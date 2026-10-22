use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use interprocess::local_socket::{
    ConnectOptions, GenericFilePath, ListenerOptions, Name, ToFsName,
};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use uuid::Uuid;

pub const TOKEN_FILE: &str = "host.token";
const MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct HostPaths {
    pub state_dir: PathBuf,
    pub socket: String,
    pub token: PathBuf,
    pub lock: PathBuf,
}

impl HostPaths {
    pub fn discover(override_dir: Option<&Path>) -> Self {
        let state_dir = override_dir
            .map(Path::to_path_buf)
            .unwrap_or_else(default_state_dir);
        let socket = if cfg!(windows) {
            "ancleto-code-host".to_owned()
        } else {
            state_dir.join("host.sock").to_string_lossy().into_owned()
        };

        Self {
            token: state_dir.join(TOKEN_FILE),
            lock: state_dir.join("host.lock"),
            state_dir,
            socket,
        }
    }
}

pub fn ensure_state_dir(paths: &HostPaths) -> io::Result<()> {
    fs::create_dir_all(&paths.state_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&paths.state_dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn read_token(paths: &HostPaths) -> io::Result<String> {
    fs::read_to_string(&paths.token).map(|token| token.trim().to_owned())
}

pub fn ensure_token(paths: &HostPaths) -> io::Result<String> {
    ensure_state_dir(paths)?;
    if paths.token.is_file() {
        return read_token(paths);
    }

    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    fs::write(&paths.token, format!("{token}\n"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&paths.token, fs::Permissions::from_mode(0o600))?;
    }
    Ok(token)
}

pub fn default_state_dir() -> PathBuf {
    if let Some(dir) = env::var_os("ANCLETO_CODE_STATE") {
        return PathBuf::from(dir);
    }
    #[cfg(windows)]
    {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local).join("AncletoCode").join("state");
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(runtime) = env::var_os("XDG_RUNTIME_DIR") {
            return PathBuf::from(runtime).join("ancleto-code");
        }
        if let Some(state) = env::var_os("XDG_STATE_HOME") {
            return PathBuf::from(state).join("ancleto-code");
        }
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("state")
                .join("ancleto-code");
        }
    }
    PathBuf::from(".ancleto-code")
}

pub fn default_database_path() -> PathBuf {
    if let Some(dir) = env::var_os("ANCLETO_CODE_HOME") {
        return PathBuf::from(dir).join("ancleto-code.sqlite");
    }
    #[cfg(windows)]
    {
        if let Some(appdata) = env::var_os("APPDATA") {
            return PathBuf::from(appdata)
                .join("AncletoCode")
                .join("ancleto-code.sqlite");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("AncletoCode")
                .join("ancleto-code.sqlite");
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(config)
                .join("ancleto-code")
                .join("ancleto-code.sqlite");
        }
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("ancleto-code")
                .join("ancleto-code.sqlite");
        }
    }
    PathBuf::from("ancleto-code.sqlite")
}

pub fn listen_stream(paths: &HostPaths) -> io::Result<interprocess::local_socket::Listener> {
    ensure_state_dir(paths)?;
    let name = socket_name(&paths.socket)?;
    ListenerOptions::new().name(name).create_sync()
}

pub fn connect_stream(paths: &HostPaths) -> io::Result<interprocess::local_socket::Stream> {
    let name = socket_name(&paths.socket)?;
    ConnectOptions::new().name(name).connect_sync()
}

fn socket_name(socket: &str) -> io::Result<Name<'_>> {
    #[cfg(windows)]
    {
        socket.to_ns_name::<GenericNamespaced>()
    }
    #[cfg(not(windows))]
    {
        socket.to_fs_name::<GenericFilePath>()
    }
}

pub fn read_message(reader: &mut impl Read) -> io::Result<serde_json::Value> {
    let mut length = [0u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame inválido"));
    }

    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;
    serde_json::from_slice(&buffer)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn write_message(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame demasiado grande",
        ));
    }
    writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}
