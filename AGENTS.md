# Agent Guidance for `nanai-shiftjis-reader`

## What this repository is

- A Windows-focused Rust application for reading and displaying Shift_JIS-encoded text.
- Main executable path: `src/main.rs`.
- Secondary crate target: `lib.rs` / `src/lib_dll.rs`, compiled as `cdylib` for Windows COM integration.
- Packaging is Windows/MSIX-centered, with `appxmanifest.xml`, `winapp.yaml`, and `nanai-txt-viewer.msix` present.

## Key code areas

- `src/main.rs`
  - Entry point. Detects package state, decodes a file, then launches the GUI.
- `src/file_process.rs`
  - Reads the first CLI argument or defaults to `hello.shiftjis`.
  - Decodes Shift_JIS bytes into UTF-8 using `encoding_rs::SHIFT_JIS`.
- `src/ui.rs`
  - Renders the decoded text in a `gpui` window.
  - The UI is a simple `Render` implementation with a centered window.
- `src/lib_dll.rs`
  - Implements a Windows Explorer command COM object via the `windows` crate.
  - Built when targeting Windows and exposed through the `explorer_com` cdylib.
- `Cargo.toml`
  - Defines dependencies: `encoding_rs`, `gpui`, `windows`, `windows-core`.
  - Library crate type is `cdylib`, so this repo is not only an app but also a native Windows library target.

## Build and run guidance

- Standard build: `cargo build`
- Release build: `cargo build --release`
- Run the GUI app with an optional file path: `cargo run -- <path-to-shiftjis-file>`
- The repository contains Windows packaging metadata; do not assume cross-platform packaging.

## Platform and toolchain notes

- This project is intended for Windows. `src/lib_dll.rs` and `src/lib.rs` are gated with `#[cfg(windows)]`.
- `config.toml` contains a nightly Rust toolchain hint and `sccache`/linker settings.
- Use the Windows toolchain and verify installed SDK components before editing packaging or COM code.

## Agent priorities

- Preserve the Shift_JIS decoding logic and CLI fallback behavior in `src/file_process.rs`.
- Keep the UI simple and centered in `src/ui.rs`; new features should respect the existing `gpui` render pattern.
- Changes to `src/lib_dll.rs` should consider Windows COM lifetime and interface requirements.
- Do not modify packaging manifests unless you understand MSIX and WinApp SDK integration.

## When asked for help

- Prefer source code over assumptions: this repo has no README or project docs.
- If a task involves Windows shell integration, inspect `src/lib_dll.rs` and `Cargo.toml` first.
- If a task involves packaging, inspect `appxmanifest.xml` and `winapp.yaml`.
