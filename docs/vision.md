# Vision

## What QuailDE should feel like

QuailDE should feel:

- fast on older hardware
- calm and uncluttered
- keyboard-friendly without punishing mouse users
- modern in animation and visual hierarchy without being bloated
- hackable by one person or a small team

## Product principles

### 1. Lightweight by design

- keep the always-running core small
- separate optional services from required ones
- favor a few reliable background processes over many tiny helpers

### 2. Modern without excess

- Wayland-first
- fractional scaling, sane multi-monitor handling, and smooth animations
- notification, launcher, and control surfaces that are simple and legible

### 3. Cohesive system

- settings, launcher, panel, lock screen, and notifications should behave like one product
- use shared design tokens and shared state contracts

### 4. Build in layers

- session first
- compositor second
- shell services third
- polished UI last

## Scope decisions

QuailDE should include:

- session bootstrap
- compositor/window management
- panel
- launcher
- notification daemon
- settings daemon
- lock screen

QuailDE should not initially include:

- a full file manager
- office apps
- a web browser
- every desktop utility under the sun

## First milestone

The first serious milestone is not "a complete DE". It is:

> Boot into a QuailDE session, start a compositor process, and display one shell surface reliably.
