mod branch;
mod clone;
mod fetch;

pub mod exclude;

pub use branch::checkout_branch;
pub use clone::{clone_repository, CloneOptions};
pub use fetch::fetch;
