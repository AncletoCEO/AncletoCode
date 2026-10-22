pub mod domain;
pub mod drivers;
pub mod host;
pub mod ide;
pub mod orch;
pub mod persistence;
pub mod projects;
pub mod providers;
pub mod session;
pub mod vendored;

pub use domain::{
    Conversation, ConversationState, CoreError, FileNode, Project, Result, Turn, TurnMode,
};
pub use drivers::{
    AgentDriver, AgentDriverDescriptor, Capabilities, DriverRegistry, DriverStatus, ToolStatus,
};
pub use host::{HostClient, HostPaths};
pub use orch::{ConversationEvent, Orchestrator, StartConversation};
pub use persistence::{SavedSession, Store};
pub use projects::{FileEntry, ProjectService};
pub use providers::Providers;
pub use session::{
    SessionBackend, SessionEvent, SessionInfo, SessionManager, SessionSpec, SessionStatus,
};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
