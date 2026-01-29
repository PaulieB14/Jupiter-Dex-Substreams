//! Jupiter DEX Substreams Constants
//!
//! Central location for all program IDs and configuration constants.
//! This is the single source of truth for Jupiter program addresses.

/// Solana Token Program ID
pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Jupiter v6 Aggregator Program (latest)
pub const JUPITER_V6_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Jupiter v4/v3 Aggregator Program
pub const JUPITER_V4_PROGRAM_ID: &str = "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB";

/// Jupiter v3 Aggregator Program
pub const JUPITER_V3_PROGRAM_ID: &str = "JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph";

/// Jupiter v2 Aggregator Program
pub const JUPITER_V2_PROGRAM_ID: &str = "JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo";

/// Jupiter Limit Orders Program
pub const JUPITER_LIMIT_ORDERS_PROGRAM_ID: &str = "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu";

/// Jupiter DCA (Dollar Cost Averaging) Program
pub const JUPITER_DCA_PROGRAM_ID: &str = "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M";

/// All Jupiter Program IDs for filtering
pub const JUPITER_PROGRAM_IDS: [&str; 6] = [
    JUPITER_V6_PROGRAM_ID,
    JUPITER_V4_PROGRAM_ID,
    JUPITER_V3_PROGRAM_ID,
    JUPITER_V2_PROGRAM_ID,
    JUPITER_LIMIT_ORDERS_PROGRAM_ID,
    JUPITER_DCA_PROGRAM_ID,
];

/// Check if a program ID is a Jupiter swap program (v2-v6)
#[inline]
pub fn is_jupiter_swap_program(program_id: &str) -> bool {
    matches!(
        program_id,
        JUPITER_V6_PROGRAM_ID
            | JUPITER_V4_PROGRAM_ID
            | JUPITER_V3_PROGRAM_ID
            | JUPITER_V2_PROGRAM_ID
    )
}

/// Check if a program ID is Jupiter Limit Orders
#[inline]
pub fn is_jupiter_limit_orders(program_id: &str) -> bool {
    program_id == JUPITER_LIMIT_ORDERS_PROGRAM_ID
}

/// Check if a program ID is Jupiter DCA
#[inline]
pub fn is_jupiter_dca(program_id: &str) -> bool {
    program_id == JUPITER_DCA_PROGRAM_ID
}

/// Check if a program ID is any Jupiter program
#[inline]
pub fn is_any_jupiter_program(program_id: &str) -> bool {
    JUPITER_PROGRAM_IDS.contains(&program_id)
}

/// Get the Jupiter program version from a program ID
pub fn get_jupiter_version(program_id: &str) -> Option<&'static str> {
    match program_id {
        JUPITER_V6_PROGRAM_ID => Some("v6"),
        JUPITER_V4_PROGRAM_ID => Some("v4"),
        JUPITER_V3_PROGRAM_ID => Some("v3"),
        JUPITER_V2_PROGRAM_ID => Some("v2"),
        JUPITER_LIMIT_ORDERS_PROGRAM_ID => Some("limit_orders"),
        JUPITER_DCA_PROGRAM_ID => Some("dca"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_jupiter_swap_program() {
        assert!(is_jupiter_swap_program(JUPITER_V6_PROGRAM_ID));
        assert!(is_jupiter_swap_program(JUPITER_V4_PROGRAM_ID));
        assert!(!is_jupiter_swap_program(JUPITER_LIMIT_ORDERS_PROGRAM_ID));
        assert!(!is_jupiter_swap_program(JUPITER_DCA_PROGRAM_ID));
        assert!(!is_jupiter_swap_program("random_program"));
    }

    #[test]
    fn test_is_any_jupiter_program() {
        for program_id in JUPITER_PROGRAM_IDS {
            assert!(is_any_jupiter_program(program_id));
        }
        assert!(!is_any_jupiter_program("not_jupiter"));
    }

    #[test]
    fn test_get_jupiter_version() {
        assert_eq!(get_jupiter_version(JUPITER_V6_PROGRAM_ID), Some("v6"));
        assert_eq!(
            get_jupiter_version(JUPITER_LIMIT_ORDERS_PROGRAM_ID),
            Some("limit_orders")
        );
        assert_eq!(get_jupiter_version("random"), None);
    }
}
