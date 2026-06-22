use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod v0;
pub use v0::StageV0;

/// A physical positioning stage for structured light scanning.
///
/// See [`StageV0`] for the current version of the stage schema.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "version", rename_all = "snake_case")]
pub enum Stage {
    V0(StageV0),
}

