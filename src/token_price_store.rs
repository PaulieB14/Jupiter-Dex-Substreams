use substreams::prelude::*;
use substreams_solana::pb::sf::solana::r#type::v1::Transactions;
use substreams_solana::pb::sf::substreams::foundational_store::v1::{Entries, Entry};
use prost::Message;
use prost_types::Any;

#[substreams::handlers::map]
fn map_token_prices(
    transactions: Transactions,
) -> Result<Entries, Error> {
    let mut entries: Vec<Entry> = Vec::new();

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if is_jupiter_swap_instruction(&instruction) {
                if let Ok(price_data) = extract_token_price_data(instruction) {
                    let key = format!("token_price_{}", hex::encode(&price_data.mint_address));
                    
                    let mut buf = Vec::new();
                    price_data.encode(&mut buf)?;

                    let entry = Entry {
                        key: key.as_bytes().to_vec(),
                        value: Some(Any {
                            type_url: "type.googleapis.com/sf.jupiter.v1.TokenPrice".to_string(),
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

fn is_jupiter_swap_instruction(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> bool {
    // Check if this is a Jupiter swap instruction
    instruction.program_id_index < instruction.accounts.len() &&
    (instruction.accounts[instruction.program_id_index as usize] == "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" ||
     instruction.accounts[instruction.program_id_index as usize] == "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB")
}

fn extract_token_price_data(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> Result<TokenPrice, Error> {
    // Parse Jupiter swap instruction to extract price information
    let data = &instruction.data;
    
    // This is a simplified example - you'd need to implement proper Jupiter instruction parsing
    let price_data = TokenPrice {
        mint_address: instruction.accounts[0].clone().into_bytes(),
        price_usd: 0.0, // Extract from instruction data
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        volume_24h: 0.0,
        price_change_24h: 0.0,
    };

    Ok(price_data)
}

fn successful_transactions(transactions: &Transactions) -> Vec<&substreams_solana::pb::sf::solana::r#type::v1::ConfirmedTransaction> {
    transactions.transactions.iter()
        .filter_map(|tx| tx.transaction.as_ref())
        .filter_map(|tx| tx.meta.as_ref())
        .filter(|meta| meta.err.is_none())
        .map(|tx| tx)
        .collect()
}

// Define TokenPrice struct for price tracking
#[derive(Clone, PartialEq, Message)]
pub struct TokenPrice {
    #[prost(bytes, tag = "1")]
    pub mint_address: Vec<u8>,
    #[prost(double, tag = "2")]
    pub price_usd: f64,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    #[prost(double, tag = "4")]
    pub volume_24h: f64,
    #[prost(double, tag = "5")]
    pub price_change_24h: f64,
}