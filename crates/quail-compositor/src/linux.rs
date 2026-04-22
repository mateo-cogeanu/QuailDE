#[cfg(target_os = "linux")]
mod platform {
    use std::fs::{self, File, OpenOptions};
    use std::io::{ErrorKind, Read};
    use std::mem::{size_of, zeroed};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result, bail};
    use memmap2::MmapMut;

    use crate::software::{SoftwareFrame, compose_scene};
    use crate::state::CompositorState;

    const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
    const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
    const KDSETMODE: libc::c_ulong = 0x4B3A;
    const KD_TEXT: libc::c_ulong = 0x00;
    const KD_GRAPHICS: libc::c_ulong = 0x01;

    const EV_KEY: u16 = 0x01;
    const EV_REL: u16 = 0x02;
    const REL_X: u16 = 0x00;
    const REL_Y: u16 = 0x01;
    const KEY_ESC: u16 = 1;
    const KEY_UP: u16 = 103;
    const KEY_LEFT: u16 = 105;
    const KEY_RIGHT: u16 = 106;
    const KEY_DOWN: u16 = 108;
    const BTN_LEFT: u16 = 272;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct FbBitfield {
        offset: u32,
        length: u32,
        msb_right: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    struct FbVarScreeninfo {
        xres: u32,
        yres: u32,
        xres_virtual: u32,
        yres_virtual: u32,
        xoffset: u32,
        yoffset: u32,
        bits_per_pixel: u32,
        grayscale: u32,
        red: FbBitfield,
        green: FbBitfield,
        blue: FbBitfield,
        transp: FbBitfield,
        nonstd: u32,
        activate: u32,
        height: u32,
        width: u32,
        accel_flags: u32,
        pixclock: u32,
        left_margin: u32,
        right_margin: u32,
        upper_margin: u32,
        lower_margin: u32,
        hsync_len: u32,
        vsync_len: u32,
        sync: u32,
        vmode: u32,
        rotate: u32,
        colorspace: u32,
        reserved: [u32; 4],
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    struct FbFixScreeninfo {
        id: [u8; 16],
        smem_start: libc::c_ulong,
        smem_len: u32,
        type_: u32,
        type_aux: u32,
        visual: u32,
        xpanstep: u16,
        ypanstep: u16,
        ywrapstep: u16,
        line_length: u32,
        mmio_start: libc::c_ulong,
        mmio_len: u32,
        accel: u32,
        capabilities: u16,
        reserved: [u16; 2],
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    struct InputEvent {
        time: libc::timeval,
        type_: u16,
        code: u16,
        value: i32,
    }

    /// LinuxPlatform owns the first visible raw Linux backend for QuailDE:
    /// fbdev for pixels and evdev for mouse/keyboard input.
    pub struct LinuxPlatform {
        console: Option<ConsoleModeGuard>,
        framebuffer: LinuxFramebuffer,
        input_devices: Vec<InputDevice>,
    }

    struct ConsoleModeGuard {
        file: File,
    }

    struct LinuxFramebuffer {
        _file: File,
        pixels: MmapMut,
        width: usize,
        height: usize,
        stride: usize,
        bits_per_pixel: u32,
    }

    struct InputDevice {
        _path: PathBuf,
        file: File,
    }

    impl LinuxPlatform {
        pub fn create(
            state: &mut CompositorState,
            framebuffer_path: &Path,
            input_dir: &Path,
        ) -> Result<Self> {
            let console = match ConsoleModeGuard::enter_graphics_mode() {
                Ok(console) => Some(console),
                Err(error) => {
                    eprintln!(
                        "warning: could not switch the active tty into graphics mode: {error}"
                    );
                    None
                }
            };
            let framebuffer = LinuxFramebuffer::open(framebuffer_path)?;
            let input_devices = discover_input_devices(input_dir)?;

            state.outputs.detected_outputs = 1;
            state.outputs.layout = format!(
                "linux fbdev {}x{} @ {}bpp",
                framebuffer.width, framebuffer.height, framebuffer.bits_per_pixel
            );
            state.composed_width = framebuffer.width as i32;
            state.composed_height = framebuffer.height as i32;
            state.cursor_x = state.composed_width / 2;
            state.cursor_y = state.composed_height / 2;
            state.clamp_cursor();
            state.update_input_focus();
            state.stage = "linux-live";
            state.backend.renderer = "software composition to fbdev";
            state.backend.input = if console.is_some() {
                "evdev pointer and keyboard"
            } else {
                "evdev pointer and keyboard (tty graphics mode unavailable)"
            };

            Ok(Self {
                console,
                framebuffer,
                input_devices,
            })
        }

        pub fn tick(&mut self, state: &mut CompositorState) -> Result<()> {
            self.poll_input(state)?;
            state.clamp_cursor();
            state.update_input_focus();

            let frame = compose_scene(state);
            self.framebuffer.present(&frame)?;
            state.last_frame_checksum = frame.checksum;
            state.last_frame_painted_surfaces = frame.painted_surfaces;
            state.presented_frames += 1;
            Ok(())
        }

        fn poll_input(&mut self, state: &mut CompositorState) -> Result<()> {
            let event_size = size_of::<InputEvent>();
            let mut buffer = vec![0_u8; event_size.saturating_mul(32)];

            for device in &mut self.input_devices {
                loop {
                    match device.file.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(read_len) => {
                            let complete_len = read_len - (read_len % event_size);
                            for chunk in buffer[..complete_len].chunks_exact(event_size) {
                                // The kernel writes a packed `input_event` array,
                                // so reinterpreting each full chunk is the most
                                // direct way to keep this raw input path minimal.
                                let event = unsafe {
                                    std::ptr::read_unaligned(chunk.as_ptr() as *const InputEvent)
                                };
                                handle_input_event(state, event);
                            }
                        }
                        Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                        Err(error) => {
                            return Err(error).context("failed to read evdev input event");
                        }
                    }
                }
            }

            Ok(())
        }
    }

    impl ConsoleModeGuard {
        fn enter_graphics_mode() -> Result<Self> {
            for path in candidate_console_paths()? {
                let file = match OpenOptions::new().read(true).write(true).open(&path) {
                    Ok(file) => file,
                    Err(_) => continue,
                };

                // The Linux text console keeps drawing characters until the tty
                // is switched into graphics mode, so QuailDE tries the current
                // VT devices directly instead of assuming `/dev/tty` is enough.
                let status = unsafe { libc::ioctl(file.as_raw_fd(), KDSETMODE, KD_GRAPHICS) };
                if status >= 0 {
                    return Ok(Self { file });
                }
            }

            Err(std::io::Error::last_os_error())
                .context("failed to switch any available Linux virtual console into graphics mode")
        }
    }

    impl Drop for ConsoleModeGuard {
        fn drop(&mut self) {
            // Restoring text mode makes sure the Linux tty comes back after
            // QuailDE exits, even if the compositor returns early on error.
            let _ = unsafe { libc::ioctl(self.file.as_raw_fd(), KDSETMODE, KD_TEXT) };
        }
    }

    impl LinuxFramebuffer {
        fn open(path: &Path) -> Result<Self> {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .with_context(|| format!("failed to open framebuffer {}", path.display()))?;

            let mut var: FbVarScreeninfo = unsafe { zeroed() };
            let mut fix: FbFixScreeninfo = unsafe { zeroed() };

            // fbdev still exposes the simplest direct pixel path on Linux, so
            // QuailDE uses it as the first visible backend before DRM/KMS.
            let var_status =
                unsafe { libc::ioctl(file.as_raw_fd(), FBIOGET_VSCREENINFO, &mut var) };
            if var_status < 0 {
                return Err(std::io::Error::last_os_error())
                    .context("failed to query variable framebuffer info");
            }

            let fix_status =
                unsafe { libc::ioctl(file.as_raw_fd(), FBIOGET_FSCREENINFO, &mut fix) };
            if fix_status < 0 {
                return Err(std::io::Error::last_os_error())
                    .context("failed to query fixed framebuffer info");
            }

            let width = usize::try_from(var.xres).context("invalid framebuffer width")?;
            let height = usize::try_from(var.yres).context("invalid framebuffer height")?;
            let stride = usize::try_from(fix.line_length).context("invalid framebuffer stride")?;
            let len = usize::try_from(fix.smem_len).context("invalid framebuffer memory length")?;
            let pixels = unsafe { MmapMut::map_mut(&file) }
                .or_else(|_| unsafe { memmap2::MmapOptions::new().len(len).map_mut(&file) })
                .context("failed to mmap framebuffer memory")?;

            Ok(Self {
                _file: file,
                pixels,
                width,
                height,
                stride,
                bits_per_pixel: var.bits_per_pixel,
            })
        }

        fn present(&mut self, frame: &SoftwareFrame) -> Result<()> {
            let width = frame.width.min(self.width);
            let height = frame.height.min(self.height);

            match self.bits_per_pixel {
                32 => {
                    for y in 0..height {
                        let row_offset = y.saturating_mul(self.stride);
                        for x in 0..width {
                            let pixel = frame.pixels[y * frame.width + x];
                            let dst = row_offset.saturating_add(x.saturating_mul(4));
                            if dst.saturating_add(4) > self.pixels.len() {
                                continue;
                            }
                            self.pixels[dst..dst + 4].copy_from_slice(&pixel.to_le_bytes());
                        }
                    }
                }
                16 => {
                    for y in 0..height {
                        let row_offset = y.saturating_mul(self.stride);
                        for x in 0..width {
                            let pixel = frame.pixels[y * frame.width + x];
                            let red = ((pixel >> 16) & 0xFF) as u16;
                            let green = ((pixel >> 8) & 0xFF) as u16;
                            let blue = (pixel & 0xFF) as u16;
                            let rgb565 = ((red >> 3) << 11) | ((green >> 2) << 5) | (blue >> 3);
                            let dst = row_offset.saturating_add(x.saturating_mul(2));
                            if dst.saturating_add(2) > self.pixels.len() {
                                continue;
                            }
                            self.pixels[dst..dst + 2].copy_from_slice(&rgb565.to_le_bytes());
                        }
                    }
                }
                other => {
                    bail!("unsupported framebuffer depth: {other}bpp");
                }
            }

            self.pixels
                .flush()
                .context("failed to flush framebuffer mmap")
        }
    }

    fn discover_input_devices(input_dir: &Path) -> Result<Vec<InputDevice>> {
        let mut event_paths = fs::read_dir(input_dir)
            .with_context(|| format!("failed to read input directory {}", input_dir.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("event"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        event_paths.sort();

        if event_paths.is_empty() {
            bail!("no evdev devices found in {}", input_dir.display());
        }

        let mut devices = Vec::with_capacity(event_paths.len());
        for path in event_paths {
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&path)
                .with_context(|| format!("failed to open input device {}", path.display()))?;
            devices.push(InputDevice { _path: path, file });
        }

        Ok(devices)
    }

    fn candidate_console_paths() -> Result<Vec<PathBuf>> {
        let mut paths = vec![PathBuf::from("/dev/tty"), PathBuf::from("/dev/console")];

        // Linux exposes the foreground VT name here, which is more reliable in
        // VMs than guessing the controlling tty path from the current process.
        if let Ok(active) = fs::read_to_string("/sys/class/tty/tty0/active") {
            let active = active.trim();
            if !active.is_empty() {
                paths.insert(0, PathBuf::from(format!("/dev/{active}")));
            }
        }

        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn handle_input_event(state: &mut CompositorState, event: InputEvent) {
        state.input_events_processed += 1;
        state.last_input_event = format!(
            "type={} code={} value={}",
            event.type_, event.code, event.value
        );

        match (event.type_, event.code, event.value) {
            (EV_REL, REL_X, delta) => state.cursor_x = state.cursor_x.saturating_add(delta),
            (EV_REL, REL_Y, delta) => state.cursor_y = state.cursor_y.saturating_add(delta),
            (EV_KEY, BTN_LEFT, 1) => state.pointer_buttons_pressed = 1,
            (EV_KEY, BTN_LEFT, 0) => state.pointer_buttons_pressed = 0,
            (EV_KEY, KEY_LEFT, 1) => state.cursor_x = state.cursor_x.saturating_sub(24),
            (EV_KEY, KEY_RIGHT, 1) => state.cursor_x = state.cursor_x.saturating_add(24),
            (EV_KEY, KEY_UP, 1) => state.cursor_y = state.cursor_y.saturating_sub(24),
            (EV_KEY, KEY_DOWN, 1) => state.cursor_y = state.cursor_y.saturating_add(24),
            (EV_KEY, KEY_ESC, 1) => state.quit_requested = true,
            _ => {}
        }
    }

    pub fn create_linux_platform(
        state: &mut CompositorState,
        framebuffer_path: &Path,
        input_dir: &Path,
    ) -> Result<LinuxPlatform> {
        LinuxPlatform::create(state, framebuffer_path, input_dir)
    }

    pub use LinuxPlatform as Platform;
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use std::path::Path;

    use anyhow::{Result, bail};

    use crate::state::CompositorState;

    pub struct Platform;

    impl Platform {
        pub fn tick(&mut self, _state: &mut CompositorState) -> Result<()> {
            bail!("the live raw QuailDE backend only runs on Linux")
        }
    }

    pub fn create_linux_platform(
        _state: &mut CompositorState,
        _framebuffer_path: &Path,
        _input_dir: &Path,
    ) -> Result<Platform> {
        bail!("the live raw QuailDE backend only runs on Linux")
    }
}

pub use platform::{Platform as LinuxPlatform, create_linux_platform};
