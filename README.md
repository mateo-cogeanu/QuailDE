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
- architecture and vision docs
- a project layout that can expand into a real shell

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
cargo run -p quail-compositor -- --once --session QuailDE
```

That command initializes `wl_display`, binds a `quailde-*` socket, reports the socket name, and exits.

QuailDE also now advertises a real `wl_compositor` global and can initialize `wl_surface`, `wl_region`, and frame callback objects. It still does not render yet, but clients can begin binding core objects against the server.

The next protocol layer is now present too: QuailDE advertises `wl_shm`, accepts shared-memory pool creation, and tracks `wl_buffer` objects. That means clients can start negotiating software-rendered buffers with the compositor, even though QuailDE still does not paint them yet.

## Near-term roadmap

- create a real session process and lifecycle manager
- add configuration loading from XDG paths
- introduce a Wayland compositor crate
- add shell services for notifications, launcher, and panel state
- prototype the first visible shell surface

See [`docs/vision.md`](docs/vision.md) and [`docs/architecture.md`](docs/architecture.md).
