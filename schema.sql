-- Jupiter DEX Substreams PostgreSQL Schema
-- Version: 0.5.0
-- Supports: substreams-sink-sql with PostgreSQL
-- Features: OHLCV Candles, Token Stats, Trader Analytics, Delta Updates

--------------------------------------------------------------------------------
-- CORE TABLES
--------------------------------------------------------------------------------

-- Individual swap events (fact table)
CREATE TABLE IF NOT EXISTS jupiter_swaps (
    id VARCHAR(256) PRIMARY KEY,
    tx_hash VARCHAR(88) NOT NULL,
    program_id VARCHAR(44) NOT NULL,
    slot BIGINT NOT NULL,
    block_time BIGINT NOT NULL,
    amount_in NUMERIC(78,0) NOT NULL,
    amount_out NUMERIC(78,0) NOT NULL,
    input_mint VARCHAR(44),
    output_mint VARCHAR(44),
    user_wallet VARCHAR(44),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for swap queries
CREATE INDEX IF NOT EXISTS idx_swaps_tx_hash ON jupiter_swaps(tx_hash);
CREATE INDEX IF NOT EXISTS idx_swaps_slot ON jupiter_swaps(slot DESC);
CREATE INDEX IF NOT EXISTS idx_swaps_block_time ON jupiter_swaps(block_time DESC);
CREATE INDEX IF NOT EXISTS idx_swaps_user ON jupiter_swaps(user_wallet);
CREATE INDEX IF NOT EXISTS idx_swaps_program ON jupiter_swaps(program_id);
CREATE INDEX IF NOT EXISTS idx_swaps_input_mint ON jupiter_swaps(input_mint);
CREATE INDEX IF NOT EXISTS idx_swaps_output_mint ON jupiter_swaps(output_mint);
CREATE INDEX IF NOT EXISTS idx_swaps_pair ON jupiter_swaps(input_mint, output_mint);

--------------------------------------------------------------------------------
-- OHLCV CANDLES (Delta Updates)
--------------------------------------------------------------------------------

-- Candle data for trading pairs with multiple intervals
-- Uses delta operations: set_if_null(open), set(close), max(high), min(low), add(volume)
CREATE TABLE IF NOT EXISTS candles (
    pair_id VARCHAR(128) NOT NULL,
    interval_seconds BIGINT NOT NULL,
    timestamp BIGINT NOT NULL,
    input_mint VARCHAR(44),
    output_mint VARCHAR(44),
    open BIGINT,
    close BIGINT,
    high BIGINT,
    low BIGINT,
    volume_in NUMERIC(78,0) DEFAULT 0,
    volume_out NUMERIC(78,0) DEFAULT 0,
    trade_count BIGINT DEFAULT 0,
    PRIMARY KEY (pair_id, interval_seconds, timestamp)
);

-- Indexes for candle queries
CREATE INDEX IF NOT EXISTS idx_candles_pair_time ON candles(pair_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_candles_interval ON candles(interval_seconds, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_candles_input_mint ON candles(input_mint);
CREATE INDEX IF NOT EXISTS idx_candles_output_mint ON candles(output_mint);

--------------------------------------------------------------------------------
-- TOKEN PAIR STATISTICS (Delta Updates)
--------------------------------------------------------------------------------

-- Aggregated statistics per trading pair
CREATE TABLE IF NOT EXISTS token_pairs (
    pair_id VARCHAR(128) PRIMARY KEY,
    input_mint VARCHAR(44),
    output_mint VARCHAR(44),
    swap_count BIGINT DEFAULT 0,
    total_volume_in NUMERIC(78,0) DEFAULT 0,
    total_volume_out NUMERIC(78,0) DEFAULT 0,
    last_swap_slot BIGINT,
    last_swap_time BIGINT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pairs_input ON token_pairs(input_mint);
CREATE INDEX IF NOT EXISTS idx_pairs_output ON token_pairs(output_mint);
CREATE INDEX IF NOT EXISTS idx_pairs_volume ON token_pairs(total_volume_in DESC);
CREATE INDEX IF NOT EXISTS idx_pairs_count ON token_pairs(swap_count DESC);

--------------------------------------------------------------------------------
-- TOKEN STATISTICS (Delta Updates)
--------------------------------------------------------------------------------

-- Per-token aggregated statistics
CREATE TABLE IF NOT EXISTS token_stats (
    mint_address VARCHAR(44) PRIMARY KEY,
    total_swaps_as_input BIGINT DEFAULT 0,
    total_swaps_as_output BIGINT DEFAULT 0,
    total_volume_as_input NUMERIC(78,0) DEFAULT 0,
    total_volume_as_output NUMERIC(78,0) DEFAULT 0,
    last_seen_slot BIGINT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_token_volume_in ON token_stats(total_volume_as_input DESC);
CREATE INDEX IF NOT EXISTS idx_token_swaps ON token_stats(total_swaps_as_input DESC);

--------------------------------------------------------------------------------
-- TRADER STATISTICS (Delta Updates)
--------------------------------------------------------------------------------

-- Per-wallet trading statistics
CREATE TABLE IF NOT EXISTS trader_stats (
    wallet_address VARCHAR(44) PRIMARY KEY,
    total_swaps BIGINT DEFAULT 0,
    total_volume NUMERIC(78,0) DEFAULT 0,
    last_swap_slot BIGINT,
    last_swap_time BIGINT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trader_swaps ON trader_stats(total_swaps DESC);
CREATE INDEX IF NOT EXISTS idx_trader_volume ON trader_stats(total_volume DESC);
CREATE INDEX IF NOT EXISTS idx_trader_last_active ON trader_stats(last_swap_time DESC);

--------------------------------------------------------------------------------
-- TIME-BASED AGGREGATIONS (Delta Updates)
--------------------------------------------------------------------------------

-- Daily aggregated statistics
CREATE TABLE IF NOT EXISTS daily_stats (
    date VARCHAR(10) PRIMARY KEY,
    swap_count BIGINT DEFAULT 0,
    total_volume NUMERIC(78,0) DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_daily_date ON daily_stats(date DESC);

-- Hourly aggregated statistics
CREATE TABLE IF NOT EXISTS hourly_stats (
    hour VARCHAR(13) PRIMARY KEY,
    swap_count BIGINT DEFAULT 0,
    total_volume NUMERIC(78,0) DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_hourly_hour ON hourly_stats(hour DESC);

--------------------------------------------------------------------------------
-- PROGRAM STATISTICS (Delta Updates)
--------------------------------------------------------------------------------

-- Per-program (Jupiter version) statistics
CREATE TABLE IF NOT EXISTS program_stats (
    program_id VARCHAR(44) PRIMARY KEY,
    instruction_count BIGINT DEFAULT 0,
    total_volume NUMERIC(78,0) DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- PROTOCOL METRICS (Delta Updates)
--------------------------------------------------------------------------------

-- Global protocol-wide metrics
CREATE TABLE IF NOT EXISTS protocol_metrics (
    protocol VARCHAR(32) PRIMARY KEY,
    total_swaps BIGINT DEFAULT 0,
    total_volume NUMERIC(78,0) DEFAULT 0,
    unique_accounts BIGINT DEFAULT 0,
    unique_mints BIGINT DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

--------------------------------------------------------------------------------
-- ANALYTICS VIEWS
--------------------------------------------------------------------------------

-- Latest candle prices (1-hour interval)
CREATE OR REPLACE VIEW latest_prices AS
SELECT DISTINCT ON (pair_id)
    pair_id,
    input_mint,
    output_mint,
    close as price,
    high,
    low,
    volume_in,
    trade_count,
    timestamp,
    TO_TIMESTAMP(timestamp) as candle_time
FROM candles
WHERE interval_seconds = 3600
ORDER BY pair_id, timestamp DESC;

-- Top tokens by 24h volume
CREATE OR REPLACE VIEW top_tokens_24h AS
SELECT
    mint_address,
    (total_swaps_as_input + total_swaps_as_output) as total_swaps,
    total_volume_as_input,
    total_volume_as_output,
    (total_volume_as_input + total_volume_as_output) as total_volume
FROM token_stats
ORDER BY total_volume DESC
LIMIT 100;

-- Top trading pairs by volume
CREATE OR REPLACE VIEW top_pairs AS
SELECT
    pair_id,
    input_mint,
    output_mint,
    swap_count,
    total_volume_in,
    total_volume_out,
    last_swap_time,
    TO_TIMESTAMP(last_swap_time) as last_swap_at
FROM token_pairs
ORDER BY total_volume_in DESC
LIMIT 100;

-- Top traders by volume
CREATE OR REPLACE VIEW top_traders AS
SELECT
    wallet_address,
    total_swaps,
    total_volume,
    last_swap_time,
    TO_TIMESTAMP(last_swap_time) as last_active
FROM trader_stats
ORDER BY total_volume DESC
LIMIT 100;

-- Daily volume trend
CREATE OR REPLACE VIEW daily_volume_trend AS
SELECT
    date,
    swap_count,
    total_volume,
    LAG(total_volume) OVER (ORDER BY date) as prev_volume,
    CASE
        WHEN LAG(total_volume) OVER (ORDER BY date) > 0
        THEN ((total_volume - LAG(total_volume) OVER (ORDER BY date))::NUMERIC / LAG(total_volume) OVER (ORDER BY date) * 100)
        ELSE 0
    END as volume_change_pct
FROM daily_stats
ORDER BY date DESC
LIMIT 30;

-- Hourly volume for last 24 hours
CREATE OR REPLACE VIEW hourly_volume_24h AS
SELECT
    hour,
    swap_count,
    total_volume
FROM hourly_stats
ORDER BY hour DESC
LIMIT 24;

-- Program distribution
CREATE OR REPLACE VIEW program_distribution AS
SELECT
    program_id,
    instruction_count,
    total_volume,
    CASE program_id
        WHEN 'JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4' THEN 'Jupiter v6'
        WHEN 'JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB' THEN 'Jupiter v4'
        WHEN 'JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph' THEN 'Jupiter v3'
        WHEN 'JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo' THEN 'Jupiter v2'
        WHEN 'jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu' THEN 'Limit Orders'
        WHEN 'DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M' THEN 'DCA'
        ELSE 'Unknown'
    END as program_name
FROM program_stats
ORDER BY total_volume DESC;

-- Candle chart data helper (5-minute candles, last 24 hours)
CREATE OR REPLACE VIEW candles_5m_24h AS
SELECT
    pair_id,
    input_mint,
    output_mint,
    timestamp,
    TO_TIMESTAMP(timestamp) as candle_time,
    open,
    high,
    low,
    close,
    volume_in,
    volume_out,
    trade_count
FROM candles
WHERE interval_seconds = 300
  AND timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '24 hours')
ORDER BY pair_id, timestamp;

-- Candle chart data helper (1-hour candles, last 7 days)
CREATE OR REPLACE VIEW candles_1h_7d AS
SELECT
    pair_id,
    input_mint,
    output_mint,
    timestamp,
    TO_TIMESTAMP(timestamp) as candle_time,
    open,
    high,
    low,
    close,
    volume_in,
    volume_out,
    trade_count
FROM candles
WHERE interval_seconds = 3600
  AND timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')
ORDER BY pair_id, timestamp;

-- Candle chart data helper (daily candles, last 30 days)
CREATE OR REPLACE VIEW candles_1d_30d AS
SELECT
    pair_id,
    input_mint,
    output_mint,
    timestamp,
    TO_TIMESTAMP(timestamp) as candle_time,
    open,
    high,
    low,
    close,
    volume_in,
    volume_out,
    trade_count
FROM candles
WHERE interval_seconds = 86400
  AND timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '30 days')
ORDER BY pair_id, timestamp;

--------------------------------------------------------------------------------
-- UTILITY FUNCTIONS
--------------------------------------------------------------------------------

-- Get candles for a specific pair and interval
-- Usage: SELECT * FROM get_candles('mint1:mint2', 3600, 100);
CREATE OR REPLACE FUNCTION get_candles(
    p_pair_id VARCHAR,
    p_interval BIGINT,
    p_limit INT DEFAULT 100
)
RETURNS TABLE (
    timestamp BIGINT,
    open BIGINT,
    high BIGINT,
    low BIGINT,
    close BIGINT,
    volume_in NUMERIC,
    volume_out NUMERIC,
    trade_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        c.timestamp,
        c.open,
        c.high,
        c.low,
        c.close,
        c.volume_in,
        c.volume_out,
        c.trade_count
    FROM candles c
    WHERE c.pair_id = p_pair_id
      AND c.interval_seconds = p_interval
    ORDER BY c.timestamp DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;
