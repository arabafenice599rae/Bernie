//! Stato del vault e aritmetica di Bernie v1.6 (§2–§7), generici su `SCALE`.
//!
//! Stesse formule, stesso ordine dei controlli e stessi arrotondamenti del modello
//! di riferimento (Appendice A): i vettori in `vectors/` lo verificano bit per bit.
//! Ogni operazione lavora su una copia e la scrive solo se riesce, quindi un errore
//! lascia lo stato intatto, come l'annullamento della transazione.
//!
//! `SCALE` è anche `10^d`, quindi `k₀ = P` (§2). Su Solana `SCALE = 10⁹`.

use crate::error::BernieError;

pub const BPS: u128 = 10_000;
pub const FEE_C: u128 = 20;
pub const FEE_P: u128 = 20;
pub const PEN_MIN: u16 = 100;
pub const PEN_MAX: u16 = 1_000;

/// SCALE su Solana (§20).
pub const SOLANA_SCALE: u128 = 1_000_000_000;
/// Limiti di prezzo su Solana, in lamport per token intero (§20).
pub const MIN_PRICE: u64 = 1_000_000;
pub const MAX_PRICE: u64 = 1_000_000_000_000_000;

/// Limiti dei metadati in byte UTF-8 (§6, create; errore 16).
pub const NAME_MAX: usize = 32;
pub const SYMBOL_MAX: usize = 10;
pub const URI_MAX: usize = 200;

pub type Result<T> = core::result::Result<T, BernieError>;

/// Parte mutabile e parametri economici dell'account vault v4 (§3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vault<const SCALE: u128> {
    pub penalty_bps: u16,
    pub entry_bps: u16,
    /// Sotto-unità per unità base.
    pub k: u128,
    /// `R`: backing contabilizzato, in sotto-unità.
    pub reserve: u128,
    /// `Q`: quota in attesa di absorb, in sotto-unità.
    pub residual: u128,
    /// `S`: supply in unità base.
    pub supply: u64,
}

/// Fee con arrotondamento unico (§5), in unità native.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fees {
    pub total: u64,
    pub creator: u64,
    pub protocol: u64,
}

/// Esito di un mint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mint {
    /// `c`: backing più penalità d'ingresso, in unità native.
    pub cost: u64,
    /// Fee calcolate su `b = ⌈full / SCALE⌉`.
    pub fees: Fees,
    /// `full = u · k'`, in sotto-unità.
    pub full: u128,
    /// Penalità d'ingresso, in sotto-unità.
    pub epen: u128,
}

impl Mint {
    /// Quanto paga l'utente: `c + ft`.
    /// Solana: `c + fc` al vault, `fp` alla tesoreria (v1.6).
    pub fn total_paid(&self) -> u64 {
        // c + ft ≤ max_cost ≤ u64::MAX, verificato in `mint`.
        self.cost + self.fees.total
    }
}

/// Esito di un redeem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Redeem {
    /// `g`: lordo in unità native.
    pub gross: u64,
    pub fees: Fees,
    /// `out = g − ft`: all'utente.
    pub out: u64,
    pub full: u128,
    pub pen: u128,
}

fn cdiv(a: u128, b: u128) -> u128 {
    // a / b per eccesso; b > 0 in ogni chiamata.
    a.div_ceil(b)
}

fn to_u64(x: u128) -> Result<u64> {
    u64::try_from(x).map_err(|_| BernieError::Overflow)
}

/// `fees(base)` di §5: un solo arrotondamento sulla fee totale.
pub fn split_fees(base: u64) -> Fees {
    let ft = u128::from(base) * (FEE_C + FEE_P) / BPS;
    let fp = ft * FEE_P / (FEE_C + FEE_P);
    // ft ≤ base, quindi entrambe stanno in u64.
    Fees {
        total: ft as u64,
        creator: (ft - fp) as u64,
        protocol: fp as u64,
    }
}

/// Parametri di `create` su Solana (§6): prezzo, penalità.
pub fn validate_params(price: u64, penalty_bps: u16, entry_bps: u16) -> Result<()> {
    if !(MIN_PRICE..=MAX_PRICE).contains(&price) {
        return Err(BernieError::PriceOutOfRange);
    }
    validate_penalties(penalty_bps, entry_bps)
}

pub fn validate_penalties(penalty_bps: u16, entry_bps: u16) -> Result<()> {
    if !(PEN_MIN..=PEN_MAX).contains(&penalty_bps) || entry_bps > penalty_bps {
        return Err(BernieError::PenaltyOutOfRange);
    }
    Ok(())
}

/// Metadati di `create`: nome 1–32, simbolo 1–10, URI 0–200 byte (errore 16).
pub fn validate_metadata(name: &[u8], symbol: &[u8], uri: &[u8]) -> Result<()> {
    let ok = (1..=NAME_MAX).contains(&name.len())
        && (1..=SYMBOL_MAX).contains(&symbol.len())
        && uri.len() <= URI_MAX;
    if ok {
        Ok(())
    } else {
        Err(BernieError::MetadataTooLong)
    }
}

impl<const SCALE: u128> Vault<SCALE> {
    /// Stato iniziale: `k = P`, `R = Q = S = 0`. I limiti di prezzo sono di
    /// `validate_params` (dipendono dalla chain); qui solo penalità e `P > 0`.
    pub fn create(price: u64, penalty_bps: u16, entry_bps: u16) -> Result<Self> {
        validate_penalties(penalty_bps, entry_bps)?;
        if price == 0 {
            return Err(BernieError::PriceOutOfRange);
        }
        Ok(Self {
            penalty_bps,
            entry_bps,
            k: u128::from(price),
            reserve: 0,
            residual: 0,
            supply: 0,
        })
    }

    /// `absorb` di §5: distribuisce `⌊Q / S⌋` su ogni unità. Conserva `R + Q`.
    pub fn absorb(&mut self) -> Result<()> {
        if self.supply == 0 {
            return Ok(());
        }
        let s = u128::from(self.supply);
        let delta = self.residual / s;
        let moved = delta * s; // ≤ Q
        self.k = self.k.checked_add(delta).ok_or(BernieError::Overflow)?;
        self.reserve = self
            .reserve
            .checked_add(moved)
            .ok_or(BernieError::Overflow)?;
        self.residual -= moved;
        Ok(())
    }

    /// I1, I3, I4, I5, I6 (§4). I2 dipende dal saldo: `check_solvency`.
    pub fn check(&self, k_prev: u128) -> Result<()> {
        let s = u128::from(self.supply);
        let i1 = s.checked_mul(self.k) == Some(self.reserve);
        let i3 = self.k >= k_prev;
        let i4 = self.supply > 0 || self.residual == 0;
        let i5 = match self.reserve.checked_add(self.residual) {
            Some(t) => t % SCALE == 0,
            None => false,
        };
        let i6 = self.supply == 0 || self.residual < s;
        if i1 && i3 && i4 && i5 && i6 {
            Ok(())
        } else {
            Err(BernieError::InvariantViolated)
        }
    }

    /// Obbligazioni del vault in unità native: `(R + Q) / SCALE`, esatto per I5.
    pub fn liabilities(&self) -> Result<u64> {
        let t = self
            .reserve
            .checked_add(self.residual)
            .ok_or(BernieError::Overflow)?;
        to_u64(t / SCALE)
    }

    /// I2 su Solana: `available` sono i lamport del vault meno il rent.
    pub fn check_solvency(&self, available: u64) -> Result<()> {
        if available >= self.liabilities()? {
            Ok(())
        } else {
            Err(BernieError::InvariantViolated)
        }
    }

    /// Excess (§3): `available − (R + Q)/SCALE`. Su Solana `available` è
    /// lamport − rent; su EVM è saldo − totalFeesOwed.
    pub fn excess(&self, available: u64) -> Result<u64> {
        available
            .checked_sub(self.liabilities()?)
            .ok_or(BernieError::InvariantViolated)
    }

    /// `sweep` (§6): importo dovuto al creator; non modifica lo stato.
    pub fn sweep_amount(&self, available: u64) -> Result<u64> {
        match self.excess(available)? {
            0 => Err(BernieError::NothingToClaim),
            x => Ok(x),
        }
    }

    /// `mint(u, max_cost)` di §6.
    pub fn mint(&mut self, u: u64, max_cost: u64) -> Result<Mint> {
        if u == 0 {
            return Err(BernieError::ZeroAmount);
        }
        let mut s = *self;
        let k_prev = s.k;
        let uu = u128::from(u);
        let epen = if s.supply == 0 {
            0
        } else {
            let x = uu
                .checked_mul(s.k)
                .and_then(|x| x.checked_mul(u128::from(s.entry_bps)))
                .ok_or(BernieError::Overflow)?;
            cdiv(x, BPS)
        };
        s.residual = s.residual.checked_add(epen).ok_or(BernieError::Overflow)?;
        s.absorb()?; // solo gli holder esistenti
        let full = uu.checked_mul(s.k).ok_or(BernieError::Overflow)?;
        let owed = full.checked_add(epen).ok_or(BernieError::Overflow)?;
        let c = to_u64(cdiv(owed, SCALE))?;
        let fees = split_fees(to_u64(cdiv(full, SCALE))?);
        match c.checked_add(fees.total) {
            Some(t) if t <= max_cost => {}
            Some(_) => return Err(BernieError::Slippage),
            None => return Err(BernieError::Overflow),
        }
        // Solo il resto di arrotondamento: c·SCALE − full − epen < SCALE.
        let paid = u128::from(c)
            .checked_mul(SCALE)
            .ok_or(BernieError::Overflow)?;
        s.residual = s
            .residual
            .checked_add(paid - owed)
            .ok_or(BernieError::Overflow)?;
        s.supply = s.supply.checked_add(u).ok_or(BernieError::Overflow)?;
        s.reserve = s.reserve.checked_add(full).ok_or(BernieError::Overflow)?;
        s.absorb()?;
        s.check(k_prev)?;
        *self = s;
        Ok(Mint {
            cost: c,
            fees,
            full,
            epen,
        })
    }

    /// `redeem(u, min_out)` di §6.
    pub fn redeem(&mut self, u: u64, min_out: u64) -> Result<Redeem> {
        if u == 0 {
            return Err(BernieError::ZeroAmount);
        }
        if u > self.supply {
            return Err(BernieError::ExceedsSupply);
        }
        let mut s = *self;
        let k_prev = s.k;
        let full = u128::from(u)
            .checked_mul(s.k)
            .ok_or(BernieError::Overflow)?;
        let pen = cdiv(
            full.checked_mul(u128::from(s.penalty_bps))
                .ok_or(BernieError::Overflow)?,
            BPS,
        );
        if full <= pen {
            return Err(BernieError::Dust);
        }
        let g = to_u64((full - pen) / SCALE)?;
        let fees = split_fees(g);
        let out = g - fees.total;
        if out == 0 {
            return Err(BernieError::ZeroPayout);
        }
        if out < min_out {
            return Err(BernieError::Slippage);
        }
        s.supply -= u;
        s.reserve = s
            .reserve
            .checked_sub(full)
            .ok_or(BernieError::InvariantViolated)?;
        if s.supply == 0 {
            s.residual = 0; // penalità e resti diventano excess
        } else {
            // full − g·SCALE ≥ pen > 0
            let rest = full - u128::from(g) * SCALE;
            s.residual = s.residual.checked_add(rest).ok_or(BernieError::Overflow)?;
            s.absorb()?;
        }
        s.check(k_prev)?;
        *self = s;
        Ok(Redeem {
            gross: g,
            fees,
            out,
            full,
            pen,
        })
    }

    /// `donate(a)` di §6.
    pub fn donate(&mut self, a: u64) -> Result<()> {
        if a == 0 {
            return Err(BernieError::ZeroAmount);
        }
        if self.supply == 0 {
            return Err(BernieError::NoHolders);
        }
        let mut s = *self;
        let k_prev = s.k;
        let add = u128::from(a)
            .checked_mul(SCALE)
            .ok_or(BernieError::Overflow)?;
        s.residual = s.residual.checked_add(add).ok_or(BernieError::Overflow)?;
        s.absorb()?;
        s.check(k_prev)?;
        *self = s;
        Ok(())
    }
}

/// Harness di §17 per Kani, su `SCALE = 10`.
///
/// Gli input nascono come `u8`/`u16` e vengono estesi a u128: i bit alti sono costanti,
/// quindi il solver non esplora divisioni a 128 bit libere. La corrispondenza con le
/// SCALE reali è coperta dai vettori differenziali.
#[cfg(kani)]
mod verification {
    use super::*;

    type V = Vault<10>;

    fn any_canonical() -> V {
        let k = u128::from(kani::any::<u8>());
        kani::assume(k >= 1);
        let supply = u64::from(kani::any::<u8>() & 0x1f);
        let residual = u128::from(kani::any::<u8>() & 0x1f);
        kani::assume(if supply == 0 {
            residual == 0
        } else {
            residual < u128::from(supply)
        });
        let reserve = k * u128::from(supply);
        kani::assume((reserve + residual) % 10 == 0);
        let penalty_bps: u16 = kani::any();
        kani::assume((PEN_MIN..=PEN_MAX).contains(&penalty_bps));
        let entry_bps: u16 = kani::any();
        kani::assume(entry_bps <= penalty_bps);
        V {
            penalty_bps,
            entry_bps,
            k,
            reserve,
            residual,
            supply,
        }
    }

    fn any_amount() -> u64 {
        let u = u64::from(kani::any::<u8>() & 0x1f);
        kani::assume(u >= 1);
        u
    }

    fn canonical(v: &V) -> bool {
        if v.supply == 0 {
            v.residual == 0 && v.reserve == 0
        } else {
            v.residual < u128::from(v.supply)
        }
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    fn absorb_conserves_and_canonicalizes() {
        let mut v = any_canonical();
        v.residual += u128::from(kani::any::<u16>());
        let before = v.reserve + v.residual;
        v.absorb().unwrap();
        assert_eq!(v.reserve + v.residual, before);
        assert_eq!(v.reserve, u128::from(v.supply) * v.k);
        assert!(v.supply == 0 || v.residual < u128::from(v.supply));
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    fn mint_canonical_or_error() {
        let v = any_canonical();
        let u = any_amount();
        let mut w = v;
        match w.mint(u, u64::MAX) {
            Ok(m) => {
                assert!(canonical(&w));
                assert!(w.k >= v.k);
                assert_eq!(
                    w.reserve + w.residual,
                    v.reserve + v.residual + u128::from(m.cost) * 10
                );
                assert!(u128::from(m.cost) * 10 >= m.full + m.epen);
                assert!(m.full.div_ceil(10) <= u128::from(m.cost)); // b ≤ c
            }
            Err(_) => assert_eq!(w, v),
        }
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    fn redeem_canonical_or_error() {
        let v = any_canonical();
        let u = any_amount();
        let mut w = v;
        match w.redeem(u, 0) {
            Ok(r) => {
                assert!(canonical(&w));
                assert!(w.k >= v.k);
                assert!(r.out <= r.gross);
                assert!(u128::from(r.gross) * 10 <= r.full - r.pen);
                if w.supply > 0 {
                    assert_eq!(
                        w.reserve + w.residual,
                        v.reserve + v.residual - u128::from(r.gross) * 10
                    );
                }
            }
            Err(_) => assert_eq!(w, v),
        }
    }

    #[kani::proof]
    #[kani::solver(cadical)]
    fn donate_canonical_or_error() {
        let v = any_canonical();
        let a = u64::from(kani::any::<u8>());
        let mut w = v;
        match w.donate(a) {
            Ok(()) => {
                assert!(canonical(&w));
                assert!(w.k >= v.k);
                assert_eq!(
                    w.reserve + w.residual,
                    v.reserve + v.residual + u128::from(a) * 10
                );
            }
            Err(_) => assert_eq!(w, v),
        }
    }

    /// Qui l'intero dominio u64: split_fees usa solo moltiplicazioni e divisioni per costanti.
    #[kani::proof]
    #[kani::solver(cadical)]
    fn fees_step_and_split() {
        let base: u64 = kani::any();
        kani::assume(base < u64::MAX);
        let a = split_fees(base);
        let b = split_fees(base + 1);
        assert!(b.total - a.total <= 1);
        assert_eq!(a.creator + a.protocol, a.total);
        // P7: out = g − ft non decrescente.
        assert!(base + 1 - b.total >= base - a.total);
    }
}
