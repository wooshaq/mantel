# Mantel WsQ, native build (Slint)

The same Mantel without WebView2: the Rust backend from `src-tauri/src` with a native interface
in [Slint](https://slint.dev). Uses the same data folder (`%APPDATA%\com.playmantel.launcher`), so
the library, playtime, favorites and settings carry over from the web build.

Built in stages; this one (0.6.0-alpha.1) has Home (shelf, game header, Play, favorites), the
Library grid, launching and session tracking, the controller (XInput) and keyboard, and all six
languages. Sign-in, stores, settings, the control center and search come in later stages.

```bash
cargo build --release                                              # Linux, for development
cargo xwin build --release --target x86_64-pc-windows-msvc         # Windows exe
MANTEL_DEMO=1 MANTEL_WINDOWED=1 cargo run                          # sample library, in a window
```

Keys: arrows, Enter (A), Esc (B), F or X (favorite), Q/E or PageUp/PageDown (LB/RB), F5 (scan
again), F11 (full screen).
