#![feature(unchecked_math)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token(pub u8);

#[derive(Debug, Clone, Copy)]
pub enum Strategy {
    RoundRobin,
    IpHash,
}

pub trait Balance {
    type State;

    fn new(weights: &[u8]) -> Self;

    fn next(&self, state: &Self::State) -> Option<Token>;
}

pub mod ip_hash;
pub mod round_robin;

mod balancer;
pub use balancer::{Balancer, BalanceCtx};
