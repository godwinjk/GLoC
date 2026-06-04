# Changelog

All notable changes to the GLoC Reactor Generator plugin are documented here.

## [Unreleased]

## [0.1.0] - 2026-06-04

### Added
- **New → GLoC Reactor** context-menu action in the RustRover Project view
- Dialog built with JetBrains UI DSL v2 — native look and feel
- Live preview of the generated Rust source before file creation
- Two reactor modes: simple (`#[reactor(state = …)]`) and with neutrons
  (`#[reactor(state = …, neutrons = …)]` + `fire()` / `on_event()` handler)
- PascalCase name validation with inline error feedback
- Duplicate-file guard — blocks creation if `<Name>.rs` already exists in the
  target directory
- Generated file opens automatically in the editor after creation
- Restricted to RustRover via `<depends>com.jetbrains.rust</depends>`
