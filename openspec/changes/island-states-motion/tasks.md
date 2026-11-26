## 1. Tokens e estado (puro, testável)

- [ ] 1.1 `src/theme.rs` com os tokens da D9 (tamanhos/raios por estado, hover, cores, contorno, sombra, mola, tipografia)
- [ ] 1.2 `src/island.rs`: máquina de estados (D6) + testes unitários cobrindo hover, clique em Idle/Compact, sair + timeout, voltar antes do timeout, início/fim de atividade, bloqueio
- [ ] 1.3 `layout::place`: janela visual dimensionada para Expanded + hover + folga da sombra (D3); função pura que devolve o retângulo da forma (tela) para um `Shape`; testes atualizados

## 2. Composição: forma, contorno, sombra, molas

- [ ] 2.1 Árvore da D4 (`LayerVisual` + `DropShadow`, contorno 1 px), com cores dos tokens; fallback sem sombra com log
- [ ] 2.2 `Renderer::set_shape(shape, animate)`: molas `SpringVector2` em Size/Offset/CornerRadius (D1); `Set*` direto quando `animate == false`
- [ ] 2.3 Região/posição da janela de entrada direto para a forma de destino (D2), com `ponytail:` comment
- [ ] 2.4 Verificar: `check-window.ps1` continua 12/12 em Idle; adicionar cheque de click-through 2 px abaixo do Expanded e sobre a sombra

## 3. Interação

- [ ] 3.1 `WM_MOUSEMOVE` + `TrackMouseEvent(TME_LEAVE)` → `MouseEnter/MouseLeave`; `WM_LBUTTONUP` → `Click`
- [ ] 3.2 Timer de disparo único de 1 s para recolher (D7); `KillTimer` ao disparar/cancelar
- [ ] 3.3 `SPI_GETCLIENTAREAANIMATION` na inicialização e em `WM_SETTINGCHANGE` (D8)
- [ ] 3.4 Bloqueio/suspensão → `SessionLock` aplicado sem animação (D10)
- [ ] 3.5 Menu Demo só em debug (D11); confirmar ausência no build de release

## 4. Texto

- [ ] 4.1 Features do `windows` para D3D11/D2D/DirectWrite/DXGI/interop; dispositivo criado sob demanda (D5)
- [ ] 4.2 `CompositionDrawingSurface` + desenho do placeholder Compact (slots esquerdo/direito) e Expanded (título/subtítulo) em px físicos
- [ ] 4.3 Opacidade do conteúdo: some antes do morph, aparece depois (D4)
- [ ] 4.4 `RenderingDeviceReplaced` / `DEVICE_REMOVED` → recriar e redesenhar; redesenhar em `relayout` (DPI)

## 5. Validação

- [ ] 5.1 `cargo test`, `cargo clippy --all-targets` limpos
- [ ] 5.2 Benchmark em repouso 10 min, depois de passar por Compact/Expanded (dispositivo D3D criado), registrado em `docs/benchmarks.md`; se > 50 MB, liberar dispositivo em Idle (D5)
- [ ] 5.3 Benchmark de transições contínuas por 30 s (CPU média + retorno ao repouso em ≤ 2 s) em `docs/benchmarks.md`
- [ ] 5.4 Manual: interrupção da mola sem salto, assentamento ≤ 600 ms, animações desligadas → instantâneo, Win+L com Expanded, troca de escala com texto, `Win+Ctrl+Shift+B` (perda de dispositivo), foco inalterado ao clicar — em `docs/testing.md`
- [ ] 5.5 Screenshots dos 3 estados sobre fundo claro e escuro comparados com as refs 1, 2, 4 e 5; ajustar tokens
- [ ] 5.6 Atualizar grafo com Graphify (`--update`)
- [ ] 5.7 `openspec validate island-states-motion --strict`
