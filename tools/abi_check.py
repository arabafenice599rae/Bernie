#!/usr/bin/env python3
"""Verifica l'interfaccia EVM del README (§11 e Appendice B).

1. Ricalcola con keccak256 ogni selettore e topic dell'Appendice B.
2. Controlla che firme di §11 e righe dell'Appendice B coincidano, in entrambe le direzioni.
3. Controlla i selettori ERC-20 canonici.
4. Se solc è disponibile (PATH o $SOLC), compila evm/src/IBernie.sol e confronta
   i selettori e i topic prodotti dal compilatore con quelli del README.

Solo libreria standard. Uscita 0 se tutto coincide, 1 altrimenti.
"""
import json
import os
import re
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from keccak import selector, topic  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ERC20 = {
    "name()": "0x06fdde03",
    "symbol()": "0x95d89b41",
    "decimals()": "0x313ce567",
    "totalSupply()": "0x18160ddd",
    "balanceOf(address)": "0x70a08231",
}


def section(readme, title):
    m = re.search(rf"^##+ {re.escape(title)}.*?$(.*?)(?=^## |\Z)", readme, re.S | re.M)
    if not m:
        raise SystemExit(f"sezione non trovata: {title}")
    return m.group(1)


def canonical(name, params):
    types = []
    for p in filter(None, (x.strip() for x in params.split(","))):
        words = [w for w in p.split() if w not in ("indexed", "calldata", "memory")]
        types.append(words[0])
    return f"{name}({','.join(types)})"


def appendix_b(readme):
    body = section(readme, "Appendice B")
    rows = re.findall(r"^\|\s*`([^`]+)`\s*\|\s*`(0x[0-9a-f]+)`\s*\|", body, re.M)
    funcs = {s: h for s, h in rows if len(h) == 10}
    events = {s: h for s, h in rows if len(h) == 66}
    return funcs, events


def section_11_evm(readme):
    body = section(readme, "11. Interfaccia on-chain")
    body = body[body.index("### Robinhood Chain"):]
    funcs, events = set(), set()
    for name, params in re.findall(r"\bfunction\s+(\w+)\s*\(([^)]*)\)", body):
        funcs.add(canonical(name, params))
    for name, params in re.findall(r"\bevent\s+(\w+)\s*\(([^)]*)\)", body):
        events.add(canonical(name, params))
    # Blocco "Token, letture": elenco di chiamate separate da virgole.
    reads = re.search(r"\*\*Token, letture:\*\*\s*```(.*?)```", body, re.S).group(1)
    for name, params in re.findall(r"(\w+)\(([^)]*)\)", reads):
        funcs.add(canonical(name, params))
    return funcs, events


def solc_hashes(solc):
    src = os.path.join(ROOT, "evm", "src", "IBernie.sol")
    out = subprocess.run([solc, "--combined-json", "abi,hashes", src],
                         check=True, capture_output=True, text=True).stdout
    funcs, events = {}, {}
    for c in json.loads(out)["contracts"].values():
        for sig, h in c["hashes"].items():
            funcs[sig] = "0x" + h
        for item in c["abi"]:
            if item["type"] == "event":
                sig = f"{item['name']}({','.join(i['type'] for i in item['inputs'])})"
                events[sig] = topic(sig)
    return funcs, events


def main():
    with open(os.path.join(ROOT, "README.md"), encoding="utf-8") as f:
        readme = f.read()
    errors = []
    funcs, events = appendix_b(readme)
    if not funcs or not events:
        errors.append("Appendice B vuota o non riconosciuta")

    for sig, h in funcs.items():
        if selector(sig) != h:
            errors.append(f"selettore errato: {sig} README {h} calcolato {selector(sig)}")
    for sig, h in events.items():
        if topic(sig) != h:
            errors.append(f"topic errato: {sig} README {h} calcolato {topic(sig)}")
    for sig, h in ERC20.items():
        if selector(sig) != h:
            errors.append(f"selettore ERC-20 errato: {sig}")

    s11_funcs, s11_events = section_11_evm(readme)
    for sig in sorted(s11_funcs - set(funcs) - set(ERC20)):
        errors.append(f"§11 dichiara {sig} ma manca nell'Appendice B")
    for sig in sorted(set(funcs) - s11_funcs):
        errors.append(f"Appendice B elenca {sig} ma §11 non lo dichiara")
    for sig in sorted(s11_events ^ set(events)):
        errors.append(f"evento non allineato tra §11 e Appendice B: {sig}")

    solc = os.environ.get("SOLC") or shutil.which("solc")
    if solc:
        c_funcs, c_events = solc_hashes(solc)
        want_funcs = dict(funcs, **ERC20)
        if c_funcs != want_funcs:
            for sig in sorted(set(c_funcs) | set(want_funcs)):
                if c_funcs.get(sig) != want_funcs.get(sig):
                    errors.append(f"solc/README divergono su {sig}: "
                                  f"solc {c_funcs.get(sig)} README {want_funcs.get(sig)}")
        if c_events != events:
            errors.append(f"eventi solc {sorted(c_events)} ≠ README {sorted(events)}")
        print(f"solc: {len(c_funcs)} selettori e {len(c_events)} eventi confrontati")
    else:
        print("solc non trovato: confronto con il compilatore saltato")

    if errors:
        print("\n".join("ERRORE " + e for e in errors))
        return 1
    print(f"ok: {len(funcs)} selettori, {len(events)} topic, {len(ERC20)} selettori ERC-20")
    return 0


if __name__ == "__main__":
    sys.exit(main())
