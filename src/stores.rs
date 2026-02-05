//! Store modules for Jupiter DEX Substreams
//!
//! Provides persistent state tracking across blocks for:
//! - Cumulative swap volumes by token pair
//! - Unique trader (wallet) tracking
//! - Daily/hourly aggregations
//! - Token statistics

use crate::pb::sf::jupiter::v1::TradingDataList;
use substreams::scalar::BigInt;
use substreams::store::{StoreAdd, StoreAddBigInt, StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsString};

/// Store handler for tracking cumulative swap volumes by token pair
///
/// Key format: `{input_mint}:{output_mint}`
/// Value: Cumulative volume in input token units (bigint)
#[substreams::handlers::store]
pub fn store_swap_volumes(trading_data: TradingDataList, store: StoreAddBigInt) {
    for trade in &trading_data.items {
        // Skip non-swap instructions
        if trade.amount_in == 0 {
            continue;
        }

        // Store volume by trading pair
        let pair_key = format!("pair:{}:{}", trade.input_mint, trade.output_mint);
        store.add(0, &pair_key, &BigInt::from(trade.amount_in));

        // Store volume by input token
        let input_key = format!("token:volume_in:{}", trade.input_mint);
        store.add(0, &input_key, &BigInt::from(trade.amount_in));

        // Store volume by output token
        let output_key = format!("token:volume_out:{}", trade.output_mint);
        store.add(0, &output_key, &BigInt::from(trade.amount_out));

        // Store total protocol volume
        store.add(0, "total:volume", &BigInt::from(trade.amount_in));

        // Store swap count
        store.add(0, "total:swap_count", &BigInt::from(1u64));

        // Store daily volume (key includes date)
        let date = format_date(trade.block_time);
        let daily_key = format!("daily:{}:volume", date);
        store.add(0, &daily_key, &BigInt::from(trade.amount_in));

        let daily_count_key = format!("daily:{}:count", date);
        store.add(0, &daily_count_key, &BigInt::from(1u64));

        // Store hourly volume
        let hour = format_hour(trade.block_time);
        let hourly_key = format!("hourly:{}:volume", hour);
        store.add(0, &hourly_key, &BigInt::from(trade.amount_in));

        // Store volume by program
        let program_key = format!("program:{}:volume", trade.program_id);
        store.add(0, &program_key, &BigInt::from(trade.amount_in));

        let program_count_key = format!("program:{}:count", trade.program_id);
        store.add(0, &program_count_key, &BigInt::from(1u64));
    }
}

/// Store handler for tracking unique traders (wallets)
///
/// Key format: `trader:{wallet_address}`
/// Value: First seen slot (stored only if not exists)
#[substreams::handlers::store]
pub fn store_unique_traders(trading_data: TradingDataList, store: StoreSetIfNotExistsString) {
    for trade in &trading_data.items {
        // Skip if no wallet info
        if trade.user_wallet.is_empty() {
            continue;
        }

        // Track unique trader with first seen slot
        let trader_key = format!("trader:{}", trade.user_wallet);
        let value = format!("{}:{}", trade.slot, trade.block_time);
        store.set_if_not_exists(0, &trader_key, &value);

        // Track daily unique traders
        let date = format_date(trade.block_time);
        let daily_trader_key = format!("daily:{}:trader:{}", date, trade.user_wallet);
        store.set_if_not_exists(0, &daily_trader_key, &trade.slot.to_string());

        // Track traders per token
        if !trade.input_mint.is_empty() {
            let token_trader_key = format!("token:{}:trader:{}", trade.input_mint, trade.user_wallet);
            store.set_if_not_exists(0, &token_trader_key, &trade.slot.to_string());
        }
    }
}

/// Store handler for tracking token statistics
///
/// Tracks first/last seen, trade counts, and volume per token
#[substreams::handlers::store]
pub fn store_token_stats(trading_data: TradingDataList, store: StoreAddBigInt) {
    for trade in &trading_data.items {
        if trade.amount_in == 0 {
            continue;
        }

        // Track input token stats
        if !trade.input_mint.is_empty() {
            let count_key = format!("token:{}:trade_count", trade.input_mint);
            store.add(0, &count_key, &BigInt::from(1u64));
        }

        // Track output token stats
        if !trade.output_mint.is_empty() {
            let count_key = format!("token:{}:trade_count", trade.output_mint);
            store.add(0, &count_key, &BigInt::from(1u64));
        }

        // Track unique pairs count
        let pair_key = format!("pairs:{}:{}", trade.input_mint, trade.output_mint);
        store.add(0, &pair_key, &BigInt::from(1u64));
    }
}

/// Format Unix timestamp to YYYY-MM-DD date string
fn format_date(timestamp: u64) -> String {
    let days = timestamp / 86400;
    let mut year = 1970u64;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// Format Unix timestamp to YYYY-MM-DD-HH hour string
fn format_hour(timestamp: u64) -> String {
    let date = format_date(timestamp);
    let hour = (timestamp % 86400) / 3600;
    format!("{}-{:02}", date, hour)
}

/// Check if a year is a leap year
#[inline]
fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_date(1705276800), "2024-01-15");
        assert_eq!(format_date(1582934400), "2020-02-29");
    }

    #[test]
    fn test_format_hour() {
        assert_eq!(format_hour(0), "1970-01-01-00");
        assert_eq!(format_hour(3600), "1970-01-01-01");
        assert_eq!(format_hour(86399), "1970-01-01-23");
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(2023));
    }
}
