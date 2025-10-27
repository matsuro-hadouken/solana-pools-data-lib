//! Pool authority definitions and utilities.
//!
//! This module contains the embedded list of known stake pool authorities
//! and provides utilities for working with pool information.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Information about a stake pool
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolInfo {
    /// Human-readable name of the pool
    pub name: String,
    /// Base58-encoded authority public key
    pub authority: String,
}

impl PoolInfo {
    /// Create a new `PoolInfo`
    pub fn new(name: impl Into<String>, authority: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            authority: authority.into(),
        }
    }
}

/// Static registry of all known pools
/// This replaces the external JSON file with embedded data
static POOLS_REGISTRY: Lazy<Vec<PoolInfo>> = Lazy::new(|| {
    vec![
        PoolInfo::new("foundation", "mpa4abUkjQoAvPzREkh5Mo75hZhPFQ2FSH6w7dWKuQ5"),
        PoolInfo::new("firedancer_delegation", "FiRep26iRQbMaKbqhhs5CqXqy7YrHn462LbnQhXzB2ps",),
        PoolInfo::new("double_zero", "4cpnpiwgBfUgELVwNYiecwGti45YHSH3R72CPkFTiwJt",),
        PoolInfo::new("jpool", "HbJTxftxnXgpePCshA8FubsRj9MW4kfPscfuUfn44fnt"),
        PoolInfo::new("jito", "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
        PoolInfo::new("marinade", "4bZ6o3eUUNXhKuqjdCnCoPAoLgWiuLYixKaxoa8PpiKk"),
        PoolInfo::new("marinade_native", "ex9CfkBZZd6Nv9XdnoDmmB45ymbu4arXVk7g5pWnt3N",),
        PoolInfo::new("marinade_native_2", "stWirqFCf2Uts1JBL1Jsd3r6VBWhgnpdPxCTe1MFjrq",),
        PoolInfo::new("socean", "AzZRvyyMHBm8EHEksWxq4ozFL7JxLMydCDMGhqM6BVck"),
        PoolInfo::new("lido", "W1ZQRwUfSkDKy2oefRBUWph82Vr2zg9txWMA8RQazN5"),
        PoolInfo::new("eversol", "C4NeuptywfXuyWB9A7H7g5jHVDE8L6Nj2hS53tA71KPn"),
        PoolInfo::new("edgevana", "FZEaZMmrRC3PDPFMzqooKLS2JjoyVkKNd2MkHjr7Xvyq"),
        PoolInfo::new("blazestake", "6WecYymEARvjG5ZyqkrVQ6YkhPfujNzWpSPwNKXHCbV2"),
        PoolInfo::new("daopool", "BbyX1GwUNsfbcoWwnkZDo8sqGmwNDzs2765RpjyQ1pQb"),
        PoolInfo::new("bonk", "9LcmMfufi8YUcx83RALwF9Y9BPWZ7SqGy4D9VLe2nhhA"),
        PoolInfo::new("sanctum", "EjYFnQcNDmfYQqT5B2R2239i781D5wNXrqA2qx2gYJo1"),
        PoolInfo::new("sanctum_2", "3rBnnH9TTgd3xwu48rnzGsaQkSr1hR64nY71DrDt6VrQ"),
        PoolInfo::new("binance", "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
        PoolInfo::new("jupiter", "EMjuABxELpYWYEwjkKmQKBNCwdaFAy4QYAs6W9bDQDNw"),
        PoolInfo::new("binance_2", "75NPzpxoh8sXGuSENFMREidq6FMzEx4g2AfcBEB6qjCV"),
        PoolInfo::new("solayer", "H5rmot8ejBUWzMPt6E44h27xj5obbSz3jVuK4AsJpHmv"),
        PoolInfo::new("bybit", "3pFTQjRVwcJHSpUNH5n1hx6Jwx7V3EzJDDHaKuwExyGJ"),
        PoolInfo::new("shinobi", "EpH4ZKSeViL5qAHA9QANYVHxdmuzbUH2T79f32DmSCaM"),
        PoolInfo::new("helius", "2rMuGTyXCqCHZBSu6NZR9Aq8MhZX9gLkCHoQsPhSj2YF"),
        PoolInfo::new("marginfi", "3b7XQeZ8nSMyjcQGTFJS5kBw4pXS2SqtB9ooHCnF2xV9"),
        PoolInfo::new("vault", "GdNXJobf8fbTR5JSE7adxa6niaygjx4EEbnnRaDCHMMW"),
        PoolInfo::new("drift", "6727ZvQ2YEz8jky1Z9fqDFG5mYuAvC9G34o2MxwzmrUK"),
        PoolInfo::new("aerosol", "AKJt3m2xJ6ANda9adBGqb5BMrheKJSwxyCfYkLuZNmjn"),
        PoolInfo::new("ftx", "H4yiPhdSsmSMJTznXzmZvdqWuhxDRzzkoQMEWXZ6agFZ"),
        PoolInfo::new("juicy", "FKDyJz5tPUy1ArAUba7ziQLbMKzaivRnHiW4FHzCSE9t"),
        PoolInfo::new("picosol", "4At8nQXanWgRvjbrVXmxMBBdfz39txWVm4SiXEoP1kGh"),
    ]
});

/// Index by pool name for fast lookups
static POOLS_BY_NAME: Lazy<HashMap<String, PoolInfo>> = Lazy::new(|| {
    POOLS_REGISTRY
        .iter()
        .map(|pool| (pool.name.clone(), pool.clone()))
        .collect()
});

/// Index by authority for fast reverse lookups
static POOLS_BY_AUTHORITY: Lazy<HashMap<String, PoolInfo>> = Lazy::new(|| {
    POOLS_REGISTRY
        .iter()
        .map(|pool| (pool.authority.clone(), pool.clone()))
        .collect()
});

/// Get all available pools
#[must_use]
pub fn get_all_pools() -> &'static [PoolInfo] {
    &POOLS_REGISTRY
}

/// Get pool info by name
pub fn get_pool_by_name(name: &str) -> Option<&PoolInfo> {
    POOLS_BY_NAME.get(name)
}

/// Get pool info by authority
pub fn get_pool_by_authority(authority: &str) -> Option<&PoolInfo> {
    POOLS_BY_AUTHORITY.get(authority)
}

/// Get multiple pools by names
#[must_use]
pub fn get_pools_by_names(names: &[&str]) -> Vec<PoolInfo> {
    names
        .iter()
        .filter_map(|name| get_pool_by_name(name))
        .cloned()
        .collect()
}

/// Check if a pool name exists
pub fn pool_exists(name: &str) -> bool {
    POOLS_BY_NAME.contains_key(name)
}

/// Get all pool names
pub fn get_all_pool_names() -> Vec<String> {
    POOLS_REGISTRY
        .iter()
        .map(|pool| pool.name.clone())
        .collect()
}

/// Get all authorities
pub fn get_all_authorities() -> Vec<String> {
    POOLS_REGISTRY
        .iter()
        .map(|pool| pool.authority.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_registry_not_empty() {
        assert!(!POOLS_REGISTRY.is_empty());
        assert!(POOLS_REGISTRY.len() > 30); // We have 32 pools
    }

    #[test]
    fn test_get_pool_by_name() {
        let jito = get_pool_by_name("jito").unwrap();
        assert_eq!(jito.name, "jito");
        assert_eq!(
            jito.authority,
            "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"
        );
    }

    #[test]
    fn test_get_pool_by_authority() {
        let marinade =
            get_pool_by_authority("4bZ6o3eUUNXhKuqjdCnCoPAoLgWiuLYixKaxoa8PpiKk").unwrap();
        assert_eq!(marinade.name, "marinade");
    }

    #[test]
    fn test_unknown_pool() {
        assert!(get_pool_by_name("unknown_pool").is_none());
    }

    #[test]
    fn test_get_multiple_pools() {
        let pools = get_pools_by_names(&["jito", "marinade", "unknown"]);
        assert_eq!(pools.len(), 2);
        assert!(pools.iter().any(|p| p.name == "jito"));
        assert!(pools.iter().any(|p| p.name == "marinade"));
    }

    #[test]
    fn test_pool_exists() {
        assert!(pool_exists("jito"));
        assert!(!pool_exists("nonexistent"));
    }

    #[test]
    fn test_all_pools_have_unique_names() {
        let mut names = std::collections::HashSet::new();
        for pool in get_all_pools() {
            assert!(
                names.insert(&pool.name),
                "Duplicate pool name: {}",
                pool.name
            );
        }
    }

    #[test]
    fn test_all_pools_have_valid_authorities() {
        for pool in get_all_pools() {
            // Basic validation - authorities should be base58 strings of length 32-44
            assert!(!pool.authority.is_empty());
            assert!(pool.authority.len() >= 32);
            assert!(pool.authority.len() <= 44);
            // Should only contain base58 characters
            assert!(pool
                .authority
                .chars()
                .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)));
        }
    }
}
