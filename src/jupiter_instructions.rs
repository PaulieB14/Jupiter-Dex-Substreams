use std::collections::{HashMap, HashSet};

use crate::constants::JUPITER_PROGRAM_IDS;
use crate::pb::sf::jupiter::v1::{
    AccountOwnerRecords, EnrichedAccount, JupiterInstruction, JupiterInstructions, TokenPriceList,
    TradingDataList,
};
use substreams::errors::Error;
use substreams_solana::{base58, pb::sf::solana::r#type::v1::Block};

#[substreams::handlers::map]
pub fn map_jupiter_instructions(
    block: Block,
    owner_records: AccountOwnerRecords,
    trading_data: TradingDataList,
    token_prices: TokenPriceList,
) -> Result<JupiterInstructions, Error> {
    // Build indexes once, reuse for all instructions
    let owner_index = build_owner_index(owner_records);
    let price_index = build_price_index(token_prices);
    let trades_by_tx = group_trades_by_tx(&trading_data);

    // Pre-allocate with estimated capacity
    let mut instructions = Vec::with_capacity(32);
    let mut total_volume: u64 = 0;
    let mut instruction_count: u32 = 0;

    let block_time = block
        .block_time
        .as_ref()
        .map(|ts| ts.timestamp.max(0) as u64)
        .unwrap_or_default();

    let slot = block.slot;

    for trx in block.transactions() {
        // Cache tx_id once per transaction
        let tx_id = trx.id();
        let trade_data = trades_by_tx.get(&tx_id);

        for instruction in trx.walk_instructions() {
            let program_id_str = instruction.program_id().to_string();
            if !is_jupiter_program(&program_id_str) {
                continue;
            }

            // Enrich accounts with ownership info
            let enriched_accounts: Vec<EnrichedAccount> = instruction
                .accounts()
                .iter()
                .map(|address| {
                    let addr_str = address.to_string();
                    enrich_account(addr_str, &owner_index, &price_index)
                })
                .collect();

            // Get instruction data
            let data = instruction.data().to_vec();

            // Extract swap info from trading data if available
            let (amount_in, amount_out, input_mint, output_mint) =
                if let Some(trades) = trade_data {
                    if let Some(first_trade) = trades.first() {
                        (
                            first_trade.amount_in,
                            first_trade.amount_out,
                            first_trade.input_mint.clone(),
                            first_trade.output_mint.clone(),
                        )
                    } else {
                        (0, 0, String::new(), String::new())
                    }
                } else {
                    (0, 0, String::new(), String::new())
                };

            if amount_in > 0 {
                total_volume = total_volume.saturating_add(amount_in);
            }
            instruction_count += 1;

            instructions.push(JupiterInstruction {
                program_id: program_id_str,
                transaction_id: tx_id.clone(),
                accounts: enriched_accounts,
                data,
                slot,
                block_time,
                amount_in,
                amount_out,
                input_mint,
                output_mint,
            });
        }
    }

    Ok(JupiterInstructions {
        instructions,
        total_volume,
        instruction_count,
    })
}

/// Build an index mapping account addresses to their (owner, mint) pairs
fn build_owner_index(records: AccountOwnerRecords) -> HashMap<String, (String, String)> {
    let capacity = records.records.len();
    let mut index = HashMap::with_capacity(capacity);

    for record in records.records {
        let account = base58::encode(&record.account);
        let owner = base58::encode(&record.owner);
        let mint = base58::encode(&record.mint);
        index.insert(account, (owner, mint));
    }

    index
}

/// Build a set of known token mint addresses for price lookups
fn build_price_index(token_prices: TokenPriceList) -> HashSet<String> {
    token_prices
        .items
        .into_iter()
        .map(|price| price.mint_address)
        .collect()
}

/// Group trading data by transaction ID for efficient lookup
fn group_trades_by_tx(trading_data: &TradingDataList) -> HashMap<String, Vec<&crate::pb::sf::jupiter::v1::TradingData>> {
    let mut map: HashMap<String, Vec<_>> = HashMap::with_capacity(trading_data.items.len());
    for trade in &trading_data.items {
        map.entry(trade.transaction_id.clone())
            .or_default()
            .push(trade);
    }
    map
}

/// Enrich an account address with ownership and mint information
#[inline]
fn enrich_account(
    address: String,
    owner_index: &HashMap<String, (String, String)>,
    price_index: &HashSet<String>,
) -> EnrichedAccount {
    // Check if we have owner info for this account
    if let Some((owner, mint)) = owner_index.get(&address) {
        return EnrichedAccount {
            address,
            owner: owner.clone(),
            mint: mint.clone(),
        };
    }

    // Check if this address is a known token mint
    let mint = if price_index.contains(&address) {
        address.clone()
    } else {
        String::new()
    };

    EnrichedAccount {
        address,
        owner: String::new(),
        mint,
    }
}

/// Check if a program ID is a Jupiter program
#[inline]
fn is_jupiter_program(program_id: &str) -> bool {
    JUPITER_PROGRAM_IDS.iter().any(|entry| entry == &program_id)
}
