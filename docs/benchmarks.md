# Benchmarks

Metodologia: `scripts/bench.ps1` inicia o build release, espera 5 s, amostra a cada 5 s `PrivateMemorySize64`, `WorkingSet64` e o tempo total de CPU do processo; CPU média = tempo de CPU / (duração × threads lógicas). Mede só o processo do app: a superfície da cápsula é composta pelo DWM (`dwm.exe`), cujo custo não aparece aqui (para uma forma vetorial é desprezível, mas não foi medido isoladamente).

Inicialização: o app registra `startup_ms` no log (início do `main` → janelas exibidas).

## Repouso (cápsula Idle, sem atividades)

| Data | Change | Duração | Private (MB) | Working set (MB) | CPU média | Inicialização | Binário |
|---|---|---|---|---|---|---|---|
| 2026-11-25 01:48 | setup-foundation | 10 min | 2,8 avg / 3,0 peak | 14,7 avg / 14,8 peak | 0,001% (12 threads) | 36 ms | 328 KB |

Ambiente: i5-13420H, 23,7 GB, RTX 4050 Laptop + Intel UHD, Windows 11 Home Single Language 26200, alimentação na tomada, monitor único 125%, VS Code e navegador abertos.
Metas: private < 50 MB ✅ · CPU < 0,5% ✅ · inicialização < 2 s ✅

Não medido ainda: GPU, bateria, CPU durante animações (não há animações até a Fase 2).

## island-states-motion — 2026-11-26

Métrica da meta: **RAM privada** (conjunto de trabalho privado, a coluna "Memória" do Gerenciador de Tarefas), < 50 MB. O commit privado fica registrado só como informação: depois que o texto é desenhado pela primeira vez, o `dxgi` reserva ~54–110 MB de commit que nunca é tocado e não volta (detalhes no design da mudança, em Riscos).

| Data | Estado | Duração | RAM privada (MB, meta < 50) | Commit privado (MB) | Working set (MB) | CPU média | Binário |
|---|---|---|---|---|---|---|---|
| 2026-11-26 23:48 | Oculta, sem atividade (release) | 10 min | 1,4 avg / 1,7 peak | 2,9 avg / 3,1 peak | 15,2 avg / 15,4 peak | 0,000% de 12 threads | 352 KB |
| 2026-11-26 21:26 | Compact (D3D/texto carregados), build profiling | 10 min | 38,8 avg / 39,4 peak | 60,8 avg / 61,0 peak | 72,2 avg / 72,8 peak | 0,000% de 12 threads | 372 KB |

Medições pontuais (release, após a correção do carregamento sob demanda do `d3d11`):
- Oculta/Idle sem texto: 3,0 MB de commit e 1,6 MB de RAM privada; `dxgi`/`d3d11` não carregam.
- Antes da correção: 56,5 MB de commit em Idle.

A rodada de repouso das 21:16 foi descartada porque a ilha foi revelada e clicada durante a medição (o que carregou o texto); a das 23:48 rodou sem interação.

### Transições (tarefa 5.3)

| Data | Cenário | CPU app (transições) | CPU dwm (transições) | CPU app 0–2 s depois | CPU app 2–12 s depois |
|---|---|---|---|---|---|
| 2026-11-26 21:05 | 84 transições de hover em 30 s (Compact, profiling) | 0,04% | n/d (exige elevação) | 0,00% | 0,00% |

As molas rodam no compositor (DWM), não no processo do app.

## media-now-playing — 2026-11-27

Release com WARP (texto e capa rasterizados na CPU, sem o driver da GPU no processo).

| Data | Estado | Duração | RAM privada (MB, meta < 50) | Commit privado (MB) | Working set (MB) | CPU média | Binário |
|---|---|---|---|---|---|---|---|
| 2026-11-27 07:10 | Compact com música (Spotify tocando; entrada no fim: 313x46) | 10 min | 9,2 avg / 10,6 peak | 68,8 avg / 71,7 peak | 41,5 avg / 43,1 peak | 0,006% de 12 threads | 418 KB |
| 2026-11-27 01:38 | Oculta (a música não estava na ilha; entrada no fim: 158x3) | 10 min | 2,5 avg / 2,5 peak | 4,2 avg / 4,3 peak | 25,0 avg / 25,1 peak | 0,002% de 12 threads | 418 KB |

- A rodada das 01:38 foi rotulada como "com música", mas a ilha estava oculta (faixa de revelação de 158x3). Fica registrada só como mais uma medição da ilha oculta.
- Antes de iniciar a rodada das 07:10, o app foi aberto para conferir: entrada 250x37 (Compact) e dispositivo de texto criado. O tamanho de 313x46 no fim é maior que o Compact em repouso e é compatível com o hover leve (cursor sobre a ilha).
- Medição pontual anterior, com o Expanded de música aberto: 10,2 MB de RAM privada e cerca de 0,9% de CPU (atualização dos tempos a cada 1 s).
- As barrinhas e a barra de progresso rodam no compositor: com música tocando, a CPU média do app ficou em 0,006%.
