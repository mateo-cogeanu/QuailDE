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
