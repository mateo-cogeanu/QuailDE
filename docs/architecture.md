# Architecture

## Stack choice

QuailDE starts in Rust because:

- memory safety matters for long-lived desktop processes
- performance is strong enough for low-end hardware
- the ecosystem is mature enough to support Linux graphics work

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

1. bootstrap binary
2. config loading
3. session lifecycle
4. compositor crate
5. first panel surface
6. launcher and notifications

## Important constraint

If we truly avoid all external foundations, progress will be extremely slow. The healthy interpretation of "based on nothing" is:

- own the product and architecture ourselves
- keep dependencies narrow and intentional
- do not build on a heavy existing desktop environment
