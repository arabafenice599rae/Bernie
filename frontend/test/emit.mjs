// Esegue i costruttori di istruzioni del frontend (ixCreate, ixMint, ixApprove, ixRedeem,
// ixDonate, ixSweep, ixAta, withBudget) su chiavi fisse e scrive le transazioni risultanti
// in fixtures.json. Il test Mollusk solana/program/tests/frontend.rs le esegue contro il
// programma: così ordine degli account, dati e budget di calcolo del frontend sono legati
// al programma reale, non verificati a vista.
//
// Uso: node emit.mjs          (rigenera fixtures.json)
//      node emit.mjs --check  (fallisce se fixtures.json non è aggiornato)
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import * as W from '@solana/web3.js';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '..', '..');
const html = readFileSync(join(root, 'frontend', 'index.html'), 'utf8');

function region(name) {
  const m = html.match(new RegExp(`/\\* ${name}-BEGIN \\*/([\\s\\S]*?)/\\* ${name}-END \\*/`));
  if (!m) throw new Error(`marcatore ${name} non trovato in index.html`);
  return m[1];
}

// Program ID e tesorerie dal programma, così il test usa gli stessi indirizzi.
const lib = readFileSync(join(root, 'solana', 'program', 'src', 'lib.rs'), 'utf8');
const programB58 = lib.match(/declare_id!\("(\w+)"\)/)[1];
const treasuryB58 = [...lib.matchAll(/from_str_const\("(\w+)"\)/g)].map(m => m[1]);

const programId = new W.PublicKey(programB58);
const treasuries = [new W.PublicKey(treasuryB58[0])];   // una sola: pickTreasury diventa deterministico
const cfg = { priority: 1000 };
const code = [region('CONST'), region('ENC'), region('IX')].join('\n');
const api = new Function('W', 'programId', 'treasuries', 'cfg',
  `'use strict';\n${code}\nreturn { PRESETS, TAG, ERR_CODES, CU_LIMIT, ixCreate, ixMint, ixApprove, ixRedeem, ixDonate, ixSweep, ixAta, withBudget, vaultPda, ataOf };`,
)(W, programId, treasuries, cfg);

// Il preset Devnet deve puntare al programma e alle tesorerie compilate in lib.rs.
if (api.PRESETS.devnet.program !== programB58) throw new Error(`PRESETS.devnet.program ${api.PRESETS.devnet.program} ≠ declare_id ${programB58}`);
if (api.PRESETS.devnet.treasury !== treasuryB58.join(',')) throw new Error('PRESETS.devnet.treasury diverso da TREASURIES in lib.rs');
// I preset devono coincidere con deployments.json, il registro dei deploy pubblici.
const deps = JSON.parse(readFileSync(join(root, 'deployments.json'), 'utf8'));
const sd = deps['solana-devnet'], rt = deps['robinhood-testnet'], pt = api.PRESETS.testnet;
if (sd.program !== programB58 || sd.rpc !== api.PRESETS.devnet.rpc) throw new Error('deployments.json solana-devnet diverso da lib.rs o dal preset');
if (pt.factory !== rt.factory || pt.chainId !== rt.chain_id || pt.fromBlock !== rt.factory_block || pt.evmRpc !== rt.rpc || pt.explorer !== rt.explorer)
  throw new Error('PRESETS.testnet diverso da deployments.json robinhood-testnet');

const key = n => new W.PublicKey(new Uint8Array(32).fill(n));
const creator = key(1), user = key(2), mint = key(3), whale = key(4);
const SOL = 1_000_000_000n;

const txs = {
  create: api.withBudget([api.ixCreate(creator, mint, SOL, 200, 100, 'Bernie Test', 'BRN', 'https://example.invalid/b.json')]),
  // Primo ingresso, grande: ATA creato più aritmetica su numeri grandi (caso peggiore di CU).
  mint_big: api.withBudget([api.ixAta(whale, whale, mint), api.ixMint(whale, mint, 400_000_000n * SOL, 500_000_000n * SOL)]),
  mint: api.withBudget([api.ixAta(user, user, mint), api.ixMint(user, mint, 2n * SOL, 10n * SOL)]),
  mint_again: api.withBudget([api.ixAta(user, user, mint), api.ixMint(user, mint, SOL, 10n * SOL)]),
  redeem: api.withBudget([api.ixApprove(user, mint, SOL, 9), api.ixRedeem(user, mint, SOL, 1n)]),
  donate: api.withBudget([api.ixDonate(user, mint, 10_000_000n)]),
  sweep: api.withBudget([api.ixSweep(creator.toBase58(), mint)]),
};

const hex = b => Buffer.from(b).toString('hex');
const out = {
  generator: 'frontend/test/emit.mjs',
  program: programB58,
  keys: { creator: creator.toBase58(), user: user.toBase58(), whale: whale.toBase58(), mint: mint.toBase58(), treasury: treasuryB58[0] },
  vault: api.vaultPda(mint.toBase58()).toBase58(),
  ata: api.ataOf(user, mint.toBase58()).toBase58(),
  err_codes: api.ERR_CODES,
  order: Object.keys(txs),
  txs: Object.fromEntries(Object.entries(txs).map(([name, ixs]) => [name, ixs.map(ix => ({
    program: ix.programId.toBase58(),
    keys: ix.keys.map(k => ({ pubkey: k.pubkey.toBase58(), signer: k.isSigner, writable: k.isWritable })),
    data: hex(ix.data),
  }))])),
};
const text = JSON.stringify(out, null, 1) + '\n';
const path = join(here, 'fixtures.json');
if (process.argv.includes('--check')) {
  if (readFileSync(path, 'utf8') !== text) {
    console.error('fixtures.json non aggiornato: eseguire node frontend/test/emit.mjs');
    process.exit(1);
  }
  console.log('fixtures.json aggiornato');
} else {
  writeFileSync(path, text);
  console.log(`${path}: ${Object.keys(txs).length} transazioni`);
}
