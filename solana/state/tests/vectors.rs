//! Differenziale: `state.rs` contro i vettori generati dal modello (Appendice A).
//! SCALE 10 (host) e 10⁹ (Solana). I vettori 10¹⁸ sono per l'EVM (uint256).

use bernie_state::{BernieError, Vault};
use serde_json::Value;

fn int(v: &Value) -> u128 {
    v.as_str()
        .expect("interi come stringhe")
        .parse()
        .expect("intero decimale")
}

fn opt_u64(v: &Value, default: u64) -> u64 {
    if v.is_null() {
        default
    } else {
        u64::try_from(int(v)).expect("argomento in u64")
    }
}

fn replay<const SCALE: u128>(file: &str) -> usize {
    let path = format!("{}/../../vectors/{}", env!("CARGO_MANIFEST_DIR"), file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let doc: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(int(&doc["scale"]), SCALE);
    let mut steps = 0;
    for case in doc["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let prm = &case["params"];
        let price = u64::try_from(int(&prm["price"])).unwrap();
        let p = prm["penalty_bps"].as_u64().unwrap() as u16;
        let e = prm["entry_bps"].as_u64().unwrap() as u16;
        let mut v = Vault::<SCALE>::create(price, p, e).unwrap();
        // Contabilità esterna allo stato, come nel modello: saldo (senza fee) e fee cumulative.
        let (mut bal, mut fc, mut fp) = (0u64, 0u64, 0u64);
        for (i, st) in case["steps"].as_array().unwrap().iter().enumerate() {
            let op = st["op"].as_str().unwrap();
            let got: Result<Option<u64>, BernieError> = match op {
                "mint" => v
                    .mint(opt_u64(&st["u"], 0), opt_u64(&st["max_cost"], u64::MAX))
                    .map(|m| {
                        bal += m.cost;
                        fc += m.fees.creator;
                        fp += m.fees.protocol;
                        Some(m.total_paid())
                    }),
                "redeem" => v
                    .redeem(opt_u64(&st["u"], 0), opt_u64(&st["min_out"], 0))
                    .map(|r| {
                        bal -= r.gross;
                        fc += r.fees.creator;
                        fp += r.fees.protocol;
                        Some(r.out)
                    }),
                "donate" => {
                    let a = opt_u64(&st["a"], 0);
                    v.donate(a).map(|()| {
                        bal += a;
                        None
                    })
                }
                other => panic!("operazione sconosciuta {other}"),
            };
            let ctx = format!("{file} caso {name} passo {i} ({op})");
            let exp = &st["expect"];
            if exp["ok"].as_bool().unwrap() {
                let got = got.unwrap_or_else(|err| panic!("{ctx}: atteso ok, errore {err:?}"));
                let want = if exp["result"].is_null() {
                    None
                } else {
                    Some(int(&exp["result"]) as u64)
                };
                assert_eq!(got, want, "{ctx}: risultato");
            } else {
                let err = got.expect_err(&format!("{ctx}: atteso errore"));
                assert_eq!(err.name(), exp["error"].as_str().unwrap(), "{ctx}: errore");
            }
            let s = &st["state"];
            assert_eq!(v.k, int(&s["k"]), "{ctx}: k");
            assert_eq!(v.reserve, int(&s["R"]), "{ctx}: R");
            assert_eq!(v.residual, int(&s["Q"]), "{ctx}: Q");
            assert_eq!(u128::from(v.supply), int(&s["S"]), "{ctx}: S");
            assert_eq!(u128::from(bal), int(&s["bal"]), "{ctx}: saldo");
            assert_eq!(u128::from(fc), int(&s["fc"]), "{ctx}: fee creator");
            assert_eq!(u128::from(fp), int(&s["fp"]), "{ctx}: fee protocollo");
            v.check(v.k).unwrap();
            v.check_solvency(bal).unwrap();
            assert_eq!(
                u128::from(v.excess(bal).unwrap()) + (int(&s["R"]) + int(&s["Q"])) / SCALE,
                u128::from(bal)
            );
            steps += 1;
        }
    }
    steps
}

#[test]
fn vectors_scale_10() {
    assert!(replay::<10>("scale_10.json") > 1000);
}

#[test]
fn vectors_scale_1e9() {
    assert!(replay::<1_000_000_000>("scale_1e9.json") > 1000);
}
