/// Peer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token(pub u8);

/// Health check configuration.
#[derive(Debug, Clone, Copy)]
pub struct HealthCheckConfig {
    pub max_fails: u32,
    pub fail_timeout_secs: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            max_fails: 2,
            fail_timeout_secs: 120,
        }
    }
}

/// Load balance traits.
pub trait Balance {
    type State;

    /// Constructor.
    fn new(weights: &[u8], config: Option<HealthCheckConfig>) -> Self;

    /// Get next peer.
    fn next(&self, state: &Self::State) -> Option<Token>;

    /// Total peers.
    fn total(&self) -> u8;

    /// Record success for a peer.
    fn on_success(&self, token: Token);

    /// Record failure for a peer.
    fn on_failure(&self, token: Token);
}

/// Iphash impl.
pub mod ip_hash;

/// Round-robin impl.
pub mod round_robin;

mod balancer;
pub use balancer::{Balancer, BalanceCtx, Strategy};
