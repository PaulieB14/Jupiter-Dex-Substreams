//! Official tokenized-stock registry lookups (xStocks by Backed on Solana).
use crate::stocks_registry::XSTOCKS;

/// Info for a tokenized-stock mint.
pub struct StockInfo {
    pub symbol: &'static str,
    pub ticker: &'static str,
    pub isin: &'static str,
}

/// Look up a mint in the official xStocks registry (binary search over a sorted table).
pub fn lookup(mint: &str) -> Option<StockInfo> {
    // Every xStocks Solana mint carries the "Xs" vanity prefix; cheap pre-filter.
    if !mint.starts_with("Xs") {
        return None;
    }
    XSTOCKS
        .binary_search_by(|(m, _, _, _)| (*m).cmp(mint))
        .ok()
        .map(|i| {
            let (_, symbol, ticker, isin) = XSTOCKS[i];
            StockInfo { symbol, ticker: if ticker.is_empty() { symbol } else { ticker }, isin }
        })
}

/// Underlying ticker for a mint, or empty string.
pub fn ticker(mint: &str) -> String {
    lookup(mint).map(|s| s.ticker.to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_sorted_and_unique() {
        for w in XSTOCKS.windows(2) {
            assert!(w[0].0 < w[1].0, "unsorted or duplicate: {} {}", w[0].0, w[1].0);
        }
    }

    #[test]
    fn known_mints() {
        assert_eq!(ticker("XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp"), "AAPL");
        assert_eq!(ticker("XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB"), "TSLA");
        assert_eq!(ticker("Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh"), "NVDA");
        assert_eq!(lookup("XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W").unwrap().symbol, "SPYx");
        assert_eq!(ticker("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), "");
        assert_eq!(ticker("XsNOTAREALMINT1111111111111111111111111111"), "");
    }
}
