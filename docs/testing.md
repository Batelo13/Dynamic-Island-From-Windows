# Registro de testes

Legenda: ✅ aprovado · ❌ reprovado · ⏳ não executado (motivo indicado)

## setup-foundation — 2026-11-25

Ambiente: i5-13420H, 24 GB, RTX 4050 Laptop + Intel UHD, Windows 11 Home 26200, monitor único a 125% (1920×1080 lógico), build release.

### Automatizados

| Teste | Como | Resultado |
|---|---|---|
| Cálculo de posição 100%/150%, área útil com barra no topo, segundo monitor, fallback de monitor | `cargo test` (layout) | ✅ |
| Config ausente, roundtrip, JSON parcial, JSON corrompido → `.bak`, clamp da margem | `cargo test` (settings) | ✅ |
| Rotação do log em 1 MB (≤ 2 MB em disco) | `cargo test` (log) | ✅ |
| Janela criada / tempo até existir | `check-window.ps1` | ✅ 157 ms (log do app: `startup_ms=36`) |
| Tamanho a 125%: 158×46 px (= 126×37 DIP), topo a 10 px (= 8 DIP) | `check-window.ps1` | ✅ |
| Centralizada na área útil (±1 px) | `check-window.ps1` | ✅ |
| Fora da barra de tarefas/Alt+Tab (`TOOLWINDOW`) e sem ativação (`NOACTIVATE`) | `check-window.ps1` | ✅ |
| Hit-test: dentro → ilha; 2 px à esquerda, 2 px acima e canto arredondado → janela de baixo | `check-window.ps1` | ✅ |
| Clique real 2 px fora chega à janela de baixo; clique dentro não chega | `check-window.ps1` | ✅ |
| Foco do teclado inalterado após clicar na cápsula | `check-window.ps1` | ✅ |
| Instância única: 2ª execução sai (código 0), segue 1 processo | manual por script | ✅ |
| Sem elevação: executado em sessão não elevada, sem prompt de UAC | manual por script | ✅ |

### Manuais — 2026-11-26

| Cenário | Como | Status |
|---|---|---|
| Menu da bandeja: "Ocultar ilha" ↔ "Mostrar ilha" (alterna) e "Sair", textos em pt-BR | usuário | ✅ |
| Explorer reiniciado → ícone da bandeja volta (`TaskbarCreated`) | script reinicia o Explorer + usuário confirma o ícone | ✅ |
| Mudança de escala 125% → 150% → 125% com o app aberto | usuário | ✅ |
| Bloqueio/desbloqueio (Win+L) | usuário | ✅ |
| Suspensão/retomada | usuário | ✅ |
| Janela maximizada por baixo continua coberta pela ilha (hit-test no centro → ilha; abaixo → janela) | script | ✅ |
| Segundo monitor / monitor configurado desconectado | — | ⏳ só há um monitor disponível (coberto por teste unitário) |
| Barra de tarefas no topo (comportamento real) | — | ⏳ Windows 11 não permite barra no topo sem ferramentas externas (coberto por teste unitário) |

### Comparação visual (tarefa 5.4)

Screenshot da cápsula Idle (125%): 158×46 px, proporção 3,43:1, raio = altura/2 (pontas semicirculares), preto opaco, borda antisserrilhada, transparência real ao redor.
Refs 1 e 2 (estado Default/Idle): proporção ≈ 3,4:1, pílula preta com pontas semicirculares → **compatível**.
Pendências visuais (Fase 2): sombra discreta, indicador de câmera decorativo, contraste da cápsula preta sobre barras de título escuras.

## island-states-motion — 2026-11-26

Ambiente: o mesmo da Fase 1 (monitor único a 125%). Builds: release, debug e `profiling` (release com o menu Demo).

### Automatizados

| Teste | Como | Resultado |
|---|---|---|
| Máquina de estados: oculta ao iniciar, revelar só após permanência, passagem rápida não revela, ocultar 1 s após sair, voltar antes do tempo mantém, hover ≤ 8%, clique em Idle → relógio, atividade → Compact sem gesto, fim de atividade oculta (ou Idle sob o cursor), bloqueio | `cargo test` (island, 9 testes) | ✅ |
| Faixa de revelação: posição, largura do Idle, 2 DIP colada ao topo | `cargo test` (layout) | ✅ |
| Oculta ao iniciar: entrada = faixa 158×3 px no topo, centralizada | `check-window.ps1` | ✅ |
| Faixa recebe; 1 px ao lado e 1 px abaixo passam; onde a cápsula apareceria passa | `check-window.ps1` | ✅ |
| Revela após permanecer na faixa | `check-window.ps1` | ✅ 157–173 ms |
| Cápsula revelada: 158×46 px, centralizada, hit-test interno/2 px fora/canto/sombra | `check-window.ps1` | ✅ |
| Clique real fora passa; clique dentro não passa e não muda o foco | `check-window.ps1` | ✅ |
| Clique em Idle expande (relógio) 450×200 px; 2 px abaixo e sobre a sombra passam | `check-window.ps1` | ✅ |
| Some 1 s depois que o cursor sai | `check-window.ps1` | ✅ |
| Demo: atividade mostra Compact (313×46) sem gesto; encerrar oculta | `check-window.ps1 -Demo` (profiling) | ✅ |
| Menu Demo ausente no release (string UTF-16 não está no binário) | busca no `.exe` | ✅ |
| Resultado total | release / profiling + Demo | ✅ 25/25 · 27/27 |

### Comparação visual (tarefa 5.5)

Capturas a 125% sobre fundo branco, preto e barra de título escura:
- **Idle** 126×37 DIP: igual à Fase 1. A sombra suave aparece no branco; no preto, o contorno de 1 px `#2A2A2A` destaca a forma.
- **Compact** 250×37 DIP (6,8:1): bate com as pílulas "Silent"/"Ring" da ref 2 (~7:1). Ponto verde à esquerda e "0:08" à direita, como as refs 2 e 4.
- **Expanded** 360×160 DIP, raio 32 (20% da altura): bate com o card de mídia da ref 4 (~19%). Título e subtítulo hierarquizados como nas refs 1 e 5; o relógio sem atividade usa Segoe UI Variable 40 DIP.
- Veredito: compatível; tokens mantidos.

### Manuais (tarefa 5.4) — 2026-11-26

Feitos pelo usuário no build `profiling` (com menu Demo).

| Cenário | Status |
|---|---|
| Interrupção da mola sem salto (clicar e sair no meio da expansão) | ✅ |
| Assentamento visual ≤ 600 ms | ✅ |
| "Efeitos de animação" desligado → aparece/some instantâneo | ✅ |
| Win+L com a ilha expandida → desbloqueia oculta (ou Compact com atividade) | ✅ |
| Troca de escala com texto visível (125% ↔ 150%) | ✅ |
| `Win+Ctrl+Shift+B` (perda de dispositivo) com texto visível → texto volta sozinho | ✅ |
