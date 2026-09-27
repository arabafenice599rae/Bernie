# Sicurezza

## Segnalare una vulnerabilità

Non aprire issue pubbliche, pull request o discussioni per le vulnerabilità.

Usa la **segnalazione privata di GitHub**: scheda **Security** del repository →
**Report a vulnerability**. La segnalazione resta visibile solo ai manutentori finché la
correzione non è pubblicata.

Includi, se puoi:

- componente e versione (commit) interessati;
- descrizione del problema e del suo impatto (fondi a rischio, invarianti violate, blocco);
- passi per riprodurlo, idealmente un test Mollusk, Foundry o un caso per `model/bernie.py`;
- una correzione proposta, se ne hai una.

## Tempi

- Conferma di ricezione entro **3 giorni**.
- Prima valutazione (gravità, perimetro) entro **7 giorni**.
- Correzione e divulgazione coordinate con chi segnala. Chiediamo di non divulgare nulla
  prima della correzione, o prima di **90 giorni** dalla segnalazione se non concordato
  diversamente.

## Perimetro

Dentro il perimetro:

- programma Solana: `solana/program`, `solana/state`;
- contratti EVM: `evm/src`;
- frontend: `frontend/index.html`, in particolare la costruzione delle transazioni;
- la specifica (`README.md`), quando un errore della specifica porta a codice vulnerabile.

Fuori dal perimetro:

- i deploy su devnet e testnet elencati in `deployments.json`: sono ambienti di prova,
  con tesorerie temporanee e fondi senza valore;
- dipendenze di terzi (Pinocchio, SPL Token-2022, OpenZeppelin, web3.js): segnalale ai
  rispettivi progetti, salvo che Bernie le usi in modo scorretto;
- attacchi che richiedono il controllo della chiave del creator, della tesoreria o
  dell'upgrade authority;
- MEV e ordinamento delle transazioni già descritti nella specifica (§8, §10).

## Stato

Il codice **non ha ancora avuto un audit esterno** e non è distribuito su mainnet.
La verifica attuale (modello di riferimento, vettori differenziali, fuzz, Mollusk, Foundry,
Kani, Halmos) è descritta in `README.md` §16–§17.
