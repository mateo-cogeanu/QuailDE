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
    pub workspace: usize,
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
    workspace: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    lines: Vec<String>,
    current_line: String,
    title: String,
    shift_pressed: bool,
    caps_lock: bool,
}

impl BuiltinTerminalState {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(TerminalShared {
                visible: false,
                focused: false,
                started: false,
                workspace: 0,
                x: 160,
                y: 112,
                width: 960,
                height: 540,
                lines: vec!["Quail Terminal ready".to_string()],
                current_line: String::new(),
                title: "Quail Terminal".to_string(),
                shift_pressed: false,
                caps_lock: false,
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
            workspace: shared.workspace,
            x: shared.x,
            y: shared.y,
            width: shared.width,
            height: shared.height,
            lines,
            title: shared.title.clone(),
        }
    }

    /// set_workspace keeps the terminal on the currently active workspace so it
    /// behaves more like a real desktop window than a global floating overlay.
    pub fn set_workspace(&self, workspace: usize) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.workspace = workspace;
        }
    }

    /// visible_on_workspace tells the compositor whether the terminal belongs
    /// on the workspace currently being painted.
    pub fn visible_on_workspace(&self, workspace: usize) -> bool {
        self.shared
            .lock()
            .map(|shared| shared.visible && shared.workspace == workspace)
            .unwrap_or(false)
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
        let Ok(mut shared) = self.shared.lock() else {
            return false;
        };
        if linux_key_code == 42 || linux_key_code == 54 {
            shared.shift_pressed = pressed;
            return shared.focused;
        }
        if linux_key_code == 58 && pressed {
            shared.caps_lock = !shared.caps_lock;
            return shared.focused;
        }
        if !pressed || !shared.focused {
            return false;
        }
        let shifted = shared.shift_pressed;
        let caps_lock = shared.caps_lock;
        drop(shared);

        let Some(bytes) = translate_linux_key(linux_key_code, shifted, caps_lock) else {
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

fn translate_linux_key(linux_key_code: u32, shifted: bool, caps_lock: bool) -> Option<Vec<u8>> {
    let uppercase = shifted ^ caps_lock;
    let byte = match linux_key_code {
        2 => {
            if shifted {
                b'!'
            } else {
                b'1'
            }
        }
        3 => {
            if shifted {
                b'@'
            } else {
                b'2'
            }
        }
        4 => {
            if shifted {
                b'#'
            } else {
                b'3'
            }
        }
        5 => {
            if shifted {
                b'$'
            } else {
                b'4'
            }
        }
        6 => {
            if shifted {
                b'%'
            } else {
                b'5'
            }
        }
        7 => {
            if shifted {
                b'^'
            } else {
                b'6'
            }
        }
        8 => {
            if shifted {
                b'&'
            } else {
                b'7'
            }
        }
        9 => {
            if shifted {
                b'*'
            } else {
                b'8'
            }
        }
        10 => {
            if shifted {
                b'('
            } else {
                b'9'
            }
        }
        11 => {
            if shifted {
                b')'
            } else {
                b'0'
            }
        }
        12 => {
            if shifted {
                b'_'
            } else {
                b'-'
            }
        }
        13 => {
            if shifted {
                b'+'
            } else {
                b'='
            }
        }
        15 => b'\t',
        16 => {
            if uppercase {
                b'Q'
            } else {
                b'q'
            }
        }
        17 => {
            if uppercase {
                b'W'
            } else {
                b'w'
            }
        }
        18 => {
            if uppercase {
                b'E'
            } else {
                b'e'
            }
        }
        19 => {
            if uppercase {
                b'R'
            } else {
                b'r'
            }
        }
        20 => {
            if uppercase {
                b'T'
            } else {
                b't'
            }
        }
        21 => {
            if uppercase {
                b'Y'
            } else {
                b'y'
            }
        }
        22 => {
            if uppercase {
                b'U'
            } else {
                b'u'
            }
        }
        23 => {
            if uppercase {
                b'I'
            } else {
                b'i'
            }
        }
        24 => {
            if uppercase {
                b'O'
            } else {
                b'o'
            }
        }
        25 => {
            if uppercase {
                b'P'
            } else {
                b'p'
            }
        }
        26 => {
            if shifted {
                b'{'
            } else {
                b'['
            }
        }
        27 => {
            if shifted {
                b'}'
            } else {
                b']'
            }
        }
        28 => b'\n',
        30 => {
            if uppercase {
                b'A'
            } else {
                b'a'
            }
        }
        31 => {
            if uppercase {
                b'S'
            } else {
                b's'
            }
        }
        32 => {
            if uppercase {
                b'D'
            } else {
                b'd'
            }
        }
        33 => {
            if uppercase {
                b'F'
            } else {
                b'f'
            }
        }
        34 => {
            if uppercase {
                b'G'
            } else {
                b'g'
            }
        }
        35 => {
            if uppercase {
                b'H'
            } else {
                b'h'
            }
        }
        36 => {
            if uppercase {
                b'J'
            } else {
                b'j'
            }
        }
        37 => {
            if uppercase {
                b'K'
            } else {
                b'k'
            }
        }
        38 => {
            if uppercase {
                b'L'
            } else {
                b'l'
            }
        }
        39 => {
            if shifted {
                b':'
            } else {
                b';'
            }
        }
        40 => {
            if shifted {
                b'"'
            } else {
                b'\''
            }
        }
        41 => {
            if shifted {
                b'~'
            } else {
                b'`'
            }
        }
        43 => {
            if shifted {
                b'|'
            } else {
                b'\\'
            }
        }
        44 => {
            if uppercase {
                b'Z'
            } else {
                b'z'
            }
        }
        45 => {
            if uppercase {
                b'X'
            } else {
                b'x'
            }
        }
        46 => {
            if uppercase {
                b'C'
            } else {
                b'c'
            }
        }
        47 => {
            if uppercase {
                b'V'
            } else {
                b'v'
            }
        }
        48 => {
            if uppercase {
                b'B'
            } else {
                b'b'
            }
        }
        49 => {
            if uppercase {
                b'N'
            } else {
                b'n'
            }
        }
        50 => {
            if uppercase {
                b'M'
            } else {
                b'm'
            }
        }
        51 => {
            if shifted {
                b'<'
            } else {
                b','
            }
        }
        52 => {
            if shifted {
                b'>'
            } else {
                b'.'
            }
        }
        53 => {
            if shifted {
                b'?'
            } else {
                b'/'
            }
        }
        57 => b' ',
        _ => {
            return match linux_key_code {
                14 => Some(vec![0x7f]),
                102 => Some(b"\x1b[H".to_vec()),
                107 => Some(b"\x1b[F".to_vec()),
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
