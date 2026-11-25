# Idle resource benchmark. Starts the app, samples every 5 s for -Minutes, prints a markdown summary.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\bench.ps1 [-Minutes 10] [-Exe target\release\dynamic-island.exe]
param(
    [double]$Minutes = 10,
    [string]$Exe = "$PSScriptRoot\..\target\release\dynamic-island.exe"
)
$ErrorActionPreference = 'Stop'
Get-Process dynamic-island -ErrorAction SilentlyContinue | Stop-Process -Force
$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds 5   # let startup settle; not counted as idle
$p.Refresh()
$cores = [Environment]::ProcessorCount
$cpu0 = $p.TotalProcessorTime; $t0 = Get-Date
$priv = @(); $ws = @()
while (((Get-Date) - $t0).TotalMinutes -lt $Minutes) {
    Start-Sleep -Seconds 5
    $p.Refresh()
    $priv += $p.PrivateMemorySize64 / 1MB
    $ws += $p.WorkingSet64 / 1MB
}
$p.Refresh()
$wall = ((Get-Date) - $t0).TotalSeconds
$cpuPct = ($p.TotalProcessorTime - $cpu0).TotalSeconds / ($wall * $cores) * 100
$stats = { param($a) "{0:N1} avg / {1:N1} peak" -f ($a | Measure-Object -Average).Average, ($a | Measure-Object -Maximum).Maximum }
Stop-Process -Id $p.Id -Force

$cpuName = (Get-CimInstance Win32_Processor).Name.Trim()
$os = (Get-CimInstance Win32_OperatingSystem)
$size = "{0:N0} KB" -f ((Get-Item $Exe).Length / 1KB)
@"
| Data | Duração | Private (MB) | Working set (MB) | CPU média | Binário |
|---|---|---|---|---|---|
| $(Get-Date -Format 'yyyy-MM-dd HH:mm') | $Minutes min | $(& $stats $priv) | $(& $stats $ws) | $("{0:N3}" -f $cpuPct)% de $cores threads | $size |

Ambiente: $cpuName, $([math]::Round($os.TotalVisibleMemorySize / 1MB, 1)) GB, $($os.Caption) build $($os.BuildNumber).
"@
