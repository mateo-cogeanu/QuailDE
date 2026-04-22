# Architecture

## Stack choice

QuailDE starts in Rust because:

- memory safety matters for long-lived desktop processes
- performance is strong enough for low-end hardware
- the ecosystem is mature enough to support Linux graphics work

QuailDE uses Wayland as the protocol standard, but the compositor, shell behavior, state management, and rendering path are intended to be QuailDE-owned rather than delegated to a higher-level compositor framework.

## Current repository shape

```text
quailde/
├── docs/
├── crates/
│   └── quail-compositor/
├── src/
│   ├── app.rs
│   ├── config.rs
│   ├── main.rs
│   ├── session.rs
│   └── shell.rs
└── Cargo.toml
└── Cargo.toml
```

This is now a Cargo workspace with the bootstrap binary at the root and the first compositor crate under `crates/`.

## Planned workspace growth

```text
crates/
├── quail-compositor
├── quail-panel
├── quail-shell
├── quail-config
└── quail-ipc
```

## Runtime layers

### Session bootstrap

Responsible for:

- startup checks
- environment validation
- launching and supervising core services

### Compositor core

Responsible for:

- outputs
- input routing
- workspaces
- window management
- basic rendering and protocol handling

### Implementation strategy

- use the current raw protocol bootstrap as the primary compositor path
- grow that path into a full compositor through Wayland globals, shared-memory buffers, software composition, outputs, input, and xdg-shell
- keep QuailDE-specific shell policy, panel behavior, launcher flow, and session logic inside this repo

### Shell services

Responsible for:

- notifications
- launcher state
- settings state
- power and session actions

### Shell surfaces

Responsible for:

- panel
- launcher UI
- notifications UI
- lock screen UI

## Suggested implementation order

1. session bootstrap
2. raw compositor runtime
3. xdg-shell windows plus input/output
4. first visible shell surface
5. panel, launcher, and notifications
6. daily-ish workflow polish

## Important constraint

If we truly avoid all external foundations, progress will be extremely slow. The healthy interpretation of "based on nothing" is:

- own the product and architecture ourselves
- keep dependencies narrow and intentional
- do not build on a heavy existing desktop environment
