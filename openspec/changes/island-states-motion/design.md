## Context

Estado atual (Fase 1, `openspec/changes/archive/2026-11-26-setup-foundation/design.md`):
- Há três janelas: host, visual (click-through, composição) e input (alpha 1, recortada por `SetWindowRgn`).
- `Renderer` tem um `ShapeVisual` com uma `CompositionRoundedRectangleGeometry` de preenchimento preto. Não há laço de render.
- `layout::place` calcula a cápsula Idle e a janela visual, que é a cápsula + 16 DIP de folga.

Motivação e escopo: ver `proposal.md`. Requisitos: `specs/island-states` e `specs/island-window`.

## Goals / Non-Goals

**Goals:**
- Morph fluido entre estados sem nenhum código do app por quadro.
- Uma máquina de estados pura e testável, separada de Win32 e da composição.
- Texto via DirectWrite, com o mínimo de memória possível.

**Non-Goals:**
- Fontes de dados reais (mídia, notificações), ícones vetoriais/imagens, controles clicáveis dentro do Expanded (próximas fases).
- Gestos, arrastar, múltiplas atividades simultâneas.
- Tela de configurações para os tokens.

## Decisions

### D1 — Molas no compositor (NaturalMotion), não no processo
`geometry.Size`, `geometry.Offset` e `geometry.CornerRadius` são animados com `SpringVector2NaturalMotionAnimation`. Para isso, `Compositor.CreateSpringVector2Animation` recebe `DampingRatio` e `Period` dos tokens e `FinalValue` com o destino. O compositor roda a física na própria thread e, quando há interrupção, parte do valor atual. Assim, o app não tem timer de quadro nem precisa saber quando a mola assenta.
- **Alternativa:** mola no processo, com `DCompositionWaitForCompositorClock` ou um timer por quadro escrevendo as propriedades. Isso dá controle quadro a quadro (região de entrada exata), mas cria um laço que precisa parar sozinho e custa CPU durante a animação. Descartada.
- Raio e tamanho usam os mesmos parâmetros de mola, então chegam juntos, sem formas intermediárias estranhas.

### D2 — Região de entrada vai direto para a forma de destino
Os quadros intermediários da mola não são observáveis sem custo. Por isso, no início da transição, a janela de entrada é reposicionada e recortada (`SetWindowRgn`) já com a forma final. Durante ~400 ms, a área clicável difere da forma desenhada.
- `ponytail:` região = destino. Se isso incomodar (cliques "fantasmas" ao expandir), o upgrade é amostrar a mola no processo durante a transição.
- Isto substitui a nota da Fase 1 ("atualizar a região a cada quadro"): o mesmo objetivo, sem laço.

### D3 — Janela visual dimensionada para o maior estado, fixa
`layout::place` passa a dimensionar a janela visual pelo Expanded + hover + folga da sombra. Ela nunca muda de tamanho durante as animações; só a geometria se move dentro dela, com `Offset.X = (largura_janela − largura_forma)/2` animado junto. Como a janela é `TRANSPARENT` + `NOREDIRECTIONBITMAP`, o tamanho maior não captura cliques nem aloca bitmap de janela.

### D4 — Árvore de composição
```
ContainerVisual (raiz)
└─ LayerVisual  (Shadow = DropShadow suave, segue o alfa do conteúdo)
   ├─ ShapeVisual → SpriteShape(geometria animada): Fill preto, Stroke 1 px cinza-escuro
   └─ SpriteVisual de conteúdo (CompositionSurfaceBrush ← CompositionDrawingSurface), Opacity animada
```
- A sombra de `LayerVisual` acompanha a forma animada sem redesenho.
- O contorno é `StrokeBrush` + `StrokeThickness` na mesma shape, então anima junto.
- O conteúdo some antes do morph (opacidade → 0, ~80 ms) e aparece depois (`DelayTime` ≈ tempo da mola). É simples e atende ao cenário "Conteúdo durante a transição".

### D5 — Texto: D3D11 + D2D + DirectWrite, criados sob demanda
- `D3D11CreateDevice` (BGRA support, hardware; WARP como fallback) alimenta `D2D1CreateDevice`, que alimenta `ICompositorInterop::CreateGraphicsDevice`. A partir daí, `CreateDrawingSurface2` e o desenho via `ICompositionDrawingSurfaceInterop::BeginDraw`.
- O dispositivo só é criado quando aparece o primeiro estado com texto (Compact/Expanded). O Idle puro continua sem D3D, o que protege a meta de RAM.
- `ponytail:` o dispositivo não é liberado ao voltar para o Idle. Se a medição mostrar mais de 50 MB, liberar após N s em Idle.
- Perda de dispositivo: o evento `CompositionGraphicsDevice.RenderingDeviceReplaced` e falhas `DXGI_ERROR_DEVICE_REMOVED` no `BeginDraw` recriam D3D/D2D, chamam `SetRenderingDevice` e redesenham.
- DPI: a superfície é desenhada em px físicos (escala do monitor). Em `relayout`, ela é redesenhada.

### D6 — Máquina de estados pura (`src/island.rs`)
`Island { activity: Option<Activity>, expanded: bool, hover: bool }` com `fn on(&mut self, Event) -> Effects`. Os eventos são `MouseEnter`, `MouseLeave`, `Click`, `LeaveTimeout`, `ActivityStart`, `ActivityEnd` e `SessionLock`. `fn target(&self) -> Shape` devolve o estado visual (Idle/Compact/Expanded) e se há hover. Sem Win32, só testes unitários. A janela traduz mensagens em eventos e aplica `target()` no `Renderer` e na região.

### D7 — Mouse sem hook global
- Hover: `WM_MOUSEMOVE` na janela de entrada, com `TrackMouseEvent(TME_LEAVE)` → `WM_MOUSELEAVE`.
- Recolher: `SetTimer` de disparo único (1 s) iniciado no `MouseLeave` e cancelado no `MouseEnter`. Quando dispara, chama `KillTimer`, então não sobra timer periódico.
- "Clicar fora" é coberto pelo mesmo timer: para clicar fora, o cursor já saiu. Um hook global de mouse continua descartado (custo constante, risco com anti-cheat — Fase 1, D2).

### D8 — Redução de animações
`SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION)` é lido na inicialização e relido em `WM_SETTINGCHANGE`. Quando está desligado, as propriedades recebem `Set*` direto, sem `StartAnimation`, e o conteúdo aparece sem atraso.

### D9 — Tokens (`src/theme.rs`)
Só `const`, um arquivo, em DIP:
- tamanhos Idle 126×37, Compact 250×37, Expanded 360×160 (raio 32);
- hover ×1,06;
- cores, contorno de 1 px `#2A2A2A`, sombra (blur 12, opacidade 0,35, offset y 2);
- mola (`DampingRatio` 0,8, `Period` 60 ms);
- fonte Segoe UI Variable, 13/15/12.

Os valores são ponto de partida, ajustados na comparação visual com as refs. Não há arquivo de configuração: um valor que não muda não precisa de configuração.

### D10 — Bloqueio/suspensão
Em `WTS_SESSION_LOCK` e `PBT_APMSUSPEND`, envia `SessionLock` à máquina (que limpa expanded/hover), mata o timer e aplica a forma sem animação. O compositor já não renderiza com a sessão bloqueada, então não há o que pausar.

### D11 — Menu Demo
Fica sob `#[cfg(debug_assertions)]` em `show_tray_menu`, com os itens "Demo: iniciar atividade" e "Demo: encerrar". O texto placeholder fica fixo no código e não vai para o i18n, porque não chega ao usuário final.

## Risks / Trade-offs

- [D3D/D2D sobe a memória (~10–20 MB)] → criado só sob demanda; medir no benchmark; liberar em Idle se passar de 50 MB.
- [Região de entrada adiantada em relação à forma (D2)] → é perceptível só em ~400 ms de expansão; marcado com `ponytail:`.
- [`LayerVisual` + `DropShadow` exigem Windows 10 1803+] → o alvo é o Windows 11. Se `Shadow` falhar, segue sem sombra (log de aviso), porque o contorno garante o contraste.
- [GPU híbrida / troca de GPU] → tratado por `RenderingDeviceReplaced` (D5); teste manual derrubando o driver (`Win+Ctrl+Shift+B`).
- [A mola NaturalMotion não reporta quando terminou] → não precisamos disso (D1/D2). Se uma fase futura precisar, há `CompositionScopedBatch.Completed`.
- [Sombra fora da janela visual cortada] → a folga da janela visual (D3) inclui blur + offset da sombra.
