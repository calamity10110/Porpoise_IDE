//! SSH host-key verification policy logic.
//!
//! This module isolates the *decision* part of host-key verification from the
//! `ssh2` runtime so it can be unit-tested without a live SSH server. The runtime
//! glue (reading `~/.ssh/known_hosts`, calling `ssh2`, recording keys) lives in
//! [`crate::session`].
//!
//! Verification model (fail-closed):
//!   - `Verified`     (known_hosts `Match`)    -> always accept
//!   - `KeyMismatch`  (known_hosts `Mismatch`) -> always REJECT (possible MITM)
//!   - `UnknownHost`  (known_hosts `NotFound`) -> accept only under TOFU policy
//!   - ssh2 `Failure` / no key                 -> always REJECT

/// Outcome of checking a presented host key against the known-hosts store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKeyVerdict {
    /// Key matches a known_hosts entry for this host.
    Verified,
    /// A known_hosts entry exists but the key differs — possible MITM.
    KeyMismatch,
    /// No known_hosts entry exists for this host yet.
    UnknownHost,
}

/// Acceptance policy for previously-unseen host keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKeyPolicy {
    /// Trust-on-first-use: accept & record the first key seen, reject on change.
    TrustOnFirstUse,
    /// Strict: reject any host not already present in known_hosts.
    Strict,
}

/// Maps a raw `ssh2::CheckResult` onto our verdict vocabulary.
///
/// `ssh2::CheckResult::Failure` is collapsed to `None` so callers fail closed.
pub fn classify(result: ssh2::CheckResult) -> Option<HostKeyVerdict> {
    match result {
        ssh2::CheckResult::Match => Some(HostKeyVerdict::Verified),
        ssh2::CheckResult::Mismatch => Some(HostKeyVerdict::KeyMismatch),
        ssh2::CheckResult::NotFound => Some(HostKeyVerdict::UnknownHost),
        ssh2::CheckResult::Failure => None,
    }
}

/// Returns `true` iff `verdict` is acceptable under `policy`.
///
/// This is the security boundary. `KeyMismatch` and failures (`None`) are
/// *always* rejected regardless of policy; `UnknownHost` is accepted only for
/// the permissive trust-on-first-use policy.
pub const fn policy_accepts(verdict: Option<HostKeyVerdict>, policy: HostKeyPolicy) -> bool {
    match verdict {
        Some(HostKeyVerdict::Verified) => true,
        Some(HostKeyVerdict::KeyMismatch) => false,
        Some(HostKeyVerdict::UnknownHost) => match policy {
            HostKeyPolicy::TrustOnFirstUse => true,
            HostKeyPolicy::Strict => false,
        },
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_maps_all_ssh2_results() {
        assert_eq!(
            classify(ssh2::CheckResult::Match),
            Some(HostKeyVerdict::Verified)
        );
        assert_eq!(
            classify(ssh2::CheckResult::Mismatch),
            Some(HostKeyVerdict::KeyMismatch)
        );
        assert_eq!(
            classify(ssh2::CheckResult::NotFound),
            Some(HostKeyVerdict::UnknownHost)
        );
        assert_eq!(classify(ssh2::CheckResult::Failure), None);
    }

    #[test]
    fn key_mismatch_is_always_rejected() {
        // The critical security property: a changed key never authenticates,
        // regardless of how permissive the policy is.
        assert!(!policy_accepts(
            Some(HostKeyVerdict::KeyMismatch),
            HostKeyPolicy::TrustOnFirstUse
        ));
        assert!(!policy_accepts(
            Some(HostKeyVerdict::KeyMismatch),
            HostKeyPolicy::Strict
        ));
    }

    #[test]
    fn unknown_host_respects_policy() {
        assert!(policy_accepts(
            Some(HostKeyVerdict::UnknownHost),
            HostKeyPolicy::TrustOnFirstUse
        ));
        assert!(!policy_accepts(
            Some(HostKeyVerdict::UnknownHost),
            HostKeyPolicy::Strict
        ));
    }

    #[test]
    fn verified_is_always_accepted() {
        assert!(policy_accepts(
            Some(HostKeyVerdict::Verified),
            HostKeyPolicy::TrustOnFirstUse
        ));
        assert!(policy_accepts(
            Some(HostKeyVerdict::Verified),
            HostKeyPolicy::Strict
        ));
    }

    #[test]
    fn check_failure_fails_closed() {
        // `None` models ssh2 CheckResult::Failure — must never authenticate.
        assert!(!policy_accepts(None, HostKeyPolicy::TrustOnFirstUse));
        assert!(!policy_accepts(None, HostKeyPolicy::Strict));
    }

    #[test]
    fn injection_regression_no_silent_accept() {
        // Regression guard: every ssh2 result maps to a decision, and the two
        // dangerous outcomes (Failure, Mismatch) reject under BOTH policies.
        for r in [
            ssh2::CheckResult::Match,
            ssh2::CheckResult::Mismatch,
            ssh2::CheckResult::NotFound,
            ssh2::CheckResult::Failure,
        ] {
            let v = classify(r);
            let tof = policy_accepts(v, HostKeyPolicy::TrustOnFirstUse);
            let strict = policy_accepts(v, HostKeyPolicy::Strict);
            if matches!(v, None | Some(HostKeyVerdict::KeyMismatch)) {
                assert!(!tof && !strict, "dangerous verdict accepted: {v:?}");
            }
        }
    }
}
