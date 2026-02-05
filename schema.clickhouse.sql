-- Jupiter DEX Substreams ClickHouse Schema
-- Version: 0.5.0
-- Supports: substreams-sink-sql with ClickHouse
-- Optimized for time-series analytics and high-throughput ingestion
-- Features: OHLCV Candles, Token Stats, Trader Analytics

--------------------------------------------------------------------------------
-- CORE TABLES
--------------------------------------------------------------------------------

-- Individual swap events (main fact table)
CREATE TABLE IF NOT EXISTS jupiter_swaps (
    id String,
    tx_hash String,
    program_id LowCardinality(String),
    slot UInt64,
    block_time Int64,
    amount_in UInt256,
    amount_out UInt256,
    input_mint String,
    output_mint String,
    user_wallet String,
    date Date MATERIALIZED toDate(fromUnixTimestamp(block_time))
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (program_id, block_time, tx_hash)
SETTINGS index_granularity = 8192;

--------------------------------------------------------------------------------
-- OHLCV CANDLES
--------------------------------------------------------------------------------

-- Candle data for trading pairs with multiple intervals
-- Uses ReplacingMergeTree with delta-like operations through queries
CREATE TABLE IF NOT EXISTS candles (
    pair_id String,
    interval_seconds Int64,
    timestamp Int64,
    input_mint String,
    output_mint String,
    open Int64,
    close Int64,
    high Int64,
    low Int64,
    volume_in UInt256,
    volume_out UInt256,
    trade_count Int64,
    updated_at DateTime64(3) DEFAULT now64()
) ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(toDate(fromUnixTimestamp(timestamp)))
ORDER BY (pair_id, interval_seconds, timestamp)
SETTINGS index_granularity = 8192;

--------------------------------------------------------------------------------
-- TOKEN PAIR STATISTICS
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS token_pairs (
    pair_id String,
    input_mint String,
    output_mint String,
    swap_count Int64,
    total_volume_in UInt256,
    total_volume_out UInt256,
    last_swap_slot UInt64,
    last_swap_time Int64,
    updated_at DateTime64(3) DEFAULT now64()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY pair_id;

--------------------------------------------------------------------------------
-- TOKEN STATISTICS
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS token_stats (
    mint_address String,
    total_swaps_as_input Int64,
    total_swaps_as_output Int64,
    total_volume_as_input UInt256,
    total_volume_as_output UInt256,
    last_seen_slot UInt64,
    updated_at DateTime64(3) DEFAULT now64()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY mint_address;

--------------------------------------------------------------------------------
-- TRADER STATISTICS
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS trader_stats (
    wallet_address String,
    total_swaps Int64,
    total_volume UInt256,
    last_swap_slot UInt64,
    last_swap_time Int64,
    updated_at DateTime64(3) DEFAULT now64()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY wallet_address;

--------------------------------------------------------------------------------
-- TIME-BASED AGGREGATIONS
--------------------------------------------------------------------------------

-- Daily statistics (SummingMergeTree for efficient delta additions)
CREATE TABLE IF NOT EXISTS daily_stats (
    date String,
    swap_count Int64,
    total_volume UInt256
) ENGINE = SummingMergeTree()
ORDER BY date;

-- Hourly statistics
CREATE TABLE IF NOT EXISTS hourly_stats (
    hour String,
    swap_count Int64,
    total_volume UInt256
) ENGINE = SummingMergeTree()
ORDER BY hour;

--------------------------------------------------------------------------------
-- PROGRAM STATISTICS
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS program_stats (
    program_id LowCardinality(String),
    instruction_count Int64,
    total_volume UInt256
) ENGINE = SummingMergeTree()
ORDER BY program_id;

--------------------------------------------------------------------------------
-- PROTOCOL METRICS
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS protocol_metrics (
    protocol LowCardinality(String),
    total_swaps Int64,
    total_volume UInt256,
    unique_accounts Int64,
    unique_mints Int64,
    updated_at DateTime64(3) DEFAULT now64()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY protocol;

--------------------------------------------------------------------------------
-- MATERIALIZED VIEWS (Real-time Aggregations)
--------------------------------------------------------------------------------

-- Materialized view: Hourly volume aggregation from raw swaps
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_hourly_swap_volume
ENGINE = SummingMergeTree()
ORDER BY (hour, program_id)
AS SELECT
    toStartOfHour(fromUnixTimestamp(block_time)) AS hour,
    program_id,
    count() AS swap_count,
    sum(amount_in) AS volume_in,
    sum(amount_out) AS volume_out
FROM jupiter_swaps
GROUP BY hour, program_id;

-- Materialized view: Token pair volumes from raw swaps
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_token_pair_volumes
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_user_activity
ENGINE = AggregatingMergeTree()
ORDER BY user_wallet
AS SELECT
    user_wallet,
    countState() AS swap_count_state,
    sumState(amount_in) AS total_volume_state,
    minState(block_time) AS first_swap_state,
    maxState(block_time) AS last_swap_state
FROM jupiter_swaps
GROUP BY user_wallet;

-- Materialized view: Daily unique users
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_unique_users
ENGINE = AggregatingMergeTree()
ORDER BY date
AS SELECT
    toDate(fromUnixTimestamp(block_time)) AS date,
    uniqState(user_wallet) AS unique_users_state,
    countState() AS swap_count_state
FROM jupiter_swaps
GROUP BY date;

-- Materialized view: 5-minute candles from raw swaps
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_candles_5m
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(candle_time)
ORDER BY (pair_id, candle_time)
AS SELECT
    concat(input_mint, ':', output_mint) AS pair_id,
    input_mint,
    output_mint,
    toStartOfFiveMinutes(fromUnixTimestamp(block_time)) AS candle_time,
    argMinState(toInt64(amount_out * 1000000 / amount_in), block_time) AS open_state,
    argMaxState(toInt64(amount_out * 1000000 / amount_in), block_time) AS close_state,
    maxState(toInt64(amount_out * 1000000 / amount_in)) AS high_state,
    minState(toInt64(amount_out * 1000000 / amount_in)) AS low_state,
    sumState(amount_in) AS volume_in_state,
    sumState(amount_out) AS volume_out_state,
    countState() AS trade_count_state
FROM jupiter_swaps
WHERE amount_in > 0
GROUP BY pair_id, input_mint, output_mint, candle_time;

-- Materialized view: Hourly candles
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_candles_1h
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(candle_time)
ORDER BY (pair_id, candle_time)
AS SELECT
    concat(input_mint, ':', output_mint) AS pair_id,
    input_mint,
    output_mint,
    toStartOfHour(fromUnixTimestamp(block_time)) AS candle_time,
    argMinState(toInt64(amount_out * 1000000 / amount_in), block_time) AS open_state,
    argMaxState(toInt64(amount_out * 1000000 / amount_in), block_time) AS close_state,
    maxState(toInt64(amount_out * 1000000 / amount_in)) AS high_state,
    minState(toInt64(amount_out * 1000000 / amount_in)) AS low_state,
    sumState(amount_in) AS volume_in_state,
    sumState(amount_out) AS volume_out_state,
    countState() AS trade_count_state
FROM jupiter_swaps
WHERE amount_in > 0
GROUP BY pair_id, input_mint, output_mint, candle_time;

-- Materialized view: Daily candles
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_candles_1d
ENGINE = AggregatingMergeTree()
PARTITION BY toYear(candle_time)
ORDER BY (pair_id, candle_time)
AS SELECT
    concat(input_mint, ':', output_mint) AS pair_id,
    input_mint,
    output_mint,
    toStartOfDay(fromUnixTimestamp(block_time)) AS candle_time,
    argMinState(toInt64(amount_out * 1000000 / amount_in), block_time) AS open_state,
    argMaxState(toInt64(amount_out * 1000000 / amount_in), block_time) AS close_state,
    maxState(toInt64(amount_out * 1000000 / amount_in)) AS high_state,
    minState(toInt64(amount_out * 1000000 / amount_in)) AS low_state,
    sumState(amount_in) AS volume_in_state,
    sumState(amount_out) AS volume_out_state,
    countState() AS trade_count_state
FROM jupiter_swaps
WHERE amount_in > 0
GROUP BY pair_id, input_mint, output_mint, candle_time;

--------------------------------------------------------------------------------
-- QUERY HELPERS (Views)
--------------------------------------------------------------------------------

-- View: Finalized hourly candles
CREATE OR REPLACE VIEW v_candles_1h AS
SELECT
    pair_id,
    input_mint,
    output_mint,
    candle_time,
    argMinMerge(open_state) AS open,
    argMaxMerge(close_state) AS close,
    maxMerge(high_state) AS high,
    minMerge(low_state) AS low,
    sumMerge(volume_in_state) AS volume_in,
    sumMerge(volume_out_state) AS volume_out,
    countMerge(trade_count_state) AS trade_count
FROM mv_candles_1h
GROUP BY pair_id, input_mint, output_mint, candle_time
ORDER BY pair_id, candle_time;

-- View: Finalized daily candles
CREATE OR REPLACE VIEW v_candles_1d AS
SELECT
    pair_id,
    input_mint,
    output_mint,
    candle_time,
    argMinMerge(open_state) AS open,
    argMaxMerge(close_state) AS close,
    maxMerge(high_state) AS high,
    minMerge(low_state) AS low,
    sumMerge(volume_in_state) AS volume_in,
    sumMerge(volume_out_state) AS volume_out,
    countMerge(trade_count_state) AS trade_count
FROM mv_candles_1d
GROUP BY pair_id, input_mint, output_mint, candle_time
ORDER BY pair_id, candle_time;

-- View: User activity finalized
CREATE OR REPLACE VIEW v_user_activity AS
SELECT
    user_wallet,
    countMerge(swap_count_state) AS swap_count,
    sumMerge(total_volume_state) AS total_volume,
    minMerge(first_swap_state) AS first_swap,
    maxMerge(last_swap_state) AS last_swap
FROM mv_user_activity
GROUP BY user_wallet
ORDER BY total_volume DESC;

-- View: Daily unique users finalized
CREATE OR REPLACE VIEW v_daily_unique_users AS
SELECT
    date,
    uniqMerge(unique_users_state) AS unique_users,
    countMerge(swap_count_state) AS swap_count
FROM mv_daily_unique_users
GROUP BY date
ORDER BY date DESC;

-- View: Top trading pairs
CREATE OR REPLACE VIEW v_top_pairs AS
SELECT
    input_mint,
    output_mint,
    sum(swap_count) AS swap_count,
    sum(total_volume_in) AS total_volume_in,
    sum(total_volume_out) AS total_volume_out
FROM mv_token_pair_volumes
GROUP BY input_mint, output_mint
ORDER BY total_volume_in DESC
LIMIT 100;

-- View: Top tokens
CREATE OR REPLACE VIEW v_top_tokens AS
SELECT
    mint_address,
    total_swaps_as_input + total_swaps_as_output AS total_swaps,
    total_volume_as_input,
    total_volume_as_output,
    total_volume_as_input + total_volume_as_output AS total_volume
FROM token_stats FINAL
ORDER BY total_volume DESC
LIMIT 100;

-- View: Program distribution
CREATE OR REPLACE VIEW v_program_distribution AS
SELECT
    program_id,
    sum(instruction_count) AS instruction_count,
    sum(total_volume) AS total_volume,
    multiIf(
        program_id = 'JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4', 'Jupiter v6',
        program_id = 'JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB', 'Jupiter v4',
        program_id = 'JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph', 'Jupiter v3',
        program_id = 'JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo', 'Jupiter v2',
        program_id = 'jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu', 'Limit Orders',
        program_id = 'DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M', 'DCA',
        'Unknown'
    ) AS program_name
FROM program_stats
GROUP BY program_id
ORDER BY total_volume DESC;

-- View: Daily volume trend
CREATE OR REPLACE VIEW v_daily_volume_trend AS
SELECT
    date,
    sum(swap_count) AS swap_count,
    sum(total_volume) AS total_volume
FROM daily_stats
GROUP BY date
ORDER BY date DESC
LIMIT 30;

-- View: Hourly volume (last 24 hours)
CREATE OR REPLACE VIEW v_hourly_volume_24h AS
SELECT
    hour,
    sum(swap_count) AS swap_count,
    sum(total_volume) AS total_volume
FROM hourly_stats
WHERE hour >= toString(toStartOfHour(now() - INTERVAL 24 HOUR))
GROUP BY hour
ORDER BY hour DESC;
