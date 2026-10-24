# Jupiter DEX aggregator on Solana — Substreams

Jupiter DEX aggregator on Solana: swaps, volume, OHLCV candles, trader and token stats, plus xStocks tokenized-stock volume, delivered as Substreams modules and a SQL sink (PostgreSQL / ClickHouse).

- Package: [`jupiter-dex-substreams` on substreams.dev](https://substreams.dev/packages/jupiter-dex-substreams/v1.0.0)
- Source: https://github.com/PaulieB14/Jupiter-Dex-Substreams
- Network: Solana mainnet
- Crates: substreams 0.8, substreams-solana 0.16, substreams-database-change 5.0, buffa 0.9.2

## What it indexes

| Program | ID | What is extracted |
|---|---|---|
| Jupiter Aggregator v6 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Every swap instruction, decoded by Anchor discriminator with the on-chain IDL account layout: `route`, `route_with_token_ledger`, `exact_out_route`, `shared_accounts_route`, `shared_accounts_route_with_token_ledger`, `shared_accounts_exact_out_route`, and the 2025 v2 family `route_v2`, `exact_out_route_v2`, `shared_accounts_route_v2`, `shared_accounts_exact_out_route_v2`. Realized amounts come from the program's `SwapEvent` / `SwapsEvent` self-CPI logs. `FeeEvent` gives the platform fee. |
| JupiterZ RFQ Order Engine | `61DFfeTKM7trxYcPQCM78bJ794ddZprZpAwAnLiwTpYH`, `2En5Y11SEAGLNmEezTuRUCwTyzyNReHaMbSnS5gjGsL1` | `fill(input_amount, output_amount, expire_at)`: taker, input/output mint and amounts (`instruction_type = rfq_fill`). |
| Jupiter Limit Order v2 | `j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X` | Instructions are tagged, with no amounts of their own. |
| Jupiter Limit Order (v1) | `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu` | Instructions are tagged, with no amounts of their own. |
| Jupiter DCA / Recurring | `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M` | Instructions are tagged, with no amounts of their own. |
| Legacy aggregators v2–v4 | `JUP2…`, `JUP3…`, `JUP4…` | Instructions are tagged, with no amounts. |

**No double counting.** DCA and limit-order fills settle through a Jupiter v6 route CPI. That v6 route is the one row that carries amounts, so the order programs themselves contribute 0 volume.

### Tokenized stocks (xStocks)

Every swap whose input or output mint is an official **xStocks** (Backed) Solana mint is tagged with the underlying ticker (`input_stock_ticker` / `output_stock_ticker`, e.g. `NVDA`). Those swaps are aggregated into `stock_daily_volume` per ticker and UTC day:

- buy and sell counts;
- raw share amounts (xStocks token base units);
- `usd_volume_raw`, in micro-USD, counted only when the other side of the trade is USDC, USDT or USDG.

The registry (`src/stocks_registry.rs`, 1,474 mints at generation time) comes only from the official xStocks public API. To regenerate it:

```bash
python3 scripts/gen_stocks.py   # rewrites src/stocks_registry.rs from https://api.xstocks.fi/api/v2/public/assets
```

xStocks on Solana are Token-2022 tokens with a Scaled-UI multiplier. Raw amounts are reported as-is; multiply by the issuer's multiplier for display shares. Ondo tokenized stocks are not included yet, because Ondo's address API requires a key.

## Modules

| Module | Kind | Inputs | Output |
|---|---|---|---|
| `map_spl_initialized_account` | map | Block (filtered to Jupiter programs) | `sf.jupiter.v1.AccountOwnerRecords` |
| `map_jupiter_trading_data` | map | Block (filtered) | `sf.jupiter.v1.TradingDataList`: one item per Jupiter instruction, with decoded swap fields |
| `map_token_prices` | map | `map_jupiter_trading_data` | `sf.jupiter.v1.TokenPriceList` |
| `map_jupiter_instructions` | map | Block, `map_spl_initialized_account`, `map_jupiter_trading_data`, `map_token_prices` | `sf.jupiter.v1.JupiterInstructions` |
| `map_jupiter_analytics` | map | `map_jupiter_instructions` | `sf.jupiter.v1.JupiterAnalytics` |
| `store_swap_volumes` | store (add, bigint) | `map_jupiter_trading_data` | volume by pair / token / day / program |
| `store_unique_traders` | store (set_if_not_exists) | `map_jupiter_trading_data` | first-seen traders |
| `store_token_stats` | store (add, bigint) | `map_jupiter_trading_data` | per-token trade counts |
| `db_out` | map | `map_jupiter_trading_data`, `map_jupiter_analytics` | `sf.substreams.sink.database.v1.DatabaseChanges` |

Block filtering uses `solana-common` `program_ids_without_votes`, limited to the Jupiter program IDs above.

### Swap fields (`TradingData`)

| Field | Meaning |
|---|---|
| `amount_in`, `amount_out` | Realized swap amounts in raw token units, before platform fees (see *Amount semantics*) |
| `input_mint`, `output_mint`, `user_wallet` | From the IDL account layout. For `route`/`route_with_token_ledger`, the source mint comes from the first swap event. |
| `instruction_type` | Decoded instruction name (`route_v2`, `rfq_fill`, …). Empty for non-swap instructions. |
| `instruction_index` | Position in the transaction's instruction walk (top-level and inner), unique per transaction |
| `quoted_amount` | Quoted counter-amount from args: `quoted_out_amount` for exact-in, `quoted_in_amount` for exact-out |
| `amount_source` | `event` (from SwapEvent/SwapsEvent) or `args` (instruction args: RFQ fills, or v6 with no events) |
| `input_stock_ticker`, `output_stock_ticker` | Underlying ticker when the mint is an xStock |
| `platform_fee_amount`, `platform_fee_mint` | From `FeeEvent`. v2 routes do not emit FeeEvent, so this is 0 for them. |

## SQL output tables

The schemas are in [`schema.sql`](schema.sql) (PostgreSQL) and [`schema.clickhouse.sql`](schema.clickhouse.sql).

| Table | Key | Contents |
|---|---|---|
| `jupiter_swaps` | `id = tx_hash:instruction_index` | One row per swap, with all fields above |
| `candles` | (pair_id, interval_seconds, timestamp) | OHLCV of the out/in price ratio at 5m, 1h, 4h and 1d (delta upserts) |
| `token_pairs` | pair_id | swap count and volumes per pair |
| `token_stats` | mint | swaps and volume as input/output |
| `trader_stats` | wallet | swaps and volume per wallet |
| `daily_stats`, `hourly_stats` | date / hour | swap count and volume |
| `program_stats` | program_id | instruction count and volume |
| `protocol_metrics` | `jupiter` | totals |
| `stock_daily_volume` | (ticker, date) | xStocks buys and sells, shares, USD volume |

## Quickstart

```bash
# Stream swaps
substreams run jupiter-dex-substreams@v1.0.0 map_jupiter_trading_data \
  -e mainnet.sol.streamingfast.io:443 -s -100

# PostgreSQL sink
substreams-sink-sql setup "psql://user:pass@localhost:5432/jupiter?sslmode=disable" jupiter-dex-substreams@v1.0.0
substreams-sink-sql run   "psql://user:pass@localhost:5432/jupiter?sslmode=disable" jupiter-dex-substreams@v1.0.0 \
  -e mainnet.sol.streamingfast.io:443

# ClickHouse: same commands with a clickhouse:// DSN and --engine=clickhouse (see schema.clickhouse.sql)
```

### Endpoints

| Provider | Endpoint | Token env |
|---|---|---|
| StreamingFast | `mainnet.sol.streamingfast.io:443` | `SUBSTREAMS_API_TOKEN` |
| Pinax | `solana.substreams.pinax.network:443` | `SUBSTREAMS_API_TOKEN` (Pinax JWT) |

The stores begin at slot 31,310,775. The map modules and `db_out` don't depend on stores, so you can start them at any slot.

## Amount semantics and fees

`amount_in` / `amount_out` are the **gross swap amounts that Jupiter logs** in its `SwapEvent`/`SwapsEvent`, summed across split and multi-hop legs: input-mint legs for `amount_in`, destination-mint legs for `amount_out`. They can differ from the user's wallet balance change, which is expected:

- **Output-side platform fee.** The fee is taken from the output after the swap, so the wallet receives less than `amount_out`. v1 routes log it as `FeeEvent`, in `platform_fee_amount`. Example: 23,574,865,657 out − 235,748,656 fee = 23,339,117,001 received. v2 routes do not log the fee.
- **Input-side fees (Ultra / v2).** The wallet spends slightly more than `amount_in`. Example: 5,000,000,000 spent vs 4,978,014,400 swapped.
- **Native SOL / wSOL and owner mismatch.** Wrapping, unwrapping and ATA closes, or a delegate `user_transfer_authority` that isn't the token owner, make wallet token deltas zero or unrelated.
- **Exact-out instructions.** `amount_in` is the realized input from events, not the `quoted_in_amount` maximum (that is `quoted_amount`).

## Verification (v1.0.0)

Run on Pinax around Solana slot 460.93M (Oct 24, 2026):

| Check | Result |
|---|---|
| Swap-instruction counts vs RPC `getBlock`, 10 slots | All types match: route_v2 34/34, route 24/24, shared_accounts_route_v2 7/7, route_with_token_ledger 3/3, shared_accounts_route 2/2, rfq_fill 1/1 |
| Pack amounts vs SwapEvent amounts decoded independently from RPC `getTransaction` | Every one of the 29 sampled v6 rows whose amounts differed from wallet balance changes matches the logged events one-to-one. The 1 RFQ fill uses arg amounts, and its output matched the wallet exactly. Multi-swap transactions sum exactly. The wallet differences are explained by fees, wSOL and owner mismatch (above). |
| FeeEvent | `platform_fee_amount` 235,748,656 equals the on-chain FeeEvent |
| PostgreSQL sink, 300 blocks, fresh DB | setup and run rc=0, no errors; 1,798 swaps, 7 xStocks tickers in `stock_daily_volume` |
| Unit tests | 30/30 pass |
| Speed, `db_out` over 1,000 blocks (Pinax) | v0.6.0 85.2s / 84.7s; v1.0.0 88.3s / 87.1s |
| Coverage vs v0.6.0, same 40 slots | v0.6.0 reported 253 swaps, 147 with wrong mints (`route_v2` was unsupported); v1.0.0 reports 362, all with valid mints |

## Breaking changes in v1.0.0 (resync required)

- `jupiter_swaps.id` is now `tx_hash:instruction_index`. It was `tx:slot:program`, which collided when a transaction contained several swaps.
- The heuristic amount scanning is removed. Only decoded swap instructions carry amounts, so volume figures change, and are now correct.
- New `jupiter_swaps` columns: `instruction_type`, `instruction_index`, `quoted_amount`, `amount_source`, `input_stock_ticker`, `output_stock_ticker`, `platform_fee_amount`, `platform_fee_mint`. New table: `stock_daily_volume`.

Existing SQL sinks must drop their tables, run `setup` with the new schema, and resync from their start block.

## Changelog

See [CHANGELOG.md](CHANGELOG.md). Recent releases:

- **v1.0.0**: exact v6 decoding including `route_v2`, event-sourced amounts and fees, JupiterZ RFQ fills, Limit Order v2 tagging, xStocks tagging and `stock_daily_volume`, unique swap IDs.
- **v0.6.0**: buffa migration (substreams 0.8 / substreams-solana 0.16 / database-change 5.0). Output identical to the v0.5.1 source tree.
- **v0.5.1**: Jupiter v4-protocol support; mint-extraction fix.

## License

MIT, see [LICENSE](LICENSE).
