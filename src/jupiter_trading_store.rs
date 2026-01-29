use crate::constants::JUPITER_PROGRAM_IDS;
use crate::pb::sf::jupiter::v1::{TradingData, TradingDataList};
use substreams::errors::Error;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

// Jupiter v6 instruction discriminators
const ROUTE_DISCRIMINATOR: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];
const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [208, 51, 239, 151, 123, 43, 237, 92];
const SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [176, 209, 105, 168, 154, 125, 69, 62];

// Maximum reasonable token amount (10^18 - prevents parsing garbage data as amounts)
const MAX_REASONABLE_AMOUNT: u64 = 1_000_000_000_000_000_000;
// Minimum amount to consider valid (filters dust/noise)
const MIN_VALID_AMOUNT: u64 = 1000;

/// Parsed swap result with all extracted fields
#[derive(Default)]
struct ParsedSwap {
    amount_in: u64,
    amount_out: u64,
    input_mint: String,
    output_mint: String,
    user_wallet: String,
}

#[substreams::handlers::map]
pub fn map_jupiter_trading_data(block: Block) -> Result<TradingDataList, Error> {
    // Pre-allocate with estimated capacity to avoid reallocations
    let mut items = Vec::with_capacity(64);
    let mut total_volume: u64 = 0;
    let mut swap_count: u32 = 0;

    let block_time = block
        .block_time
        .as_ref()
        .map(|ts| ts.timestamp.max(0) as u64)
        .unwrap_or_default();

    let slot = block.slot;

    for trx in block.transactions() {
        // Cache tx_id once per transaction (avoid repeated clones)
        let tx_id = trx.id();

        for instruction in trx.walk_instructions() {
            // Check program ID without converting to String first
            let program_id_bytes = instruction.program_id();
            let program_id_str = program_id_bytes.to_string();

            if !is_jupiter_program(&program_id_str) {
                continue;
            }

            // Convert accounts to strings (required for output)
            let accounts: Vec<String> = instruction
                .accounts()
                .iter()
                .map(|address| address.to_string())
                .collect();

            let data = instruction.data();

            // Parse swap amounts from instruction data
            let parsed = parse_jupiter_instruction(data, &accounts);

            if parsed.amount_in > 0 {
                total_volume = total_volume.saturating_add(parsed.amount_in);
                swap_count += 1;
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
/// Returns a ParsedSwap struct with all extracted fields
fn parse_jupiter_instruction(data: &[u8], accounts: &[String]) -> ParsedSwap {
    if data.len() < 8 {
        return ParsedSwap::default();
    }

    // Safe discriminator extraction
    let discriminator: [u8; 8] = match data[0..8].try_into() {
        Ok(d) => d,
        Err(_) => return ParsedSwap::default(),
    };

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
fn parse_route_instruction(data: &[u8], accounts: &[String]) -> ParsedSwap {
    // Route instruction has variable length route_plan first
    // We need to find the amounts at the end
    if data.len() < 24 {
        return ParsedSwap::default();
    }

    let len = data.len();

    // Amounts are usually at specific offsets after the route plan
    // Common pattern: last 18-19 bytes contain amounts + slippage
    if len >= 26 {
        let amount_in = read_u64_le(&data[len - 18..len - 10]);
        let amount_out = read_u64_le(&data[len - 10..len - 2]);

        if is_valid_amount(amount_in) && amount_out < MAX_REASONABLE_AMOUNT {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return ParsedSwap {
                amount_in,
                amount_out,
                input_mint,
                output_mint,
                user_wallet,
            };
        }
    }

    ParsedSwap::default()
}

/// Parse SharedAccountsRoute instruction (Jupiter v6)
/// Most common instruction type
fn parse_shared_accounts_route(data: &[u8], accounts: &[String]) -> ParsedSwap {
    // SharedAccountsRoute format:
    // discriminator(8) + id(1) + route_plan + in_amount(8) + quoted_out_amount(8) + slippage_bps(2) + platform_fee_bps(1)

    if data.len() < 30 {
        return ParsedSwap::default();
    }

    let len = data.len();

    // Find in_amount and quoted_out_amount (they're 8 bytes each, near the end)
    // Pattern: ...route_plan_data... + in_amount(8) + quoted_out_amount(8) + slippage(2) + platform_fee(1)
    if len >= 19 {
        let amount_in = read_u64_le(&data[len - 19..len - 11]);
        let amount_out = read_u64_le(&data[len - 11..len - 3]);

        if is_valid_amount(amount_in) && amount_out < MAX_REASONABLE_AMOUNT {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return ParsedSwap {
                amount_in,
                amount_out,
                input_mint,
                output_mint,
                user_wallet,
            };
        }
    }

    // Alternative: amounts might be at a fixed offset after discriminator
    if len >= 25 {
        let amount_in = read_u64_le(&data[9..17]);
        let amount_out = read_u64_le(&data[17..25]);

        if is_valid_amount(amount_in) && amount_out < MAX_REASONABLE_AMOUNT {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return ParsedSwap {
                amount_in,
                amount_out,
                input_mint,
                output_mint,
                user_wallet,
            };
        }
    }

    ParsedSwap::default()
}

/// Parse ExactOutRoute instruction
fn parse_exact_out_route(data: &[u8], accounts: &[String]) -> ParsedSwap {
    if data.len() < 24 {
        return ParsedSwap::default();
    }

    // ExactOut has out_amount specified, in_amount is maximum
    let len = data.len();
    if len >= 19 {
        let amount_out = read_u64_le(&data[len - 19..len - 11]);
        let max_amount_in = read_u64_le(&data[len - 11..len - 3]);

        if is_valid_amount(max_amount_in) && amount_out < MAX_REASONABLE_AMOUNT {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return ParsedSwap {
                amount_in: max_amount_in,
                amount_out,
                input_mint,
                output_mint,
                user_wallet,
            };
        }
    }

    ParsedSwap::default()
}

/// Parse SharedAccountsExactOutRoute instruction
fn parse_shared_accounts_exact_out(data: &[u8], accounts: &[String]) -> ParsedSwap {
    parse_exact_out_route(data, accounts)
}

/// Generic swap parsing for unknown instruction formats
/// Uses heuristics to find likely swap amounts in instruction data
fn parse_generic_swap(data: &[u8], accounts: &[String]) -> ParsedSwap {
    if data.len() < 16 {
        return ParsedSwap::default();
    }

    // Scan through data looking for reasonable u64 values
    // Start after discriminator (8 bytes)
    let end = data.len().saturating_sub(16);
    for i in 8..end {
        let val1 = read_u64_le(&data[i..i + 8]);
        let val2 = read_u64_le(&data[i + 8..i + 16]);

        // Check if values look like token amounts
        if is_valid_amount(val1) && val2 > 0 && val2 < MAX_REASONABLE_AMOUNT {
            let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(accounts);
            return ParsedSwap {
                amount_in: val1,
                amount_out: val2,
                input_mint,
                output_mint,
                user_wallet,
            };
        }
    }

    ParsedSwap::default()
}

/// Validate that an amount looks like a real token amount
#[inline]
fn is_valid_amount(amount: u64) -> bool {
    amount >= MIN_VALID_AMOUNT && amount < MAX_REASONABLE_AMOUNT
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create test instruction data with route discriminator
    fn create_route_instruction(amount_in: u64, amount_out: u64) -> Vec<u8> {
        let mut data = Vec::with_capacity(34);
        // Route discriminator
        data.extend_from_slice(&ROUTE_DISCRIMINATOR);
        // Padding (route plan placeholder)
        data.extend_from_slice(&[0u8; 8]);
        // Amount in (little-endian)
        data.extend_from_slice(&amount_in.to_le_bytes());
        // Amount out (little-endian)
        data.extend_from_slice(&amount_out.to_le_bytes());
        // Slippage (2 bytes)
        data.extend_from_slice(&[0u8, 0u8]);
        data
    }

    /// Helper to create test instruction data with shared accounts route discriminator
    fn create_shared_accounts_instruction(amount_in: u64, amount_out: u64) -> Vec<u8> {
        let mut data = Vec::with_capacity(38);
        // Shared accounts route discriminator
        data.extend_from_slice(&SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR);
        // ID byte
        data.push(0);
        // Padding (route plan placeholder)
        data.extend_from_slice(&[0u8; 10]);
        // Amount in (little-endian)
        data.extend_from_slice(&amount_in.to_le_bytes());
        // Amount out (little-endian)
        data.extend_from_slice(&amount_out.to_le_bytes());
        // Slippage (2 bytes) + platform fee (1 byte)
        data.extend_from_slice(&[0u8, 0u8, 0u8]);
        data
    }

    /// Helper to create test accounts
    fn create_test_accounts() -> Vec<String> {
        vec![
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(), // Token program
            "UserWallet123456789012345678901234567890AB".to_string(),  // User
            "SourceTokenAccount12345678901234567890ABCD".to_string(),  // Source
            "DestTokenAccount123456789012345678901234AB".to_string(),  // Dest
            "IntermediateAccount12345678901234567890AB".to_string(),   // Intermediate
            "OutputMintAddress12345678901234567890ABCD".to_string(),   // Output mint
        ]
    }

    #[test]
    fn test_is_valid_amount() {
        // Valid amounts
        assert!(is_valid_amount(1_000_000)); // 1 token with 6 decimals
        assert!(is_valid_amount(1_000_000_000_000)); // 1 million tokens
        assert!(is_valid_amount(MIN_VALID_AMOUNT));

        // Invalid amounts
        assert!(!is_valid_amount(0));
        assert!(!is_valid_amount(999)); // Below minimum
        assert!(!is_valid_amount(MAX_REASONABLE_AMOUNT)); // At max
        assert!(!is_valid_amount(u64::MAX)); // Way too large
    }

    #[test]
    fn test_read_u64_le() {
        let data = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(read_u64_le(&data), 1);

        let data = [0x00, 0xCA, 0x9A, 0x3B, 0x00, 0x00, 0x00, 0x00]; // 1 billion
        assert_eq!(read_u64_le(&data), 1_000_000_000);

        // Short data returns 0
        assert_eq!(read_u64_le(&[1, 2, 3]), 0);
        assert_eq!(read_u64_le(&[]), 0);
    }

    #[test]
    fn test_extract_mints_from_accounts() {
        let accounts = create_test_accounts();
        let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(&accounts);

        assert_eq!(user_wallet, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert_eq!(input_mint, "SourceTokenAccount12345678901234567890ABCD");
        assert_eq!(output_mint, "OutputMintAddress12345678901234567890ABCD");
    }

    #[test]
    fn test_extract_mints_empty_accounts() {
        let accounts: Vec<String> = vec![];
        let (input_mint, output_mint, user_wallet) = extract_mints_from_accounts(&accounts);

        assert!(user_wallet.is_empty());
        assert!(input_mint.is_empty());
        assert!(output_mint.is_empty());
    }

    #[test]
    fn test_parse_jupiter_instruction_short_data() {
        let short_data = vec![0u8; 5];
        let accounts = create_test_accounts();

        let result = parse_jupiter_instruction(&short_data, &accounts);

        assert_eq!(result.amount_in, 0);
        assert_eq!(result.amount_out, 0);
    }

    #[test]
    fn test_parse_jupiter_instruction_unknown_discriminator() {
        let mut data = vec![0xFFu8; 32]; // Unknown discriminator
        // Add some amount-like values
        let amount_in: u64 = 1_000_000_000;
        let amount_out: u64 = 500_000_000;
        data[8..16].copy_from_slice(&amount_in.to_le_bytes());
        data[16..24].copy_from_slice(&amount_out.to_le_bytes());

        let accounts = create_test_accounts();
        let result = parse_jupiter_instruction(&data, &accounts);

        // Generic parser should find the amounts
        assert_eq!(result.amount_in, amount_in);
        assert_eq!(result.amount_out, amount_out);
    }

    #[test]
    fn test_is_jupiter_program() {
        // Valid Jupiter programs
        assert!(is_jupiter_program("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"));
        assert!(is_jupiter_program("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB"));
        assert!(is_jupiter_program("jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu")); // Limit orders
        assert!(is_jupiter_program("DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M")); // DCA

        // Invalid programs
        assert!(!is_jupiter_program("RandomProgram123456789012345678901234567"));
        assert!(!is_jupiter_program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"));
        assert!(!is_jupiter_program(""));
    }

    #[test]
    fn test_parsed_swap_default() {
        let swap = ParsedSwap::default();

        assert_eq!(swap.amount_in, 0);
        assert_eq!(swap.amount_out, 0);
        assert!(swap.input_mint.is_empty());
        assert!(swap.output_mint.is_empty());
        assert!(swap.user_wallet.is_empty());
    }
}
