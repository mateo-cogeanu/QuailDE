#[cfg(target_os = "linux")]
use std::fs::{File, OpenOptions};
#[cfg(target_os = "linux")]
use std::os::fd::{AsFd, BorrowedFd};
#[cfg(target_os = "linux")]
use std::path::Path;

#[cfg(target_os = "linux")]
use anyhow::{Context, Result, bail};
#[cfg(target_os = "linux")]
use drm::Device as BasicDevice;
#[cfg(target_os = "linux")]
use drm::buffer::Buffer as DrmBuffer;
#[cfg(target_os = "linux")]
use drm::buffer::DrmFourcc;
#[cfg(target_os = "linux")]
use drm::control::connector;
#[cfg(target_os = "linux")]
use drm::control::{Device as ControlDevice, crtc, framebuffer};

#[cfg(target_os = "linux")]
use crate::software::SoftwareFrame;
#[cfg(target_os = "linux")]
use crate::state::CompositorState;

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct Card(File);

#[cfg(target_os = "linux")]
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

#[cfg(target_os = "linux")]
impl BasicDevice for Card {}

#[cfg(target_os = "linux")]
impl ControlDevice for Card {}

#[cfg(target_os = "linux")]
pub struct DrmOutput {
    card: Card,
    connector: connector::Info,
    crtc: crtc::Handle,
    saved_crtc: crtc::Info,
    mode: drm::control::Mode,
    framebuffer: framebuffer::Handle,
    dumb_buffer: drm::control::dumbbuffer::DumbBuffer,
}

#[cfg(target_os = "linux")]
impl DrmOutput {
    /// open configures a legacy DRM/KMS scanout path on the chosen card using a
    /// dumb buffer so QuailDE can present its software-composed frame directly.
    pub fn open(path: &Path, state: &mut CompositorState) -> Result<Self> {
        let card = Card(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .with_context(|| format!("failed to open DRM device {}", path.display()))?,
        );
        card.acquire_master_lock()
            .context("failed to acquire DRM master lock")?;

        let resources = card
            .resource_handles()
            .context("failed to query DRM resources")?;
        let connector = resources
            .connectors()
            .iter()
            .filter_map(|handle| card.get_connector(*handle, true).ok())
            .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
            .context("no connected DRM connector with a valid mode")?;
        let mode = *connector
            .modes()
            .first()
            .context("connected DRM connector had no modes")?;
        let encoder_handle = connector
            .current_encoder()
            .or_else(|| connector.encoders().first().copied())
            .context("connected DRM connector had no usable encoder")?;
        let encoder = card
            .get_encoder(encoder_handle)
            .context("failed to inspect DRM encoder")?;
        let crtc = encoder
            .crtc()
            .or_else(|| {
                resources
                    .filter_crtcs(encoder.possible_crtcs())
                    .into_iter()
                    .next()
            })
            .context("DRM encoder did not expose a compatible CRTC")?;
        let saved_crtc = card
            .get_crtc(crtc)
            .context("failed to inspect current CRTC state")?;

        let (width, height) = mode.size();
        let mut dumb_buffer = card
            .create_dumb_buffer((width.into(), height.into()), DrmFourcc::Xrgb8888, 32)
            .context("failed to create DRM dumb buffer")?;
        let framebuffer = card
            .add_framebuffer(&dumb_buffer, 24, 32)
            .context("failed to create DRM framebuffer object")?;

        {
            let mut map = card
                .map_dumb_buffer(&mut dumb_buffer)
                .context("failed to map DRM dumb buffer")?;
            for byte in map.as_mut() {
                *byte = 0;
            }
        }

        card.set_crtc(
            crtc,
            Some(framebuffer),
            (0, 0),
            &[connector.handle()],
            Some(mode),
        )
        .context("failed to set DRM CRTC mode")?;

        state.outputs.detected_outputs = 1;
        state.outputs.layout = format!("linux drm {} {}", connector, mode.name().to_string_lossy());
        state.composed_width = i32::try_from(width).unwrap_or(1280);
        state.composed_height = i32::try_from(height).unwrap_or(720);
        state.cursor_x = state.composed_width / 2;
        state.cursor_y = state.composed_height / 2;
        state.clamp_cursor();
        state.update_input_focus();
        state.stage = "linux-drm-live";
        state.backend.renderer = "software composition to drm dumb buffer";
        state.backend.input = "evdev pointer and keyboard";

        Ok(Self {
            card,
            connector,
            crtc,
            saved_crtc,
            mode,
            framebuffer,
            dumb_buffer,
        })
    }

    /// present copies the composed software frame into the mapped dumb buffer
    /// that the CRTC is already scanning out.
    pub fn present(&mut self, frame: &SoftwareFrame) -> Result<()> {
        let (width, height) = self.dumb_buffer.size();
        let width = usize::try_from(width).context("invalid DRM buffer width")?;
        let height = usize::try_from(height).context("invalid DRM buffer height")?;
        let pitch = usize::try_from(self.dumb_buffer.pitch()).context("invalid DRM pitch")?;
        let copy_width = frame.width.min(width);
        let copy_height = frame.height.min(height);

        let mut map = self
            .card
            .map_dumb_buffer(&mut self.dumb_buffer)
            .context("failed to remap DRM dumb buffer")?;
        let bytes = map.as_mut();

        for y in 0..copy_height {
            let row_offset = y.saturating_mul(pitch);
            for x in 0..copy_width {
                let pixel = frame.pixels[y * frame.width + x];
                let dst = row_offset.saturating_add(x.saturating_mul(4));
                if dst.saturating_add(4) > bytes.len() {
                    bail!("DRM dumb buffer write would overflow the mapped scanout buffer");
                }
                bytes[dst..dst + 4].copy_from_slice(&pixel.to_le_bytes());
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for DrmOutput {
    fn drop(&mut self) {
        let _ = self.card.set_crtc(
            self.saved_crtc.handle(),
            self.saved_crtc.framebuffer(),
            self.saved_crtc.position(),
            &[self.connector.handle()],
            self.saved_crtc.mode(),
        );
        let _ = self.card.destroy_framebuffer(self.framebuffer);
        let _ = self.card.destroy_dumb_buffer(self.dumb_buffer);
        let _ = self.card.release_master_lock();
        let _ = self.mode;
        let _ = self.crtc;
    }
}

#[cfg(not(target_os = "linux"))]
pub struct DrmOutput;

#[cfg(not(target_os = "linux"))]
impl DrmOutput {
    pub fn open(
        _path: &std::path::Path,
        _state: &mut crate::state::CompositorState,
    ) -> anyhow::Result<Self> {
        anyhow::bail!("the DRM/KMS output backend only runs on Linux")
    }

    pub fn present(&mut self, _frame: &crate::software::SoftwareFrame) -> anyhow::Result<()> {
        anyhow::bail!("the DRM/KMS output backend only runs on Linux")
    }
}
