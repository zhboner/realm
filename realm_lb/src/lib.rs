/// Peer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token(pub u8);

/// Load balance traits.
pub trait Balance {
    type State;

    /// Constructor.
    fn new(weights: &[u8]) -> Self;

    /// Get next peer.
    fn next(&self, state: &Self::State) -> Option<Token>;

    /// Total peers.
    fn total(&self) -> u8;
}

/// Iphash impl.
pub mod ip_hash;

/// Round-robin impl.
pub mod round_robin;

mod balancer;
pub use balancer::{Balancer, BalanceCtx, Strategy};
