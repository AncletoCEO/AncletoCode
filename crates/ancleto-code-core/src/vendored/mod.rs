#[path = "../../../../third_party/pandacode/src-tauri/src/providers/support.rs"]
pub mod support;

pub use support::limited;

#[allow(dead_code, clippy::unwrap_used)]
#[path = "../../../../third_party/pandacode/src-tauri/src/providers/text_stream.rs"]
pub mod text_stream;

#[cfg(windows)]
#[path = "../../../../third_party/pandacode/src-tauri/src/providers/windows_job.rs"]
pub mod windows_job;
