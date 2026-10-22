pub mod client;
pub mod protocol;
pub mod transport;

pub use client::HostClient;
pub use protocol::{
    HostStatus, RpcError, RpcNotification, RpcRequest, RpcResponse, PROTOCOL_VERSION,
};
pub use transport::{
    connect_stream, ensure_token, listen_stream, read_message, read_token, write_message,
    HostPaths, TOKEN_FILE,
};
