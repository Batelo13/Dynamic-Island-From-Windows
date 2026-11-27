## 1. Tokens e estado (puro, testável)

- [x] 1.1 `src/theme.rs` com os tokens da D9 (tamanhos/raios por estado, hover, cores, contorno, sombra, mola, tipografia)
- [x] 1.2 `src/island.rs`: máquina de estados (D6) + testes unitários cobrindo hover, clique em Idle/Compact, sair + timeout, voltar antes do timeout, início/fim de atividade, bloqueio
- [x] 1.3 `layout::place`: janela visual dimensionada para Expanded + hover + folga da sombra (D3); função pura que devolve o retângulo da forma (tela) para um `Shape`; testes atualizados

## 2. Composição: forma, contorno, sombra, molas

- [x] 2.1 Árvore da D4 (`SpriteVisual` com `DropShadow` mascarada por `CompositionVisualSurface`, contorno 1 px), com cores dos tokens; fallback sem sombra com log
- [x] 2.2 `Renderer::apply(frame, visual, dpi, animate)`: molas escalares por componente em Size/Offset/CornerRadius (D1); `Set*` direto quando `animate == false`
- [x] 2.3 Região/posição da janela de entrada direto para a forma de destino (D2), com `ponytail:` comment
- [x] 2.4 Verificar: `check-window.ps1` continua 12/12 em Idle; adicionar cheque de click-through 2 px abaixo do Expanded e sobre a sombra

## 3. Interação

- [x] 3.1 `WM_MOUSEMOVE` + `TrackMouseEvent(TME_LEAVE)` → `MouseEnter/MouseLeave`; `WM_LBUTTONUP` → `Click`
- [x] 3.2 Timer de disparo único de 1 s para recolher (D7); `KillTimer` ao disparar/cancelar
- [x] 3.3 `SPI_GETCLIENTAREAANIMATION` na inicialização e em `WM_SETTINGCHANGE` (D8)
- [x] 3.4 Bloqueio/suspensão → `SessionLock` aplicado sem animação (D10)
- [x] 3.5 Menu Demo só em debug (D11); confirmar ausência no build de release

## 4. Texto

- [x] 4.1 Features do `windows` para D3D11/D2D/DirectWrite/DXGI/interop; dispositivo criado sob demanda (D5)
- [x] 4.2 `CompositionDrawingSurface` + desenho do placeholder Compact (slots esquerdo/direito) e Expanded (título/subtítulo) em px físicos
- [x] 4.3 Opacidade do conteúdo: some antes do morph, aparece depois (D4)
- [x] 4.4 `RenderingDeviceReplaced` / `DEVICE_REMOVED` → recriar e redesenhar; redesenhar em `relayout` (DPI)

## 5. Validação

- [x] 5.1 `cargo test`, `cargo clippy --all-targets` limpos
- [ ] 5.2 Benchmark em repouso 10 min, depois de passar por Compact/Expanded (dispositivo D3D criado), registrado em `docs/benchmarks.md` (conjunto de trabalho privado < 50 MB; commit como informação, ver Riscos do design)
- [ ] 5.3 Benchmark de transições contínuas por 30 s (CPU média + retorno ao repouso em ≤ 2 s) em `docs/benchmarks.md`
- [ ] 5.4 Manual: interrupção da mola sem salto, assentamento ≤ 600 ms, animações desligadas → instantâneo, Win+L com Expanded, troca de escala com texto, `Win+Ctrl+Shift+B` (perda de dispositivo), foco inalterado ao clicar — em `docs/testing.md`
- [ ] 5.5 Screenshots dos 3 estados sobre fundo claro e escuro comparados com as refs 1, 2, 4 e 5; ajustar tokens
- [ ] 5.6 Atualizar grafo com Graphify (`--update`)
- [ ] 5.7 `openspec validate island-states-motion --strict`

## 6. Auto-ocultar e revelar pelo topo (revisão de 2026-11-26)

- [x] 6.1 `Island`: estado Oculta, `revealed`, `RevealTimeout`, `Timer::StartReveal`; sair por 1 s → Compact (com atividade) ou Oculta (sem); clique em Idle → Expanded; testes unitários
- [x] 6.2 `window.rs`: janela de entrada vira a faixa de 2 px × largura Idle quando Oculta (D12); timer de 150 ms para revelar, cancelado no `WM_MOUSELEAVE`
- [x] 6.3 `layout`/`render`: forma Oculta acima da borda superior (Offset.Y negativo), mesma mola; instantâneo com animações desligadas
- [x] 6.4 Expanded sem atividade mostra hora e data (placeholder, desenhado ao expandir; sem relógio ao vivo)
- [x] 6.5 `check-window.ps1`: nada visível ao iniciar; faixa recebe e 1 px ao lado/abaixo passa; onde a cápsula apareceria passa; revela após 150 ms; oculta 1 s após sair; checagens de Idle passam a rodar com a ilha revelada
- [ ] 6.6 Refazer 5.2/5.3 com o comportamento novo (o benchmark de 2026-11-26 mediu Idle sempre visível)
