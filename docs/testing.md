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
