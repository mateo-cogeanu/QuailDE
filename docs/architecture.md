# Architecture

## Stack choice

QuailDE starts in Rust because:

- memory safety matters for long-lived desktop processes
- performance is strong enough for low-end hardware
- the ecosystem is mature enough to support Linux graphics work

For a daily-ish desktop, QuailDE should target a Smithay-oriented compositor architecture. The current hand-rolled protocol layer is still useful for learning and smoke tests, but it is not the fastest route to windows, input, outputs, and rendering.

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

- use the current raw protocol bootstrap as a thin compatibility layer for smoke tests
- move feature work toward a Smithay-backed compositor path
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
2. Smithay-backed compositor runtime
3. xdg-shell windows plus input/output
4. first visible shell surface
5. panel, launcher, and notifications
6. daily-ish workflow polish

## Important constraint

If we truly avoid all external foundations, progress will be extremely slow. The healthy interpretation of "based on nothing" is:

- own the product and architecture ourselves
- keep dependencies narrow and intentional
- do not build on a heavy existing desktop environment
