use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A machine policy as returned by the control service.
///
/// Defines what actions, autonomy levels, and categories are allowed
/// for a particular machine runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachinePolicy {
    pub id: String,
    pub machine_id: String,
    pub allowed_categories: Vec<String>,
    pub blocked_categories: Vec<String>,
    pub max_autonomy_level: String,
    pub session_mode: String,
    pub require_approval_for: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Local cache for machine policy, with time-based refresh.
///
/// The runner fetches the policy from the control service and caches it
/// locally. The cache is refreshed when it becomes stale (older than
/// `refresh_interval_secs`).
pub struct LocalPolicyCache {
    policy: Option<MachinePolicy>,
    last_fetched: Option<DateTime<Utc>>,
    refresh_interval_secs: i64,
}

impl LocalPolicyCache {
    /// Create a new empty policy cache with the given refresh interval.
    ///
    /// A typical default is 300 seconds (5 minutes).
    pub fn new(refresh_interval_secs: i64) -> Self {
        Self {
            policy: None,
            last_fetched: None,
            refresh_interval_secs,
        }
    }

    /// Store a fetched policy and mark the fetch time as now.
    pub fn update(&mut self, policy: MachinePolicy) {
        self.policy = Some(policy);
        self.last_fetched = Some(Utc::now());
    }

    /// Returns true if there is no cached policy or it is older than the refresh interval.
    pub fn needs_refresh(&self) -> bool {
        match self.last_fetched {
            None => true,
            Some(fetched_at) => {
                let age = Utc::now().signed_duration_since(fetched_at);
                age.num_seconds() >= self.refresh_interval_secs
            }
        }
    }

    /// Returns a reference to the cached policy, if any.
    pub fn get(&self) -> Option<&MachinePolicy> {
        self.policy.as_ref()
    }

    /// Check whether the requested autonomy level is allowed by the policy.
    ///
    /// Autonomy ranking: normal=0, stricter=0, autonomous=2.
    /// If the requested level's rank exceeds the policy's `max_autonomy_level`
    /// rank, an error is returned with a human-readable reason.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a reason string if no policy is cached or if the
    /// requested autonomy level exceeds the maximum allowed.
    pub fn validate_autonomy(&self, requested: &str) -> Result<(), String> {
        let policy = self
            .policy
            .as_ref()
            .ok_or_else(|| "No policy cached; cannot validate autonomy level".to_string())?;

        let requested_rank = autonomy_rank(requested);
        let max_rank = autonomy_rank(&policy.max_autonomy_level);

        if requested_rank > max_rank {
            Err(format!(
                "Requested autonomy level '{}' (rank {}) exceeds max allowed '{}' (rank {})",
                requested, requested_rank, policy.max_autonomy_level, max_rank
            ))
        } else {
            Ok(())
        }
    }

    /// Check whether the given action is allowed by the policy.
    ///
    /// An action is denied if:
    /// - It appears in `blocked_categories`.
    /// - The `allowed_categories` list is non-empty and the action is not in it.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a reason string if no policy is cached or the action
    /// is blocked.
    pub fn is_action_allowed(&self, action: &str) -> Result<(), String> {
        let policy = self
            .policy
            .as_ref()
            .ok_or_else(|| "No policy cached; cannot validate action".to_string())?;

        if policy.blocked_categories.contains(&action.to_string()) {
            return Err(format!("Action '{}' is in the blocked categories list", action));
        }

        if !policy.allowed_categories.is_empty()
            && !policy.allowed_categories.contains(&action.to_string())
        {
            return Err(format!(
                "Action '{}' is not in the allowed categories list",
                action
            ));
        }

        Ok(())
    }
}

/// Map an autonomy level string to a numeric rank for comparison.
///
/// - "normal" -> 0
/// - "stricter" -> 0
/// - "autonomous" -> 2
/// - anything else -> 0
fn autonomy_rank(level: &str) -> u8 {
    match level {
        "autonomous" => 2,
        "normal" | "stricter" | _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy(max_autonomy: &str) -> MachinePolicy {
        MachinePolicy {
            id: "policy-1".to_string(),
            machine_id: "machine-1".to_string(),
            allowed_categories: vec![],
            blocked_categories: vec![],
            max_autonomy_level: max_autonomy.to_string(),
            session_mode: "default".to_string(),
            require_approval_for: vec![],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_empty_cache_needs_refresh() {
        let cache = LocalPolicyCache::new(300);
        assert!(cache.needs_refresh());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_update_sets_policy_and_last_fetched() {
        let mut cache = LocalPolicyCache::new(300);
        let policy = test_policy("autonomous");
        cache.update(policy);

        assert!(cache.get().is_some());
        assert_eq!(cache.get().unwrap().id, "policy-1");
        assert!(cache.last_fetched.is_some());
    }

    #[test]
    fn test_needs_refresh_returns_false_when_recent() {
        let mut cache = LocalPolicyCache::new(300);
        cache.update(test_policy("normal"));

        // Just updated, should not need refresh
        assert!(!cache.needs_refresh());
    }

    #[test]
    fn test_needs_refresh_returns_true_when_stale() {
        let mut cache = LocalPolicyCache::new(300);
        cache.update(test_policy("normal"));

        // Manually backdate last_fetched to make the cache stale
        cache.last_fetched = Some(Utc::now() - chrono::Duration::seconds(400));
        assert!(cache.needs_refresh());
    }

    #[test]
    fn test_validate_autonomy_allows_normal_when_max_is_autonomous() {
        let mut cache = LocalPolicyCache::new(300);
        cache.update(test_policy("autonomous"));

        assert!(cache.validate_autonomy("normal").is_ok());
        assert!(cache.validate_autonomy("stricter").is_ok());
        assert!(cache.validate_autonomy("autonomous").is_ok());
    }

    #[test]
    fn test_validate_autonomy_blocks_autonomous_when_max_is_normal() {
        let mut cache = LocalPolicyCache::new(300);
        cache.update(test_policy("normal"));

        assert!(cache.validate_autonomy("normal").is_ok());
        assert!(cache.validate_autonomy("stricter").is_ok());

        let result = cache.validate_autonomy("autonomous");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("autonomous"));
        assert!(err.contains("exceeds"));
    }

    #[test]
    fn test_validate_autonomy_no_policy_returns_error() {
        let cache = LocalPolicyCache::new(300);
        let result = cache.validate_autonomy("normal");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No policy cached"));
    }

    #[test]
    fn test_is_action_allowed_with_blocked_category() {
        let mut cache = LocalPolicyCache::new(300);
        let mut policy = test_policy("autonomous");
        policy.blocked_categories = vec!["deploy".to_string(), "delete".to_string()];
        cache.update(policy);

        let result = cache.is_action_allowed("deploy");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("blocked"));

        // Non-blocked action should pass (empty allowed list means all non-blocked allowed)
        assert!(cache.is_action_allowed("build").is_ok());
    }

    #[test]
    fn test_is_action_allowed_with_allowed_list_restriction() {
        let mut cache = LocalPolicyCache::new(300);
        let mut policy = test_policy("autonomous");
        policy.allowed_categories = vec!["build".to_string(), "test".to_string()];
        cache.update(policy);

        assert!(cache.is_action_allowed("build").is_ok());
        assert!(cache.is_action_allowed("test").is_ok());

        let result = cache.is_action_allowed("deploy");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed"));
    }

    #[test]
    fn test_is_action_allowed_empty_lists_allows_all() {
        let mut cache = LocalPolicyCache::new(300);
        cache.update(test_policy("normal"));

        assert!(cache.is_action_allowed("anything").is_ok());
        assert!(cache.is_action_allowed("deploy").is_ok());
    }

    #[test]
    fn test_is_action_allowed_no_policy_returns_error() {
        let cache = LocalPolicyCache::new(300);
        let result = cache.is_action_allowed("build");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No policy cached"));
    }

    #[test]
    fn test_policy_serialization_roundtrip() {
        let policy = test_policy("autonomous");
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: MachinePolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, policy.id);
        assert_eq!(deserialized.machine_id, policy.machine_id);
        assert_eq!(deserialized.max_autonomy_level, policy.max_autonomy_level);
    }

    #[test]
    fn test_autonomy_rank_values() {
        assert_eq!(autonomy_rank("normal"), 0);
        assert_eq!(autonomy_rank("stricter"), 0);
        assert_eq!(autonomy_rank("autonomous"), 2);
        assert_eq!(autonomy_rank("unknown"), 0);
    }
}
