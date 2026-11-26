## Why

A Fase 1 entregou só a cápsula Idle estática. Para virar uma Dynamic Island de verdade, a ilha precisa mudar de forma com fluidez (Idle → Compact → Expanded), responder ao mouse e mostrar texto — antes de ligarmos fontes de dados reais (mídia, notificações), que dependem desse motor visual.

## What Changes

- Design system em um único módulo de tokens: tamanhos e raios por estado, cores, contorno, sombra, tipografia e parâmetros de mola.
- Três estados visuais, conforme as refs 1, 2, 4 e 5:
  - **Idle**: 126 × 37 DIP.
  - **Compact**: pílula larga, com um slot à esquerda para ícone e um à direita para informação.
  - **Expanded**: retângulo arredondado grande, raio ~32 DIP.
- Morph por mola entre estados (tamanho, raio e posição). A transição pode ser interrompida a qualquer momento, sem saltos, e não há trabalho periódico depois que a forma assenta.
- Interação com o mouse:
  - hover dá um leve crescimento;
  - clique expande quando há atividade;
  - a ilha recolhe ~1 s depois que o cursor sai, o que cobre também o clique fora.
- A área clicável acompanha a forma atual.
- Texto renderizado com Direct2D/DirectWrite dentro da composição, com recuperação de perda de dispositivo gráfico.
- Sombra suave e contorno de 1 px cinza-escuro, para contraste sobre fundos escuros.
- Respeita "Efeitos de animação" desligado no Windows: as transições ficam instantâneas.
- Menu **Demo** na bandeja, só em builds de debug, para acionar os estados com conteúdo de exemplo. Fontes de dados reais ficam para as próximas fases.

## Capabilities

### New Capabilities
- `island-states`: estados visuais da ilha, transições por mola, interação com o mouse, texto, redução de animações, custo da animação e modo demo.

### Modified Capabilities
- `island-window`: a cápsula Idle passa a ter sombra suave e contorno sutil. O click-through passa a acompanhar a forma atual (não só a cápsula Idle).

## Impact

- Código:
  - `src/render.rs`: árvore de composição com sombra, contorno, superfície de texto e animações.
  - `src/window.rs`: mouse, timers e menu Demo.
  - `src/layout.rs`: janela visual dimensionada para o maior estado.
  - Novos `src/theme.rs` (tokens) e `src/island.rs` (máquina de estados pura, testável).
- Dependências: só novas features do crate `windows` (Direct3D11, Direct2D, DirectWrite, DXGI e interop de composição). Nenhum crate novo.
- Desempenho: o dispositivo D3D/D2D aumenta a memória. A meta de < 50 MB ociosos continua valendo e é medida de novo. A CPU é medida também durante as animações.
