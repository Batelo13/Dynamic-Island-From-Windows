## 1. Estado (puro, testável)

- [x] 1.1 `Island`: `dormant`, eventos `Paused`/`Resumed`/`PauseTimeout`, `Timer::StartPause` (30 s); revelar com `dormant` → Compact; testes dos cenários de "Pausa e visibilidade" (D3)
- [x] 1.2 `layout::media_buttons(frame, scale)` com os 3 retângulos de clique + teste (D5)
- [x] 1.3 Tokens novos em `theme.rs`: capa (56 DIP, raio 10; mini do Compact), barrinhas, progresso, botões, `PAUSE_HIDE_MS = 30000`

## 2. Fonte de mídia

- [x] 2.1 Features `Media_Control`, `Storage_Streams`, `Win32_Graphics_Imaging`; `src/media.rs` com thread MTA, gerenciador e assinatura dos eventos, reassinando ao trocar de sessão (D1)
- [x] 2.2 `Snapshot`: título, artista, tocando, comandos disponíveis, posição/duração/instante; entrega via `Arc<Mutex>` + `WM_MEDIA`
- [x] 2.3 Capa: stream → WIC → BGRA 160×160 pré-multiplicado, desenhada com filtro cúbico de alta qualidade, só quando a faixa muda (D2)
- [x] 2.4 Comandos anterior/tocar-pausar/próxima pelo canal, sem esperar resultado
- [x] 2.5 Falha de `RequestAsync` → log de aviso e app segue sem música

## 3. Integração na janela

- [x] 3.1 `WM_MEDIA` → eventos `ActivityStart/End`, `Paused/Resumed`; timer de 30 s (`TIMER_PAUSE`)
- [x] 3.2 Clique no Expanded de música → `media_buttons` → comando; botões desabilitados e cliques antes do conteúdo aparecer são ignorados
- [x] 3.3 Timer de 1 s para os tempos só com o Expanded visível; morto ao recolher

## 4. Renderização

- [x] 4.1 Compact: capa mini arredondada (ou nota em fundo cinza) + 4 barrinhas com animação infinita no compositor; pausado → paradas em 0,3 (D4)
- [x] 4.2 Expanded: capa, título/artista com reticências, tempos, botões com Segoe Fluent Icons (esmaecidos se indisponíveis), ícone tocar/pausar conforme estado (D5)
- [x] 4.3 Barra de progresso com `Scale.X` linear no compositor, reiniciada a cada snapshot; oculta sem duração
- [x] 4.4 Chave de conteúdo inclui faixa e estado; barrinhas e progresso não forçam redesenho (D6)
- [x] 4.5 Hover nos botões do Expanded: círculo claro com crescimento (compositor) e cursor de mão; só em botões habilitados
- [x] 4.6 Sessão sem título (player fechado para a bandeja) = sem música

## 5. Validação

- [x] 5.1 `cargo test` e `cargo clippy --all-targets` limpos
- [ ] 5.2 Manual com Spotify: aparece sozinho em ≤ 1 s, troca de faixa, pausar/tocar/próxima/anterior pela ilha sem mudar o foco, pausa > 30 s some, revelar com pausada mostra Compact, fechar o Spotify oculta; YouTube no navegador como segundo player — em `docs/testing.md`
- [ ] 5.3 `check-window.ps1` continua passando (sem música tocando)
- [x] 5.4 Benchmark 10 min com música tocando em Compact (RAM privada < 50 MB, CPU < 0,5%) em `docs/benchmarks.md`
- [ ] 5.5 Screenshots Compact/Expanded com música comparados às refs 1, 4 e 5; ajustar tokens (inclusive altura do Expanded)
- [x] 5.6 Graphify `--update`
- [ ] 5.7 `openspec validate media-now-playing --strict`
