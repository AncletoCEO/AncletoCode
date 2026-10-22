use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ancleto_code_core::host::HostClient;
use ancleto_code_core::{Result, SessionBackend};
use serde_json::Value;

use crate::vt::{Selection, TerminalSnapshot, TerminalState, UiEvent};

pub struct TerminalSession {
    client: Arc<HostClient>,
    session_id: String,
    state: Arc<Mutex<TerminalState>>,
    stop: Arc<AtomicBool>,
    reader: Option<JoinHandle<()>>,
}

impl TerminalSession {
    pub fn attach(
        client: Arc<HostClient>,
        session_id: impl Into<String>,
        columns: usize,
        lines: usize,
    ) -> Result<Self> {
        let session_id = session_id.into();
        let mut state = TerminalState::new(columns, lines);
        let snapshot = client.attach(&session_id)?;
        state.feed(&snapshot);

        let state = Arc::new(Mutex::new(state));
        let stop = Arc::new(AtomicBool::new(false));
        let events = client.events();
        let reader_state = Arc::clone(&state);
        let reader_stop = Arc::clone(&stop);
        let reader_id = session_id.clone();
        let reader = thread::spawn(move || {
            while !reader_stop.load(Ordering::SeqCst) {
                match events.recv_timeout(Duration::from_millis(100)) {
                    Ok(notification) => {
                        let id = notification.params.get("id").and_then(Value::as_str);
                        if id != Some(reader_id.as_str()) {
                            continue;
                        }
                        match notification.method.as_str() {
                            "session.output" => {
                                if let Ok(data) = serde_json::from_value::<Vec<u8>>(
                                    notification.params["data"].clone(),
                                ) {
                                    if let Ok(mut state) = reader_state.lock() {
                                        state.feed(&data);
                                    }
                                }
                            }
                            "session.exit" => break,
                            _ => {}
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(Self {
            client,
            session_id,
            state,
            stop,
            reader: Some(reader),
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.client.write_session(&self.session_id, data)
    }

    pub fn resize(&self, columns: usize, lines: usize) -> Result<()> {
        self.client
            .resize_session(&self.session_id, columns as u16, lines as u16)?;
        if let Ok(mut state) = self.state.lock() {
            state.resize(columns, lines);
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Option<TerminalSnapshot> {
        self.state.lock().ok().map(|state| state.snapshot())
    }

    pub fn search(&self, query: &str, case_sensitive: bool) -> Vec<(i32, usize)> {
        self.state
            .lock()
            .map(|state| state.search(query, case_sensitive))
            .unwrap_or_default()
    }

    pub fn incremental_search(
        &self,
        query: &str,
        from_line: i32,
        case_sensitive: bool,
    ) -> Option<(i32, usize)> {
        self.state
            .lock()
            .ok()
            .and_then(|mut state| state.incremental_search(query, from_line, case_sensitive))
    }

    pub fn scroll(&self, delta: i32) {
        if let Ok(mut state) = self.state.lock() {
            state.scroll(delta);
        }
    }

    pub fn selection_text(&self, selection: &Selection) -> String {
        self.state
            .lock()
            .map(|state| state.selection_text(selection))
            .unwrap_or_default()
    }

    pub fn take_events(&self) -> Vec<UiEvent> {
        self.state
            .lock()
            .map(|mut state| state.take_events())
            .unwrap_or_default()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.client.detach(&self.session_id);
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
