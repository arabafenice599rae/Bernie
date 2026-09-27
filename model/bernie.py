FEE_C = 20; FEE_P = 20; BPS = 10_000
def cdiv(a, b): return -(-a // b)
class Err(Exception): pass

def split_fees(base):
    ft = base * (FEE_C + FEE_P) // BPS          # un solo arrotondamento
    fp = ft * FEE_P // (FEE_C + FEE_P)
    return ft, ft - fp, fp                       # totale, creator, protocollo

class Vault:
    def __init__(s, P, p, e, SCALE):
        if not (0 <= e <= p): raise Err("PenaltyOutOfRange")
        s.k, s.R, s.Q, s.S, s.SC, s.p, s.e = P, 0, 0, 0, SCALE, p, e
        s.bal = 0; s.fc = 0; s.fp = 0
    def absorb(s):
        if s.S == 0: return
        d = s.Q // s.S; s.k += d; s.R += d * s.S; s.Q -= d * s.S
    def inv(s, k_prev):
        assert s.R == s.S * s.k                      # I1
        assert s.bal * s.SC >= s.R + s.Q             # I2
        assert s.k >= k_prev                         # I3
        assert s.S > 0 or s.Q == 0                   # I4
        assert (s.R + s.Q) % s.SC == 0               # I5
        assert s.S == 0 or s.Q < s.S                 # I6
    def mint(s, u, max_cost=None):
        if u <= 0: raise Err("ZeroAmount")
        k0 = s.k
        epen = 0 if s.S == 0 else cdiv(u * s.k * s.e, BPS)
        s.Q += epen; s.absorb()
        full = u * s.k
        c = cdiv(full + epen, s.SC)
        ft, fc, fp = split_fees(cdiv(full, s.SC))
        if max_cost is not None and c + ft > max_cost: raise Err("Slippage")
        s.Q += c * s.SC - full - epen
        s.S += u; s.R += full; s.absorb()
        s.bal += c; s.fc += fc; s.fp += fp
        s.inv(k0); return c + ft
    def redeem(s, u, min_out=0):
        if u <= 0: raise Err("ZeroAmount")
        if u > s.S: raise Err("ExceedsSupply")
        k0 = s.k
        full = u * s.k; pen = cdiv(full * s.p, BPS)
        if full <= pen: raise Err("Dust")
        g = (full - pen) // s.SC
        ft, fc, fp = split_fees(g); out = g - ft
        if out <= 0: raise Err("ZeroPayout")
        if out < min_out: raise Err("Slippage")
        s.S -= u; s.R -= full
        if s.S == 0: s.Q = 0
        else: s.Q += full - g * s.SC; s.absorb()
        s.bal -= g; s.fc += fc; s.fp += fp
        s.inv(k0); return out
    def donate(s, a):
        if a <= 0: raise Err("ZeroAmount")
        if s.S == 0: raise Err("NoHolders")
        k0 = s.k; s.Q += a * s.SC; s.absorb(); s.bal += a; s.inv(k0)
    def excess(s): return s.bal - (s.R + s.Q) // s.SC
    def value(s, units): return units * s.k // s.SC

class Ledger:
    """Saldi individuali sopra Vault; ogni operazione è atomica (copia e commit)."""
    def __init__(s, vault, wallets):
        import copy; s._copy = copy.deepcopy
        s.v = vault; s.w = dict(wallets); s.tok = {a: 0 for a in wallets}
        s.total0 = sum(s.w.values()) + vault.bal
    def _commit(s, v): s.v = v; s.check()
    def mint(s, a, u, max_cost=None):
        v = s._copy(s.v); cost = v.mint(u, max_cost)
        if s.w[a] < cost: raise Err("InsufficientFunds")
        s.w[a] -= cost; s.tok[a] += u; s._commit(v); return cost
    def redeem(s, a, u, min_out=0):
        if u > s.tok[a]: raise Err("InsufficientBalance")
        v = s._copy(s.v); out = v.redeem(u, min_out)
        s.w[a] += out; s.tok[a] -= u; s._commit(v); return out
    def donate(s, a, x):
        if s.w[a] < x: raise Err("InsufficientFunds")
        v = s._copy(s.v); v.donate(x); s.w[a] -= x; s._commit(v)
    def value(s, a): return s.v.value(s.tok[a])
    def check(s):
        assert sum(s.tok.values()) == s.v.S
        assert sum(s.w.values()) + s.v.fc + s.v.fp + s.v.bal == s.total0
        assert s.v.excess() >= 0
