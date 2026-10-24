//! Jupiter trading data extraction.
//!
//! v1.0.0: exact decoding of Jupiter Aggregator v6 swap instructions (v1 and the 2025 `*_v2`
//! family, incl. token-ledger variants) using Anchor discriminators and the on-chain IDL account
//! layouts, with realized amounts taken from the program's `SwapEvent` / `SwapsEvent` self-CPI
//! logs. JupiterZ RFQ (Order Engine) fills are decoded from their instruction args.
//! Other Jupiter programs (DCA, limit orders) are emitted as instructions with zero amounts:
//! their fills settle through a v6 route CPI, which is what gets counted (no double counting).
use crate::constants::{
    JUPITER_ORDER_ENGINE_2_PROGRAM_ID, JUPITER_ORDER_ENGINE_PROGRAM_ID, JUPITER_PROGRAM_IDS,
    JUPITER_V6_PROGRAM_ID,
};
use crate::pb::sf::jupiter::v1::{TradingData, TradingDataList};
use crate::stocks;
use substreams::errors::Error;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

// Jupiter v6 instruction discriminators (sha256("global:<name>")[..8])
const ROUTE_DISCRIMINATOR: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
const ROUTE_WITH_TOKEN_LEDGER: [u8; 8] = [150, 86, 71, 116, 167, 93, 14, 104];
const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [208, 51, 239, 151, 123, 43, 237, 92];
const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];
const SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER: [u8; 8] = [230, 121, 143, 80, 119, 159, 106, 170];
const SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [176, 209, 105, 168, 154, 125, 69, 62];
const ROUTE_V2: [u8; 8] = [187, 100, 250, 204, 49, 196, 175, 20];
const EXACT_OUT_ROUTE_V2: [u8; 8] = [157, 138, 184, 82, 21, 244, 243, 36];
const SHARED_ACCOUNTS_ROUTE_V2: [u8; 8] = [209, 152, 83, 147, 124, 254, 216, 233];
const SHARED_ACCOUNTS_EXACT_OUT_ROUTE_V2: [u8; 8] = [53, 96, 229, 202, 216, 187, 250, 24];

// Anchor event CPI: EVENT_IX_TAG (0x1d9acb512ea545e4 LE) + event discriminator
const EVENT_IX_TAG: [u8; 8] = [0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d];
const SWAP_EVENT: [u8; 8] = [64, 198, 205, 232, 38, 8, 113, 226];
const SWAPS_EVENT: [u8; 8] = [152, 47, 78, 235, 192, 96, 110, 106];

// JupiterZ Order Engine `fill(input_amount u64, output_amount u64, expire_at i64)`
const RFQ_FILL: [u8; 8] = [168, 96, 183, 163, 92, 10, 40, 160];

/// Account-index layout of a swap instruction (from the on-chain IDL).
struct Layout {
    name: &'static str,
    wallet: usize,
    src_mint: Option<usize>,
    dst_mint: usize,
    exact_out: bool,
}

/// Decoded swap instruction (args-level).
#[derive(Default, Debug, Clone, PartialEq)]
struct ParsedSwap {
    name: &'static str,
    amount_in: u64,
    amount_out: u64,
    quoted: u64,
    input_mint: String,
    output_mint: String,
    user_wallet: String,
    exact_out: bool,
    is_v6_route: bool,
}

/// One swap leg from a SwapEvent / SwapsEvent log.
#[derive(Debug, Clone, PartialEq)]
struct SwapLeg {
    input_mint: String,
    input_amount: u64,
    output_mint: String,
    output_amount: u64,
}

struct RouteCtx {
    item: usize,
    legs: Vec<SwapLeg>,
}

#[substreams::handlers::map]
pub fn map_jupiter_trading_data(block: Block) -> Result<TradingDataList, Error> {
    let mut items: Vec<TradingData> = Vec::with_capacity(64);
    let block_time = block.block_time.timestamp.max(0) as u64;
    let slot = block.slot;

    for trx in block.transactions() {
        let tx_id = trx.id();
        let mut routes: Vec<RouteCtx> = Vec::new();

        for (ix_index, instruction) in trx.walk_instructions().enumerate() {
            let program_id_str = instruction.program_id().to_string();
            if !is_jupiter_program(&program_id_str) {
                continue;
            }
            let data = instruction.data();

            // Self-CPI event log: attach legs to the enclosing route.
            if program_id_str == JUPITER_V6_PROGRAM_ID && data.len() >= 16 && data[0..8] == EVENT_IX_TAG {
                if let Some(ctx) = routes.last_mut() {
                    ctx.legs.extend(parse_event(&data[8..]));
                }
            }

            let accounts: Vec<String> = instruction.accounts().iter().map(|a| a.to_string()).collect();
            let parsed = parse_instruction(&program_id_str, data, &accounts);
            if parsed.is_v6_route {
                routes.push(RouteCtx { item: items.len(), legs: Vec::new() });
            }

            items.push(TradingData {
                program_id: program_id_str,
                transaction_id: tx_id.clone(),
                accounts,
                data: data.to_vec(),
                slot,
                block_time,
                amount_in: parsed.amount_in,
                amount_out: parsed.amount_out,
                input_mint: parsed.input_mint,
                output_mint: parsed.output_mint,
                user_wallet: parsed.user_wallet,
                instruction_type: parsed.name.to_string(),
                instruction_index: ix_index as u32,
                quoted_amount: parsed.quoted,
                amount_source: if parsed.name.is_empty() { String::new() } else { "args".to_string() },
                ..Default::default()
            });
        }

        // Realized amounts from event logs.
        for ctx in routes {
            let item = &mut items[ctx.item];
            if let Some((src, a_in, a_out)) = realized(&item.input_mint, &item.output_mint, &ctx.legs) {
                item.input_mint = src;
                item.amount_in = a_in;
                item.amount_out = a_out;
                item.amount_source = "event".to_string();
            }
        }
    }

    let mut total_volume: u64 = 0;
    let mut swap_count: u32 = 0;
    for item in items.iter_mut() {
        if item.amount_in > 0 {
            total_volume = total_volume.saturating_add(item.amount_in);
            swap_count += 1;
            item.input_stock_ticker = stocks::ticker(&item.input_mint);
            item.output_stock_ticker = stocks::ticker(&item.output_mint);
        }
    }

    Ok(TradingDataList { items, total_volume, swap_count })
}

/// Realized (source_mint, amount_in, amount_out) from event legs.
fn realized(src: &str, dst: &str, legs: &[SwapLeg]) -> Option<(String, u64, u64)> {
    if legs.is_empty() || dst.is_empty() {
        return None;
    }
    let src = if src.is_empty() { legs[0].input_mint.clone() } else { src.to_string() };
    let a_in: u64 = legs.iter().filter(|l| l.input_mint == src).fold(0u64, |a, l| a.saturating_add(l.input_amount));
    let a_out: u64 = legs.iter().filter(|l| l.output_mint == dst).fold(0u64, |a, l| a.saturating_add(l.output_amount));
    if a_in == 0 || a_out == 0 {
        return None;
    }
    Some((src, a_in, a_out))
}

/// Parse an Anchor event payload (after EVENT_IX_TAG).
fn parse_event(d: &[u8]) -> Vec<SwapLeg> {
    if d.len() < 8 {
        return vec![];
    }
    let body = &d[8..];
    match <[u8; 8]>::try_from(&d[0..8]).unwrap_or_default() {
        // SwapEvent { amm, input_mint, input_amount, output_mint, output_amount }
        SWAP_EVENT if body.len() >= 112 => vec![SwapLeg {
            input_mint: b58(&body[32..64]),
            input_amount: read_u64_le(&body[64..72]),
            output_mint: b58(&body[72..104]),
            output_amount: read_u64_le(&body[104..112]),
        }],
        // SwapsEvent { swap_events: Vec<SwapEventV2 { input_mint, input_amount, output_mint, output_amount, amm }> }
        SWAPS_EVENT if body.len() >= 4 => {
            let n = u32::from_le_bytes(body[0..4].try_into().unwrap()) as usize;
            let mut out = Vec::with_capacity(n.min(64));
            for i in 0..n {
                let o = 4 + i * 112;
                if body.len() < o + 112 {
                    break;
                }
                let e = &body[o..o + 112];
                out.push(SwapLeg {
                    input_mint: b58(&e[0..32]),
                    input_amount: read_u64_le(&e[32..40]),
                    output_mint: b58(&e[40..72]),
                    output_amount: read_u64_le(&e[72..80]),
                });
            }
            out
        }
        _ => vec![],
    }
}

fn b58(b: &[u8]) -> String {
    bs58::encode(b).into_string()
}

/// Decode a Jupiter instruction into args-level swap fields.
fn parse_instruction(program_id: &str, data: &[u8], accounts: &[String]) -> ParsedSwap {
    if data.len() < 8 {
        return ParsedSwap::default();
    }
    let disc: [u8; 8] = data[0..8].try_into().unwrap();

    if program_id == JUPITER_ORDER_ENGINE_PROGRAM_ID || program_id == JUPITER_ORDER_ENGINE_2_PROGRAM_ID {
        if disc == RFQ_FILL && data.len() >= 24 && accounts.len() >= 9 {
            return ParsedSwap {
                name: "rfq_fill",
                amount_in: read_u64_le(&data[8..16]),
                amount_out: read_u64_le(&data[16..24]),
                quoted: read_u64_le(&data[16..24]),
                input_mint: accounts[6].clone(),
                output_mint: accounts[8].clone(),
                user_wallet: accounts[0].clone(),
                ..Default::default()
            };
        }
        return ParsedSwap::default();
    }
    if program_id != JUPITER_V6_PROGRAM_ID {
        return ParsedSwap::default();
    }

    // (layout, args): v1 args have amounts at the tail (route_plan is variable length);
    // v2 args have amounts right after the discriminator (and the shared `id` byte).
    enum Args { Tail19, TailLedger11, Head(usize) }
    let (layout, args) = match disc {
        ROUTE_DISCRIMINATOR => (Layout { name: "route", wallet: 1, src_mint: None, dst_mint: 5, exact_out: false }, Args::Tail19),
        ROUTE_WITH_TOKEN_LEDGER => (Layout { name: "route_with_token_ledger", wallet: 1, src_mint: None, dst_mint: 5, exact_out: false }, Args::TailLedger11),
        EXACT_OUT_ROUTE_DISCRIMINATOR => (Layout { name: "exact_out_route", wallet: 1, src_mint: Some(5), dst_mint: 6, exact_out: true }, Args::Tail19),
        SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR => (Layout { name: "shared_accounts_route", wallet: 2, src_mint: Some(7), dst_mint: 8, exact_out: false }, Args::Tail19),
        SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER => (Layout { name: "shared_accounts_route_with_token_ledger", wallet: 2, src_mint: Some(7), dst_mint: 8, exact_out: false }, Args::TailLedger11),
        SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR => (Layout { name: "shared_accounts_exact_out_route", wallet: 2, src_mint: Some(7), dst_mint: 8, exact_out: true }, Args::Tail19),
        ROUTE_V2 => (Layout { name: "route_v2", wallet: 0, src_mint: Some(3), dst_mint: 4, exact_out: false }, Args::Head(8)),
        EXACT_OUT_ROUTE_V2 => (Layout { name: "exact_out_route_v2", wallet: 0, src_mint: Some(3), dst_mint: 4, exact_out: true }, Args::Head(8)),
        SHARED_ACCOUNTS_ROUTE_V2 => (Layout { name: "shared_accounts_route_v2", wallet: 1, src_mint: Some(6), dst_mint: 7, exact_out: false }, Args::Head(9)),
        SHARED_ACCOUNTS_EXACT_OUT_ROUTE_V2 => (Layout { name: "shared_accounts_exact_out_route_v2", wallet: 1, src_mint: Some(6), dst_mint: 7, exact_out: true }, Args::Head(9)),
        _ => return ParsedSwap::default(),
    };
    let max_idx = layout.dst_mint.max(layout.wallet).max(layout.src_mint.unwrap_or(0));
    if accounts.len() <= max_idx {
        return ParsedSwap::default();
    }
    let len = data.len();
    // (first, second) u64 args: exact-in => (in_amount, quoted_out), exact-out => (out_amount, quoted_in)
    let (first, second) = match args {
        Args::Tail19 if len >= 8 + 19 => (read_u64_le(&data[len - 19..len - 11]), read_u64_le(&data[len - 11..len - 3])),
        Args::TailLedger11 if len >= 8 + 11 => (0, read_u64_le(&data[len - 11..len - 3])),
        Args::Head(o) if len >= o + 16 => (read_u64_le(&data[o..o + 8]), read_u64_le(&data[o + 8..o + 16])),
        _ => return ParsedSwap::default(),
    };
    let (amount_in, amount_out, quoted) = if layout.exact_out {
        (second, first, second)
    } else {
        (first, second, second)
    };
    ParsedSwap {
        name: layout.name,
        amount_in,
        amount_out,
        quoted,
        input_mint: layout.src_mint.map(|i| accounts[i].clone()).unwrap_or_default(),
        output_mint: accounts[layout.dst_mint].clone(),
        user_wallet: accounts[layout.wallet].clone(),
        exact_out: layout.exact_out,
        is_v6_route: true,
    }
}

/// Read u64 from bytes in little-endian
fn read_u64_le(data: &[u8]) -> u64 {
    if data.len() < 8 {
        return 0;
    }
    u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8]))
}

fn is_jupiter_program(program_id: &str) -> bool {
    JUPITER_PROGRAM_IDS.iter().any(|id| id == &program_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accts(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("acct{}", i)).collect()
    }

    #[test]
    fn route_v2_head_args() {
        let mut d = ROUTE_V2.to_vec();
        d.extend_from_slice(&1_000_000u64.to_le_bytes());
        d.extend_from_slice(&2_500u64.to_le_bytes());
        d.extend_from_slice(&[50, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // slippage, fees, empty plan
        let p = parse_instruction(JUPITER_V6_PROGRAM_ID, &d, &accts(10));
        assert_eq!((p.name, p.amount_in, p.amount_out), ("route_v2", 1_000_000, 2_500));
        assert_eq!((p.user_wallet.as_str(), p.input_mint.as_str(), p.output_mint.as_str()), ("acct0", "acct3", "acct4"));
    }

    #[test]
    fn shared_v1_tail_args() {
        let mut d = SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR.to_vec();
        d.push(3);
        d.extend_from_slice(&[0u8; 13]); // route plan
        d.extend_from_slice(&7_000u64.to_le_bytes());
        d.extend_from_slice(&9_000u64.to_le_bytes());
        d.extend_from_slice(&[0, 0, 0]);
        let p = parse_instruction(JUPITER_V6_PROGRAM_ID, &d, &accts(13));
        assert_eq!((p.amount_in, p.amount_out, p.user_wallet.as_str(), p.input_mint.as_str(), p.output_mint.as_str()), (7_000, 9_000, "acct2", "acct7", "acct8"));
    }

    #[test]
    fn route_v1_tail_is_19_bytes() {
        let mut d = ROUTE_DISCRIMINATOR.to_vec();
        d.extend_from_slice(&[0u8; 9]);
        d.extend_from_slice(&5_000u64.to_le_bytes());
        d.extend_from_slice(&6_000u64.to_le_bytes());
        d.extend_from_slice(&[0, 0, 0]);
        let p = parse_instruction(JUPITER_V6_PROGRAM_ID, &d, &accts(9));
        assert_eq!((p.amount_in, p.amount_out, p.output_mint.as_str(), p.input_mint.as_str()), (5_000, 6_000, "acct5", ""));
    }

    #[test]
    fn exact_out_v2_swaps_order() {
        let mut d = EXACT_OUT_ROUTE_V2.to_vec();
        d.extend_from_slice(&100u64.to_le_bytes()); // out_amount
        d.extend_from_slice(&250u64.to_le_bytes()); // quoted_in
        d.extend_from_slice(&[0u8; 10]);
        let p = parse_instruction(JUPITER_V6_PROGRAM_ID, &d, &accts(10));
        assert!(p.exact_out);
        assert_eq!((p.amount_in, p.amount_out), (250, 100));
    }

    #[test]
    fn rfq_fill() {
        let mut d = RFQ_FILL.to_vec();
        d.extend_from_slice(&11u64.to_le_bytes());
        d.extend_from_slice(&22u64.to_le_bytes());
        d.extend_from_slice(&0i64.to_le_bytes());
        let p = parse_instruction(JUPITER_ORDER_ENGINE_PROGRAM_ID, &d, &accts(11));
        assert_eq!((p.name, p.amount_in, p.amount_out, p.input_mint.as_str(), p.output_mint.as_str(), p.user_wallet.as_str()), ("rfq_fill", 11, 22, "acct6", "acct8", "acct0"));
        assert!(!p.is_v6_route);
    }

    #[test]
    fn unknown_and_other_programs_yield_nothing() {
        let d = vec![0xFFu8; 64];
        assert_eq!(parse_instruction(JUPITER_V6_PROGRAM_ID, &d, &accts(20)), ParsedSwap::default());
        assert_eq!(parse_instruction("DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M", &ROUTE_V2, &accts(20)), ParsedSwap::default());
        assert_eq!(parse_instruction(JUPITER_V6_PROGRAM_ID, &[1, 2, 3], &accts(20)), ParsedSwap::default());
    }

    #[test]
    fn too_few_accounts_rejected() {
        let mut d = ROUTE_V2.to_vec();
        d.extend_from_slice(&[1u8; 26]);
        assert_eq!(parse_instruction(JUPITER_V6_PROGRAM_ID, &d, &accts(4)), ParsedSwap::default());
    }

    fn pk(n: u8) -> [u8; 32] { [n; 32] }

    #[test]
    fn swaps_event_and_realized() {
        let mut d = SWAPS_EVENT.to_vec();
        d.extend_from_slice(&2u32.to_le_bytes());
        for (i, o, ai, ao) in [(1u8, 2u8, 100u64, 50u64), (2, 3, 50, 7)] {
            d.extend_from_slice(&pk(i)); d.extend_from_slice(&ai.to_le_bytes());
            d.extend_from_slice(&pk(o)); d.extend_from_slice(&ao.to_le_bytes());
            d.extend_from_slice(&pk(9));
        }
        let legs = parse_event(&d);
        assert_eq!(legs.len(), 2);
        let (src, a, b) = realized("", &b58(&pk(3)), &legs).unwrap();
        assert_eq!((src, a, b), (b58(&pk(1)), 100, 7));
    }

    #[test]
    fn swap_event_v1_split_route() {
        let mk = |i: u8, o: u8, ai: u64, ao: u64| {
            let mut d = SWAP_EVENT.to_vec();
            d.extend_from_slice(&pk(9)); d.extend_from_slice(&pk(i)); d.extend_from_slice(&ai.to_le_bytes());
            d.extend_from_slice(&pk(o)); d.extend_from_slice(&ao.to_le_bytes());
            parse_event(&d)
        };
        let mut legs = mk(1, 2, 60, 30);
        legs.extend(mk(1, 2, 40, 21));
        let (_, a, b) = realized(&b58(&pk(1)), &b58(&pk(2)), &legs).unwrap();
        assert_eq!((a, b), (100, 51));
        assert!(realized(&b58(&pk(1)), &b58(&pk(5)), &legs).is_none());
    }

    #[test]
    fn test_read_u64_le() {
        assert_eq!(read_u64_le(&[1, 0, 0, 0, 0, 0, 0, 0]), 1);
        assert_eq!(read_u64_le(&[1, 2, 3]), 0);
    }

    #[test]
    fn test_is_jupiter_program() {
        assert!(is_jupiter_program("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"));
        assert!(is_jupiter_program("j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X"));
        assert!(is_jupiter_program("61DFfeTKM7trxYcPQCM78bJ794ddZprZpAwAnLiwTpYH"));
        assert!(!is_jupiter_program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"));
    }
}
