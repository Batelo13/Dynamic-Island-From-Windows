# Dynamic Island for Windows

Native Rust app (`windows-rs`, Win32 + Windows.UI.Composition). Goal: premium Dynamic-Island-style capsule, idle RAM < 50 MB, idle CPU < 0.5%.

## Build / test
- Toolchain: `stable-x86_64-pc-windows-gnu`. Needs MinGW `dlltool` on PATH (WinLibs via winget):
  `export PATH="$HOME/.cargo/bin:$LOCALAPPDATA/Microsoft/WinGet/Packages/BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe/mingw64/bin:$PATH"`
- `cargo build --release` · `cargo test` · `cargo clippy --all-targets`
- Window behavior check (moves mouse briefly): `powershell -ExecutionPolicy Bypass -File scripts/check-window.ps1`
- Idle benchmark: `scripts/bench.ps1 -Minutes 10` → append to `docs/benchmarks.md`
- Logs: `%LOCALAPPDATA%\DynamicIsland\app.log` · settings: `%APPDATA%\DynamicIsland\settings.json`

## Workflow
- Every relevant feature goes through OpenSpec (`openspec/changes/`, `/opsx:propose` → `/opsx:apply` → `/opsx:archive`). Specs/design are the source of truth for decisions — read them, don't duplicate here.
- Branch `develop`; small commits; never merge to `main` or push without user approval.
- Ponytail: simplest working solution, no speculative abstractions; `ponytail:` comments mark known ceilings.
- Docs/specs in pt-BR; code in English. Visual references: `c:\Users\batel\Downloads\Dynamic 1..5` (JPEG, no extension).

## Architecture notes (non-obvious)
- Three windows (`src/window.rs`): hidden host (tray + system broadcasts), click-through visual (composition), invisible input window clipped by `SetWindowRgn` to the capsule. Don't merge them: this is what makes click-through work across processes.
- No render loop: composition is retained-mode. Any future animation loop must stop when idle.
- Wndproc re-entrancy: use `with_app` (try_borrow) and defer work via `WM_RELAYOUT`.
