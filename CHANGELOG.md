# Changelog

## 2026-04-21
- Added a repo-local workflow in `AGENTS.md` to keep code comments, changelog updates, and GitHub pushes top of mind for every change.
- Created `CHANGELOG.md` so each future change has a dedicated place to be recorded.
- Split `quail-compositor` into backend, output, shell, and shared state modules so the compositor skeleton has a clearer path toward a real DE runtime.
- Added a real Wayland bootstrap path to `quail-compositor` using `wayland-server`, including `wl_display` creation, listening socket binding, and a minimal client dispatch loop.
- Added a real `wl_compositor` global plus placeholder `wl_surface`, `wl_region`, and frame callback handlers so QuailDE can initialize core Wayland objects for clients.
- Added `wl_shm`, `wl_shm_pool`, and `wl_buffer` protocol handling so QuailDE can accept shared-memory buffer objects as the next step toward visible rendering.
- Added per-surface pending and committed buffer tracking so QuailDE now retains scene state instead of only counting protocol requests.

## 2026-04-22
- Pivoted QuailDE’s runtime and docs toward a Smithay-oriented backend path so future work is aimed at a usable daily-ish desktop instead of only raw protocol experiments.
- Restored raw QuailDE as the primary architecture in docs and config, keeping Wayland as the protocol but moving feature work back onto QuailDE’s own compositor path.
- Added real shared-memory pool mapping and in-memory software composition so committed surfaces now produce a composed software frame instead of only metadata counters.
- Added `--dump-frame` support so QuailDE can export its current software-composed output to a PPM image for real-world inspection before a visible display backend exists.
- Added `xdg_wm_base`, `xdg_surface`, and `xdg_toplevel` handling so QuailDE now exposes the first desktop-window protocol layer expected by modern Wayland apps.
- Added `wl_seat`, `wl_pointer`, and `wl_keyboard` groundwork so QuailDE now exposes the first input globals and capability metadata a real desktop session needs.
- Added a first visible raw Linux backend using `fbdev` and `evdev`, including a rendered desktop background, software cursor, basic focus tracking, and live mouse or keyboard cursor movement.
- Switched the raw Linux backend to claim the active tty in graphics mode while QuailDE runs, then restore text mode on exit so the framebuffer output can actually stay visible on a Debian console.
- Hardened tty graphics-mode startup so QuailDE now tries the active Linux VT explicitly and falls back to a warning instead of aborting when a VM does not expose a switchable console device.
- Made tty graphics-mode opt-in instead of implicit so QuailDE no longer risks trapping the user on a Linux console during routine testing.
- Added a preferred DRM/KMS live output path on `/dev/dri/card0` using a dumb buffer and legacy modesetting, with `fbdev` kept as a fallback when direct DRM setup fails.
- Fixed the DRM/KMS output module to import the dumb-buffer trait methods explicitly so Linux builds can call `size()` and `pitch()` correctly.
- Reworked the software shell frame into a clearly visible desktop mockup with a brighter wallpaper, top bar, dock, desktop icons, and placeholder windows so QuailDE now looks like a standard DE before real apps arrive.
