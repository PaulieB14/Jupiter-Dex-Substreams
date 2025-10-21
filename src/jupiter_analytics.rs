use substreams::prelude::*;
use substreams::store::FoundationalStore;
use substreams_solana::pb::sf::solana::r#type::v1::Transactions;
use substreams_solana::pb::sf::substreams::solana::spl::v1::AccountOwner;
use substreams_solana::pb::sf::substreams::foundational_store::v1::ResponseCode;
use prost::Message;

#[substreams::handlers::map]
fn map_jupiter_analytics(
    transactions: Transactions,
    account_owner_store: FoundationalStore,
    trading_data_store: FoundationalStore,
    token_price_store: FoundationalStore,
) -> Result<JupiterAnalytics, Error> {
    let mut analytics = JupiterAnalytics {
        total_volume_24h: 0.0,
        total_trades_24h: 0,
        top_tokens: Vec::new(),
        user_activity: Vec::new(),
        price_movements: Vec::new(),
    };

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if is_jupiter_instruction(&instruction) {
                // Aggregate trading volume
                if let Ok(volume) = extract_trading_volume(instruction) {
                    analytics.total_volume_24h += volume;
                    analytics.total_trades_24h += 1;
                }

                // Track top tokens
                if let Ok(token_info) = extract_token_info(instruction, &token_price_store) {
                    analytics.top_tokens.push(token_info);
                }

                // Track user activity
                if let Ok(user_activity) = extract_user_activity(instruction, &account_owner_store) {
                    analytics.user_activity.push(user_activity);
                }

                // Track price movements
                if let Ok(price_movement) = extract_price_movement(instruction, &token_price_store) {
                    analytics.price_movements.push(price_movement);
                }
            }
        }
    }

    Ok(analytics)
}

fn extract_trading_volume(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> Result<f64, Error> {
    // Extract trading volume from Jupiter instruction
    // This is a simplified example - implement proper volume extraction
    Ok(1000.0) // Placeholder
}

fn extract_token_info(
    instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction,
    token_price_store: &FoundationalStore,
) -> Result<TokenInfo, Error> {
    let mint_address = instruction.accounts[0].clone();
    let price_key = format!("token_price_{}", hex::encode(mint_address.as_bytes()));
    
    // Query token price from foundational store
    let response = token_price_store.get(&price_key.as_bytes().to_vec());
    
    let token_info = TokenInfo {
        mint_address: mint_address.clone(),
        symbol: "UNKNOWN".to_string(),
        price_usd: 0.0,
        volume_24h: 0.0,
        price_change_24h: 0.0,
    };

    Ok(token_info)
}

fn extract_user_activity(
    instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction,
    account_owner_store: &FoundationalStore,
) -> Result<UserActivity, Error> {
    // Extract user activity from instruction
    let user_activity = UserActivity {
        user_address: "unknown".to_string(),
        action: "swap".to_string(),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        amount: 0.0,
    };

    Ok(user_activity)
}

fn extract_price_movement(
    instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction,
    token_price_store: &FoundationalStore,
) -> Result<PriceMovement, Error> {
    // Extract price movement data
    let price_movement = PriceMovement {
        mint_address: instruction.accounts[0].clone(),
        price_change: 0.0,
        volume: 0.0,
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };

    Ok(price_movement)
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

// Define analytics structs
#[derive(Clone, PartialEq, Message)]
pub struct JupiterAnalytics {
    #[prost(double, tag = "1")]
    pub total_volume_24h: f64,
    #[prost(uint64, tag = "2")]
    pub total_trades_24h: u64,
    #[prost(message, repeated, tag = "3")]
    pub top_tokens: Vec<TokenInfo>,
    #[prost(message, repeated, tag = "4")]
    pub user_activity: Vec<UserActivity>,
    #[prost(message, repeated, tag = "5")]
    pub price_movements: Vec<PriceMovement>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TokenInfo {
    #[prost(string, tag = "1")]
    pub mint_address: String,
    #[prost(string, tag = "2")]
    pub symbol: String,
    #[prost(double, tag = "3")]
    pub price_usd: f64,
    #[prost(double, tag = "4")]
    pub volume_24h: f64,
    #[prost(double, tag = "5")]
    pub price_change_24h: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct UserActivity {
    #[prost(string, tag = "1")]
    pub user_address: String,
    #[prost(string, tag = "2")]
    pub action: String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    #[prost(double, tag = "4")]
    pub amount: f64,
}

#[derive(Clone, PartialEq, Message)]
pub struct PriceMovement {
    #[prost(string, tag = "1")]
    pub mint_address: String,
    #[prost(double, tag = "2")]
    pub price_change: f64,
    #[prost(double, tag = "3")]
    pub volume: f64,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}