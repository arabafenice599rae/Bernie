# Bernie — Specifica v1.5 (candidate freeze)

## Cosa cambia rispetto alla v1.4

- **Nome.** Il protocollo si chiama **Bernie** (prima Residual Peg). Il termine "peg" sparisce anche dall'interfaccia, perché suggeriva un valore stabile.
- **Interfaccia on-chain definita e normativa.** Codifica delle istruzioni Solana, layout del vault e lettura dei metadati Token-2022. ABI EVM completa con selettori, eventi e ordine di emissione.
- **Vincoli di design e principio di neutralità.** Sono requisiti normativi, sia per il software sia per la comunicazione.
- **Nuovo errore 16, `MetadataTooLong`.** Limiti di nome, simbolo e URI validati on-chain.
- **Tesoreria Solana come lista costante di N indirizzi.** Il client ne sceglie uno per operazione, per distribuire i write lock.
- **Nuove sezioni:**
  - frontend;
  - contesto regolamentare aggiornato a settembre 2026;
  - Appendice B con selettori e topic EVM.

## 1. Panoramica

Bernie crea token sempre interamente coperti nell'asset nativo e riscattabili in ogni momento. Ogni riscatto trattiene una penalità fissa che alza il valore unitario k di chi resta; una penalità d'ingresso opzionale fa lo stesso a ogni mint. La disciplina è quella di SolPeg: nessun admin, nessun upgrade, k monotono non decrescente, contabilità esatta senza perdere lamport o wei.

Le implementazioni sono due: **Solana** (Pinocchio + Token-2022) e **Robinhood Chain** (Solidity + OpenZeppelin 5.x). Un unico modello di riferimento in Python (Appendice A) genera i vettori di test, che programmi e frontend devono riprodurre bit per bit.

Decisioni prese:

- Penalità in uscita `penalty_bps`: fissa, scelta alla creazione, immutabile.
- Penalità in ingresso `entry_bps`: scelta dal creator con 0 ≤ e ≤ p, immutabile, distribuita solo agli holder già presenti.
- k ad alta precisione tramite `SCALE` (10⁹ su Solana, 10¹⁸ su EVM).
- Absorb immediato nella stessa istruzione: a fine istruzione vale `residual < S`, oppure `S == 0`.
- La penalità dell'ultimo uscente (S → 0) diventa excess e va al creator.
- Fee dello 0,2% al creator e dello 0,2% al protocollo, calcolate solo sul backing, in ingresso e in uscita, con un solo arrotondamento sulla fee totale.
- Backing solo nell'asset nativo (SOL, ETH).
- Metadati on-chain immutabili.
- La cattura della penalità è accettata come limite documentato, con le garanzie P6a–P6d.
- Neutralità: il protocollo e l'interfaccia non prevedono né promettono eventi che non possono determinare (sezione 10).

Escluso per scelta: penalità dinamica (richiede un oracolo), penalità progressiva (aggirabile spezzando il redeem), vault lockato, distribuzione ritardata della penalità (rompe I6), tetto alla supply.

## 2. Notazione e unità

Tutta la contabilità interna è in **sotto-unità**: 1 unità nativa (lamport o wei) vale `SCALE` sotto-unità.

| Simbolo | Significato | Solana | EVM |
|---|---|---|---|
| `SCALE` | sotto-unità per unità nativa | 10⁹ | 10¹⁸ |
| `d` | decimali del token (fissi) | 9 | 18 |
| `P` | prezzo iniziale per token intero | lamport | wei |
| `k` | valore di 1 unità base | sotto-lamport | sotto-wei |
| `S` | supply in unità base | u64 | uint256 |
| `R` | `reserve`, backing contabilizzato | u128 | uint256 |
| `Q` | `residual`, quota in attesa di absorb | u128 | uint256 |
| `p` | `penalty_bps`, penalità in uscita | u16 | uint16 |
| `e` | `entry_bps`, penalità in ingresso | u16 | uint16 |
| `τ` | turnover giornaliero = (volume in ingresso + volume in uscita) / TVL | — | — |

- **Prezzo iniziale:** `k₀ = P · SCALE / 10^d`. Con `SCALE = 10^d` vale `k₀ = P`. Numericamente, quindi, k coincide sempre con il prezzo di 1 token intero in unità native.
- **Precisione:** k cresce a passi di 1 sotto-unità per unità base (passo relativo `1/k`), sotto 10⁻⁶ se `P ≥ MIN_PRICE`.
- **Overflow:** `R` corrisponde al saldo reale del vault moltiplicato per `SCALE`, quindi resta sotto ~6·10²⁶ su Solana (u128) e sotto ~10⁵⁴ su EVM (uint256). Poiché `u·k ≤ R`, nessun prodotto intermedio supera `R`, tranne `full · p`, che aggiunge al massimo 4 cifre.

## 3. Stato

Il vault ha quattro valori mutabili (`k`, `R`, `Q`, `S`); tutto il resto è fissato alla creazione.

### Solana: account vault v4 (128 byte)

PDA `[b"vault", mint]`, owner il programma.

| Offset | Campo | Tipo | Note |
|---|---|---|---|
| 0 | `discriminator` | u8 | `0x52` |
| 1 | `version` | u8 | 4 |
| 2 | `bump` | u8 | bump del PDA |
| 3 | `_pad0` | u8 | 0 |
| 4 | `penalty_bps` | u16 | immutabile |
| 6 | `entry_bps` | u16 | immutabile, ≤ `penalty_bps` |
| 8 | `creator` | [u8; 32] | immutabile |
| 40 | `mint` | [u8; 32] | immutabile |
| 72 | `k` | u128 | sotto-lamport per unità base |
| 88 | `reserve` | u128 | sotto-lamport |
| 104 | `residual` | u128 | sotto-lamport |
| 120 | `supply` | u64 | deve coincidere con la supply del mint |

Tutti gli interi sono little-endian.

**Mint:** Token-2022 con 9 decimali, mint authority = PDA del vault, freeze authority nulla. Estensioni ammesse: solo MetadataPointer (che punta al mint stesso) e TokenMetadata, con update authority nulla.

### EVM: clone ERC-20

Argomenti immutabili, letti con `Clones.fetchCloneArgs`: `creator` (address), `penaltyBps` (uint16), `entryBps` (uint16), `k0` (uint256), `name` (string), `symbol` (string).

Storage:

| Campo | Tipo | Note |
|---|---|---|
| `kGrowth` | uint256 | `k = k0 + kGrowth`; parte da 0, quindi niente initializer |
| `reserve` | uint256 | sotto-wei |
| `residual` | uint256 | sotto-wei |
| `feesOwed` | mapping(address ⇒ uint256) | fee in pull, in wei |
| `totalFeesOwed` | uint256 | somma di `feesOwed` |

`S` è `totalSupply()` dell'ERC-20 di OZ; `decimals()` = 18. La tesoreria è una costante dell'implementazione.

**Excess**, per differenza su entrambe le chain: `saldo − (R + Q)/SCALE − fee dovute − rent`. Non tocca mai k.

## 4. Invarianti

Asseriti on-chain alla fine di ogni istruzione che modifica lo stato. Se uno fallisce, la transazione viene annullata.

| ID | Invariante | Cosa impedisce |
|---|---|---|
| I1 | `R == S · k` | backing disallineato dalla supply |
| I2 | `saldo ≥ (R + Q)/SCALE + fee dovute + rent` | vault insolvente |
| I3 | `k_dopo ≥ k_prima` | k che scende |
| I4 | `S == 0 ⇒ Q == 0` | residuo catturabile dal primo nuovo entrante |
| I5 | `(R + Q) % SCALE == 0` | contabilità scalata non convertibile in unità intere |
| I6 | `S == 0 ∨ Q < S` | penalità ferme tra un'istruzione e l'altra |

I6 rende lo stato **canonico**. Su Solana si aggiunge il controllo `vault.supply == mint.supply` dopo ogni CPI di mint o burn.

## 5. Aritmetica

Ogni arrotondamento favorisce chi resta nel vault, mai chi opera. Ogni resto finisce in `Q` oppure, quando S == 0, diventa excess: nessun resto esce dalla contabilità.

### Absorb

```
absorb(s):
    if s.S == 0: return
    δ   = s.Q / s.S            // floor
    s.k = s.k + δ
    s.R = s.R + δ · s.S
    s.Q = s.Q − δ · s.S        // ora Q < S  (I6)
```

### Fee con arrotondamento unico

```
fees(base):
    ft = floor(base · (FEE_C + FEE_P) / 10_000)    // un solo arrotondamento
    fp = floor(ft · FEE_P / (FEE_C + FEE_P))       // quota protocollo
    fc = ft − fp                                   // quota creator
    return ft, fc, fp
```

Con tasso `(FEE_C + FEE_P)/10.000 < 1`, `ft` cresce al massimo di 1 per ogni unità di `base`. Quindi `out = g − ft` ha incrementi di 0 o 1 e non scende mai (P7).

### Conversioni

| Grandezza | Formula | Arrotondamento | Resto |
|---|---|---|---|
| costo del mint `c` | `⌈(full + epen) / SCALE⌉` | per eccesso | `c·SCALE − full − epen` in Q |
| base fee del mint `b` | `⌈full / SCALE⌉` | per eccesso | — |
| penalità `pen` | `⌈full · p / 10 000⌉` | per eccesso | tutta in Q |
| lordo del redeem `g` | `⌊(full − pen) / SCALE⌋` | per difetto | `full − pen − g·SCALE` in Q |
| fee totale `ft` | `fees(b)` nel mint, `fees(g)` nel redeem | per difetto, una volta | resta all'utente |

`full = u · k`. Le fee non passano mai per `R` o `Q`, quindi I5 resta vero per costruzione.

## 6. Operazioni

Solana ha cinque istruzioni: create, mint, redeem, donate e close. EVM ha le stesse, con `sweep` al posto di `close` e in più `claimFees`. Per I6 basta l'absorb finale. La transazione è atomica: un errore annulla ogni modifica.

### create(P, p, e, metadati)

```
require MIN_PRICE ≤ P ≤ MAX_PRICE
require PEN_MIN ≤ p ≤ PEN_MAX
require 0 ≤ e ≤ p
require 1 ≤ len(nome) ≤ 32, 1 ≤ len(simbolo) ≤ 10, len(uri) ≤ 200   // byte UTF-8
k = P ; R = 0 ; Q = 0 ; S = 0
```

### mint(u, max_cost)

```
require u > 0
epen = (S == 0) ? 0 : ceil(u · k · e / 10_000)   // sul k corrente
Q += epen
absorb()                                      // S precedente: solo gli holder esistenti
full = u · k                                  // k già aggiornato
c    = ceil((full + epen) / SCALE)
ft, fc, fp = fees(ceil(full / SCALE))         // fee sul solo backing
require c + ft ≤ max_cost                     // slippage
Q += c·SCALE − full − epen                    // solo il resto di arrotondamento
S += u ; R += full
absorb()
assert I1..I6
// effetti: utente → vault c, fc → creator, fp → protocollo, mint di u all'utente
```

**Semantica del prezzo d'ingresso.** La sequenza è `k_old → epen → absorb → k' → full = u·k'`. L'unica penalità economica è `epen`. Il termine `u·(k' − k_old)` è backing riscattabile del nuovo entrante, necessario per mantenere I1.

### redeem(u, min_out)

```
require 0 < u ≤ S
full = u · k
pen  = ceil(full · p / 10_000)
require full > pen                           // Dust
g    = floor((full − pen) / SCALE)
ft, fc, fp = fees(g)
out  = g − ft
require out > 0                              // ZeroPayout
require out ≥ min_out                        // slippage
S −= u ; R −= full
if S == 0: Q = 0                             // penalità + resti → excess
else:      Q += full − g·SCALE ; absorb()
assert I1..I6
// effetti: burn di u, vault → utente out, fc → creator, fp → protocollo
```

### donate(a)

```
require a > 0
require S > 0                                // NoHolders
Q += a · SCALE
absorb()
assert I1..I6
```

### close (solo Solana)

```
require S == 0                               // per I4 anche Q == 0 e R == 0
// tutto il saldo del vault, rent compreso, va al creator; account chiuso
```

### sweep (solo EVM, chiamabile da chiunque)

```
excess = balance − (R + Q)/SCALE − totalFeesOwed
require excess > 0
// excess → creator; sicuro con qualsiasi S
```

### claimFees (solo EVM)

```
amt = feesOwed[caller] ; require amt > 0
feesOwed[caller] = 0 ; totalFeesOwed −= amt
// sendValue(caller, amt)
```

## 7. Casi limite ed errori

| Codice | Errore | Quando |
|---|---|---|
| 1 | `ZeroAmount` | `u == 0` o `a == 0` |
| 2 | `ExceedsSupply` | `u > S` |
| 3 | `Dust` | `full ≤ pen` |
| 4 | `ZeroPayout` | `out == 0` dopo le fee |
| 5 | `Slippage` | costo oltre `max_cost` o uscita sotto `min_out` |
| 6 | `NoHolders` | `donate` con `S == 0` |
| 7 | `NotEmpty` | `close` con `S > 0` |
| 8 | `PenaltyOutOfRange` | `p` fuori da `[PEN_MIN, PEN_MAX]`, oppure `e > p` |
| 9 | `PriceOutOfRange` | `P` fuori da `[MIN_PRICE, MAX_PRICE]` |
| 10 | `Overflow` | un'operazione checked fallisce |
| 11 | `InvariantViolated` | un'asserzione I1–I6 fallisce: segnala un bug |
| 12 | `NothingToClaim` | `claimFees` o `sweep` senza importo (solo EVM) |
| 13 | `TransferToSelf` | trasferimento ERC-20 verso il contratto stesso (solo EVM) |
| 14 | `SupplyMismatch` | `vault.supply ≠ mint.supply` (solo Solana) |
| 15 | `MissingDelegation` | delega al PDA assente o inferiore a `u` nel redeem (solo Solana) |
| 16 | `MetadataTooLong` | nome, simbolo o URI fuori dai limiti (nuovo) |

Casi limite con comportamento definito:

- **Ultimo uscente.** Paga la penalità come tutti. Penalità e resti diventano excess e vanno sempre al creator, chiunque sia l'ultimo holder.
- **Penalità senza altri holder.** È redistributiva solo se esistono altri holder; altrimenti diventa excess del creator.
- **Rientro dopo lo svuotamento.** k resta al valore raggiunto (I3). Il nuovo primo entrante non cattura nulla, perché I4 ha azzerato Q.
- **Redeem minuscoli.** Falliscono con `Dust` o `ZeroPayout`.
- **ETH o lamport inviati direttamente.** Diventano excess; su EVM non c'è `receive()`.
- **CPI Guard.** Il redeem su Solana usa sempre il burn come delegato, quindi funziona con o senza CPI Guard (sezione 11).

## 8. Proprietà economiche

### Teorema di canonicità

Da ogni stato che soddisfa I1–I6, ogni transizione riuscita produce un unico stato valido e canonico: `Q < S` se `S > 0`; `Q == 0 ∧ R == 0` se `S == 0`.

### Conservazione

- I1, I2, I5.
- **P3, conservazione globale.** Somma dei wallet + fee del creator + fee del protocollo + saldo del vault = costante, al lamport o al wei.
- **P3b, contabilità dei resti.** Ogni resto va in Q oppure, quando `S == 0`, diventa excess.

### Monotonicità

- I3: k non scende mai.
- I4 e I6: stato vuoto e stato non vuoto canonici.
- **P7, monotonia.** `out` è non decrescente in `g`, con incrementi di 0 o 1 per unità. Il costo del mint è non decrescente in `u`.

### Economia

- **P1, round-trip in perdita.** Senza flussi da terzi tra la prima e l'ultima operazione, qualunque sequenza di mint e redeem dello stesso attore restituisce meno del capitale versato.
- **P2, holder passivo.** Chi non opera non vede mai scendere il valore di backing della propria posizione. `k_out ≥ k_in` non significa che ogni ciclo sia in guadagno.
- **P4, penalità nominale non frazionabile.** `⌈a⌉ + ⌈b⌉ ≥ ⌈a + b⌉`. Riguarda solo le penalità nominali, non il costo effettivo.
- **P5, nessun auto-rimborso d'ingresso.** Il nuovo entrante non riceve alcuna parte della propria `epen`.
- **P6, cattura limitata.** Un attaccante che minta `A` prima del redeem `V` di una vittima può appropriarsi di parte della penalità:
    - **P6a, vittima indenne.** A parità di stato iniziale, stesso `V` e nessun'altra operazione, vale `out_V(con attaccante) ≥ out_V(senza attaccante)`, esattamente.
    - **P6b, nessuna perdita di valore di backing.** Per ogni holder passivo con H unità vale `H·k_dopo ≥ H·k_prima`. Il minore guadagno rispetto allo scenario senza attacco è la diluizione accettata.
    - **P6c, cattura limitata alla penalità.** Il guadagno dell'attaccante non supera mai la penalità della vittima.
    - **P6d, soglia.** Con `O` = holder diversi da vittima e attaccanti, e senza flussi in ingresso tra l'entrata dell'attaccante e il redeem della vittima: se `p · V ≤ (e + FEE_C + FEE_P) · O`, il profitto mark-to-k dell'attaccante è ≤ ε = 1 unità nativa. Con donazioni intermedie l'attaccante ne cattura una quota: è la classe del front-running delle donazioni, già accettata in SolPeg.

**Metrica.** Profitto mark-to-k = (unità detenute × k finale + nativo ricevuto) − nativo versato. È la metrica più conservativa: qualunque uscita, anche frazionata, realizza al massimo questo valore.

**Perché la cattura esiste.** La penalità della vittima si divide su `O + A` unità:

```
guadagno_attaccante = pen_V · A / (O + A)
costo_attaccante    ≥ (e + FEE_C + FEE_P) · A · k
```

Il guadagno non cresce linearmente con A, il costo sì. Con O piccolo nessuna `e ≤ p` può chiudere il vettore.

**Esempio verificato**: p = 500, e = 460, O = 5, V = 20, A = 5; 1 unità base = 10⁶ unità native.

| Soggetto | Prima | Dopo il redeem, senza attacco | Dopo il redeem, con attacco |
|---|---|---|---|
| Holder O (backing) | 5.000.000 | 6.000.000 | 5.550.600 |
| Vittima (incasso) | — | 18.924.000 | 19.098.102 |
| Attaccante (profitto) | — | — | +253.496 |

## 9. Guida ai parametri

**Limiti tecnici:** 100 ≤ p ≤ 1000 bps, 0 ≤ e ≤ p.

**Riferimento per il trading frequente:** p = 200 bps, e = 100 bps.

| Grandezza | Valore |
|---|---|
| Costo di un round-trip immediato | 3,74% |
| Crescita di k necessaria per il pareggio | +3,89% |
| Protezione dalla cattura, v* | 41% |

- **Crescita di k:** circa `(e + p) · τ / 2` al giorno, **solo se** il turnover τ si realizza.
- **Protezione:** `v* = r / (1 + r)`, con `r = (e + FEE_C + FEE_P) / p`. Conta la ripartizione tra e e p, non solo la somma: con somma 3%, la configurazione 3% / 0% dà circa 12%, 2% / 1% dà 41%, 1,5% / 1,5% dà 56%.
- **Nessun rendimento esterno.** La crescita di k è un trasferimento da chi fa trading a chi detiene, al netto delle fee. Senza operazioni k resta fermo.
- **Calibrazione.** La grandezza rilevante è `(e + p) · τ(costo)`. L'elasticità del volume rispetto al costo non è nota: i parametri vanno scelti con dati reali.

Simulazione a turnover costante (limite superiore ottimistico):

| p | e | Round-trip | v* | k/anno, τ = 1%/g | k/anno, τ = 5%/g |
|---|---|---|---|---|---|
| 1% | 0,5% | 2,28% | 47% | +2,8% | +14,6% |
| 2% | 1% | 3,74% | 41% | +5,7% | +31% |
| 3% | 1,5% | 5,19% | 39% | +8,6% | +51% |
| 5% | 2,5% | 8,05% | 37% | +14,7% | +98% |

## 10. Vincoli di design e neutralità

Requisiti normativi per programmi, interfaccia e comunicazione. Descrivono il software; non sono un parere sulla qualificazione giuridica dei token.

1. **Neutralità su ciò che non può determinare.** Il protocollo esegue solo operazioni già firmate. Non prevede, stima o garantisce volume futuro, ingressi, uscite, donazioni, prezzi di SOL o ETH, mercati esterni. Nessun oracolo. k cambia solo dopo operazioni avvenute.
2. **Nessuna promessa di rendimento.** Nessun tasso di crescita di k è promesso, né on-chain né nell'interfaccia. I dati storici sono etichettati come tali.
3. **Nessuno sforzo gestionale da cui dipenda il valore.** P, p, e, fee e formula sono immutabili. Niente admin, upgrade, pause o parametri modificabili.
4. **Nessuna allocazione privilegiata.** Nessun pre-mint, whitelist, riserva o accesso anticipato; il creator entra alle stesse condizioni di chiunque.
5. **Nessun bisogno di compratori.** Il riscatto avviene contro il vault, anche per l'ultimo holder. Nessuna dipendenza da mercati secondari.
6. **Nessuna componente aleatoria.**
7. **Custodia vincolata dal codice.** I fondi escono solo via redeem, fee ed excess.
8. **Flussi dichiarati.** Fee fisse e indipendenti dal risultato di chi opera; penalità agli holder presenti; excess al creator.
9. **Limite di cattura dichiarato**, con la soglia v* mostrata per ogni token.
10. **Preventivi.** Calcolati sullo stato attuale; la tolleranza di prezzo limita la differenza.
11. **Interfaccia.** Legge dati pubblici e prepara transazioni firmate dall'utente; non custodisce fondi; non è un'offerta.

**Regole di comunicazione:**

- Descrivere solo funzionalità presenti.
- Mai presentare la redistribuzione delle penalità come rendimento.
- Mai usare "fair launch", "garantito", "stabile", "peg" o formule equivalenti.
- Tenere il marchio separato da Bernie List, che ha componenti di gioco escluse qui.

## 11. Interfaccia on-chain

### Solana

Il byte 0 dei dati è il tag dell'istruzione; gli interi sono little-endian; le stringhe sono codificate come lunghezza u8 seguita dai byte UTF-8.

| Tag | Istruzione | Dati | Account (s = firmatario, w = scrivibile) |
|---|---|---|---|
| 0 | create | P u64, p u16, e u16, nome, simbolo, URI | creator (s,w), mint (s,w, keypair nuova), vault PDA (w), System, Token-2022 |
| 1 | mint | u u64, max_cost u64 | utente (s,w), ATA Token-2022 (w), mint (w), vault (w), creator (w), tesoreria (w), System, Token-2022 |
| 2 | redeem | u u64, min_out u64 | utente (s,w), ATA (w), mint (w), vault (w), creator (w), tesoreria (w), Token-2022 |
| 3 | donate | a u64 | donatore (s,w), mint, vault (w), System |
| 4 | close | — | creator (s,w), mint, vault (w) |

**Composizione delle transazioni da parte del client:**

- **mint:** creazione idempotente dell'ATA (programma ATA, dati `[1]`, con il program ID Token-2022) seguita da `mint`.
- **redeem:** `ApproveChecked` Token-2022 (dati `[13, u u64, decimali u8]`: source ATA, mint, delegato = vault PDA, owner) seguita da `redeem`. Il programma verifica che la delega sia ≥ `u` ed esegue `BurnChecked` come delegato con `invoke_signed`.
- **Budget di calcolo stimato**, da misurare: create 90k CU, mint 60k, redeem 45k, donate e close 10k.

**Tesoreria.** Lista costante di N indirizzi nel programma. Il client ne sceglie uno a caso per ogni operazione, per distribuire i write lock.

**Lettura dello stato:**

- **Vault:** `getProgramAccounts` con `dataSize = 128` e i primi due byte = `[0x52, 0x04]`.
- **Metadati:** dal TLV del mint Token-2022, a partire dall'offset 166, tipo 19 (TokenMetadata): update authority (32 byte), mint (32 byte), poi nome, simbolo e URI come stringhe con lunghezza u32.
- **Saldo del vault** per i preventivi: lamport − rent esente per 128 byte.

**Storico.** È ricostruibile rigiocando i dati delle istruzioni riuscite dal `create`, perché `P`, `u` e `a` determinano completamente le transizioni. Il programma emette anche un log `Peg(k, S, R, Q)` per gli indexer.

### Robinhood Chain (EVM)

**Factory:**

```
function count() view returns (uint256)
function tokens(uint256) view returns (address)
function create(uint256 price, uint16 penaltyBps, uint16 entryBps, string name, string symbol, bytes32 salt) returns (address)
event Created(address indexed token, address indexed creator)
```

**Token, letture:**

```
name(), symbol(), decimals(), totalSupply(), balanceOf(address),
k(), k0(), reserve(), residual(), penaltyBps(), entryBps(), creator(),
feesOwed(address), totalFeesOwed()
```

**Token, scritture:**

```
function mint(uint256 u) payable                 // msg.value ≥ costo, eccedenza rimborsata; altrimenti Slippage
function redeem(uint256 u, uint256 minOut)
function donate() payable
function claimFees()
function sweep()
```

**Eventi**, emessi in quest'ordine nella stessa transazione: prima l'evento dell'operazione, poi `Peg`.

```
event Minted(address indexed who, uint256 u, uint256 paid)
event Redeemed(address indexed who, uint256 u, uint256 out)
event Donated(address indexed who, uint256 amount)
event Peg(uint256 k, uint256 S, uint256 R, uint256 Q)
```

Selettori e topic sono nell'Appendice B.

## 12. Mappa Solana

- **Dipendenze:** `pinocchio`, `pinocchio-system`, `pinocchio-token-2022` 0.4.0. Le istruzioni dei metadati sono scritte a mano (~30 righe).
- **Moduli:**
  - `state.rs`: funzioni pure generiche su `const SCALE`;
  - `processor/*.rs`: uno per istruzione;
  - `token.rs`: CPI Token-2022;
  - `error.rs`: codici 1–16.
- **Sequenza di create:**
  1. account mint con spazio e rent per la dimensione finale;
  2. InitializeMetadataPointer (metadata address = mint, authority nulla);
  3. InitializeMint2 (9 decimali, authority = PDA, freeze nulla);
  4. TokenMetadata Initialize firmata dal PDA;
  5. UpdateAuthority → nulla;
  6. creazione del vault PDA.
- **Flussi di lamport:**
  - in ingresso: trasferimenti di sistema;
  - in uscita: modifica diretta dei lamport del vault, con controllo rent-exempt sui destinatari delle fee;
  - dopo ogni CPI si confronta `vault.supply` con la supply del mint.

## 13. Mappa EVM

- **Componenti OZ 5.x** (tag esatto fissato): `ERC20` (con override di `_update` contro i trasferimenti verso `address(this)`), `Clones` con argomenti immutabili, `Math.mulDiv`, `ReentrancyGuardTransient` (fallback `ReentrancyGuard`), `Address.sendValue`.
- **Contratti:**
  - `BernieMath`: library pura, l'unico codice che Halmos deve dimostrare;
  - `Bernie`: implementazione, niente `receive()`, `fallback()` o initializer;
  - `BernieFactory`: senza owner, salt effettivo `keccak256(msg.sender, salt)`, enumerazione con `count()` e `tokens(i)`.
- **Ordine in ogni funzione:**
  1. `nonReentrant`;
  2. calcolo puro;
  3. scrittura dello storage, mint o burn, accredito delle fee;
  4. asserzione di I1–I6;
  5. trasferimenti di ETH per ultimi.
- **Rete:** chain ID 4663, testnet 46630, ArbOS 51.

## 14. Frontend

Pagina singola sul layout di SolPeg, con lingue IT ed EN, pubblicata come artifact.

**Matematica.** È il porting in BigInt del modello dell'Appendice A. Il test differenziale ha dato 15.949 stati identici su 15.949. Il mint per importo usa una ricerca binaria, valida per P7.

**Viste:**

- **Lista token:** ordinamento neutrale (attività recente, TVL, nome), nessuna classifica per rendimento.
- **Pagina token:**
  - prezzo, crescita storica dal lancio;
  - barra che mostra il pareggio della posizione se l'utente ne ha una, altrimenti la protezione v*;
  - grafico a gradini;
  - round-trip, pareggio, turnover 24h;
  - Mint (per importo), Redeem (per percentuale), Dona, con preventivi esatti;
  - attività, pannello creator.
- **Creazione:** cursori per p ed e, preset 2% / 1%, anteprima di round-trip, pareggio e protezione.

**Riquadro "Il tuo wallet".** Visibile appena si collega un wallet, sia nella lista sia nel foglio Wallet. Mostra:

- il valore totale dei token Bernie;
- il saldo nativo disponibile;
- per ogni token: quantità, valore di backing e incasso in caso di uscita immediata.

In live mostra solo i token del protocollo.

**Neutralità nell'interfaccia:**

- foglio "Vincoli di design" (sezione 10);
- note sotto grafico, barra di pareggio e preventivi;
- dati storici etichettati come tali;
- demo dichiarata come simulata;
- avviso ai creator contro promesse di rendimento.

**Modalità:**

- **Demo** completa, con attività simulata che usa la stessa aritmetica.
- **Live Solana:** tramite RPC e wallet (Phantom, Backpack, Solflare).
- **Live Robinhood:** tramite RPC e `window.ethereum`.

Dentro claude.ai le richieste RPC dirette sono bloccate, quindi il live funziona con la pagina ospitata in proprio. Il live non è ancora testato, perché i programmi non esistono.

**Da allineare:** aggiungere il codice 16 `MetadataTooLong` alla mappa degli errori del frontend.

## 15. Librerie e strumenti

| Chain | Libreria o strumento | Ruolo | Versione |
|---|---|---|---|
| Solana | `pinocchio`, `pinocchio-system` | entrypoint, PDA, trasferimenti | da fissare |
| Solana | `pinocchio-token-2022` | InitializeMint2, MintTo, BurnChecked | 0.4.0 |
| Solana | `mollusk-svm`, `proptest`, Kani | test di istruzione, proprietà, prova formale | da fissare |
| EVM | OpenZeppelin Contracts 5.x | token-vault, cloni, arrotondamenti, lock | tag da fissare |
| EVM | Foundry, Halmos | test, invarianti, prova simbolica | da fissare |
| Entrambe | Python 3 (solo libreria standard) | modello di riferimento e vettori | — |
| Frontend | `@solana/web3.js` 1.98.4 (IIFE) | Solana live | fissata |

## 16. Risultati dei test

| Test | Casi | Esito |
|---|---|---|
| Fuzz multi-attore (I1–I6, P3, excess ≥ 0, P2) su SCALE 10, 10⁹, 10¹⁸ | 17.631 operazioni per SCALE | 0 violazioni |
| P6a contro lo scenario senza attaccante, ricerca casuale | 199.852 | 0 violazioni (53 con la formula della v1.3) |
| P6a, P6b, P6d con k grandi e piccoli, due attaccanti | 300.352 in totale | 0 violazioni; ε = 1 (k piccoli, SCALE 10), altrimenti 0 |
| Donazione intermedia | 27.648 | P6a e P6b reggono; P6d esclusa per definizione |
| P6b e P6c, griglia ampia | 144.000 per SCALE | cattura massima 96,8% della penalità |
| Uscita frazionata ≤ mark-to-k | 38.064 | 0 violazioni |
| P7, monotonia di `out(g)` | 10⁷ | 0 violazioni (20.000 con la formula della v1.3) |
| P1, P5 | 3.000 + 3.000 | 0 violazioni |
| Frontend contro modello (differenziale) | 15.949 stati | identici |

## 17. Piano di verifica

| Livello | Solana | EVM |
|---|---|---|
| Prova formale | Kani su `state.rs` | Halmos su `BernieMath` |
| Esaustivo | host con `SCALE = 10` | Foundry su domini piccoli |
| Proprietà | proptest + Mollusk | fuzz + invariant |
| Differenziale | vettori JSON condivisi | vettori JSON condivisi |

**Harness minimi:**

- `absorb` conserva `R + Q` e I1, e lascia `Q < S`;
- mint e redeem producono uno stato canonico oppure un errore;
- k non scende;
- nel mint `c·SCALE ≥ full + epen` e `b ≤ c`;
- nel redeem mai `out > g` né `g·SCALE > full − pen`;
- `ft(base+1) − ft(base) ∈ {0, 1}` e `fc + fp == ft`;
- P5.

**Prima del deploy:**

- [ ] Programmi scritti sull'interfaccia della sezione 11.
- [ ] Vettori con i casi peggiori portati in Mollusk e Foundry, compreso il redeem con CPI Guard attivo e disattivo.
- [ ] Probe TSTORE su testnet 46630 e misura delle CU su devnet.
- [ ] Indirizzi reali di tesoreria (N per Solana).
- [ ] Upgrade authority revocata su Solana; sorgenti verificati sugli explorer.
- [ ] Parere legale Italia/UE, più USA in caso di utenti americani.

## 18. Stato dell'arte

La cattura è la stessa struttura della JIT liquidity in Uniswap v3: chi esegue non perde, gli holder passivi vengono diluiti nel guadagno, e una durata minima richiederebbe stato per posizione. Il rimedio standard è lo sblocco graduale del profitto (Yearn), che qui è escluso perché introduce stato temporale e rompe I6. La specifica sceglie la semplicità deterministica e garantisce vittima indenne, nessuna perdita di valore di backing e cattura limitata alla penalità.

## 19. Contesto regolamentare (settembre 2026, non è parere legale)

- **Pump.fun (Aguilar v. Baton).** Nella sentenza del 31 agosto la commonality verticale non è stata riconosciuta: Baton incassava una fee dell'1% su ogni transazione indipendentemente dal profitto del trader. I predicati di wire fraud invece poggiano su post che descrivevano la piattaforma come fair launch e terreno di gioco alla pari, trattati come affermazioni fattuali. Per Bernie ne derivano fee fisse e indipendenti dal risultato, e regole di comunicazione rigide.
- **SEC, FAQ del 25 settembre.** Sono opinioni dello staff, senza forza di legge. Per lo staff un "receipt" non fornisce benefici finanziari aggiuntivi rispetto all'asset depositato. Un token Bernie li fornisce (la quota delle penalità altrui), quindi non può appoggiarsi alla categoria dei receipt o dei wrapped token. Nella stessa FAQ un buyback presentato come rendimento per gli holder può diventare una promessa di sforzi gestionali: l'analogo per Bernie è la redistribuzione delle penalità, da non presentare mai come rendimento.
- **Regulation Crypto Assets.** È una proposta, con commenti aperti fino al 20 ottobre 2026. Prevede un safe harbor condizionato, disponibile quando l'emittente ha completato o cessato in modo permanente gli sforzi gestionali promessi.
- **Money transmission.** Il 15 settembre il Senato ha respinto la cloture sul CLARITY Act 49 a 50. La protezione per gli sviluppatori che non controllano gli asset degli utenti resta un disegno di legge.
- **UE (MiCA).** L'esclusione del Recital 22 richiede che nessun soggetto controlli parametri, governance o infrastruttura e che gli utenti accedano a una risorsa comune. Da valutare: frontend, tesoreria e fee del creator. Nella tassonomia di riferimento gli ART pretendono di mantenere un valore stabile, mentre i token con riscatto non pensati per stabilizzarsi sono distinti. La classificazione può però variare tra Stati membri.

## 20. Costanti e punti aperti

| Costante | Solana | EVM |
|---|---|---|
| `SCALE` | 10⁹ | 10¹⁸ |
| decimali | 9 | 18 |
| `FEE_C` / `FEE_P` | 20 / 20 bps sul backing, arrotondamento unico | uguale |
| `PEN_MIN` / `PEN_MAX` | 100 / 1000 bps | 100 / 1000 bps |
| e | 0 ≤ e ≤ p | 0 ≤ e ≤ p |
| riferimento per il trading frequente | p = 200, e = 100 | uguale |
| ε (P6d) | 1 unità nativa | 1 unità nativa |
| `MIN_PRICE` | 10⁶ lamport | 10¹² wei |
| `MAX_PRICE` | 10¹⁵ lamport | 10²⁴ wei |
| metadati | nome ≤ 32, simbolo ≤ 10, URI ≤ 200 byte | nome ≤ 32, simbolo ≤ 10 |
| tesoreria | lista costante di N indirizzi | costante |

**Da decidere:**

- asimmetria close/sweep (proposta: sweep senza chiusura anche su Solana);
- N e indirizzi delle tesorerie, da puntare a un multisig.

**Decisi:**

- nessuna fee su `donate`;
- 0 ≤ e ≤ p;
- fee sul solo backing con arrotondamento unico;
- metrica mark-to-k;
- cattura accettata con P6a–P6d;
- redeem via burn come delegato;
- interfaccia on-chain della sezione 11;
- vincoli di design della sezione 10;
- nome Bernie.

**Stato: v1.5 candidate freeze.** La matematica è chiusa e l'interfaccia è definita. Gate rimasti:

1. Scrivere i due programmi sull'interfaccia.
2. Portare i vettori in Mollusk e Foundry.
3. Ottenere il parere legale.
4. Congelare.

## Appendice A — Modello di riferimento (Python)

```python
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
```

## Appendice B — Selettori e topic EVM (keccak256)

| Funzione | Selettore |
|---|---|
| `count()` | `0x06661abd` |
| `tokens(uint256)` | `0x4f64b2be` |
| `create(uint256,uint16,uint16,string,string,bytes32)` | `0x2b24a4ab` |
| `k()` | `0xb4f40c61` |
| `k0()` | `0x93eed093` |
| `reserve()` | `0xcd3293de` |
| `residual()` | `0x69a48b71` |
| `penaltyBps()` | `0xecfad9fd` |
| `entryBps()` | `0x81566f90` |
| `creator()` | `0x02d05d3f` |
| `feesOwed(address)` | `0x13083c34` |
| `totalFeesOwed()` | `0xaa8b869a` |
| `mint(uint256)` | `0xa0712d68` |
| `redeem(uint256,uint256)` | `0x7cbc2373` |
| `donate()` | `0xed88c68e` |
| `claimFees()` | `0xd294f093` |
| `sweep()` | `0x35faa416` |

| Evento | Topic 0 |
|---|---|
| `Peg(uint256,uint256,uint256,uint256)` | `0xcdd6d3d734a1fa212e9a18dfe6b0953a5653f62bcc9f4d686f5ec64c8aa286e9` |
| `Minted(address,uint256,uint256)` | `0x25b428dfde728ccfaddad7e29e4ac23c24ed7fd1a6e3e3f91894a9a073f5dfff` |
| `Redeemed(address,uint256,uint256)` | `0xf3a670cd3af7d64b488926880889d08a8585a138ff455227af6737339a1ec262` |
| `Donated(address,uint256)` | `0x2a01595cddf097c90216094025db714da3f4e5bd8877b56ba86a24ecead8e543` |
| `Created(address,address)` | `0x587ece4cd19692c5be1a4184503d607d45542d2aca0698c0068f52e09ccb541c` |

I selettori ERC-20 standard (`name`, `symbol`, `decimals`, `totalSupply`, `balanceOf`) sono quelli canonici, verificati nel calcolo.