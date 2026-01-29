use std::collections::{HashMap, HashSet};

use crate::pb::sf::jupiter::v1::{JupiterAnalytics, JupiterInstructions, ProgramStat};
use substreams::errors::Error;

/// Aggregate Jupiter instructions into analytics metrics
#[substreams::handlers::map]
pub fn map_jupiter_analytics(instructions: JupiterInstructions) -> Result<JupiterAnalytics, Error> {
    Ok(compute_analytics(instructions))
}

/// Core analytics computation logic (extracted for testability)
fn compute_analytics(instructions: JupiterInstructions) -> JupiterAnalytics {
    let instruction_count = instructions.instructions.len();

    // Pre-allocate collections with estimated capacity
    let mut account_set: HashSet<String> = HashSet::with_capacity(instruction_count * 4);
    let mut mint_set: HashSet<String> = HashSet::with_capacity(instruction_count);
    let mut program_stats: HashMap<String, (u64, u64)> = HashMap::with_capacity(6); // 6 Jupiter programs
    let mut total_volume: u64 = 0;
    let mut total_swaps: u64 = 0;

    for instruction in &instructions.instructions {
        // Update program stats using entry API to avoid double lookup
        program_stats
            .entry(instruction.program_id.clone())
            .and_modify(|(count, vol)| {
                *count += 1;
                *vol = vol.saturating_add(instruction.amount_in);
            })
            .or_insert((1, instruction.amount_in));

        if instruction.amount_in > 0 {
            total_volume = total_volume.saturating_add(instruction.amount_in);
            total_swaps += 1;
        }

        // Collect unique accounts and mints
        for account in &instruction.accounts {
            account_set.insert(account.address.clone());
            if !account.mint.is_empty() {
                mint_set.insert(account.mint.clone());
            }
        }
    }

    // Convert to sorted list of top programs
    let mut top_programs: Vec<ProgramStat> = program_stats
        .into_iter()
        .map(|(program_id, (count, volume))| ProgramStat {
            program_id,
            instruction_count: count,
            total_volume: volume,
        })
        .collect();

    // Sort by instruction count descending
    top_programs.sort_unstable_by(|a, b| b.instruction_count.cmp(&a.instruction_count));
    top_programs.truncate(5);

    JupiterAnalytics {
        total_instructions: instruction_count as u64,
        unique_accounts: account_set.len() as u64,
        unique_mints: mint_set.len() as u64,
        top_programs,
        total_volume,
        total_swaps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::sf::jupiter::v1::{EnrichedAccount, JupiterInstruction};

    fn create_test_instruction(program_id: &str, amount_in: u64) -> JupiterInstruction {
        JupiterInstruction {
            program_id: program_id.to_string(),
            transaction_id: "test_tx_123".to_string(),
            accounts: vec![
                EnrichedAccount {
                    address: "account1".to_string(),
                    owner: "owner1".to_string(),
                    mint: "mint1".to_string(),
                },
                EnrichedAccount {
                    address: "account2".to_string(),
                    owner: "".to_string(),
                    mint: "".to_string(),
                },
            ],
            data: vec![],
            slot: 1000,
            block_time: 1700000000,
            amount_in,
            amount_out: amount_in / 2,
            input_mint: "input_mint".to_string(),
            output_mint: "output_mint".to_string(),
        }
    }

    #[test]
    fn test_analytics_empty_instructions() {
        let instructions = JupiterInstructions {
            instructions: vec![],
            total_volume: 0,
            instruction_count: 0,
        };

        let result = compute_analytics(instructions);

        assert_eq!(result.total_instructions, 0);
        assert_eq!(result.unique_accounts, 0);
        assert_eq!(result.unique_mints, 0);
        assert_eq!(result.total_volume, 0);
        assert_eq!(result.total_swaps, 0);
        assert!(result.top_programs.is_empty());
    }

    #[test]
    fn test_analytics_single_instruction() {
        let instructions = JupiterInstructions {
            instructions: vec![create_test_instruction(
                "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
                1_000_000,
            )],
            total_volume: 1_000_000,
            instruction_count: 1,
        };

        let result = compute_analytics(instructions);

        assert_eq!(result.total_instructions, 1);
        assert_eq!(result.unique_accounts, 2);
        assert_eq!(result.unique_mints, 1); // Only account1 has a mint
        assert_eq!(result.total_volume, 1_000_000);
        assert_eq!(result.total_swaps, 1);
        assert_eq!(result.top_programs.len(), 1);
        assert_eq!(
            result.top_programs[0].program_id,
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        );
    }

    #[test]
    fn test_analytics_multiple_programs() {
        let instructions = JupiterInstructions {
            instructions: vec![
                create_test_instruction("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", 1_000_000),
                create_test_instruction("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", 2_000_000),
                create_test_instruction("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB", 500_000),
            ],
            total_volume: 3_500_000,
            instruction_count: 3,
        };

        let result = compute_analytics(instructions);

        assert_eq!(result.total_instructions, 3);
        assert_eq!(result.total_volume, 3_500_000);
        assert_eq!(result.total_swaps, 3);
        assert_eq!(result.top_programs.len(), 2);

        // Jupiter v6 should be first (2 instructions)
        assert_eq!(
            result.top_programs[0].program_id,
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        );
        assert_eq!(result.top_programs[0].instruction_count, 2);
        assert_eq!(result.top_programs[0].total_volume, 3_000_000);
    }

    #[test]
    fn test_analytics_zero_amount_not_counted_as_swap() {
        let instructions = JupiterInstructions {
            instructions: vec![
                create_test_instruction("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", 0),
                create_test_instruction("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", 1_000_000),
            ],
            total_volume: 1_000_000,
            instruction_count: 2,
        };

        let result = compute_analytics(instructions);

        assert_eq!(result.total_instructions, 2);
        assert_eq!(result.total_swaps, 1); // Only one has amount > 0
        assert_eq!(result.total_volume, 1_000_000);
    }
}
