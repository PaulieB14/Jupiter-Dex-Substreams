//! Database sink module for Jupiter DEX Substreams
//!
//! Transforms Jupiter analytics data into DatabaseChanges for SQL sinks.
//! Supports both PostgreSQL and ClickHouse via the substreams-sink-sql tool.

use crate::pb::sf::jupiter::v1::{JupiterAnalytics, TradingDataList};
use substreams::errors::Error;
use substreams_database_change::pb::sf::substreams::sink::database::v1::DatabaseChanges;
use substreams_database_change::tables::Tables;

/// Transform Jupiter trading data and analytics into database changes
///
/// This module outputs CDC (Change Data Capture) records that can be consumed
/// by substreams-sink-sql to populate PostgreSQL or ClickHouse databases.
#[substreams::handlers::map]
pub fn db_out(
    trading_data: TradingDataList,
    analytics: JupiterAnalytics,
) -> Result<DatabaseChanges, Error> {
    let mut tables = Tables::new();

    // Process individual swap events
    for trade in &trading_data.items {
        // Skip trades with no amount (non-swap instructions)
        if trade.amount_in == 0 {
            continue;
        }

        // Create unique swap ID: transaction_id:slot
        let swap_id = format!("{}:{}", trade.transaction_id, trade.slot);

        tables
            .create_row("jupiter_swaps", &swap_id)
            .set("tx_hash", &trade.transaction_id)
            .set("program_id", &trade.program_id)
            .set("slot", trade.slot)
            .set("block_time", trade.block_time)
            .set("amount_in", trade.amount_in.to_string())
            .set("amount_out", trade.amount_out.to_string())
            .set("input_mint", &trade.input_mint)
            .set("output_mint", &trade.output_mint)
            .set("user_wallet", &trade.user_wallet);
    }

    // Update daily statistics (using upsert with delta operations)
    if trading_data.swap_count > 0 {
        // Extract date from first trade's block_time (Unix timestamp)
        let date = if let Some(first_trade) = trading_data.items.first() {
            format_date(first_trade.block_time)
        } else {
            "unknown".to_string()
        };

        // Aggregate by date
        tables
            .upsert_row("daily_swap_stats", &date)
            .set("date", &date)
            .add("swap_count", trading_data.swap_count as i64)
            .add("total_volume", trading_data.total_volume.to_string());
    }

    // Update program statistics
    for program_stat in &analytics.top_programs {
        tables
            .upsert_row("program_stats", &program_stat.program_id)
            .set("program_id", &program_stat.program_id)
            .add("instruction_count", program_stat.instruction_count as i64)
            .add("total_volume", program_stat.total_volume.to_string());
    }

    // Update global metrics
    if analytics.total_swaps > 0 {
        tables
            .upsert_row("global_metrics", "jupiter")
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
    // Simple date extraction without external dependencies
    // Days since Unix epoch
    let days = timestamp / 86400;

    // Calculate year, month, day using a simplified algorithm
    let mut year = 1970;
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

    let mut month = 1;
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
        // Unix timestamp for 2024-01-15 00:00:00 UTC
        let ts = 1705276800;
        assert_eq!(format_date(ts), "2024-01-15");

        // Unix timestamp for 2020-02-29 (leap year)
        let ts_leap = 1582934400;
        assert_eq!(format_date(ts_leap), "2020-02-29");

        // Unix timestamp for 1970-01-01 (epoch)
        assert_eq!(format_date(0), "1970-01-01");
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2020)); // Divisible by 4, not by 100
        assert!(is_leap_year(2000)); // Divisible by 400
        assert!(!is_leap_year(2100)); // Divisible by 100 but not 400
        assert!(!is_leap_year(2023)); // Not divisible by 4
    }
}
