## Context

Estado atual (Fase 2, `openspec/changes/archive/2026-11-27-island-states-motion/design.md`):
- `Island` tem uma atividade booleana (só o Demo a liga) e os estados Oculta/Idle/Compact/Expanded.
- `Renderer` desenha o conteúdo em duas superfícies D2D alternadas (A/B), que trocam com fade. O texto carrega D3D sob demanda.
- A janela traduz mensagens Win32 em eventos e usa timers de disparo único. Não há laço nem polling.

Motivação e escopo: `proposal.md`. Requisitos: `specs/media-playback` e `specs/island-states`.

## Goals / Non-Goals

**Goals:**
- Uma única fonte de verdade da mídia, alimentada por eventos do sistema, sem bloquear a thread da UI.
- Animações contínuas (barrinhas, progresso) no compositor, sem trabalho periódico no app.
- Capa com custo de memória pequeno e previsível.

**Non-Goals:**
- Seek, volume, escolha manual de player, letras, cores extraídas da capa (fases futuras, se pedidas).
- Notificações e cronômetro (próximas fases; a estrutura de atividade fica pronta para eles).

## Decisions

### D1 — Thread de mídia dedicada (`src/media.rs`)
- Uma thread MTA (`CoInitializeEx(COINIT_MULTITHREADED)`) é dona do `GlobalSystemMediaTransportControlsSessionManager` (`RequestAsync().get()`).
- Ela assina estes eventos:
  - no gerenciador: `CurrentSessionChanged` e `SessionsChanged`;
  - na sessão atual: `MediaPropertiesChanged`, `PlaybackInfoChanged` e `TimelinePropertiesChanged`, reassinados quando a sessão muda.
- Os handlers só enviam `Refresh` a um `mpsc::channel`. A thread fica bloqueada no `recv()`, sem polling, e a cada mensagem monta um `Snapshot`:
  - título, artista e estado (tocando/pausado);
  - comandos disponíveis (anterior/play-pause/próxima);
  - posição, duração e instante da leitura;
  - capa já decodificada.
- O `Snapshot` vai para a UI por um `Arc<Mutex<Option<Snapshot>>>` + `PostMessage(host, WM_MEDIA)`. A UI só lê o último snapshot; snapshots intermediários podem ser descartados.
- Os comandos da UI (anterior/tocar-pausar/próxima) entram no mesmo canal (`Command(..)`) e a thread de mídia chama `Try*Async` sem esperar o resultado. A atualização de estado chega pelo evento `PlaybackInfoChanged`.
- **Alternativas:** fazer tudo na thread da UI com callbacks `SetCompleted` (mais código assíncrono espalhado e risco de reentrância no wndproc); polling de 1 s (viola o requisito de custo).

### D2 — Capa decodificada uma vez, pequena
- `Thumbnail().OpenReadAsync()`: lê o stream, `CreateStreamOverRandomAccessStream` e WIC (`IWICImagingFactory` → decoder → `IWICBitmapScaler` com `HighQualityCubic`), gerando BGRA pré-multiplicado de 160×160 px.
- A decodificação roda na thread de mídia e só refaz quando a faixa muda (chave: título + artista + tamanho do stream).
- A UI cria um `ID2D1Bitmap1` a partir dos bytes ao desenhar e o reusa no Compact (reduzido) e no Expanded, com o pincel em `D2D1_INTERPOLATION_MODE_HIGH_QUALITY_CUBIC`. O filtro linear padrão serrilhava a capa reduzida ~3× no Compact (reportado pelo usuário). Custo: ~100 KB por capa.
- `ponytail:` 160 px fixos (cobrem 56 DIP até ~285% sem ampliar). O upgrade é decodificar no tamanho exato da escala.

### D3 — Atividade e pausa na máquina de estados
- `Island` passa a ter `activity: bool` + `dormant: bool`:
  - `dormant` = música pausada há mais de 30 s;
  - a ilha fica Oculta se `!(activity && !dormant) && !revealed`;
  - revelar com `dormant` mostra Compact, porque há atividade.
- Novos eventos `Paused`/`Resumed`/`PauseTimeout`, e `ActivityStart`/`End` passam a vir também da mídia.
- `Paused` devolve `Timer::StartPause` (one-shot de 30 s, `TIMER_PAUSE`); `Resumed` cancela o timer e limpa `dormant`.
- Testes unitários cobrem os cenários de "Pausa e visibilidade".
- O Demo continua usando os mesmos eventos.

### D4 — Barrinhas e progresso no compositor
- **Barrinhas:** 4 `SpriteVisual` pequenos (cor de destaque, cantos arredondados via `CompositionGeometricClip`), ancorados embaixo (`CenterPoint.Y` = altura).
  - Cada uma tem uma `ScalarKeyFrameAnimation` em `Scale.Y`, com `IterationBehavior::Forever` e durações diferentes (~0,4–0,7 s), para parecer orgânico.
  - Pausado: `StopAnimation` + `Scale.Y = 0,3`.
  - As animações infinitas rodam no DWM, e o custo do app é zero.
- **Progresso:** `SpriteVisual` de preenchimento com `Scale.X` animado por `ScalarKeyFrameAnimation` linear, da fração atual até 1, com `Duration` = tempo restante.
  - É reiniciado a cada `Snapshot`, porque posição/estado podem ter mudado.
  - Pausado: para na fração atual.
- **Tempos decorrido/restante:** uma superfície D2D pequena, redesenhada por um timer de 1 s **só enquanto o Expanded está visível**, e morto ao recolher. A posição é extrapolada como `posição + (agora − instante da leitura)` quando está tocando.
- **Alternativa descartada:** texto de tempo animado no compositor. Não existe primitiva de texto lá; precisaria de 60 glifos pré-renderizados, código demais para um detalhe.

### D5 — Layout e botões do Expanded
- **Capa:** 56 DIP à esquerda, com raio de 10.
- **Ao lado da capa:** título (15 semibold) e artista (12, cinza), cortados com reticências (`DWRITE_TRIMMING` + `IDWriteInlineObject` de elipse).
- **Abaixo:** a barra de progresso com os tempos nas pontas e a linha de botões centralizada (anterior, tocar/pausar, próxima).
- **Ícones:** glifos da fonte de sistema **Segoe Fluent Icons** (anterior `E892`, tocar `E768`, pausar `E769`, próxima `E893`, nota `E8D6`), em vez de SVG ou desenho à mão.
- Os retângulos de clique dos botões vêm de uma função pura `layout::media_buttons(frame, scale) -> [Rect; 3]`, testável.
- A janela converte o `WM_LBUTTONUP` (coordenadas cliente da janela de entrada) num `Command` quando o clique cai num botão habilitado; fora dos botões, o Expanded ignora o clique.
- O tamanho do Expanded (360×160 DIP) é revisto na comparação visual; se não couber, só o token `EXPANDED` muda.

### D6 — Chave de conteúdo do Renderer
- O conteúdo é redesenhado quando muda (estado, atividade, DPI, `faixa`, `tocando`).
- `faixa` = título + artista + ponteiro da capa. Assim as barrinhas e o progresso não forçam redesenho das superfícies: eles vivem em visuais próprios, posicionados por estado.

### D7 — Texto rasterizado por WARP, não pela GPU (decidido na implementação)
- O que aconteceu: um processo do app (build de debug) ficou preso a 98% de um núcleo e deixou de responder. O gdb mostrou a thread principal girando dentro do driver de vídeo da Intel (`dcomp.dll` → `d3d11.dll` → `igd10um64xe.dll`). O gatilho exato não ficou provado: a pilha das nossas chamadas não chegou a ser lida.
- Decisão: `create_device` tenta primeiro `D3D_DRIVER_TYPE_WARP` (rasterizador por CPU) e só depois o hardware. Só desenhamos superfícies pequenas (≤ 450×200 px), no máximo 1×/s; a composição e as animações continuam na GPU, dentro do DWM.
- Medido (build profiling, Expanded de música aberto): nenhum driver de GPU no processo, RAM privada 10,2 MB (era ~39 MB), commit 17,2 MB (era 60–114 MB), CPU ~0,9% só enquanto o Expanded está aberto (tempos a cada 1 s). Checagens de janela: 27/27.
- **Alternativa:** manter o hardware e investigar o gatilho. Isso exigiria reproduzir um travamento raro de driver de terceiros, sem garantia; o WARP remove o componente que falhou e ainda reduz a memória.

## Risks / Trade-offs

- [Players que não publicam sessão (alguns apps antigos)] → não aparecem; limitação do Windows, documentada.
- [Navegadores publicam uma sessão por aba/vídeo e o Windows escolhe a "atual"] → seguimos a escolha do sistema (decisão do usuário); pode não ser a aba que o usuário espera.
- [Posição reportada com atraso ou irregular por alguns players (o Spotify atualiza a cada poucos segundos)] → a barra é extrapolada pelo compositor e corrigida a cada `TimelinePropertiesChanged`, então pequenos saltos são aceitos.
- [`RequestAsync` falhar (políticas de grupo)] → log de aviso; o app segue sem música, como na Fase 2.
- [Clique em botão durante o morph de expansão] → a região de entrada já é a do Expanded (D2 da Fase 2), mas os botões só ficam habilitados depois que o conteúdo aparece (≥ `CONTENT_FADE_IN_DELAY_MS`); antes disso o clique é ignorado.
- [Segoe Fluent Icons ausente (Windows 10)] → o alvo é o Windows 11; o fallback do DirectWrite mostraria quadrados. Aceito.
