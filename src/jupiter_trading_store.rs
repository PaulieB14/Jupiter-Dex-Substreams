use substreams::prelude::*;
use substreams_solana::pb::sf::solana::r#type::v1::Transactions;
use substreams_solana::pb::sf::substreams::foundational_store::v1::{Entries, Entry};
use substreams_solana::pb::sf::substreams::solana::spl::v1::AccountOwner;
use prost::Message;
use prost_types::Any;
use serde_json::Value;

#[substreams::handlers::map]
fn map_jupiter_trading_data(
    transactions: Transactions,
) -> Result<Entries, Error> {
    let mut entries: Vec<Entry> = Vec::new();

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if is_jupiter_instruction(&instruction) {
                if let Ok(trading_data) = extract_jupiter_trading_data(instruction) {
                    let key = format!("jupiter_trade_{}", hex::encode(&instruction.accounts()[0]));
                    
                    let mut buf = Vec::new();
                    trading_data.encode(&mut buf)?;

                    let entry = Entry {
                        key: key.as_bytes().to_vec(),
                        value: Some(Any {
                            type_url: "type.googleapis.com/sf.jupiter.v1.TradingData".to_string(),
                            value: buf,
                        }),
                    };
                    entries.push(entry);
                }
            }
        }
    }

    Ok(Entries { entries })
}

fn is_jupiter_instruction(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> bool {
    // Check if this is a Jupiter program instruction
    instruction.program_id_index < instruction.accounts.len() &&
    instruction.accounts[instruction.program_id_index as usize] == "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
}

fn extract_jupiter_trading_data(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> Result<TradingData, Error> {
    // Parse Jupiter instruction data to extract trading information
    let data = &instruction.data;
    
    // This is a simplified example - you'd need to implement proper Jupiter instruction parsing
    let trading_data = TradingData {
        program_id: instruction.accounts[instruction.program_id_index as usize].clone(),
        accounts: instruction.accounts.clone(),
        data: data.clone(),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        // Add more Jupiter-specific fields as needed
    };

    Ok(trading_data)
}

fn successful_transactions(transactions: &Transactions) -> Vec<&substreams_solana::pb::sf::solana::r#type::v1::ConfirmedTransaction> {
    transactions.transactions.iter()
        .filter_map(|tx| tx.transaction.as_ref())
        .filter_map(|tx| tx.meta.as_ref())
        .filter(|meta| meta.err.is_none())
        .map(|tx| tx)
        .collect()
}

// Define TradingData struct for Jupiter trades
#[derive(Clone, PartialEq, Message)]
pub struct TradingData {
    #[prost(string, tag = "1")]
    pub program_id: String,
    #[prost(string, repeated, tag = "2")]
    pub accounts: Vec<String>,
    #[prost(bytes, tag = "3")]
    pub data: Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}