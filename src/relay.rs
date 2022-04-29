use futures::future::join_all;

use realm_core::tcp::run_tcp;
use realm_core::udp::run_udp;

use crate::conf::EndpointInfo;

pub async fn run(endpoints: Vec<EndpointInfo>) {
    let mut workers = Vec::with_capacity(2 * endpoints.len());

    for EndpointInfo {
        endpoint,
        no_tcp,
        use_udp,
    } in endpoints
    {
        if use_udp {
            workers.push(tokio::spawn(run_udp(endpoint.clone())));
        }

        if !no_tcp {
            workers.push(tokio::spawn(run_tcp(endpoint)));
        }
    }

    workers.shrink_to_fit();

    join_all(workers).await;
}
