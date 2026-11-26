## Context

Repositório vazio; stack aprovada: Rust + `windows-rs` (ver `proposal.md`). Máquina de desenvolvimento: Windows 11 26200, i5-13420H, 24 GB, RTX 4050 + Intel UHD (híbrida). Sem Visual Studio: usamos o toolchain `x86_64-pc-windows-gnu`, que linka `windows-rs` sem MSVC.

## Goals / Non-Goals

**Goals:**
- Provar janela transparente + click-through + sem foco com custo ocioso ~zero.
- Base de módulos pequena que as fases de animação e integrações estendem sem reescrever.

**Non-Goals:**
- Molas/morphing, texto, ícones, estados além de Idle (Fase 2).
- Inicialização automática, MSIX, UI de configurações (Fases 7/9).

## Decisions

### D1 — Renderização: `Windows.UI.Composition` em janela Win32
A cápsula é um `ShapeVisual` com `CompositionRoundedRectangleGeometry`, ligado à janela via `ICompositorDesktopInterop::CreateDesktopWindowTarget`. A janela usa `WS_EX_NOREDIRECTIONBITMAP`, então não há superfície GDI: transparência por pixel vem de graça e o DWM compõe na GPU.
- Parado, não há nenhum quadro desenhado pelo app: CPU 0.
- Na Fase 2, tamanho e raio da geometria são animáveis diretamente (molas no processo alimentando as propriedades), e o texto entra via `CompositionDrawingSurface` + Direct2D/DirectWrite.
- **Alternativas:** DirectComposition puro (mais código para formas, mesmo resultado); Direct2D em swap chain (exige laço de render próprio); GDI/`UpdateLayeredWindow` (CPU, sem vsync).
- Requer `DispatcherQueueController` na thread da UI (`CreateDispatcherQueueController`).

### D2 — Click-through: janela visual transparente + janela de entrada recortada
Duas janelas top-level do mesmo processo e thread:
1. **Visual**: área generosa (cápsula + margem para sombra/expansão futura), `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOREDIRECTIONBITMAP`. Nunca recebe mouse.
2. **Entrada**: popup invisível (`WS_EX_LAYERED` com alfa 1, `NOACTIVATE`, `TOOLWINDOW`, `TOPMOST`), com `SetWindowRgn` igual ao contorno da cápsula. Recebe só os cliques dentro da forma.

Na Fase 2, a cada quadro de animação atualizamos apenas a região/retângulo da janela de entrada (barato, sem repintura visível); a janela visual nunca é redimensionada durante a animação, eliminando tremor.
- **Alternativas:** `WM_NCHITTEST → HTTRANSPARENT` (não repassa cliques entre processos); uma janela só com `SetWindowRgn` (recorta sombra e conteúdo que sai da forma); alternar `WS_EX_TRANSPARENT` via hook global de mouse (custo constante, risco com anti-cheat — descartado).
- **Validação primeiro** (tarefa 2): se `LAYERED + NOREDIRECTIONBITMAP` não compor corretamente, fallback documentado é a janela única com `SetWindowRgn` e sombra desenhada dentro da região.
- **Resultado (2026-11-25):** validado sem fallback — composição renderiza com `LAYERED + TRANSPARENT + NOREDIRECTIONBITMAP`; `scripts/check-window.ps1` confirma hit-test, cliques reais e foco (12/12). Uma terceira janela oculta (*host*) ficou responsável pela bandeja e mensagens do sistema, porque o menu da bandeja exige `SetForegroundWindow` e as janelas da ilha são `NOACTIVATE`.

### D3 — Foco
`WS_EX_NOACTIVATE` nas duas janelas, `ShowWindow(SW_SHOWNOACTIVATE)`, `SetWindowPos(..., SWP_NOACTIVATE)` e `WM_MOUSEACTIVATE → MA_NOACTIVATE`. `WS_EX_TOOLWINDOW` tira da barra de tarefas e do Alt+Tab.

### D4 — DPI e posicionamento
`SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)` na primeira linha do `main` (dispensa manifesto agora; o manifesto entra com o MSIX). Geometria em DIP → px com `GetDpiForMonitor`. Posição calculada por função pura `layout(work_area, dpi, settings) -> Rect`, testável. Recalcular em `WM_DISPLAYCHANGE`, `WM_DPICHANGED`, `WM_SETTINGCHANGE(SPI_SETWORKAREA)` e `WM_POWERBROADCAST(PBT_APMRESUMEAUTOMATIC)`; monitor escolhido por índice salvo + fallback para o principal.

### D5 — Bloqueio/sessão
`WTSRegisterSessionNotification` → em `WTS_SESSION_UNLOCK` recalcula layout. Nada a pausar ainda (não há laço); o gancho já fica pronto para a Fase 2 suspender animações.

### D6 — Ciclo de vida, bandeja, configurações, log
- Instância única: `CreateMutexW("Local\\DynamicIslandForWindows")`; `ERROR_ALREADY_EXISTS` → sair.
- Bandeja: `Shell_NotifyIconW` + `TrackPopupMenu`; ícone provisório `IDI_APPLICATION` até existir arte própria. Recriar o ícone em `TaskbarCreated` (Explorer reiniciado).
- Textos do menu: tabela pt-BR/en em código (`i18n.rs`) escolhida por `GetUserDefaultUILanguage`; vira arquivos quando houver mais textos (Fase 7).
- Config: `serde_json` em `%APPDATA%\DynamicIsland\settings.json`, gravação atômica (arquivo temporário + `rename`), validação/clamp na leitura, `.bak` em caso de JSON inválido.
- Log: `%LOCALAPPDATA%\DynamicIsland\app.log`, só nível erro/aviso + tempo de inicialização, rotação em 1 MB (`app.log.1`). Sem crate de log.
- Release com `#![windows_subsystem = "windows"]`; `panic = "abort"`, `opt-level = "s"`, `lto = true`, `strip = true` para binário pequeno.

### D7 — Estrutura (mínima)
```
src/main.rs      entrada, mutex, laço de mensagens, bandeja
src/window.rs    criação das janelas, estilos, mensagens de tela/energia/sessão
src/render.rs    compositor, DispatcherQueue, cápsula
src/layout.rs    cálculo puro de posição (+ testes)
src/settings.rs  carregar/salvar/validar (+ testes)
src/log.rs       log com rotação
src/i18n.rs      textos pt-BR/en
scripts/bench.ps1  medição de RAM/CPU
```
Sem traits/abstrações até haver segundo consumidor.

### D8 — Benchmark
`scripts/bench.ps1`: inicia o app, amostra `PrivateMemorySize64`, `WorkingSet64` e tempo de CPU a cada 5 s por N minutos, e reporta média/pico em `docs/benchmarks.md` com hardware, build e condições. Inicialização: o app loga `startup_ms` do início do `main` até o primeiro `Commit` do compositor + `ShowWindow`.

## Risks / Trade-offs

- [`WS_EX_LAYERED` + `NOREDIRECTIONBITMAP` + composição pode não renderizar em algumas builds] → validar logo na tarefa 2; fallback D2.
- [GPU híbrida: compositor em uma GPU, app em outra] → shapes de composição não usam device D3D do app; risco só a partir da Fase 2 (texto) — tratar `device lost` lá.
- [Toolchain GNU e futuros crates com C] → só `windows`, `windows-numerics`, `serde`, `serde_json`; se precisar de MSVC, troca de target sem mudar código. O GNU exige `dlltool` (MinGW/WinLibs) no PATH para as import libs do `windows-rs`.
- [Ícone provisório na bandeja] → aceitável até o design system; marcado como pendência visual.
- [Tela cheia exclusiva cobre a ilha] → esperado; tratado no modo jogo (Fase 7).
