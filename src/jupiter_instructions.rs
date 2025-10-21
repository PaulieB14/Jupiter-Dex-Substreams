use substreams::prelude::*;
use substreams::store::FoundationalStore;
use substreams_solana::pb::sf::solana::r#type::v1::Transactions;
use substreams_solana::pb::sf::substreams::solana::spl::v1::AccountOwner;
use substreams_solana::pb::sf::substreams::foundational_store::v1::ResponseCode;
use prost::Message;

#[substreams::handlers::map]
fn map_jupiter_instructions(
    transactions: Transactions,
    account_owner_store: FoundationalStore,
    trading_data_store: FoundationalStore,
    token_price_store: FoundationalStore,
) -> Result<JupiterInstructions, Error> {
    let mut instructions = Vec::new();

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if is_jupiter_instruction(&instruction) {
                if let Ok(jupiter_instruction) = process_jupiter_instruction(
                    instruction,
                    &account_owner_store,
                    &trading_data_store,
                    &token_price_store,
                ) {
                    instructions.push(jupiter_instruction);
                }
            }
        }
    }

    Ok(JupiterInstructions { instructions })
}

fn process_jupiter_instruction(
    instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction,
    account_owner_store: &FoundationalStore,
    trading_data_store: &FoundationalStore,
    token_price_store: &FoundationalStore,
) -> Result<JupiterInstruction, Error> {
    // Extract account addresses that need owner lookup
    let account_keys: Vec<Vec<u8>> = instruction.accounts.iter()
        .map(|addr| addr.as_bytes().to_vec())
        .collect();

    // Batch query foundational store for account owners
    let response = account_owner_store.get_all(&account_keys);

    let mut enriched_accounts = Vec::new();
    
    // Process responses and decode account owner data
    for (i, entry) in response.entries.iter().enumerate() {
        if let Some(response) = &entry.response {
            if response.response == ResponseCode::Found as i32 {
                if let Some(value) = &response.value {
                    if let Ok(account_owner) = AccountOwner::decode(&value.value) {
                        let owner_address = bs58::encode(&account_owner.owner).into_string();
                        let mint_address = bs58::encode(&account_owner.mint_address).into_string();
                        
                        enriched_accounts.push(EnrichedAccount {
                            address: instruction.accounts[i].clone(),
                            owner: owner_address,
                            mint: mint_address,
                        });
                    }
                }
            }
        }
    }

    // Query trading data and token prices for additional context
    let trading_key = format!("jupiter_trade_{}", hex::encode(&instruction.accounts[0]));
    let price_key = format!("token_price_{}", hex::encode(&instruction.accounts[0]));

    let jupiter_instruction = JupiterInstruction {
        program_id: instruction.accounts[instruction.program_id_index as usize].clone(),
        accounts: enriched_accounts,
        data: instruction.data.clone(),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        // Add more enriched data from foundational stores
    };

    Ok(jupiter_instruction)
}

fn is_jupiter_instruction(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> bool {
    instruction.program_id_index < instruction.accounts.len() &&
    (instruction.accounts[instruction.program_id_index as usize] == "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" ||
     instruction.accounts[instruction.program_id_index as usize] == "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB" ||
     instruction.accounts[instruction.program_id_index as usize] == "JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph")
}

fn successful_transactions(transactions: &Transactions) -> Vec<&substreams_solana::pb::sf::solana::r#type::v1::ConfirmedTransaction> {
    transactions.transactions.iter()
        .filter_map(|tx| tx.transaction.as_ref())
        .filter_map(|tx| tx.meta.as_ref())
        .filter(|meta| meta.err.is_none())
        .map(|tx| tx)
        .collect()
}

// Define Jupiter-specific structs
#[derive(Clone, PartialEq, Message)]
pub struct JupiterInstructions {
    #[prost(message, repeated, tag = "1")]
    pub instructions: Vec<JupiterInstruction>,
}

#[derive(Clone, PartialEq, Message)]
pub struct JupiterInstruction {
    #[prost(string, tag = "1")]
    pub program_id: String,
    #[prost(message, repeated, tag = "2")]
    pub accounts: Vec<EnrichedAccount>,
    #[prost(bytes, tag = "3")]
    pub data: Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct EnrichedAccount {
    #[prost(string, tag = "1")]
    pub address: String,
    #[prost(string, tag = "2")]
    pub owner: String,
    #[prost(string, tag = "3")]
    pub mint: String,
}