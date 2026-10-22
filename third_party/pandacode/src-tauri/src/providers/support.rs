// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 PandaCode contributors
// Source: https://github.com/aicodeking/pandacode/blob/e9f89891fd23518848765aef8219dbf39feac6e0/src-tauri/src/providers.rs
// Modified by AncletoCode: Extracción de Preview y limited para text_stream

use std::io::Write;

const PREVIEW_TRUNCATED: &str = "\n[Preview truncated]";

struct Preview {
    bytes: Vec<u8>,
    limit: usize,
    truncated: bool,
}

impl Preview {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            truncated: false,
        }
    }

    fn text(&mut self, text: &str) {
        let _ = self.write_all(text.as_bytes());
    }

    fn finish(mut self) -> String {
        if self.truncated {
            self.bytes
                .truncate(self.limit.saturating_sub(PREVIEW_TRUNCATED.len()));
        }
        if let Err(error) = std::str::from_utf8(&self.bytes) {
            self.bytes.truncate(error.valid_up_to());
        }
        let mut text = String::from_utf8(self.bytes).unwrap_or_default();
        if self.truncated {
            text.push_str(PREVIEW_TRUNCATED);
        }
        text
    }
}

impl Write for Preview {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.truncated {
            return Err(std::io::Error::other("Preview full"));
        }
        let count = bytes.len().min(self.limit.saturating_sub(self.bytes.len()));
        self.bytes.extend_from_slice(&bytes[..count]);
        if count != bytes.len() {
            self.truncated = true;
            return Err(std::io::Error::other("Preview full"));
        }
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn limited(text: &str, limit: usize) -> String {
    let mut preview = Preview::new(limit);
    preview.text(text);
    preview.finish()
}
