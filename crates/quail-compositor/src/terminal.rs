use std::fmt;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

const BUILTIN_TERMINAL_COMMAND: &str = "quail:terminal";

/// BuiltinTerminalState keeps a first-party PTY-backed shell available even on
/// minimal Linux installs that do not yet have a working Wayland terminal app.
#[derive(Clone)]
pub struct BuiltinTerminalState {
    shared: Arc<Mutex<TerminalShared>>,
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
}

#[derive(Debug, Clone)]
pub struct TerminalSnapshot {
    pub visible: bool,
    pub focused: bool,
    pub started: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub lines: Vec<String>,
    pub title: String,
}

struct TerminalShared {
    visible: bool,
    focused: bool,
    started: bool,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    lines: Vec<String>,
    current_line: String,
    title: String,
}

impl BuiltinTerminalState {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(TerminalShared {
                visible: false,
                focused: false,
                started: false,
                x: 160,
                y: 112,
                width: 960,
                height: 540,
                lines: vec!["Quail Terminal ready".to_string()],
                current_line: String::new(),
                title: "Quail Terminal".to_string(),
            })),
            writer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_builtin_terminal_command(command: &str) -> bool {
        command == BUILTIN_TERMINAL_COMMAND
    }

    pub fn builtin_command_name() -> &'static str {
        BUILTIN_TERMINAL_COMMAND
    }

    pub fn show(&self) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.visible = true;
            shared.focused = true;
        }
    }

    pub fn hide(&self) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.visible = false;
            shared.focused = false;
        }
    }

    pub fn unfocus(&self) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.focused = false;
        }
    }

    pub fn focus_if_contains(&self, cursor_x: i32, cursor_y: i32) -> bool {
        let Ok(mut shared) = self.shared.lock() else {
            return false;
        };
        let contains = shared.visible
            && cursor_x >= shared.x
            && cursor_x < shared.x + shared.width
            && cursor_y >= shared.y
            && cursor_y < shared.y + shared.height;
        shared.focused = contains;
        contains
    }

    pub fn close_button_hit(&self, cursor_x: i32, cursor_y: i32) -> bool {
        let Ok(shared) = self.shared.lock() else {
            return false;
        };
        if !shared.visible {
            return false;
        }
        let button_x = shared.x + shared.width - 28;
        let button_y = shared.y + 10;
        cursor_x >= button_x
            && cursor_x < button_x + 18
            && cursor_y >= button_y
            && cursor_y < button_y + 18
    }

    pub fn is_focused(&self) -> bool {
        self.shared
            .lock()
            .map(|shared| shared.focused)
            .unwrap_or(false)
    }

    pub fn snapshot(&self) -> TerminalSnapshot {
        let shared = self.shared.lock().expect("terminal shared poisoned");
        let mut lines = shared.lines.clone();
        if !shared.current_line.is_empty() {
            lines.push(shared.current_line.clone());
        }
        TerminalSnapshot {
            visible: shared.visible,
            focused: shared.focused,
            started: shared.started,
            x: shared.x,
            y: shared.y,
            width: shared.width,
            height: shared.height,
            lines,
            title: shared.title.clone(),
        }
    }

    pub fn ensure_started(&self) -> Result<()> {
        let already_started = self
            .shared
            .lock()
            .map(|shared| shared.started)
            .unwrap_or(false);
        if already_started {
            self.show();
            return Ok(());
        }

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: 28,
                cols: 104,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to create terminal pty")?;
        let mut command = CommandBuilder::new(shell);
        command.env("TERM", "xterm-256color");
        let _child = pair
            .slave
            .spawn_command(command)
            .context("failed to spawn shell in terminal")?;
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone terminal reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("failed to take terminal writer")?;

        if let Ok(mut writer_slot) = self.writer.lock() {
            *writer_slot = Some(writer);
        }
        if let Ok(mut shared) = self.shared.lock() {
            shared.started = true;
            shared.visible = true;
            shared.focused = true;
            shared.lines.push("Launching shell...".to_string());
        }

        let shared = self.shared.clone();
        // The PTY reader lives on a background thread so the compositor loop
        // stays focused on input, Wayland dispatch, and frame presentation.
        thread::spawn(move || read_terminal_output(reader, shared));
        Ok(())
    }

    pub fn handle_key_event(&self, linux_key_code: u32, pressed: bool) -> bool {
        if !pressed || !self.is_focused() {
            return false;
        }
        let Some(bytes) = translate_linux_key(linux_key_code) else {
            return false;
        };
        let Ok(mut writer_slot) = self.writer.lock() else {
            return false;
        };
        let Some(writer) = writer_slot.as_mut() else {
            return false;
        };
        let _ = writer.write_all(&bytes);
        let _ = writer.flush();
        true
    }
}

impl fmt::Debug for BuiltinTerminalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let snapshot = self.snapshot();
        f.debug_struct("BuiltinTerminalState")
            .field("visible", &snapshot.visible)
            .field("focused", &snapshot.focused)
            .field("started", &snapshot.started)
            .field("lines", &snapshot.lines.len())
            .finish()
    }
}

fn read_terminal_output(mut reader: Box<dyn Read + Send>, shared: Arc<Mutex<TerminalShared>>) {
    let mut buffer = [0_u8; 4096];
    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(_) => break,
        };
        append_terminal_output(&shared, &buffer[..read]);
    }
}

fn append_terminal_output(shared: &Arc<Mutex<TerminalShared>>, bytes: &[u8]) {
    let Ok(mut shared) = shared.lock() else {
        return;
    };
    // This first terminal view intentionally keeps parsing simple: printable
    // text, newlines, and backspace are enough to make a shell session usable
    // before QuailDE grows a full terminal emulation layer.
    for ch in String::from_utf8_lossy(bytes).chars() {
        match ch {
            '\r' => {}
            '\n' => {
                let line = std::mem::take(&mut shared.current_line);
                shared.lines.push(line);
            }
            '\u{0008}' => {
                shared.current_line.pop();
            }
            _ if ch.is_control() => {}
            _ => shared.current_line.push(ch),
        }
    }
    while shared.lines.len() > 512 {
        shared.lines.remove(0);
    }
}

fn translate_linux_key(linux_key_code: u32) -> Option<Vec<u8>> {
    let byte = match linux_key_code {
        2 => b'1',
        3 => b'2',
        4 => b'3',
        5 => b'4',
        6 => b'5',
        7 => b'6',
        8 => b'7',
        9 => b'8',
        10 => b'9',
        11 => b'0',
        12 => b'-',
        13 => b'=',
        15 => b'\t',
        16 => b'q',
        17 => b'w',
        18 => b'e',
        19 => b'r',
        20 => b't',
        21 => b'y',
        22 => b'u',
        23 => b'i',
        24 => b'o',
        25 => b'p',
        26 => b'[',
        27 => b']',
        28 => b'\n',
        30 => b'a',
        31 => b's',
        32 => b'd',
        33 => b'f',
        34 => b'g',
        35 => b'h',
        36 => b'j',
        37 => b'k',
        38 => b'l',
        39 => b';',
        40 => b'\'',
        41 => b'`',
        43 => b'\\',
        44 => b'z',
        45 => b'x',
        46 => b'c',
        47 => b'v',
        48 => b'b',
        49 => b'n',
        50 => b'm',
        51 => b',',
        52 => b'.',
        53 => b'/',
        57 => b' ',
        _ => {
            return match linux_key_code {
                14 => Some(vec![0x7f]),
                103 => Some(b"\x1b[A".to_vec()),
                108 => Some(b"\x1b[B".to_vec()),
                105 => Some(b"\x1b[D".to_vec()),
                106 => Some(b"\x1b[C".to_vec()),
                _ => None,
            };
        }
    };
    Some(vec![byte])
}
