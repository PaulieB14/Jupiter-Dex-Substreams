-- Jupiter DEX Substreams PostgreSQL Schema
-- Version: 0.4.0
-- Supports: substreams-sink-sql with PostgreSQL

-- Individual swap events
CREATE TABLE IF NOT EXISTS jupiter_swaps (
    id VARCHAR(128) PRIMARY KEY,
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

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_swaps_tx_hash ON jupiter_swaps(tx_hash);
CREATE INDEX IF NOT EXISTS idx_swaps_slot ON jupiter_swaps(slot DESC);
CREATE INDEX IF NOT EXISTS idx_swaps_block_time ON jupiter_swaps(block_time DESC);
CREATE INDEX IF NOT EXISTS idx_swaps_user ON jupiter_swaps(user_wallet);
CREATE INDEX IF NOT EXISTS idx_swaps_program ON jupiter_swaps(program_id);
CREATE INDEX IF NOT EXISTS idx_swaps_input_mint ON jupiter_swaps(input_mint);
CREATE INDEX IF NOT EXISTS idx_swaps_output_mint ON jupiter_swaps(output_mint);

-- Daily aggregated statistics
CREATE TABLE IF NOT EXISTS daily_swap_stats (
    date VARCHAR(10) PRIMARY KEY,
    swap_count BIGINT NOT NULL DEFAULT 0,
    total_volume NUMERIC(78,0) NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_swap_stats(date DESC);

-- Per-program statistics
CREATE TABLE IF NOT EXISTS program_stats (
    program_id VARCHAR(44) PRIMARY KEY,
    instruction_count BIGINT NOT NULL DEFAULT 0,
    total_volume NUMERIC(78,0) NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Global protocol metrics
CREATE TABLE IF NOT EXISTS global_metrics (
    protocol VARCHAR(32) PRIMARY KEY,
    total_swaps BIGINT NOT NULL DEFAULT 0,
    total_volume NUMERIC(78,0) NOT NULL DEFAULT 0,
    unique_accounts BIGINT NOT NULL DEFAULT 0,
    unique_mints BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Useful views for analytics

-- Top tokens by volume (last 24 hours)
CREATE OR REPLACE VIEW top_tokens_24h AS
SELECT
    input_mint as token,
    COUNT(*) as swap_count,
    SUM(amount_in) as total_volume
FROM jupiter_swaps
WHERE block_time > EXTRACT(EPOCH FROM NOW() - INTERVAL '24 hours')
GROUP BY input_mint
ORDER BY total_volume DESC
LIMIT 100;

-- Top trading pairs
CREATE OR REPLACE VIEW top_trading_pairs AS
SELECT
    input_mint,
    output_mint,
    COUNT(*) as swap_count,
    SUM(amount_in) as volume_in,
    SUM(amount_out) as volume_out
FROM jupiter_swaps
GROUP BY input_mint, output_mint
ORDER BY swap_count DESC
LIMIT 100;

-- Hourly volume summary
CREATE OR REPLACE VIEW hourly_volume AS
SELECT
    DATE_TRUNC('hour', TO_TIMESTAMP(block_time)) as hour,
    COUNT(*) as swap_count,
    SUM(amount_in) as total_volume
FROM jupiter_swaps
GROUP BY DATE_TRUNC('hour', TO_TIMESTAMP(block_time))
ORDER BY hour DESC;

-- Active users (wallets) in last 7 days
CREATE OR REPLACE VIEW active_users_7d AS
SELECT
    user_wallet,
    COUNT(*) as swap_count,
    SUM(amount_in) as total_volume,
    MIN(block_time) as first_swap,
    MAX(block_time) as last_swap
FROM jupiter_swaps
WHERE block_time > EXTRACT(EPOCH FROM NOW() - INTERVAL '7 days')
GROUP BY user_wallet
ORDER BY swap_count DESC;
