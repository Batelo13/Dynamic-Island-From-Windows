## 1. Projeto e ferramentas

- [x] 1.1 Criar `Cargo.toml` (deps: `windows`, `serde`, `serde_json`; perfil release enxuto) e `.gitignore` (`target/`, `graphify-out/cache`)
- [x] 1.2 Criar `CLAUDE.md` do projeto (comandos de build/teste, fluxo OpenSpec, ponteiros para docs) — curto
- [x] 1.3 `cargo build` e `cargo clippy` limpos com o toolchain GNU

## 2. Validação de click-through (risco principal — antes do resto)

- [x] 2.1 Janela visual (D2.1) com compositor + cápsula Idle 126×37 DIP centralizada no monitor principal
- [x] 2.2 Janela de entrada (D2.2) com `SetWindowRgn` arredondado; logar clique recebido
- [x] 2.3 Teste manual: clique 2 px fora da borda chega ao app de baixo; clique dentro não; foco do teclado não muda. Se falhar, aplicar fallback D2 e registrar em `design.md`

## 3. Janela e posicionamento

- [x] 3.1 `layout.rs`: cálculo puro de posição (área útil, DPI, margem, monitor ausente → principal) + testes unitários
- [x] 3.2 Per-Monitor-V2 DPI; recalcular em `WM_DISPLAYCHANGE`, `WM_DPICHANGED`, `WM_SETTINGCHANGE`
- [x] 3.3 Retomada de suspensão (`WM_POWERBROADCAST`) e desbloqueio (`WTSRegisterSessionNotification`)
- [x] 3.4 Garantir sem foco / fora da barra de tarefas e Alt+Tab (D3)

## 4. Ciclo de vida

- [x] 4.1 Mutex de instância única
- [x] 4.2 Bandeja: Mostrar/Ocultar, Sair; recriar em `TaskbarCreated`; textos pt-BR/en
- [x] 4.3 `settings.rs`: carregar/validar/clamp/`.bak`/gravação atômica + testes unitários
- [x] 4.4 `log.rs` com rotação em 1 MB + teste unitário; logar `startup_ms`

## 5. Validação

- [x] 5.1 `cargo test` passando
- [x] 5.2 `scripts/bench.ps1` e primeira medição em `docs/benchmarks.md` (10 min em repouso, hardware e build documentados)
- [ ] 5.3 Checklist manual dos cenários das specs (DPI 100/125/150%, segundo monitor se disponível, barra no topo, suspensão, bloqueio, usuário padrão) registrado em `docs/testing.md` como aprovado/reprovado/não executado
- [x] 5.4 Screenshot da cápsula Idle comparada com as refs 1 e 2 (proporção e raio)
- [x] 5.5 Gerar grafo inicial com Graphify
- [x] 5.6 `openspec validate setup-foundation --strict`
