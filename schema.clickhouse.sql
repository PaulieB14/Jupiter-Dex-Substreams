-- Jupiter DEX Substreams ClickHouse Schema
-- Version: 0.4.0
-- Supports: substreams-sink-sql with ClickHouse
-- Optimized for time-series analytics and high-throughput ingestion

-- Individual swap events (main fact table)
CREATE TABLE IF NOT EXISTS jupiter_swaps (
    id String,
    tx_hash String,
    program_id LowCardinality(String),
    slot UInt64,
    block_time DateTime64(0) CODEC(Delta, ZSTD),
    amount_in UInt256,
    amount_out UInt256,
    input_mint String,
    output_mint String,
    user_wallet String,
    date Date MATERIALIZED toDate(block_time)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (program_id, block_time, tx_hash)
SETTINGS index_granularity = 8192;

-- Daily aggregated statistics (using SummingMergeTree for efficient aggregation)
CREATE TABLE IF NOT EXISTS daily_swap_stats (
    date String,
    swap_count Int64,
    total_volume UInt256
) ENGINE = SummingMergeTree()
ORDER BY date;

-- Per-program statistics
CREATE TABLE IF NOT EXISTS program_stats (
    program_id LowCardinality(String),
    instruction_count Int64,
    total_volume UInt256
) ENGINE = SummingMergeTree()
ORDER BY program_id;

-- Global protocol metrics (using ReplacingMergeTree for latest values)
CREATE TABLE IF NOT EXISTS global_metrics (
    protocol LowCardinality(String),
    total_swaps Int64,
    total_volume UInt256,
    unique_accounts UInt64,
    unique_mints UInt64,
    updated_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY protocol;

-- Materialized view: Hourly volume aggregation
CREATE MATERIALIZED VIEW IF NOT EXISTS hourly_swap_volume
ENGINE = SummingMergeTree()
ORDER BY (hour, program_id)
AS SELECT
    toStartOfHour(block_time) AS hour,
    program_id,
    count() AS swap_count,
    sum(amount_in) AS volume_in,
    sum(amount_out) AS volume_out
FROM jupiter_swaps
GROUP BY hour, program_id;

-- Materialized view: Token pair volumes
CREATE MATERIALIZED VIEW IF NOT EXISTS token_pair_volumes
ENGINE = SummingMergeTree()
ORDER BY (input_mint, output_mint)
AS SELECT
    input_mint,
    output_mint,
    count() AS swap_count,
    sum(amount_in) AS total_volume_in,
    sum(amount_out) AS total_volume_out
FROM jupiter_swaps
GROUP BY input_mint, output_mint;

-- Materialized view: User activity summary
CREATE MATERIALIZED VIEW IF NOT EXISTS user_activity
ENGINE = SummingMergeTree()
ORDER BY user_wallet
AS SELECT
    user_wallet,
    count() AS swap_count,
    sum(amount_in) AS total_volume,
    min(block_time) AS first_swap,
    max(block_time) AS last_swap
FROM jupiter_swaps
GROUP BY user_wallet;

-- Materialized view: Daily unique users
CREATE MATERIALIZED VIEW IF NOT EXISTS daily_unique_users
ENGINE = AggregatingMergeTree()
ORDER BY date
AS SELECT
    toDate(block_time) AS date,
    uniqState(user_wallet) AS unique_users,
    count() AS swap_count
FROM jupiter_swaps
GROUP BY date;
