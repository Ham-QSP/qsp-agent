# AGENTS.md

## Project
- Name: `qsp-remote-agent`
- Type: Rust workspace
- Purpose: Service daemon (aka agent) to connect an hardware radio to external clients using webrtc through a signaling server

## Workspace Layout
- `Cargo.toml`: workspace root
- `qsp-agent/`: main daemon crate
- `lib-hamlib/`: rust binding to a radio control library crate
- `Readme.md`: project description for humans

## Working Rules
- Prefer minimal, targeted changes.
- Preserve the existing Rust workspace structure.
- Do not introduce new dependencies unless they are necessary.
- Keep file and module names consistent with the current layout.
- Avoid touching unrelated files, especially generated content under `target/`.

## Commands
- Build workspace: `cargo build`
- Test workspace: `cargo test`
- Check formatting: `cargo fmt --all --check`
- Run linting: `cargo clippy --workspace --all-targets`

## Editing Guidance
- Make changes in the smallest relevant crate.
- If a change affects shared radio behavior, inspect `lib-hamlib/` first.
- If a change affects the agent runtime or signaling flow, inspect `qsp-agent/` first.
- Update documentation when behavior or setup changes.

## Validation
- Run the narrowest useful command first.
- For code changes, prefer at least `cargo test` or `cargo check` before finishing.
- If validation cannot be run, state that explicitly.
