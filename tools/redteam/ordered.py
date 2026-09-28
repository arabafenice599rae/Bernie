"""Ordine di valutazione degli errori di Bernie (§7), per chain e per dominio numerico.

Il modello dell'Appendice A calcola i valori con interi illimitati; le implementazioni
lavorano in domini finiti e controllano gli errori in un ordine preciso. Questo file
descrive quell'ordine passo per passo, come lo fissa §7:

1. argomenti (ZeroAmount, ExceedsSupply, NoHolders);
2. calcolo, nell'ordine dei passi di §6: `Overflow` scatta nel punto in cui un valore esce
   dal dominio della chain (Solana: importi e S in u64, k, R, Q e prodotti in u128;
   EVM: tutto in uint256), mentre Dust, ZeroPayout e Slippage si valutano quando il loro
   valore è disponibile;
3. aggiornamento dello stato (ancora Overflow nel punto del calcolo).

`InvariantViolated` è fuori dall'ordine: segnala un bug e non si raggiunge da stati validi.
Unica eccezione di dominio su Solana: se dopo l'operazione R + Q non sta in u128, il
controllo delle invarianti lo riporta come InvariantViolated (I5 non è calcolabile). Su chain
non si arriva lì, perché I2 richiederebbe più lamport di quelli esistenti.

Ogni funzione restituisce il nome del primo errore oppure None. I valori (costo, uscita,
stato finale) restano quelli del modello: `oracle.py` e i generatori li confrontano.
"""

U64 = (1 << 64) - 1
U128 = (1 << 128) - 1
U256 = (1 << 256) - 1
BPS = 10_000
FEE_TOTAL = 40  # FEE_C + FEE_P (§5)


class Stop(Exception):
    pass


def _cdiv(a, b):
    return -(-a // b)


class _Dom:
    def __init__(self, chain):
        self.chain = chain
        self.W = U128 if chain == "sol" else U256   # k, R, Q, prodotti intermedi
        self.A = U64 if chain == "sol" else U256    # importi nativi e supply

    def w(self, x):
        if x > self.W:
            raise Stop("Overflow")
        return x

    def a(self, x):
        if x > self.A:
            raise Stop("Overflow")
        return x

    def absorb(self, st):
        k, R, Q, S = st
        if S == 0:
            return st
        d = Q // S
        moved = d * S if self.chain == "sol" else self.w(d * S)
        return (self.w(k + d), self.w(R + moved), Q - moved, S)

    def fees_total(self, base):
        # Solana: base in u64, il prodotto per 40 sta in u128. EVM: base·40 controllato.
        return (base * FEE_TOTAL if self.chain == "sol" else self.w(base * FEE_TOTAL)) // BPS

    def final(self, st):
        k, R, Q, S = st
        if self.chain == "sol":
            if R + Q > self.W:
                raise Stop("InvariantViolated")
        else:
            self.w(S * k)      # checkAt: s.S * s.k
            self.w(R + Q)      # checkAt: (s.R + s.Q) % scale


def _run(fn):
    try:
        fn()
    except Stop as ex:
        return ex.args[0]
    return None


def mint(chain, st, e, SC, u, limit):
    """`limit` è max_cost su Solana, msg.value su EVM (il costo oltre il limite è Slippage)."""
    d = _Dom(chain)

    def body():
        k, R, Q, S = st
        if u == 0:
            raise Stop("ZeroAmount")
        epen = 0 if S == 0 else _cdiv(d.w(d.w(u * k) * e), BPS)
        Q = d.w(Q + epen)
        k, R, Q, S = d.absorb((k, R, Q, S))
        full = d.w(u * k)
        owed = d.w(full + epen)
        c = d.a(_cdiv(owed, SC))
        base = d.a(_cdiv(full, SC))
        ft = d.fees_total(base)
        if chain == "sol":
            if d.a(c + ft) > limit:
                raise Stop("Slippage")
        paid = d.w(c * SC)
        Q = d.w(Q + paid - owed)
        S = d.a(S + u)
        R = d.w(R + full)
        k, R, Q, S = d.absorb((k, R, Q, S))
        d.final((k, R, Q, S))
        if chain == "evm":
            # Bernie.mint: paid = cost + fee dopo BernieMath.mint, poi msg.value < paid
            if d.w(c + ft) > limit:
                raise Stop("Slippage")
    return _run(body)


def redeem(chain, st, p, SC, u, min_out):
    d = _Dom(chain)

    def body():
        k, R, Q, S = st
        if u == 0:
            raise Stop("ZeroAmount")
        if u > S:
            raise Stop("ExceedsSupply")
        full = d.w(u * k)
        pen = _cdiv(d.w(full * p), BPS)
        if full <= pen:
            raise Stop("Dust")
        g = d.a((full - pen) // SC)
        ft = d.fees_total(g)
        out = g - ft
        if out == 0:
            raise Stop("ZeroPayout")
        if out < min_out:
            raise Stop("Slippage")
        S -= u
        R -= full
        if S == 0:
            Q = 0
        else:
            Q = d.w(Q + full - g * SC)
            k, R, Q, S = d.absorb((k, R, Q, S))
        d.final((k, R, Q, S))
    return _run(body)


def donate(chain, st, SC, a):
    d = _Dom(chain)

    def body():
        k, R, Q, S = st
        if a == 0:
            raise Stop("ZeroAmount")
        if S == 0:
            raise Stop("NoHolders")
        Q = d.w(Q + d.w(a * SC))
        k, R, Q, S = d.absorb((k, R, Q, S))
        d.final((k, R, Q, S))
    return _run(body)


def check(chain, v, op, x, lim=0):
    """Primo errore di `op` ("M", "R", "D") sul vault `v` del modello, nell'ordine di §7."""
    st = (v.k, v.R, v.Q, v.S)
    if op == "M":
        return mint(chain, st, v.e, v.SC, x, lim)
    if op == "R":
        return redeem(chain, st, v.p, v.SC, x, lim)
    return donate(chain, st, v.SC, x)


def cross(v, op, x, lim, err):
    """Controllo incrociato con il modello (interi illimitati): se `err` non dipende dal
    dominio numerico, il modello deve fallire con lo stesso errore; se `err` è None, deve
    riuscire. Solleva AssertionError alla prima discordanza."""
    import copy
    w = copy.copy(v)
    try:
        if op == "M":
            w.mint(x, lim)
        elif op == "R":
            w.redeem(x, lim)
        else:
            w.donate(x)
        got = None
    except Exception as ex:  # noqa: BLE001 (Err del modello)
        got = ex.args[0]
    if err in ("Overflow", "InvariantViolated"):
        return
    assert got == err, f"{op} {x} {lim}: ordine §7 {err}, modello {got}"
