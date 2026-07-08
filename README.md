# retropad

A Petzold-style Win32 Notepad clone rewritten in Rust. It keeps the classic menus, accelerators, word wrap toggle, status bar, find/replace, font picker, time/date insertion, and BOM-aware load/save. Printing is intentionally omitted.

## Prerequisites (Windows)
- [Rust](https://www.rust-lang.org/tools/install) (MSVC or GNU toolchain)
- The Rust toolchain must be able to find `rc.exe` (resource compiler) for embedding the icon/menu resources. If using the GNU toolchain, `windres` is needed.

## Build

```bat
cargo build --release
```

The binary will be at `target\release\retropad.exe`. Clean with:

```bat
cargo clean
```

## Run

```bat
target\release\retropad.exe
```

## Features & notes
- Menus/accelerators: File, Edit, Format, View, Help; classic Notepad key bindings (Ctrl+N/O/S, Ctrl+F, F3, Ctrl+H, Ctrl+G, F5, etc.).
- Word Wrap toggles horizontal scrolling; status bar auto-hides while wrapped, restored when unwrapped.
- Find/Replace dialogs (standard `FINDMSGSTRING`), Go To (disabled when word wrap is on).
- Font picker (ChooseFont), time/date insertion, drag-and-drop to open files.
- File I/O: detects UTF-8/UTF-16 BOMs, falls back to UTF-8/ANSI heuristic; saves with UTF-8 BOM by default.
- Printing/page setup menu items show a "not implemented" notice by design.
- Icon: linked as the main app icon from `res/retropad.ico` via `retropad.rc`.

## Project layout
- `src/main.rs` — entry point
- `src/win32.rs` — Win32 FFI declarations and constants
- `src/app.rs` — window proc, UI logic, menus, find/replace, layout
- `src/file_io.rs` — file open/save dialogs and encoding-aware load/save
- `src/search.rs` — find/replace/replace-all string logic
- `src/resource_ids.rs` — resource ID constants (mirrors `resource.h`)
- `retropad.rc` — menus, accelerators, dialogs, version info, icon
- `resource.h` — resource IDs (C header, consumed by `retropad.rc`)
- `res/retropad.ico` — application icon
- `build.rs` — compile resources, link system libraries
- `Cargo.toml` — Rust project manifest
