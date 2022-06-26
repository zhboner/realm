use std::sync::Mutex;

use super::{Balance, Token};

/// Round-robin node.
#[derive(Debug)]
struct Node {
    cw: i16,
    ew: u8,
    weight: u8,
    token: Token,
}

/// Round robin balancer.
#[derive(Debug)]
pub struct RoundRobin {
    nodes: Mutex<Vec<Node>>,
    total: u8,
}

impl Balance for RoundRobin {
    type State = ();

    fn total(&self) -> u8 {
        self.total
    }

    fn new(weights: &[u8]) -> Self {
        assert!(weights.len() <= u8::MAX as usize);

        if weights.len() <= 1 {
            return Self {
                nodes: Mutex::new(Vec::new()),
                total: weights.len() as u8,
            };
        }

        let nodes = weights
            .iter()
            .enumerate()
            .map(|(i, w)| Node {
                ew: *w,
                cw: 0,
                weight: *w,
                token: Token(i as u8),
            })
            .collect();
        Self {
            nodes: Mutex::new(nodes),
            total: weights.len() as u8,
        }
    }

    #[allow(clippy::significant_drop_in_scrutinee)]
    fn next(&self, _: &Self::State) -> Option<Token> {
        if self.total <= 1 {
            return Some(Token(0));
        }

        // lock the whole list
        {
            let mut nodes = self.nodes.lock().unwrap();
            let mut tw: i16 = 0;
            let mut best: Option<&mut Node> = None;
            for p in nodes.iter_mut() {
                tw += p.ew as i16;
                p.cw += p.ew as i16;

                if p.ew < p.weight {
                    p.ew += 1;
                }

                if let Some(ref x) = best {
                    if p.cw > x.cw {
                        best = Some(p);
                    }
                } else {
                    best = Some(p);
                }
            }

            best.map(|x| {
                x.cw -= tw;
                x.token
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use average::{Max, Mean, Min};

    #[test]
    fn rr_same_weight() {
        let rr = RoundRobin::new(&vec![1; 255]);
        let mut distro = [0f64; 255];

        for _ in 0..1_000_000 {
            let token = rr.next(&()).unwrap();
            distro[token.0 as usize] += 1 as f64;
        }

        let diffs: Vec<f64> = distro
            .iter()
            .map(|x| *x / 1_000_000.0 - 1.0 / 255.0)
            .map(f64::abs)
            .inspect(|x| assert!(x < &1e-3))
            .collect();

        let min_diff: Min = diffs.iter().collect();
        let max_diff: Max = diffs.iter().collect();
        let mean_diff: Mean = diffs.iter().collect();

        println!("{:?}", distro);
        println!("min diff: {}", min_diff.min());
        println!("max diff: {}", max_diff.max());
        println!("mean diff: {}", mean_diff.mean());
    }

    #[test]
    fn rr_all_weights() {
        let weights: Vec<u8> = (1..=255).collect();
        let total_weight: f64 = weights.iter().map(|x| *x as f64).sum();
        let rr = RoundRobin::new(&weights);
        let mut distro = [0f64; 255];

        for _ in 0..1_000_000 {
            let token = rr.next(&()).unwrap();
            distro[token.0 as usize] += 1 as f64;
        }

        let diffs: Vec<f64> = distro
            .iter()
            .enumerate()
            .map(|(i, x)| *x / 1_000_000.0 - (i as f64 + 1.0) / total_weight)
            .map(f64::abs)
            .inspect(|x| assert!(x < &1e-3))
            .collect();

        let min_diff: Min = diffs.iter().collect();
        let max_diff: Max = diffs.iter().collect();
        let mean_diff: Mean = diffs.iter().collect();

        println!("{:?}", distro);
        println!("min diff: {}", min_diff.min());
        println!("max diff: {}", max_diff.max());
        println!("mean diff: {}", mean_diff.mean());
    }
}
