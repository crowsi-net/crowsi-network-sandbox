//! Builds finite Linux user and network namespace command boundaries.

mod model;
mod runner;

pub use model::{ISOLATION_SCHEMA, IsolationCapabilityV1};
pub use runner::{IsolationBudget, isolated_command};
