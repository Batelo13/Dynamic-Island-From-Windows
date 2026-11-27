## Why

A ilha já tem estados, animações e auto-ocultar, mas nenhuma fonte de dados real: só o Demo de debug faz ela aparecer. Música tocando é o caso de uso mais comum de uma Dynamic Island e o primeiro que o usuário testou (Spotify), então é a primeira atividade real a ligar.

## What Changes

- Lê a sessão de mídia atual do Windows (a mesma que aparece no painel de volume do Windows 11). Funciona com Spotify, navegadores e qualquer player que publique sessão, por eventos e sem polling.
- Música tocando vira uma atividade:
  - **Compact**: capa do álbum mini à esquerda e barrinhas de equalizador animadas à direita (param quando pausa).
  - **Expanded**: capa, título, artista, barra de progresso com tempo decorrido/restante (só exibição) e botões anterior / tocar-pausar / próxima.
- Pausou: a ilha continua visível por 30 s e depois some. Revelar pelo topo enquanto a música está pausada mostra o Compact dela em vez do Idle.
- Com vários players, vale a sessão que o Windows considera atual.
- Os botões funcionam sem tirar o foco da janela em uso.
- O Demo continua existindo só em builds de debug/profiling.

## Capabilities

### New Capabilities
- `media-playback`: detectar a sessão de mídia atual e mostrá-la na ilha (Compact e Expanded), controles de reprodução, regras de pausa/30 s e custo.

### Modified Capabilities
- `island-states`:
  - *Estados visuais*: a atividade agora pode ser real (música), não só o demo.
  - *Revelar pelo topo*: com música pausada há mais de 30 s, revelar mostra o Compact da música em vez do Idle.

## Impact

- Código:
  - novo `src/media.rs` (sessão, eventos, capa);
  - `src/island.rs` (atividade real, visibilidade da pausa);
  - `src/render.rs` (capa, barrinhas, progresso e botões);
  - `src/window.rs` (cliques nos botões, timer de 30 s, mensagens dos eventos de mídia).
- Dependências: só novas features do crate `windows` (`Media_Control`, `Storage_Streams`, `Win32_Graphics_Imaging`). Nenhum crate novo.
- Desempenho: a capa decodificada ocupa memória (alvo: < 50 MB de RAM privada com música). As animações das barrinhas e do progresso rodam no compositor.
