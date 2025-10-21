use substreams::prelude::*;
use substreams_solana::pb::sf::solana::r#type::v1::Block;
use substreams_solana::pb::sf::solana::r#type::v1::ConfirmedTransaction;
use substreams_solana::pb::sf::solana::r#type::v1::Transaction;
use substreams_solana::pb::sf::solana::r#type::v1::UiTransaction;
use substreams_solana::pb::sf::solana::spl::token::v1::TokenInstruction;
use substreams_solana::pb::sf::substreams::foundational_store::v1::{Entries, Entry};
use substreams_solana::pb::sf::substreams::solana::spl::v1::AccountOwner;
use prost::Message;
use prost_types::Any;

#[substreams::handlers::map]
fn map_spl_initialized_account(
    transactions: substreams_solana::pb::sf::solana::r#type::v1::Transactions,
) -> Result<Entries, Error> {
    let mut entries: Vec<Entry> = Vec::new();

    for confirmed_trx in successful_transactions(&transactions) {
        for instruction in confirmed_trx.walk_instructions() {
            if let Ok(token_instruction) = TokenInstruction::unpack(&instruction.data()) {
                match token_instruction {
                    TokenInstruction::InitializeAccount {} => {
                        let account_owner = AccountOwner {
                            mint_address: instruction.accounts()[1].clone(),
                            owner: instruction.accounts()[2].clone(),
                        };

                        let mut buf = Vec::new();
                        account_owner.encode(&mut buf)?;

                        let entry = Entry {
                            key: instruction.accounts()[0].clone(),
                            value: Some(Any {
                                type_url: "type.googleapis.com/sf.substreams.solana.spl.v1.AccountOwner".to_string(),
                                value: buf,
                            }),
                        };
                        entries.push(entry);
                    }
                    TokenInstruction::InitializeAccount2 { owner } => {
                        let account_owner = AccountOwner {
                            mint_address: instruction.accounts()[1].clone(),
                            owner: owner.clone(),
                        };

                        let mut buf = Vec::new();
                        account_owner.encode(&mut buf)?;

                        let entry = Entry {
                            key: instruction.accounts()[0].clone(),
                            value: Some(Any {
                                type_url: "type.googleapis.com/sf.substreams.solana.spl.v1.AccountOwner".to_string(),
                                value: buf,
                            }),
                        };
                        entries.push(entry);
                    }
                    TokenInstruction::InitializeAccount3 { owner } => {
                        let account_owner = AccountOwner {
                            mint_address: instruction.accounts()[1].clone(),
                            owner: owner.clone(),
                        };

                        let mut buf = Vec::new();
                        account_owner.encode(&mut buf)?;

                        let entry = Entry {
                            key: instruction.accounts()[0].clone(),
                            value: Some(Any {
                                type_url: "type.googleapis.com/sf.substreams.solana.spl.v1.AccountOwner".to_string(),
                                value: buf,
                            }),
                        };
                        entries.push(entry);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(Entries { entries })
}

fn successful_transactions(transactions: &substreams_solana::pb::sf::solana::r#type::v1::Transactions) -> Vec<&ConfirmedTransaction> {
    transactions.transactions.iter()
        .filter_map(|tx| tx.transaction.as_ref())
        .filter_map(|tx| tx.meta.as_ref())
        .filter(|meta| meta.err.is_none())
        .map(|tx| tx)
        .collect()
}