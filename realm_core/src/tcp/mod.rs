//! TCP relay entrance.

mod socket;
mod middle;
mod plain;

#[cfg(feature = "hook")]
mod hook;

#[cfg(feature = "proxy")]
mod proxy;

#[cfg(feature = "transport")]
mod transport;

use std::io::Result;

use crate::trick::Ref;
use crate::endpoint::Endpoint;

use middle::connect_and_relay;

/// Launch a tcp relay.
pub async fn run_tcp(endpoint: Endpoint) -> Result<()> {
    let Endpoint {
        laddr,
        raddr,
        conn_opts,
        extra_raddrs,
    } = endpoint;

    let raddr = Ref::new(&raddr);
    let conn_opts = Ref::new(&conn_opts);
    let extra_raddrs = Ref::new(&extra_raddrs);

    let lis = socket::bind(&laddr).unwrap_or_else(|e| panic!("[tcp]failed to bind {}: {}", &laddr, e));

    loop {
        let (local, addr) = match lis.accept().await {
            Ok(x) => x,
            Err(e) => {
                log::error!("[tcp]failed to accept: {}", e);
                continue;
            }
        };

        // ignore error
        let _ = local.set_nodelay(true);

        tokio::spawn(async move {
            match connect_and_relay(local, raddr, conn_opts, extra_raddrs).await {
                Ok(..) => log::debug!("[tcp]{} => {}, finish", addr, raddr.as_ref()),
                Err(e) => log::error!("[tcp]{} => {}, error: {}", addr, raddr.as_ref(), e),
            }
        });
    }
}

/// Launch a tcp relay, where tasks are not detached.
#[cfg(feature = "x")]
pub async fn run_tcp_scoped(endpoint: Endpoint) -> Result<()> {
    use tokio::task::JoinSet;
    let Endpoint {
        laddr,
        raddr,
        conn_opts,
    } = endpoint;

    let raddr = Ref::new(&raddr);
    let conn_opts = Ref::new(&conn_opts);

    let lis = socket::bind(&laddr).unwrap_or_else(|e| panic!("[tcp]failed to bind {}: {}", &laddr, e));

    let mut tasks = JoinSet::new();

    loop {
        let (local, addr) = match lis.accept().await {
            Ok(x) => x,
            Err(e) => {
                log::error!("[tcp]failed to accept: {}", e);
                continue;
            }
        };

        // ignore error
        let _ = local.set_nodelay(true);

        tasks.spawn(async move {
            match connect_and_relay(local, raddr, conn_opts).await {
                Ok(..) => log::debug!("[tcp]{} => {}, finish", addr, raddr.as_ref()),
                Err(e) => log::error!("[tcp]{} => {}, error: {}", addr, raddr.as_ref(), e),
            }
        });
    }
}
