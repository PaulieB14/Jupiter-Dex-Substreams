use crate::constants::TOKEN_PROGRAM_ID;
use crate::pb::sf::jupiter::v1::{AccountOwnerRecord, AccountOwnerRecords};
use substreams::errors::Error;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

#[substreams::handlers::map]
pub fn map_spl_initialized_account(block: Block) -> Result<AccountOwnerRecords, Error> {
    let mut records = Vec::new();

    for trx in block.transactions() {
        for instruction in trx.walk_instructions() {
            if instruction.program_id().to_string() != TOKEN_PROGRAM_ID {
                continue;
            }

            let accounts = instruction.accounts();
            if accounts.len() < 3 {
                continue;
            }

            records.push(AccountOwnerRecord {
                account: accounts[0].as_ref().to_vec(),
                mint: accounts[1].as_ref().to_vec(),
                owner: accounts[2].as_ref().to_vec(),
            });
        }
    }

    Ok(AccountOwnerRecords { records })
}