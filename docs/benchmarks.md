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
