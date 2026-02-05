//! Database sink module for Jupiter DEX Substreams
//!
//! Transforms Jupiter analytics data into DatabaseChanges for SQL sinks.
//! Supports both PostgreSQL and ClickHouse via the substreams-sink-sql tool.
//!
//! Features:
//! - Individual swap events
//! - OHLCV candles with delta updates (5min, 1hr, 4hr, 1day)
//! - Token pair statistics
//! - Trader activity tracking
//! - Protocol-wide metrics

use crate::pb::sf::jupiter::v1::{JupiterAnalytics, TradingDataList};
use substreams::errors::Error;
use substreams_database_change::pb::sf::substreams::sink::database::v1::DatabaseChanges;
use substreams_database_change::tables::Tables;

/// Candle intervals in seconds
const CANDLE_INTERVALS: [i64; 4] = [
    300,    // 5 minutes
    3600,   // 1 hour
    14400,  // 4 hours
    86400,  // 1 day
];

/// Transform Jupiter trading data and analytics into database changes
///
/// This module outputs CDC (Change Data Capture) records that can be consumed
/// by substreams-sink-sql to populate PostgreSQL or ClickHouse databases.
///
/// Uses delta operations for efficient aggregations:
/// - `set_if_null`: open price (first trade wins)
/// - `set`: close price (always overwrites with latest)
/// - `max`/`min`: high/low price tracking
/// - `add`: volume and trade count accumulation
#[substreams::handlers::map]
pub fn db_out(
    trading_data: TradingDataList,
    analytics: JupiterAnalytics,
) -> Result<DatabaseChanges, Error> {
    let mut tables = Tables::new();

    // Process individual swap events and candles
    for trade in &trading_data.items {
        // Skip trades with no amount (non-swap instructions)
        if trade.amount_in == 0 {
            continue;
        }

        // Create unique swap ID: transaction_id:slot:program
        let swap_id = format!("{}:{}:{}", trade.transaction_id, trade.slot, trade.program_id);

        // Insert individual swap record
        tables
            .create_row("jupiter_swaps", &swap_id)
            .set("tx_hash", &trade.transaction_id)
            .set("program_id", &trade.program_id)
            .set("slot", trade.slot)
            .set("block_time", trade.block_time as i64)
            .set("amount_in", trade.amount_in.to_string())
            .set("amount_out", trade.amount_out.to_string())
            .set("input_mint", &trade.input_mint)
            .set("output_mint", &trade.output_mint)
            .set("user_wallet", &trade.user_wallet);

        // Calculate "price" as ratio (amount_out / amount_in scaled by 1e6)
        // This gives us a relative price for candle tracking
        let price_ratio = if trade.amount_in > 0 {
            ((trade.amount_out as f64 / trade.amount_in as f64) * 1_000_000.0) as i64
        } else {
            0
        };

        // Build candles for each trading pair at multiple intervals
        let pair_id = format!("{}:{}", trade.input_mint, trade.output_mint);
        let timestamp = trade.block_time as i64;

        for &interval in &CANDLE_INTERVALS {
            let window_start = (timestamp / interval) * interval;

            tables
                .upsert_row(
                    "candles",
                    [
                        ("pair_id", pair_id.clone()),
                        ("interval_seconds", interval.to_string()),
                        ("timestamp", window_start.to_string()),
                    ],
                )
                .set("input_mint", &trade.input_mint)
                .set("output_mint", &trade.output_mint)
                .set_if_null("open", price_ratio)
                .set("close", price_ratio)
                .max("high", price_ratio)
                .min("low", price_ratio)
                .add("volume_in", trade.amount_in.to_string())
                .add("volume_out", trade.amount_out.to_string())
                .add("trade_count", 1i64);
        }

        // Update token pair statistics using delta operations
        tables
            .upsert_row("token_pairs", &pair_id)
            .set("input_mint", &trade.input_mint)
            .set("output_mint", &trade.output_mint)
            .add("swap_count", 1i64)
            .add("total_volume_in", trade.amount_in.to_string())
            .add("total_volume_out", trade.amount_out.to_string())
            .set("last_swap_slot", trade.slot)
            .set("last_swap_time", trade.block_time as i64);

        // Update token statistics (input token)
        if !trade.input_mint.is_empty() {
            tables
                .upsert_row("token_stats", &trade.input_mint)
                .set("mint_address", &trade.input_mint)
                .add("total_swaps_as_input", 1i64)
                .add("total_volume_as_input", trade.amount_in.to_string())
                .set("last_seen_slot", trade.slot);
        }

        // Update token statistics (output token)
        if !trade.output_mint.is_empty() {
            tables
                .upsert_row("token_stats", &trade.output_mint)
                .set("mint_address", &trade.output_mint)
                .add("total_swaps_as_output", 1i64)
                .add("total_volume_as_output", trade.amount_out.to_string())
                .set("last_seen_slot", trade.slot);
        }

        // Update trader (wallet) statistics
        if !trade.user_wallet.is_empty() {
            tables
                .upsert_row("trader_stats", &trade.user_wallet)
                .set("wallet_address", &trade.user_wallet)
                .add("total_swaps", 1i64)
                .add("total_volume", trade.amount_in.to_string())
                .set("last_swap_slot", trade.slot)
                .set("last_swap_time", trade.block_time as i64);
        }
    }

    // Update daily statistics (using upsert with delta operations)
    if trading_data.swap_count > 0 {
        let date = if let Some(first_trade) = trading_data.items.first() {
            format_date(first_trade.block_time)
        } else {
            "unknown".to_string()
        };

        tables
            .upsert_row("daily_stats", &date)
            .set("date", &date)
            .add("swap_count", trading_data.swap_count as i64)
            .add("total_volume", trading_data.total_volume.to_string());
    }

    // Update hourly statistics
    if trading_data.swap_count > 0 {
        if let Some(first_trade) = trading_data.items.first() {
            let hour = format_hour(first_trade.block_time);
            tables
                .upsert_row("hourly_stats", &hour)
                .set("hour", &hour)
                .add("swap_count", trading_data.swap_count as i64)
                .add("total_volume", trading_data.total_volume.to_string());
        }
    }

    // Update program statistics
    for program_stat in &analytics.top_programs {
        tables
            .upsert_row("program_stats", &program_stat.program_id)
            .set("program_id", &program_stat.program_id)
            .add("instruction_count", program_stat.instruction_count as i64)
            .add("total_volume", program_stat.total_volume.to_string());
    }

    // Update global protocol metrics
    if analytics.total_swaps > 0 {
        tables
            .upsert_row("protocol_metrics", "jupiter")
            .set("protocol", "jupiter")
            .add("total_swaps", analytics.total_swaps as i64)
            .add("total_volume", analytics.total_volume.to_string())
            .max("unique_accounts", analytics.unique_accounts as i64)
            .max("unique_mints", analytics.unique_mints as i64);
    }

    Ok(tables.to_database_changes())
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
        assert_eq!(format_date(1705276800), "2024-01-15");
        assert_eq!(format_date(1582934400), "2020-02-29");
        assert_eq!(format_date(0), "1970-01-01");
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

    #[test]
    fn test_candle_intervals() {
        assert_eq!(CANDLE_INTERVALS[0], 300);   // 5 min
        assert_eq!(CANDLE_INTERVALS[1], 3600);  // 1 hour
        assert_eq!(CANDLE_INTERVALS[2], 14400); // 4 hours
        assert_eq!(CANDLE_INTERVALS[3], 86400); // 1 day
    }

    #[test]
    fn test_price_ratio_calculation() {
        let amount_in = 1_000_000u64;
        let amount_out = 500_000u64;
        let price_ratio = ((amount_out as f64 / amount_in as f64) * 1_000_000.0) as i64;
        assert_eq!(price_ratio, 500_000);
    }

    #[test]
    fn test_candle_window_calculation() {
        let timestamp: i64 = 1705276800; // 2024-01-15 00:00:00
        let interval: i64 = 3600; // 1 hour
        let window_start = (timestamp / interval) * interval;
        assert_eq!(window_start, 1705276800);

        // Test with offset timestamp
        let timestamp2: i64 = 1705278000; // 2024-01-15 00:20:00
        let window_start2 = (timestamp2 / interval) * interval;
        assert_eq!(window_start2, 1705276800); // Same hour window
    }
}
