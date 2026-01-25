use crate::constants::JUPITER_PROGRAM_IDS;
use crate::pb::sf::jupiter::v1::{TradingData, TradingDataList};
use substreams::errors::Error;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

// Jupiter v6 instruction discriminators
const ROUTE_DISCRIMINATOR: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];
const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [208, 51, 239, 151, 123, 43, 237, 92];
const SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [176, 209, 105, 168, 154, 125, 69, 62];

#[substreams::handlers::map]
pub fn map_jupiter_trading_data(block: Block) -> Result<TradingDataList, Error> {
    let mut items = Vec::new();
    let mut total_volume: u64 = 0;
    let mut swap_count: u32 = 0;

    let block_time = block
        .block_time
        .as_ref()
        .map(|ts| ts.timestamp.max(0) as u64)
        .unwrap_or_default();

    for trx in block.transactions() {
        let tx_id = trx.id();

        for instruction in trx.walk_instructions() {
            let program_id = instruction.program_id().to_string();
            if !is_jupiter_program(&program_id) {
                continue;
            }

            let accounts: Vec<String> = instruction
                .accounts()
                .iter()
                .map(|address| address.to_string())
                .collect();

            let data = instruction.data();

            // Parse swap amounts from instruction data
            let (amount_in, amount_out, input_mint, output_mint, user_wallet) =
                parse_jupiter_instruction(data, &accounts);

            if amount_in > 0 {
                total_volume = total_volume.saturating_add(amount_in);
                swap_count += 1;
            }

            items.push(TradingData {
                program_id,
                transaction_id: tx_id.clone(),
                accounts,
                data: data.to_vec(),
                slot: block.slot,
                block_time,
                amount_in,
                amount_out,
                input_mint,
                output_mint,
                user_wallet,
            });
        }
    }

    Ok(TradingDataList {
        items,
        total_volume,
        swap_count,
    })
}

/// Parse Jupiter instruction data to extract swap amounts
fn parse_jupiter_instruction(data: &[u8], accounts: &[String]) -> (u64, u64, String, String, String) {
    if data.len() < 8 {
        return (0, 0, String::new(), String::new(), String::new());
    }

    let discriminator: [u8; 8] = data[0..8].try_into().unwrap_or([0; 8]);

    // Route instruction: discriminator(8) + route_plan_len(1) + ...
    // SharedAccountsRoute: discriminator(8) + id(1) + route_plan_len(1) + ...
    // The amount is typically a u64 after the route plan

    match discriminator {
        ROUTE_DISCRIMINATOR => parse_route_instruction(data, accounts),
        SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR => parse_shared_accounts_route(data, accounts),
        EXACT_OUT_ROUTE_DISCRIMINATOR => parse_exact_out_route(data, accounts),
        SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR => parse_shared_accounts_exact_out(data, accounts),
        _ => {
            // Try to extract amount from generic instruction format
            parse_generic_swap(data, accounts)
        }
    }
}

/// Parse Route instruction (Jupiter v6)
/// Format: discriminator(8) + route_plan + in_amount(8) + quoted_out_amount(8) + slippage_bps(2) + platform_fee_bps(1)
fn parse_route_instruction(data: &[u8], accounts: &[String]) -> (u64, u64, String, String, String) {
    // Route instruction has variable length route_plan first
    // We need to find the amounts at the end
    if data.len() < 24 {
        return (0, 0, String::new(), String::new(), String::new());
    }

    // Try to find amounts - they're typically near the end
    // Look for the pattern: in_amount(8) + quoted_out_amount(8) + slippage_bps(2)
    let len = data.len();

    // Amounts are usually at specific offsets after the route plan
    // Common pattern: last 18-19 bytes contain amounts + slippage
    if len >= 26 {
        let amount_in = read_u64_le(&data[len - 18..len - 10]);
        let amount_out = read_u64_le(&data[len - 10..len - 2]);

        let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);

        return (amount_in, amount_out, input_mint, output_mint, user_wallet);
    }

    (0, 0, String::new(), String::new(), String::new())
}

/// Parse SharedAccountsRoute instruction (Jupiter v6)
/// Most common instruction type
fn parse_shared_accounts_route(data: &[u8], accounts: &[String]) -> (u64, u64, String, String, String) {
    // SharedAccountsRoute format:
    // discriminator(8) + id(1) + route_plan + in_amount(8) + quoted_out_amount(8) + slippage_bps(2) + platform_fee_bps(1)

    if data.len() < 30 {
        return (0, 0, String::new(), String::new(), String::new());
    }

    // Try to extract amounts from the end of the data
    let len = data.len();

    // Find in_amount and quoted_out_amount (they're 8 bytes each, near the end)
    // Pattern: ...route_plan_data... + in_amount(8) + quoted_out_amount(8) + slippage(2) + platform_fee(1)
    if len >= 19 {
        // Try reading from different offsets
        let amount_in = read_u64_le(&data[len - 19..len - 11]);
        let amount_out = read_u64_le(&data[len - 11..len - 3]);

        if amount_in > 0 && amount_in < u64::MAX / 2 {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return (amount_in, amount_out, input_mint, output_mint, user_wallet);
        }
    }

    // Alternative: amounts might be at a fixed offset after discriminator
    if len >= 25 {
        let amount_in = read_u64_le(&data[9..17]);
        let amount_out = read_u64_le(&data[17..25]);

        if amount_in > 0 && amount_in < u64::MAX / 2 {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return (amount_in, amount_out, input_mint, output_mint, user_wallet);
        }
    }

    (0, 0, String::new(), String::new(), String::new())
}

/// Parse ExactOutRoute instruction
fn parse_exact_out_route(data: &[u8], accounts: &[String]) -> (u64, u64, String, String, String) {
    if data.len() < 24 {
        return (0, 0, String::new(), String::new(), String::new());
    }

    // ExactOut has out_amount specified, in_amount is maximum
    let len = data.len();
    if len >= 19 {
        let amount_out = read_u64_le(&data[len - 19..len - 11]);
        let max_amount_in = read_u64_le(&data[len - 11..len - 3]);

        let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
        return (max_amount_in, amount_out, input_mint, output_mint, user_wallet);
    }

    (0, 0, String::new(), String::new(), String::new())
}

/// Parse SharedAccountsExactOutRoute instruction
fn parse_shared_accounts_exact_out(data: &[u8], accounts: &[String]) -> (u64, u64, String, String, String) {
    parse_exact_out_route(data, accounts)
}

/// Generic swap parsing for unknown instruction formats
fn parse_generic_swap(data: &[u8], accounts: &[String]) -> (u64, u64, String, String, String) {
    // Try to find u64 values that look like token amounts (reasonable range)
    if data.len() < 16 {
        return (0, 0, String::new(), String::new(), String::new());
    }

    // Scan through data looking for reasonable u64 values
    for i in 8..data.len().saturating_sub(16) {
        let val1 = read_u64_le(&data[i..i + 8]);
        let val2 = read_u64_le(&data[i + 8..i + 16]);

        // Check if values look like token amounts (between 1 and 10^18)
        if val1 > 1000 && val1 < 1_000_000_000_000_000_000
           && val2 > 0 && val2 < 1_000_000_000_000_000_000 {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return (val1, val2, input_mint, output_mint, user_wallet);
        }
    }

    (0, 0, String::new(), String::new(), String::new())
}

/// Extract mint addresses and user wallet from accounts
fn extract_mints_from_accounts(accounts: &[String]) -> (String, String, String) {
    // Jupiter account layout varies by instruction, but typically:
    // - Token program is always present
    // - User's wallet is usually first or second
    // - Source/destination token accounts contain the mints

    let user_wallet = accounts.first().cloned().unwrap_or_default();

    // For SharedAccountsRoute, typical layout:
    // [0] token_program, [1] user_transfer_authority, [2] user_source_token_account,
    // [3] user_destination_token_account, [4] destination_token_account (can be same as 3)
    // [5] destination_mint, [6] platform_fee_account (optional), [7+] intermediate accounts

    let input_mint = if accounts.len() > 2 {
        accounts[2].clone()
    } else {
        String::new()
    };

    let output_mint = if accounts.len() > 5 {
        accounts[5].clone()
    } else if accounts.len() > 3 {
        accounts[3].clone()
    } else {
        String::new()
    };

    (input_mint, output_mint, user_wallet)
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
