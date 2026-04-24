# Changelog

## 2026-04-24
- Added a vendored `plasma-workspace` checkout under `vendor/plasma-workspace` so QuailDE can evolve as a heavily customized Plasma-based desktop instead of only a from-scratch shell experiment.
- Added a custom Plasma look-and-feel package at `vendor/plasma-workspace/lookandfeel/org.quail.masterpiece`, giving the project its own branded dark defaults and a denser, less effect-heavy panel layout script.
- Tuned Plasma shell defaults for responsiveness by preferring non-floating opaque panels in `panelview.cpp`, extending `ShellCorona`'s config sync interval to reduce churn, and slightly debouncing Kicker runner queries so launcher search batches rapid typing more efficiently.
- Stored the current Plasma customization work as `patches/plasma-workspace-quail.patch` and ignored the live `vendor/plasma-workspace` clone in the root repo, so the QuailDE Git history stays valid without trying to vendor the entire upstream KDE workspace.
- Added a built-in PTY-backed `Quail Terminal` so QuailDE can launch and focus a real first-party terminal surface from the panel and launcher even on minimal Linux installs without a working external Wayland terminal.
- Routed Linux keyboard input through the built-in terminal before focused clients when the terminal has focus, and painted the terminal as live shell text instead of another static shell placeholder so QuailDE relies less on decorative rectangle-only UI.
- Made the built-in terminal more usable for daily shell work by adding shifted symbol support, caps lock awareness, more navigation keys, a status footer, and workspace-aware visibility so it behaves more like a real desktop terminal window.
- Added the first everyday shell features around the compositor: workspace switching from the panel, launcher search typing, shell notifications, a quick-settings popover for common toggles, a simple power menu, and workspace-aware routing for newly created client surfaces.
- Reworked shell theming around a dedicated `theme` module so the panel, launcher, terminal, window chrome, notifications, and overlays now share one cohesive dark palette instead of relying on scattered hard-coded colors.
- Broadened launcher app coverage by classifying unknown desktop entries as launchable utility apps, recognizing `/usr/bin/env` desktop-entry wrappers, showing more launcher tiles at once, and letting `Enter` launch the first search result so installed apps are easier to start.
- Made shell notifications expire automatically after one second so they behave like transient toasts instead of piling up on screen.
- Fixed a compositor panic when dragging or focusing oversized windows by clamping drag bounds safely, and tightened keyboard-focus bookkeeping plus `wl_keyboard` setup so GTK apps do not immediately lose the Wayland connection as easily when focused.

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
- Forced the Linux output path to present one full shell frame immediately at startup so QuailDE does not come up on an all-black scanout before the main loop begins.
- Fixed the startup frame presentation path to bind the Linux output backend mutably so the immediate first-frame render compiles on Linux too.
- Hardened the DRM/KMS scanout path with a direct startup test pattern and an explicit CRTC refresh on frame present so VM GPUs are less likely to stay stuck on a stale black buffer.
- Fixed two Linux-only compile issues in the DRM test-pattern path by typing the startup pixels explicitly and computing the dumb-buffer pitch before taking the mutable map borrow.
- Removed fake placeholder windows and turned the raw compositor into a basic real window manager for `xdg_toplevel` surfaces, with focus tracking, server-side decorations, and drag-to-move behavior driven by the mouse.
- Smoothed VM mouse handling by scaling absolute-pointer devices from their observed input range instead of a hard-coded constant, and added a small system-app catalog so QuailDE can auto-launch a real terminal and open installed apps from the dock.
- Replaced the narrow binary-only launcher with desktop-entry discovery plus PATH fallbacks, and switched the Linux absolute-pointer path to use kernel-reported evdev axis bounds when available for much less blocky VM mouse movement.
- Smoothed the VM pointer again with high-resolution cursor easing, advertised a real `wl_output` global plus corrected `xdg_toplevel` configure serials, broadened app launching to handle desktop-entry wrappers and absolute paths, and refreshed the shell visuals with rounded modern surfaces instead of flat debug rectangles.
- Rebuilt the shell look around a dark application launcher and bottom panel inspired by the new reference image, replaced the previous cursor art with a more standard pointer, and expanded app discovery from a tiny category shortlist into a broader launcher inventory backed by desktop entries plus executable PATH fallbacks.
- Added the first real shell/rendering foundation: system-font text rendering through `ab_glyph`, system icon loading through the icon theme and pixmaps directories, a dedicated launcher view model, and early pointer/keyboard event delivery into focused Wayland clients instead of only shell-side cursor handling.
- Replaced the shell-drawn pointer with real XCursor theme loading from the system, kept subpixel cursor positioning all the way into software rendering so VM motion looks smoother, and made QuailDE honor `XCURSOR_THEME` and `XCURSOR_SIZE` when painting the mouse cursor.
- Added an animated cursor target path so sparse VM absolute-pointer events glide instead of stepping, made the launcher act like a real closable menu toggled from the panel, and polished the dark shell layout so app launching can happen from both the menu grid and the bottom panel without the launcher permanently covering the desktop.
- Replaced the bad momentum-style VM mouse path with proper absolute-pointer batching on `SYN_REPORT`, so tablet-style devices now apply X/Y together instead of fighting the hand, and matured the launcher model with real section filtering so the menu behaves more like an actual desktop launcher than a static icon sheet.
