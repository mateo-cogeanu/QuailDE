# QuailDE

QuailDE is the starting point for a lightweight, modern Linux desktop environment.

The long-term goal is not "a pile of desktop apps", but a cohesive shell with:

- a Wayland-first session
- a minimal compositor core
- a panel, launcher, notifications, and settings surfaces
- fast startup and low memory use
- clear module boundaries so the shell can stay simple

## Why this repo exists

Building a desktop environment "from nothing" is a big systems project. The practical way to make it real is to break it into layers:

1. session bootstrap
2. compositor core
3. shell services
4. user-facing shell surfaces

This repository starts with layer 1 and the project architecture so we can grow it deliberately.

## Current status

Right now QuailDE contains:

- a Rust executable for bootstrapping and project diagnostics
- a workspace layout ready for multiple DE components
- TOML-based session configuration
- a session launcher that can dry-run or supervise child processes
- a bundled compositor skeleton binary
- a raw QuailDE compositor path built directly on the Wayland protocol
- architecture and vision docs
- a project layout that can expand into a real shell

QuailDE also now uses KDE Plasma's local [`plasma-workspace`](/Users/mateocogeanu/Downloads/QuailDE/vendor/plasma-workspace) checkout as a customization base. The current plan is to turn that upstream shell into a heavily modified Quail-flavored desktop rather than continue betting everything on the experimental compositor prototype alone. The active Plasma-side modifications are also preserved as [`patches/plasma-workspace-quail.patch`](/Users/mateocogeanu/Downloads/QuailDE/patches/plasma-workspace-quail.patch) so the main repo can track the customization work without vendoring the entire KDE workspace into Git history.

Right now the Plasma path is the one to actually try on Linux. The Rust compositor prototype is still kept here as an earlier experiment, but the practical shell work is happening in the local `plasma-workspace` checkout and the tracked patch file.

## Try The Plasma Build On Linux

To try the Plasma-based QuailDE work, you need a Linux machine with the Plasma 6 / Qt 6 / KDE Frameworks development dependencies required by `plasma-workspace`.

The repo now includes helper scripts for the dev flow:

```bash
cd /Users/mateocogeanu/Downloads/QuailDE
./scripts/build-plasma-workspace.sh
./scripts/install-plasma-dev-session.sh
```

What those do:

- build the modified local [`vendor/plasma-workspace`](/Users/mateocogeanu/Downloads/QuailDE/vendor/plasma-workspace) checkout into `~/.local/quail-plasma`
- install the generated Plasma dev session metadata so your login manager can offer the custom session

After that:

1. log out
2. choose the built-from-source Plasma Wayland dev session on the login screen
3. sign in and select the `Quail Masterpiece` global theme if it is not already active

If you already use Plasma and only want the custom theme/layout without rebuilding the shell, you can copy the look-and-feel package directly:

```bash
mkdir -p ~/.local/share/plasma/look-and-feel
cp -R /Users/mateocogeanu/Downloads/QuailDE/vendor/plasma-workspace/lookandfeel/org.quail.masterpiece ~/.local/share/plasma/look-and-feel/
```

## Run it

```bash
cargo run -- doctor
```

```bash
cargo run -- start
```

Build every QuailDE component:

```bash
cargo build --workspace
```

Use a custom config file:

```bash
cargo run -- --config ./quailde.example.toml doctor
```

The example config lives at [`quailde.example.toml`](quailde.example.toml). The default config path is `~/.config/quailde/quailde.toml`.

## Compositor

QuailDE now includes a bundled compositor placeholder at [`crates/quail-compositor`](crates/quail-compositor). It is not a real Wayland compositor yet, but it gives the session bootstrap a Quail-owned runtime target and defines the next boundary we should implement.

The compositor crate now has explicit modules for backend, output, shell-surface, runtime, and overall state so we can replace placeholders with real Wayland pieces incrementally instead of rewriting one large file later.

The current compositor bootstrap can also create a real Wayland display socket. On Linux with `XDG_RUNTIME_DIR` set, try:

```bash
cargo run -p quail-compositor -- --once --session QuailDE --backend raw
```

That command initializes `wl_display`, binds a `quailde-*` socket, reports the socket name, and exits.

To dump QuailDE's current software-composed frame into an image you can inspect:

```bash
mkdir -p /tmp/quailde-runtime
XDG_RUNTIME_DIR=/tmp/quailde-runtime cargo run -p quail-compositor -- --once --session QuailDE --backend raw --dump-frame /tmp/quailde-frame.ppm
```

Then open `/tmp/quailde-frame.ppm` with any image viewer that supports PPM files.

On macOS, `/tmp` may resolve to `/private/tmp`, so the file may appear at `/private/tmp/quailde-frame.ppm`.

QuailDE also now advertises a real `wl_compositor` global and can initialize `wl_surface`, `wl_region`, and frame callback objects. It still does not render yet, but clients can begin binding core objects against the server.

The next protocol layer is now present too: QuailDE advertises `wl_shm`, accepts shared-memory pool creation, and tracks `wl_buffer` objects. That means clients can start negotiating software-rendered buffers with the compositor, even though QuailDE still does not paint them yet.

QuailDE now also remembers pending and committed surface buffer state. That is the first real scene-management step toward a software renderer, because buffer attachments are no longer thrown away immediately after the request is parsed.

QuailDE now maps shared-memory pools and composes committed surfaces into an in-memory software output buffer. There is still no visible display backend yet, but the compositor is now reading real client pixels instead of only tracking metadata.

QuailDE now also advertises `xdg_wm_base` and can initialize `xdg_surface` and `xdg_toplevel` objects, including basic configure and ack bookkeeping. That is the protocol groundwork desktop-style Wayland applications expect before they can behave like real windows.

QuailDE now also advertises `wl_seat` with pointer and keyboard capabilities. The compositor also has a first raw Linux live path: it prefers DRM/KMS on `/dev/dri/card0`, falls back to `/dev/fb0` when DRM setup fails, and reads mouse or keyboard events from `/dev/input/event*`.

The software shell now paints a darker launcher-and-panel desktop inspired by a more traditional DE layout, manages real `xdg_toplevel` client surfaces with dark server-side decorations, focus tracking, and drag-to-move behavior, and can discover installed system apps from desktop entries plus PATH fallbacks so the launcher, bottom panel, and startup session can expose more than a tiny fixed app list. QuailDE also now renders real text from a system font, resolves app icons from the installed icon theme or pixmaps, has the first real launcher view model plus early pointer and keyboard event delivery into focused Wayland clients, paints the mouse using the system XCursor theme instead of a hard-coded bitmap, treats the launcher like a real closable menu toggled from the bottom panel, and maps VM/tablet absolute pointers on `SYN_REPORT` so the cursor follows the device more directly instead of fighting half-updated axis events. The shell also includes a built-in PTY-backed `Quail Terminal`, so the compositor can launch a real terminal surface from the menu even on minimal systems where no separate Wayland terminal is installed yet. QuailDE now also has the first set of everyday shell features around that terminal: four workspaces, launcher search, notifications, a quick-settings popover for common toggles, and a simple power menu. The current theme is no longer only scattered hard-coded shell colors: QuailDE now has a dedicated theme module, more cohesive dark surfaces, broader launcher result visibility, and launcher search can launch the first matching app directly with `Enter`. Shell notifications now auto-expire quickly, and the compositor is more defensive around large client windows so focusing or dragging them is less likely to tear the session down.

On a Linux VM with no desktop environment, you can now try the first visible QuailDE session from a text console:

```bash
cargo build --workspace
sudo mkdir -p /tmp/quailde-runtime
sudo XDG_RUNTIME_DIR=/tmp/quailde-runtime ./target/debug/quail-compositor --session QuailDE --backend raw --drm-device /dev/dri/card0 --framebuffer /dev/fb0 --input-dir /dev/input
```

You can also test a different cursor theme and size by exporting the standard environment variables before launch:

```bash
export XCURSOR_THEME=Adwaita
export XCURSOR_SIZE=24
```

Notes:

- this raw live path now prefers Linux DRM/KMS plus `evdev`, with `fbdev` kept as a fallback
- QuailDE now keeps text mode by default so testing is safer
- only use `--console-mode graphics` from a real Linux virtual console when you explicitly want the fallback `fbdev` path to take over the tty
- press `Esc` to exit
- arrow keys also move the software cursor if mouse input is unavailable
- if QuailDE logs a DRM warning and falls back to `fbdev`, the VM or permissions likely blocked modesetting on `/dev/dri/card0`
- some VMs expose `/dev/dri/card0` but require root or an active virtual console for modesetting

## Near-term roadmap

- deepen the Plasma-side customization instead of keeping the shell close to stock
- keep cutting steady-state shell work without flattening the visual polish
- replace more default Plasma branding and behavior with Quail-owned identity
- make the Linux build and dev-session workflow easier to reproduce
- keep the old compositor prototype available for experimentation without making it the main path

See [`docs/vision.md`](docs/vision.md) and [`docs/architecture.md`](docs/architecture.md).
