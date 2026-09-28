# Pre-audit interno / red team — report finale (fase 25)

Tranche red-team sulle implementazioni di Bernie v1.6 (programma Solana, contratti EVM,
crate `bernie-state`), con il modello Python dell'Appendice A come oracolo indipendente.
Branch `claude/readme-review-8901um`, dalla baseline `4583bf5`.

**Esito del gate: YELLOW.** Nessun bug nel protocollo, nessuna condizione RED. Restano aperti
un punto di deployment (tesorerie Solana segnaposto) e limiti di misura dichiarati sotto; la
precedenza degli errori (Q-1) è stata decisa e fissata in §7. Dettagli nel [gate finale](#gate-finale).

La logica del protocollo non è stata modificata: `solana/program/src`, `solana/state/src` ed
`evm/src` sono identici alla baseline. Tutte le correzioni riguardano oracolo e harness di test.

## 1. Baseline

[Baseline.md](Baseline.md): tutte le suite esistenti verdi (bernie-state 8 test, programma
12, Foundry 17, Python 17 + 10 subtest, vettori, ABI, frontend, gitleaks), Kani 5/5
VERIFIED, Halmos 9/9 (4 in locale, tutti e 9 in CI). Nessun test saltato. Versioni degli strumenti registrate.

## 2. Test aggiunti

| Dove | Cosa | Numero |
|---|---|---|
| `solana/state/tests/guards.rs` | ogni invariante in isolamento, soglie inclusive (solvenza, excess, sweep, min_out, max_cost) | 5 test |
| `solana/state/tests/boundary.rs` | 2 058 vettori di confine, absorb e idempotenza, tabella fee con oracolo intero | 3 test |
| `solana/program/tests/adversarial.rs` | vault falsi, cross-market, Token-2022 incoerente, firme, programmi, tesoreria, insolvenza, permutazioni e duplicati di account, rent esatto, log `State`, precedenza degli errori (§7) | 14 test |
| `solana/program/tests/differential.rs` | differential e state-machine su Mollusk + regressione T-2 | 2 test |
| `solana/state/examples/diff_stream.rs` | consumatore ad alto volume del differential sullo stato | harness |
| `evm/test/Adversarial.t.sol` | invarianti in isolamento, stato corrotto, rientri su ogni percorso di ETH, cloni, furti, doppi prelievi, atomicità, precedenza degli errori (§7) | 19 test |
| `evm/test/Boundary.t.sol` | 978 vettori di confine SCALE 10¹⁸, absorb | 2 test |
| `evm/test/Differential.t.sol` | differential e state-machine su Foundry | 1 test |
| `tests/test_redteam.py` | determinismo del generatore, replay, regressione T-1 | 4 test |
| `tools/redteam/` | oracolo (`oracle.py`), ordine degli errori di §7 per chain (`ordered.py`), generatore (`gen.py`), stream (`stream.py`), vettori (`boundary.py`), minimizzatore (`minimize.py`) | harness |
| `tools/mutation/` | harness di mutazione, cataloghi mirati (55 Solana, 24 stato, 43 EVM), equivalenti motivati | harness |

CI: vettori di confine e uno stream breve a ogni push (`ci.yml`); campagne lunghe e mutation
testing ogni settimana (`redteam.yml`).

## 3. Proprietà verificate

Matrice completa proprietà → implementazione → test in [TestMatrix.md](TestMatrix.md). Ogni
proprietà economica critica (I1–I6, conservazione, fee, sweep, solvenza, autorizzazioni)
ha almeno un test positivo e uno avversariale. Oracoli: modello Python per la matematica,
invarianti della specifica per le proprietà, rifiuto più stato invariato per gli attacchi
(fase 24). Verifica formale: Kani (5 harness su `bernie-state`) e Halmos (9 harness su
`BernieMath`), tutti verificati in CI sul commit `430808f`.

## 4. Campagne di fuzzing (state machine)

Sequenze multi-utente con operazioni valide e non valide (mint, redeem con e senza approve,
donate, sweep, transfer, revoke, claim), errori attesi dall'oracolo, invarianti dopo ogni passo.

| Campagna | Seed | Sequenze × passi | Utenti | Esito |
|---|---|---|---|---|
| Mollusk (programma SBF reale) | 1000 | 1 000 × 1 000 | 8 | 10⁶ passi, 0 divergenze |
| Mollusk, 10 lotti | 20000–29999 | 10 000 × 1 000 | 8 | 10⁷ passi, 0 divergenze (10 lotti su 10 verdi) |
| Foundry (contratti reali) | 5000 | 1 000 × 500 | 8 | 5 × 10⁵ passi, 0 divergenze |
| Foundry, 10 lotti | 60000–69999 | 10 000 × 500 | 8 | 5 × 10⁶ passi, 0 divergenze (10 lotti su 10 verdi) |
| Marathon Solana | 778 | 1 × 10 000 | 120 | 0 divergenze |
| Marathon Solana | 777 | 1 × 100 000 | 120 | 0 divergenze (stato utenti completo ogni 1 000 passi) |

Determinismo: due generazioni indipendenti degli stessi seed (Solana, EVM, stream) danno file
identici byte per byte (SHA-256 `cb4658e3…`); `tests/test_redteam.py` lo verifica in CI.

## 5. Campagne differential

| Livello | Seed | Volume | Esito |
|---|---|---|---|
| `bernie-state` contro oracolo, SCALE 10 | 424242 | 50 000 × 1 000 = 5 × 10⁷ passi | identici |
| `bernie-state` contro oracolo, SCALE 10⁹ | 424242 | 50 000 × 1 000 = 5 × 10⁷ passi | identici |
| Vettori di confine Solana | — | 2 058 casi | identici |
| Vettori di confine EVM | — | 978 casi | identici |
| Programma e contratti | vedi §4 | ogni passo confronta esito, k, R, Q, S, lamport/ETH e token di ogni utente, deleghe, fee dovute, tesoreria | identici |

Totale sul solo livello di stato: 10⁵ sequenze × 10³ passi = 10⁸ passi. In queste campagne gli
errori ammessi erano insiemi quando più condizioni valevano insieme. Dopo la decisione su Q-1
l'oracolo è esatto (un solo errore, nell'ordine di §7) e le campagne di verifica sono state
ripetute in quella forma: vedi [Q-1](#domande-sulla-specifica).

## 6. Mutation testing

| Target | KILLED | EQUIVALENT | COMPILE_ERROR | SURVIVED |
|---|---|---|---|---|
| Solana (programma) | 174 | 13 | 36 | **0** |
| EVM | 178 | 3 | 0 | **0** |
| `bernie-state` (cargo-mutants + catalogo) | 155 | 0 | 0 | **0** |

Report per mutante: [mutation/](mutation/). Primo giro: 61 sopravvissuti Solana e 38 EVM;
ognuno è stato analizzato ([survivors.md](mutation/survivors.md)): 83 uccisi da test nuovi
(la proprietà mancante è il nome del test), 16 classificati EQUIVALENT con motivazione in
`tools/mutation/equivalent.json`. Nessun mutante realistico e pericoloso sopravvive.
Gli equivalenti sono: trasferimenti di 0 lamport, un buffer locale più grande, controlli
difensivi veri per costruzione (supply dopo mint/burn, solvenza dopo sweep, underflow che il
runtime rifiuta comunque), l'identità ⌊⌊b/250⌋/2⌋ = ⌊b/500⌋ (vera finché FEE_C = FEE_P) e la
lettura difensiva del tag del delegato (Token-2022 azzera anche la chiave, vedi O-4). I COMPILE_ERROR non sono contati come prova di forza della suite.

## 7. Attacchi (fase 18)

Tutti rifiutati, con stato invariato verificato account per account (Solana: intero store
Mollusk; EVM: stato e saldi):

- **Furto / inflazione / deflazione**: stato di input corrotto (ogni invariante violata da
  sola), supply ±1 rispetto a S, vault con lamport/ETH mancanti (`insolvent_vault…`,
  `test_short_vault…`), donazioni e absorb ai confini.
- **Creator theft**: sweep solo al creator registrato (beneficiario falso → rifiuto), fee del
  creator non reclamabili da altri (`claimFeesFor` paga l'account, non il chiamante).
- **Treasury theft**: tesoreria fuori lista rifiutata (Solana); treasury immutabile, un clone
  arbitrario non la redirige (EVM).
- **Doppio prelievo**: redeem, sweep e claim ripetuti, rientri durante mint (rimborso), redeem,
  claim e sweep.
- **Cross-market**: vault di un mercato con mint di un altro, header manomessi, vault posseduto
  da un altro programma.
- **Account**: tutte le permutazioni e i duplicati degli account di ogni istruzione, firmatari
  e scrivibilità mancanti, programmi sbagliati, PDA con seed errati, secondo create.
- **Redeem forzato**: senza il controllo del firmatario chiunque potrebbe riscattare da un conto
  con delega attiva; il test lo prova e il mutante che toglie il controllo muore.

## 8. Conservazione economica (fase 19)

Controllata dopo ogni passo di ogni campagna, in modo indipendente dalle singole funzioni:
somma dei lamport (Solana) o degli ETH (EVM) di utenti, vault/contratto e tesorerie costante;
somma dei saldi token = supply = S; I2 (backing ≥ (R + Q)/SCALE + fee dovute); excess ≥ 0.
L'oracolo tiene il bilancio completo (valore utenti + fee creator + fee protocollo + backing +
excess) e lo confronta con il totale iniziale. Nessun denaro creato, distrutto o contato due volte.

## 9. Risultati Solana

- Differential e fuzzing: §4, §5. Atomicità: ogni passo fallito lascia lo store identico.
- Deployment devnet verificato: il binario on-chain di `8pDmtT…` è identico byte per byte alla
  build locale del sorgente attuale (SHA-256 `47aa178a…`, 51 960 byte); upgrade authority
  `VUpwi769…` come nel registro. Le quattro tesorerie hanno ≥ 0,001 SOL (rent-exempt).
- Le tesorerie in `TREASURIES` sono **segnaposto**: vanno sostituite prima di mainnet (§17).
- Coverage: il programma gira in SBF su Mollusk, dove llvm-cov non misura; al suo posto vale
  il mutation testing (0 sopravvissuti non classificati). `bernie-state`: 95,7 % righe
  (`state.rs` 99,1 %), 100 % funzioni; righe scoperte dai soli test unitari: nomi degli errori, `create` con prezzo
  0, overflow u64 del costo di mint (esercitati da stream e programma).

## 10. Risultati EVM

- Differential e fuzzing: §4, §5. Rientri, pagamenti esatti a contratti senza `receive`,
  cloni dell'implementazione, sale legato al mittente: §7.
- Deployment testnet Robinhood verificato: factory `0x39e1…F305` e implementazione
  `0xd73e…e449` hanno bytecode identico alla build locale (metadata compresi) a meno degli
  immutabili; gli immutabili sono l'implementazione attesa e la tesoreria `0xC72B…778c`
  del registro; `treasury()` restituisce lo stesso indirizzo.
- Coverage `src/`: 100 % righe, statement, branch e funzioni (`forge coverage --ir-minimum`).

## 11. Limiti noti

- **Mollusk non applica la regola di rent del runtime**: l'harness la riproduce (T-2). Altre
  regole a livello di transazione (fee, compute budget per transazione, lock degli account)
  non sono modellate; i test su CU restano quelli della suite esistente.
- **Coverage del programma SBF** non misurabile con llvm-cov (vedi §9).
- **Kani `mint_canonical_or_error`** richiede 25 minuti in CI (2 191 s in locale) su un timeout di 45: margine da tenere d'occhio se la matematica cresce.
- **Domini numerici**: il modello dell'Appendice A usa interi illimitati; il primo errore con i
  domini finiti (u64/u128 su Solana, uint256 su EVM) lo decide `tools/redteam/ordered.py`
  seguendo §7, con un controllo incrociato contro il modello su ogni errore non di dominio.
- **Cloni EVM**: chiunque può clonare l'implementazione con parametri non validati; la tesoreria
  resta fissa, ma i token vanno riconosciuti tramite il registro della factory (O-2).
- **Mainnet**: nessun deployment di produzione; le tesorerie Solana sono segnaposto.

## 12. Bug trovati

**Protocollo: nessuno.** Nessuna divergenza tra implementazioni e oracolo, nessuna invariante
violata, nessun movimento di asset non autorizzato. I difetti trovati erano negli strumenti
di test; tutti corretti, ciascuno con un test di regressione dove ha senso.

| ID | Severità | Componente | Descrizione | Riproduzione | Causa | Fix | Regressione | Stato |
|---|---|---|---|---|---|---|---|---|
| T-1 | tooling | oracolo + harness Foundry | un mint o donate EVM con `msg.value` oltre il saldo portava l'oracolo a un saldo negativo invece di un errore | `minimize.py evm --seed 31337 --ops 200 --users 4` su un mutante | il caso "il wallet non può inviare la transazione" non era modellato; il generatore non lo produce, la minimizzazione sì | `InsufficientNative` nell'oracolo e nell'harness | `tests/test_redteam.py::test_evm_value_above_balance` | chiuso |
| T-2 | tooling | harness Mollusk | un mint che lascia il pagatore rent-paying passava su Mollusk, l'oracolo si aspettava `InsufficientNative` | `gen.py sol --seed 1413 --seqs 1 --ops 1000` e marathon seed 777 | Mollusk non applica la regola di transizione del rent (Agave); il runtime reale rifiuta la transazione | l'harness applica la regola e ripristina lo store | `differential.rs::payer_cannot_end_rent_paying` (confini 891 883 / 891 884 dall'oracolo) | chiuso |
| T-3 | tooling | campagne Foundry | la campagna 1 000 × 500 falliva con `EvmError: Revert` | `forge test --match-contract DifferentialTest` su 5 × 10⁵ passi | limite di gas di default per test (2³⁰) | `--gas-limit` in `redteam.yml` e nella documentazione | campagna 5 × 10⁵ passi verde | chiuso |
| T-4 | tooling | generatore | importi oltre u64 potevano arrivare al consumatore Solana | generatore prima della correzione | nessun limite al dominio u64 | importi limitati a u64; il consumatore rifiuta (non tronca) | `u64v` in `differential.rs` | chiuso |
| T-5 | tooling | oracolo | redeem con u = 0 e nessuna delega: errore diverso dal programma | sequenze Solana con revoke | Token-2022 azzera il delegato a quantità 0; non era modellato | `has_delegate` nell'oracolo | differential Solana | chiuso |
| T-6 | tooling | oracolo | costo di mint oltre u64 su pool piccoli (salto di k): Slippage nel modello, Overflow on-chain | stream SCALE 10⁹ | dominio illimitato del modello | prima insiemi `Overflow\|Slippage`; dopo Q-1 valutatore d'ordine per chain (`ordered.py`), errore unico | `boundary.py --check`, stream in CI, `error_precedence_follows_spec` | chiuso |
| T-7 | tooling | report | O-4 del report affermava che Token-2022 lascia i byte del vecchio delegato dopo revoke | rilancio del mutante `token.rs:73` dopo Q-1 (sopravvissuto) e lettura dell'account su Mollusk | ipotesi non verificata | O-4 e motivazione dell'equivalente corrette | lettura dei byte del token account dopo revoke e dopo uso completo | chiuso |

### Osservazioni (nessuna azione sul codice)

- **O-1 (economica)**: con S molto piccolo la penalità d'uscita divisa su pochi token fa saltare
  k, e il costo di ingresso cresce in modo quadratico; `max_cost` protegge l'utente (§6).
- **O-2 (informativa, EVM)**: cloni arbitrari dell'implementazione possono usare parametri non
  validati; la tesoreria resta quella dell'implementazione. Il frontend e gli integratori devono
  riconoscere i token tramite `BernieFactory.tokens`.
- **O-3**: il controllo che il firmatario del redeem sia il proprietario del token account è
  l'unica difesa contro il riscatto forzato di chi ha una delega attiva (coperto da test e mutanti).
- **O-4**: quando il delegato torna None (revoke o delega usata tutta) Token-2022 azzera tag,
  chiave e quantità (verificato su Mollusk). La lettura del tag del `COption` nel programma è
  quindi difensiva; il mutante che la toglie è equivalente (`token.rs:73`).
- **O-5**: la rent-exemption del pagatore è garantita dal runtime, non dal programma (T-2).
- **O-6**: `forge lint` segnala missing-zero-check, calls-loop e reentrancy-events: scelte
  documentate, nessun effetto sul comportamento.

### Domande sulla specifica

- **Q-1 — precedenza degli errori. Chiusa.** Decisione: si scrive in §7 l'ordine applicato oggi
  dalle implementazioni, senza cambiare il codice. L'ordine coincide sulle due chain per create
  (Price → Penalty → Metadata; la numerazione dei codici non è l'ordine), redeem e donate. Nel
  mint diverge solo la posizione di `InvariantViolated` rispetto a `Slippage`; `InvariantViolated`
  è stato messo fuori dall'ordine perché segnala un bug e non si raggiunge da stati validi.
  `Overflow` scatta nel punto del calcolo, con domini per chain. L'Appendice A verifica il saldo
  token del redeem (e il saldo nativo del donate) dopo il calcolo, come le implementazioni.
  Oracolo, generatori e consumatori sono esatti: un solo errore per passo. Nuovi test:
  `error_precedence_follows_spec` (Mollusk e Foundry). Verifica con l'oracolo esatto:
  vettori di confine (2 058 + 978, 243 casi passati da insieme a errore unico), stream
  4 × 10⁵ passi, Mollusk 3 × 10⁵ (marathon 100k compresa), Foundry 10⁵, tutti identici.

## Gate finale

| Criterio GREEN | Stato |
|---|---|
| baseline verde | sì |
| invarianti verdi | sì (Kani, Halmos, guards, differential) |
| differential testing verde | sì, 10⁸ passi di stato + programma e contratti reali |
| adversarial testing verde | sì |
| mutation testing senza mutanti realistici sopravvissuti | sì (0 SURVIVED non classificati) |
| nessun bug critico/high aperto | sì (nessun bug di protocollo) |
| failure atomicity verificata | sì (ogni passo fallito di ogni campagna, più i test avversariali) |
| deployment configuration verificata | **parziale**: devnet e testnet verificati; tesorerie Solana di produzione ancora segnaposto |

Nessuna condizione RED (nessuna violazione di invarianti, divergenza, movimento non autorizzato,
divergenza supply/backing, mutante critico sopravvissuto, corruzione di stato, doppio prelievo,
furto). Q-1 è chiusa. Il risultato è **YELLOW** per due motivi:

1. configurazione di produzione non verificabile: le tesorerie Solana sono segnaposto e non
   esiste un deployment mainnet;
2. componenti non misurabili con la coverage (programma SBF), compensati dal mutation testing.

Per GREEN: tesorerie reali in `TREASURIES`, stessa verifica binario/immutabili fatta qui sul
deployment di produzione.
