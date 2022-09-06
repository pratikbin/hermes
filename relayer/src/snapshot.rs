use async_trait::async_trait;

pub mod memory;
pub mod psql;
mod util;

mod data;
pub use data::*;

use crate::{chain::requests::QueryHeight, error::Error};

pub const KEEP_SNAPSHOTS: u64 = 8;

#[async_trait]
pub trait SnapshotManager {
    async fn fetch_snapshot(&self, query_height: QueryHeight) -> Result<IbcSnapshot, Error>;

    async fn update_snapshot(&mut self, snapshot: &IbcSnapshot) -> Result<(), Error>;

    async fn vacuum_snapshots(&mut self, at_or_below: u64) -> Result<(), Error>;
}
