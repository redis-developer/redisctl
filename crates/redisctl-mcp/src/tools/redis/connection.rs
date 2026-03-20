//! Unified Redis connection type supporting both standalone and cluster modes.

use redis::aio::{ConnectionLike, MultiplexedConnection};
use redis::cluster_async::ClusterConnection;
use redis::{Cmd, RedisFuture, Value};

/// A Redis connection that transparently handles both standalone and cluster topologies.
///
/// Implements `ConnectionLike` so it can be used anywhere a Redis connection is expected
/// (`cmd.query_async()`, `pipe.query_async()`, etc.).
#[derive(Clone)]
pub enum RedisConnection {
    Standalone(MultiplexedConnection),
    Cluster(ClusterConnection),
}

impl ConnectionLike for RedisConnection {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, Value> {
        match self {
            RedisConnection::Standalone(conn) => conn.req_packed_command(cmd),
            RedisConnection::Cluster(conn) => conn.req_packed_command(cmd),
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> RedisFuture<'a, Vec<Value>> {
        match self {
            RedisConnection::Standalone(conn) => conn.req_packed_commands(cmd, offset, count),
            RedisConnection::Cluster(conn) => conn.req_packed_commands(cmd, offset, count),
        }
    }

    fn get_db(&self) -> i64 {
        match self {
            RedisConnection::Standalone(conn) => conn.get_db(),
            RedisConnection::Cluster(conn) => conn.get_db(),
        }
    }
}
