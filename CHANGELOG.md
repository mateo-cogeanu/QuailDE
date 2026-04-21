# Changelog

## 2026-04-21
- Added a repo-local workflow in `AGENTS.md` to keep code comments, changelog updates, and GitHub pushes top of mind for every change.
- Created `CHANGELOG.md` so each future change has a dedicated place to be recorded.
- Split `quail-compositor` into backend, output, shell, and shared state modules so the compositor skeleton has a clearer path toward a real DE runtime.
- Added a real Wayland bootstrap path to `quail-compositor` using `wayland-server`, including `wl_display` creation, listening socket binding, and a minimal client dispatch loop.
- Added a real `wl_compositor` global plus placeholder `wl_surface`, `wl_region`, and frame callback handlers so QuailDE can initialize core Wayland objects for clients.
- Added `wl_shm`, `wl_shm_pool`, and `wl_buffer` protocol handling so QuailDE can accept shared-memory buffer objects as the next step toward visible rendering.
