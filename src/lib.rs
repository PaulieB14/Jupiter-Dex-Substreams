use substreams::prelude::*;
use substreams_solana::pb::sol::v1::Block;

// Declare the pb module
mod pb;

// Import the generated protobuf bindings
use crate::pb::jupiter::events::v1::{
    JupiterEvents, SwapEvent, LimitOrderEvent, DcaEvent, AggregationEvent,
    SwapStatus, LimitOrderType, LimitOrderStatus, DcaStatus, AggregationType
};

#[substreams::handlers::map]
pub fn jupiter_events(block: substreams_solana::pb::sol::v1::Block) -> Result<JupiterEvents, substreams::errors::Error> {
    let mut swap_events = vec![];
    let mut limit_order_events = vec![];
    let mut dca_events = vec![];
    let mut aggregation_events = vec![];

    // Process transactions to find Jupiter events
    for transaction in &block.transactions {
        if let Some(meta) = &transaction.meta {
            if meta.err.is_some() {
                continue; // Skip failed transactions
            }
        }

        if let Some(transaction) = &transaction.transaction {
            if let Some(message) = &transaction.message {
                for instruction in &message.instructions {
                    let program_id_index = instruction.program_id_index;
                    if let Some(account_key) = message.account_keys.get(program_id_index as usize) {
                        let program_id = bs58::encode(account_key).into_string();
                        
                        if is_jupiter_program(&program_id) {
                            let transaction_signature = bs58::encode(&transaction.signatures[0]).into_string();
                            let timestamp = block.block_time.as_ref().map(|t| t.timestamp as u64).unwrap_or(0);
                            
                            // Create appropriate event based on program ID
                            if program_id.starts_with("JUP") {
                                // Jupiter swap event
                                swap_events.push(SwapEvent {
                                    transaction_signature: transaction_signature.clone(),
                                    user: "".to_string(),
                                    input_mint: "".to_string(),
                                    output_mint: "".to_string(),
                                    input_amount: 0,
                                    output_amount: 0,
                                    minimum_amount_out: 0,
                                    price_impact_pips: 0,
                                    routes: vec![],
                                    program_id: program_id.clone(),
                                    slot: block.slot,
                                    timestamp,
                                    version: get_jupiter_version(&program_id),
                                    status: SwapStatus::Success as i32,
                                    error_message: "".to_string(),
                                });
                            } else if program_id == "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu" {
                                // Jupiter limit order event
                                limit_order_events.push(LimitOrderEvent {
                                    transaction_signature: transaction_signature.clone(),
                                    user: "".to_string(),
                                    order_id: "".to_string(),
                                    input_mint: "".to_string(),
                                    output_mint: "".to_string(),
                                    input_amount: 0,
                                    output_amount: 0,
                                    price: 0,
                                    order_type: LimitOrderType::Buy as i32,
                                    status: LimitOrderStatus::Pending as i32,
                                    slot: block.slot,
                                    timestamp,
                                    error_message: "".to_string(),
                                });
                            } else if program_id == "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M" {
                                // Jupiter DCA event
                                dca_events.push(DcaEvent {
                                    transaction_signature: transaction_signature.clone(),
                                    user: "".to_string(),
                                    dca_id: "".to_string(),
                                    input_mint: "".to_string(),
                                    output_mint: "".to_string(),
                                    amount_per_interval: 0,
                                    interval_seconds: 0,
                                    next_execution: 0,
                                    status: DcaStatus::Active as i32,
                                    slot: block.slot,
                                    timestamp,
                                    error_message: "".to_string(),
                                });
                            }
                            
                            // Create aggregation event for routing decisions
                            aggregation_events.push(AggregationEvent {
                                transaction_signature: transaction_signature.clone(),
                                user: "".to_string(),
                                available_dexs: vec!["Raydium".to_string(), "Orca".to_string()],
                                selected_route: "Jupiter".to_string(),
                                alternative_routes: vec![],
                                price_impact_pips: 0,
                                estimated_slippage: 0,
                                r#type: AggregationType::RouteSelection as i32,
                                slot: block.slot,
                                timestamp,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(JupiterEvents {
        swap_events,
        limit_order_events,
        dca_events,
        aggregation_events,
        block_number: block.slot,
        block_hash: bs58::encode(&block.blockhash).into_string(),
        timestamp: block.block_time.as_ref().map(|t| t.timestamp as u64).unwrap_or(0),
    })
}

// Helper functions
fn is_jupiter_program(program_id: &str) -> bool {
    matches!(program_id, 
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" | 
        "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB" |
        "JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph" |
        "JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo" |
        "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu" |
        "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M"
    )
}

fn get_jupiter_version(program_id: &str) -> String {
    match program_id {
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => "v6".to_string(),
        "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB" => "v4".to_string(),
        "JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph" => "v2".to_string(),
        "JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo" => "v1".to_string(),
        _ => "unknown".to_string(),
    }
}