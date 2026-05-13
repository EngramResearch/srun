pub mod cli;
pub mod detect;
pub mod exec;
pub mod manifest;
pub mod model;
pub mod resolve;

pub use detect::detect_project;
pub use model::{Intent, ProjectInfo, ResolvedCommand, SrunError};
pub use resolve::resolve_intent;
