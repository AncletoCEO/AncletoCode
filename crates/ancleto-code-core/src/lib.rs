pub mod domain;
pub mod drivers;
pub mod orch;
pub mod persistence;
pub mod projects;
pub mod providers;
pub mod session;

pub use domain::{CoreError, FileNode, Project, Result};
pub use drivers::DriverRegistry;
pub use orch::Orchestrator;
pub use persistence::Store;
pub use projects::ProjectService;
pub use providers::Providers;
pub use session::{SessionEvent, SessionInfo, SessionManager, SessionSpec, SessionStatus};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
