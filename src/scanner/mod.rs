pub mod project;
pub mod walk;

pub use project::{ProjectKind, ScannedProject};
pub use walk::scan_directory;
