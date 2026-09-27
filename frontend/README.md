# Frontend di Bernie

`index.html` è la **fonte unica** del frontend. L'artifact pubblicato si ripubblica da
questo file; le modifiche si fanno qui, non nell'artifact.

## Cosa lega la pagina al protocollo

- **Matematica:** il blocco `MATH-BEGIN … MATH-END` è il porting in BigInt del modello
  dell'Appendice A.
- **Istruzioni Solana:** i blocchi `CONST`, `ENC` e `IX` (costanti, codifica, costruttori
  `ixCreate`, `ixAta`, `ixMint`, `ixApprove`, `ixRedeem`, `ixDonate`, `ixSweep`,
  `withBudget`) sono estratti da `test/emit.mjs` ed eseguiti su Mollusk contro il
  programma (`solana/program/tests/frontend.rs`): ordine degli account, dati, vault PDA,
  mappa degli errori e limiti di CU.
- **EVM:** selettori e topic (`SEL`, `TOPIC`) coincidono con l'Appendice B, verificata
  da `tools/abi_check.py`.

I marcatori di commento (`/* CONST-BEGIN */` e simili) vanno mantenuti.

## Dopo una modifica

```
cd frontend/test && npm ci && node emit.mjs      # rigenera fixtures.json
cd ../../solana && cargo test -p bernie-program --release --test frontend
```

La CI fallisce se `fixtures.json` non è aggiornato o se le transazioni del frontend non
passano sul programma.
