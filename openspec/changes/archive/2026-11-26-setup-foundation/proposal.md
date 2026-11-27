## Why

O projeto ainda não tem código (auditoria da Fase 0, 2026-11-25). Antes de design, animação ou integrações, precisamos de uma base nativa que prove os requisitos mais arriscados da janela — transparência por pixel, nunca roubar o foco e **click-through fora da forma** — dentro das metas de consumo (RAM em repouso < 50 MB, CPU < 0,5%).

## What Changes

- Projeto Rust (`windows-rs`) com um único executável, sem console em release.
- Janela da ilha: sem borda, sempre visível, fora da barra de tarefas, sem ativação/foco, transparente por pixel via composição do Windows.
- Desenho de uma cápsula estática (estado Idle) centralizada no topo da área útil do monitor escolhido, com margem superior configurável.
- Cliques fora da cápsula atravessam para as janelas de baixo; cliques na cápsula chegam ao app.
- Suporte a DPI por monitor, múltiplos monitores, mudança de resolução/área útil, suspensão/retomada e bloqueio/desbloqueio.
- Ciclo de vida: instância única, ícone na bandeja (mostrar/ocultar, sair), configurações persistidas em JSON local.
- Log técnico local com tamanho limitado, sem conteúdo sensível.
- Roteiro de benchmark reproduzível (RAM, CPU, tempo de inicialização) e primeira medição documentada.
- `CLAUDE.md` do projeto e `.gitignore`.

## Capabilities

### New Capabilities
- `island-window`: comportamento da janela da ilha — aparência mínima (cápsula Idle), posicionamento, DPI, monitores, foco, click-through, suspensão/bloqueio.
- `app-lifecycle`: instância única, bandeja do sistema, encerramento, persistência de configurações, log e metas de consumo em repouso.

### Modified Capabilities
<!-- nenhuma: projeto sem specs anteriores -->

## Impact

- Código novo: `Cargo.toml`, `src/`.
- Dependências: `windows` (bindings oficiais Microsoft), `serde`, `serde_json`.
- Toolchain: Rust stable (`x86_64-pc-windows-gnu` ou `-msvc`) — sem exigir Visual Studio.
- Nenhuma permissão administrativa; nada é instalado no sistema além de `%APPDATA%\DynamicIsland\` e `%LOCALAPPDATA%\DynamicIsland\`.
- Fora do escopo: animações/molas, design system completo, mídia, notificações, IA, inicialização automática, empacotamento MSIX (fases seguintes).
