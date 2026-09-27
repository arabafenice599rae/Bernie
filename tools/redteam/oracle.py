"""Oracolo indipendente per il differential testing (pre-audit, fasi 2–4, 19–21).

L'aritmetica è il modello di riferimento dell'Appendice A (`model/bernie.py`), non il
codice delle implementazioni: costo, fee e uscita di ogni operazione si ricavano dalle
variazioni di `bal`, `fc` e `fp` del modello, senza riscrivere formule. Attorno al
modello questo file descrive ciò che la specifica aggiunge per ciascuna chain:

- Solana (§3, §6, §11, §12): il creator non compare in mint e redeem; la sua fee resta
  nel vault come excess e va al creator con `sweep`; la fee del protocollo va alla
  tesoreria nella stessa istruzione. Il redeem brucia come delegato del vault, quindi
  serve una delega Token-2022 sufficiente sull'account dell'utente (§11).
- EVM (§6, §11, §13): entrambe le fee maturano in `feesOwed` e si ritirano in pull con
  `claimFees`, `claimFeesFor` o `factory.claimAll`; `sweep` invia al creator
  `saldo − (R + Q)/SCALE − totalFeesOwed`; `mint` rimborsa l'eccedenza di `msg.value`.

Ogni operazione restituisce l'esito atteso: `{"ok": True}` oppure `{"err": [...]}` con
l'insieme degli errori ammessi. Quando più condizioni d'errore valgono insieme, la
specifica non fissa quale prevalga: l'oracolo le accetta tutte. Un'operazione fallita non
cambia nulla (atomicità).
"""
import copy
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "model"))
from bernie import Vault, Err  # noqa: E402

U64 = (1 << 64) - 1
RENT0 = 890_880  # minimo rent-exempt di un account di sistema senza dati (Solana)

# Errori di piattaforma, non di Bernie. I consumatori li riconoscono dal programma o
# dall'errore che fallisce (System, Token-2022, ERC-20, chiamata con valore).
NATIVE = "InsufficientNative"   # saldo nativo insufficiente
TOKENS = "InsufficientTokens"   # saldo token insufficiente (burn, transfer)
CREATOR = 0                     # l'utente 0 è il creator del token
TREASURY = "T"                  # chiave della tesoreria in fees_owed (EVM)


class Fail(Exception):
    def __init__(self, *names):
        super().__init__(sorted(set(names)))


class Market:
    """Un token Bernie su una chain, con N utenti."""

    def __init__(self, chain, P, p, e, wallets):
        assert chain in ("sol", "evm")
        self.chain = chain
        self.SC = 10**9 if chain == "sol" else 10**18
        self.v = Vault(P, p, e, self.SC)
        self.n = len(wallets)
        self.native = list(wallets)
        self.tokens = [0] * self.n
        self.delegated = [0] * self.n   # Solana: quantità delegata al vault
        self.has_delegate = [False] * self.n  # Solana: il vault è il delegato (anche con quantità 0)
        self.treasury_in = 0            # Solana: nativo ricevuto dalla tesoreria
        self.swept = 0                  # totale inviato al creator con sweep
        self.fees_owed = {}             # EVM: {utente | "T": fee dovute}
        self.claimed = 0                # EVM: fee già ritirate
        self.total0 = self.system_total()

    # ── grandezze derivate ──

    def available(self):
        """Solana: lamport del vault oltre il rent. EVM: saldo del contratto."""
        if self.chain == "sol":
            return self.v.bal + self.v.fc - self.swept
        return self.v.bal + self.v.fc + self.v.fp - self.claimed - self.swept

    def total_fees_owed(self):
        return sum(self.fees_owed.values())

    def liabilities(self):
        return (self.v.R + self.v.Q) // self.SC

    def excess(self):
        x = self.available() - self.liabilities()
        return x - self.total_fees_owed() if self.chain == "evm" else x

    def system_total(self):
        return sum(self.native) + self.treasury_in + self.available()

    def snapshot(self):
        return {
            "k": self.v.k, "R": self.v.R, "Q": self.v.Q, "S": self.v.S,
            "native": list(self.native), "tokens": list(self.tokens),
            "available": self.available(), "treasury_in": self.treasury_in,
            "fees_owed": [self.fees_owed.get(i, 0) for i in range(self.n)] + [self.fees_owed.get(TREASURY, 0)],
            "total_fees_owed": self.total_fees_owed(),
            "delegated": list(self.delegated),
        }

    # ── invarianti, indipendenti dalle singole funzioni (fasi 3 e 19) ──

    def check(self, k_prev):
        v = self.v
        assert v.R == v.S * v.k, "I1"
        assert self.available() * self.SC >= v.R + v.Q, "I2"
        assert v.k >= k_prev, "I3"
        assert v.S > 0 or v.Q == 0, "I4"
        assert (v.R + v.Q) % self.SC == 0, "I5"
        assert v.S == 0 or v.Q < v.S, "I6"
        assert min(v.R, v.Q, v.S) >= 0
        assert sum(self.tokens) == v.S, "supply = somma dei saldi"
        assert self.excess() >= 0, "excess ≥ 0"
        assert min(self.native + self.tokens + self.delegated) >= 0
        assert self.system_total() == self.total0, "conservazione del nativo"

    # ── esecuzione atomica ──

    def _atomic(self, fn):
        saved = copy.deepcopy(self.__dict__)
        k_prev = self.v.k
        try:
            fn()
        except Fail as ex:
            self.__dict__.clear()
            self.__dict__.update(saved)
            return {"err": ex.args[0]}
        except Err as ex:  # errore del modello: atomico per costruzione
            self.__dict__.clear()
            self.__dict__.update(saved)
            return {"err": [ex.args[0]]}
        self.check(k_prev)
        return {"ok": True}

    def _math(self, op, *args):
        """Operazione sul modello: restituisce il vault nuovo e le variazioni di bal, fc, fp."""
        v = copy.deepcopy(self.v)
        b0, c0, p0 = v.bal, v.fc, v.fp
        ret = getattr(v, op)(*args)
        return v, ret, v.bal - b0, v.fc - c0, v.fp - p0

    # ── operazioni degli utenti ──

    def mint(self, a, u, pay):
        """Solana: pay = max_cost. EVM: pay = msg.value (l'eccedenza torna indietro)."""
        def run():
            v, total, c, fc, fp = self._math("mint", u)      # total = c + ft
            errs = []
            if total > pay:
                errs.append("Slippage")
            if self.chain == "sol":
                if total > U64 or v.S > U64:
                    errs.append("Overflow")
                if self.native[a] - total < RENT0:
                    errs.append(NATIVE)
            if errs:
                raise Fail(*errs)
            self.v = v
            self.native[a] -= total
            self.tokens[a] += u
            if self.chain == "sol":
                self.treasury_in += fp            # fc resta nel vault (v.fc)
            else:
                self.fees_owed[CREATOR] = self.fees_owed.get(CREATOR, 0) + fc
                self.fees_owed[TREASURY] = self.fees_owed.get(TREASURY, 0) + fp
        return self._atomic(run)

    def redeem(self, a, u, min_out, approve=None):
        """Solana: `approve` è la delega impostata nella stessa transazione (None = nessuna
        istruzione di approve: resta la delega precedente)."""
        def run():
            errs = []
            if self.chain == "sol":
                if approve is not None:               # l'approve fa parte della stessa transazione
                    self.delegated[a] = approve
                    self.has_delegate[a] = True
                if not self.has_delegate[a] or self.delegated[a] < u:
                    errs.append("MissingDelegation")
            if u > self.tokens[a]:
                errs.append(TOKENS)
            try:
                v, out, dbal, fc, fp = self._math("redeem", u, min_out)
            except Err as ex:
                errs.append(ex.args[0])
            if errs:
                raise Fail(*errs)
            self.v = v
            self.tokens[a] -= u
            self.native[a] += out
            if self.chain == "sol":
                self.delegated[a] -= u
                if self.delegated[a] == 0:            # Token-2022 azzera il delegato a quantità 0
                    self.has_delegate[a] = False
                self.treasury_in += fp
            else:
                self.fees_owed[CREATOR] = self.fees_owed.get(CREATOR, 0) + fc
                self.fees_owed[TREASURY] = self.fees_owed.get(TREASURY, 0) + fp
        return self._atomic(run)

    def donate(self, a, x):
        def run():
            v, _, dbal, _, _ = self._math("donate", x)
            if self.chain == "sol" and self.native[a] - x < RENT0:
                raise Fail(NATIVE)
            self.v = v
            self.native[a] -= x
        return self._atomic(run)

    def sweep(self):
        """Chiamabile da chiunque; l'excess va sempre al creator (utente 0)."""
        def run():
            x = self.excess()
            if x == 0:
                raise Fail("NothingToClaim")
            self.swept += x
            self.native[CREATOR] += x
        return self._atomic(run)

    def transfer(self, a, b, x):
        """Trasferimento di token tra utenti: non tocca lo stato di Bernie."""
        def run():
            if x > self.tokens[a]:
                raise Fail(TOKENS)
            self.tokens[a] -= x
            self.tokens[b] += x
        return self._atomic(run)

    def revoke(self, a):
        """Solana: Revoke della delega (sempre riuscito)."""
        def run():
            self.delegated[a] = 0
            self.has_delegate[a] = False
        return self._atomic(run)

    def claim(self, who):
        """EVM: claimFees da parte di `who` (indice utente o "T")."""
        def run():
            amt = self.fees_owed.get(who, 0)
            if amt == 0:
                raise Fail("NothingToClaim")
            self.fees_owed[who] = 0
            self.claimed += amt
            if who == TREASURY:
                self.treasury_in += amt
            else:
                self.native[who] += amt
        return self._atomic(run)
