use substreams::prelude::*;
use substreams::store::FoundationalStore;
use substreams_solana::pb::sf::solana::r#type::v1::Transactions;
use substreams_solana::pb::sf::substreams::solana::spl::v1::AccountOwner;
use substreams_solana::pb::sf::substreams::foundational_store::v1::ResponseCode;
use prost::Message;
use serde_json::Value;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[substreams::handlers::map]
fn map_jwt_test(
    transactions: Transactions,
    account_owner_store: FoundationalStore,
    trading_data_store: FoundationalStore,
    token_price_store: FoundationalStore,
) -> Result<JwtTestResults, Error> {
    let mut test_results = JwtTestResults {
        authenticated_requests: 0,
        failed_authentications: 0,
        foundational_store_accesses: 0,
        account_owner_resolutions: 0,
        trading_data_queries: 0,
        token_price_queries: 0,
        errors: Vec::new(),
        performance_metrics: Vec::new(),
    };

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if is_jupiter_instruction(&instruction) {
                // Test JWT authentication
                if let Ok(auth_result) = test_jwt_authentication(instruction) {
                    if auth_result.is_authenticated {
                        test_results.authenticated_requests += 1;
                        
                        // Test foundational store access with authentication
                        if let Ok(store_result) = test_foundational_store_access(
                            instruction,
                            &account_owner_store,
                            &trading_data_store,
                            &token_price_store,
                        ) {
                            test_results.foundational_store_accesses += store_result.accesses;
                            test_results.account_owner_resolutions += store_result.account_owner_resolutions;
                            test_results.trading_data_queries += store_result.trading_data_queries;
                            test_results.token_price_queries += store_result.token_price_queries;
                        }
                    } else {
                        test_results.failed_authentications += 1;
                        test_results.errors.push(format!("Authentication failed for instruction: {}", hex::encode(&instruction.data)));
                    }
                }
            }
        }
    }

    Ok(test_results)
}

#[substreams::handlers::map]
fn map_jwt_auth_test(
    transactions: Transactions,
) -> Result<JwtAuthResults, Error> {
    let mut auth_results = JwtAuthResults {
        total_requests: 0,
        successful_auths: 0,
        failed_auths: 0,
        token_validation_errors: Vec::new(),
        performance_metrics: Vec::new(),
    };

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if is_jupiter_instruction(&instruction) {
                auth_results.total_requests += 1;
                
                // Simulate JWT token validation
                if let Ok(token) = extract_jwt_token(instruction) {
                    if validate_jwt_token(&token) {
                        auth_results.successful_auths += 1;
                    } else {
                        auth_results.failed_auths += 1;
                        auth_results.token_validation_errors.push("Invalid token signature".to_string());
                    }
                } else {
                    auth_results.failed_auths += 1;
                    auth_results.token_validation_errors.push("No JWT token found".to_string());
                }
            }
        }
    }

    Ok(auth_results)
}

fn test_jwt_authentication(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> Result<AuthResult, Error> {
    // Extract JWT token from instruction data (simplified example)
    let token = extract_jwt_token(instruction)?;
    
    // Validate JWT token
    let is_authenticated = validate_jwt_token(&token);
    
    Ok(AuthResult {
        is_authenticated,
        token: "***REDACTED***".to_string(), // Never expose actual tokens
        user_id: "test_user".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
    })
}

fn test_foundational_store_access(
    instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction,
    account_owner_store: &FoundationalStore,
    trading_data_store: &FoundationalStore,
    token_price_store: &FoundationalStore,
) -> Result<StoreAccessResult, Error> {
    let mut result = StoreAccessResult {
        accesses: 0,
        account_owner_resolutions: 0,
        trading_data_queries: 0,
        token_price_queries: 0,
    };

    // Test account owner store access
    let account_keys: Vec<Vec<u8>> = instruction.accounts.iter()
        .map(|addr| addr.as_bytes().to_vec())
        .collect();

    let response = account_owner_store.get_all(&account_keys);
    result.accesses += 1;
    result.account_owner_resolutions += response.entries.len() as u64;

    // Test trading data store access
    let trading_key = format!("jupiter_trade_{}", hex::encode(&instruction.accounts[0]));
    let _trading_response = trading_data_store.get(&trading_key.as_bytes().to_vec());
    result.accesses += 1;
    result.trading_data_queries += 1;

    // Test token price store access
    let price_key = format!("token_price_{}", hex::encode(&instruction.accounts[0]));
    let _price_response = token_price_store.get(&price_key.as_bytes().to_vec());
    result.accesses += 1;
    result.token_price_queries += 1;

    Ok(result)
}

fn extract_jwt_token(instruction: &substreams_solana::pb::sf::solana::r#type::v1::CompiledInstruction) -> Result<String, Error> {
    // In a real implementation, you'd extract the JWT from the instruction data
    // This is a simplified example that reads from environment variables
    let token = std::env::var("SUBSTREAMS_API_TOKEN")
        .or_else(|_| std::env::var("JWT_TOKEN"))
        .map_err(|_| Error::msg("No JWT token found in environment variables"))?;
    
    Ok(token)
}

fn validate_jwt_token(token: &str) -> bool {
    // In a real implementation, you'd validate the JWT signature
    // This is a simplified example that checks for basic JWT structure
    if token.is_empty() {
        return false;
    }
    
    // Check if it's a valid JWT format (3 parts separated by dots)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    
    // In a real implementation, you'd validate the signature
    // For now, just check if it's not empty and has the right format
    !token.is_empty()
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

// Define JWT test result structs
#[derive(Clone, PartialEq, Message)]
pub struct JwtTestResults {
    #[prost(uint64, tag = "1")]
    pub authenticated_requests: u64,
    #[prost(uint64, tag = "2")]
    pub failed_authentications: u64,
    #[prost(uint64, tag = "3")]
    pub foundational_store_accesses: u64,
    #[prost(uint64, tag = "4")]
    pub account_owner_resolutions: u64,
    #[prost(uint64, tag = "5")]
    pub trading_data_queries: u64,
    #[prost(uint64, tag = "6")]
    pub token_price_queries: u64,
    #[prost(string, repeated, tag = "7")]
    pub errors: Vec<String>,
    #[prost(message, repeated, tag = "8")]
    pub performance_metrics: Vec<PerformanceMetric>,
}

#[derive(Clone, PartialEq, Message)]
pub struct JwtAuthResults {
    #[prost(uint64, tag = "1")]
    pub total_requests: u64,
    #[prost(uint64, tag = "2")]
    pub successful_auths: u64,
    #[prost(uint64, tag = "3")]
    pub failed_auths: u64,
    #[prost(string, repeated, tag = "4")]
    pub token_validation_errors: Vec<String>,
    #[prost(message, repeated, tag = "5")]
    pub performance_metrics: Vec<PerformanceMetric>,
}

#[derive(Clone, PartialEq, Message)]
pub struct AuthResult {
    #[prost(bool, tag = "1")]
    pub is_authenticated: bool,
    #[prost(string, tag = "2")]
    pub token: String,
    #[prost(string, tag = "3")]
    pub user_id: String,
    #[prost(string, repeated, tag = "4")]
    pub permissions: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StoreAccessResult {
    #[prost(uint64, tag = "1")]
    pub accesses: u64,
    #[prost(uint64, tag = "2")]
    pub account_owner_resolutions: u64,
    #[prost(uint64, tag = "3")]
    pub trading_data_queries: u64,
    #[prost(uint64, tag = "4")]
    pub token_price_queries: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct PerformanceMetric {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(double, tag = "2")]
    pub value: f64,
    #[prost(string, tag = "3")]
    pub unit: String,
}