use std::net::IpAddr;
use std::sync::Arc;

use crate::{Token, Balance, Strategy};
use crate::ip_hash::IpHash;
use crate::round_robin::RoundRobin;

#[derive(Default, Debug)]
pub struct BalanceCtx<'a> {
    pub src_ip: Option<&'a IpAddr>,
}

#[derive(Debug, Clone)]
pub enum Balancer {
    IpHash(Arc<IpHash>),
    RoundRobin(Arc<RoundRobin>),
}

impl Balancer {
    pub fn new(weights: &[u8], strategy: Strategy) -> Self {
        match strategy {
            Strategy::IpHash => Self::IpHash(Arc::new(IpHash::new(weights))),
            Strategy::RoundRobin => Self::RoundRobin(Arc::new(RoundRobin::new(weights))),
        }
    }

    pub fn next(&self, ctx: BalanceCtx) -> Option<Token> {
        match self {
            Balancer::IpHash(iphash) => iphash.next(ctx.src_ip.unwrap()),
            Balancer::RoundRobin(rr) => rr.next(&()),
        }
    }
}
