# Mutation testing: state

Generato da `tools/mutation/mutate.py state`. KILLED: 24

| Esito | Mutante | Rilevato da |
|---|---|---|
| KILLED | absorb: floor(Q/S) → ceil | every_transition_is_canonical_or_rejected |
| KILLED | donate: a · (SCALE − 1) | every_transition_is_canonical_or_rejected |
| KILLED | penalità d'ingresso: ceil → floor | vectors_scale_10 |
| KILLED | penalità d'ingresso anche con S = 0 | vectors_scale_1e9 |
| KILLED | fee: due arrotondamenti separati invece di uno (§5) | fees_single_rounding |
| KILLED | fee creator 0,20% → 0,21% | fees_single_rounding |
| KILLED | fee protocollo 0,20% → 0,30% | fees_single_rounding |
| KILLED | split fee: floor → ceil sulla quota protocollo | vectors_scale_10 |
| KILLED | fee totale: floor → ceil | fees_single_rounding |
| KILLED | mint: nessun absorb prima dell'ingresso (epen diluita al nuovo entrante) | vectors_scale_10 |
| KILLED | mint: c = ceil → floor | every_transition_is_canonical_or_rejected |
| KILLED | mint: base delle fee ceil → floor | vectors_scale_10 |
| KILLED | mint: fee su backing + epen invece che sul backing | vectors_scale_1e9 |
| KILLED | mint: R += full → R += full − epen | every_transition_is_canonical_or_rejected |
| KILLED | mint: resto di arrotondamento non in Q | every_transition_is_canonical_or_rejected |
| KILLED | mint: S += u → S += u − 1 | every_transition_is_canonical_or_rejected |
| KILLED | create: e > p ammesso | create_params |
| KILLED | create: MIN_PRICE escluso | create_params |
| KILLED | redeem: con S → 0 Q non azzerato | every_transition_is_canonical_or_rejected |
| KILLED | redeem: g floor → ceil | every_transition_is_canonical_or_rejected |
| KILLED | redeem: fee del creator non trattenuta | vectors_scale_10 |
| KILLED | redeem: penalità ceil → floor | vectors_scale_10 |
| KILLED | redeem: 1 sotto-unità di penalità persa | every_transition_is_canonical_or_rejected |
| KILLED | metadati: URI fino a 201 byte | metadata_limits |
