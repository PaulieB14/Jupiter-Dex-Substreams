use std::collections::{HashMap, HashSet};

use crate::pb::sf::jupiter::v1::{JupiterAnalytics, JupiterInstructions, ProgramStat};
use substreams::errors::Error;

#[substreams::handlers::map]
pub fn map_jupiter_analytics(instructions: JupiterInstructions) -> Result<JupiterAnalytics, Error> {
    let mut account_set = HashSet::new();
    let mut mint_set = HashSet::new();
    let mut program_stats: HashMap<String, (u64, u64)> = HashMap::new(); // (count, volume)
    let mut total_volume: u64 = 0;
    let mut total_swaps: u64 = 0;

    for instruction in instructions.instructions.iter() {
        let entry = program_stats
            .entry(instruction.program_id.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        entry.1 = entry.1.saturating_add(instruction.amount_in);

        if instruction.amount_in > 0 {
            total_volume = total_volume.saturating_add(instruction.amount_in);
            total_swaps += 1;
        }

        for account in instruction.accounts.iter() {
            account_set.insert(account.address.clone());
            if !account.mint.is_empty() {
                mint_set.insert(account.mint.clone());
            }
        }
    }

    let mut top_programs = program_stats
        .into_iter()
        .map(|(program_id, (instruction_count, volume))| ProgramStat {
            program_id,
            instruction_count,
            total_volume: volume,
        })
        .collect::<Vec<_>>();
    top_programs.sort_by(|a, b| b.instruction_count.cmp(&a.instruction_count));
    top_programs.truncate(5);

    Ok(JupiterAnalytics {
        total_instructions: instructions.instructions.len() as u64,
        unique_accounts: account_set.len() as u64,
        unique_mints: mint_set.len() as u64,
        top_programs,
        total_volume,
        total_swaps,
    })
}
