# Resource benchmark. Starts the app, samples every 5 s for -Minutes, prints a markdown summary.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\bench.ps1 [-Minutes 10] [-Exe target\release\dynamic-island.exe]
#   -Demo         drive the tray Demo (Compact -> Expanded -> Idle) before measuring idle, so the text/D3D stack is loaded.
#                 Needs a build with the Demo menu: cargo build --profile profiling (default exe when -Demo is given).
#   -Transitions  instead of idling: hover in/out of the island continuously for 30 s (moves the mouse!), report CPU
#                 during the transitions and in the 2 s after they stop. Compositor work runs in dwm.exe; reported too.
param(
    [double]$Minutes = 10,
    [string]$Exe = "",
    [switch]$Demo,
    [switch]$Transitions,
    # Label for the result row (e.g. "Compact com música"); default derives from -Demo.
    [string]$State = ""
)
$ErrorActionPreference = 'Stop'
if (-not $Exe) { $Exe = if ($Demo -or $Transitions) { "$PSScriptRoot\..\target\profiling\dynamic-island.exe" } else { "$PSScriptRoot\..\target\release\dynamic-island.exe" } }
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class B {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int x, int y, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
}
"@
[void][B]::SetProcessDpiAwarenessContext([IntPtr]-4)
function Tray($downs) {
    # WM_APP+1 / WM_RBUTTONUP opens the tray menu; Demo items are 2nd (start) and 3rd (end).
    [void][B]::PostMessage([B]::FindWindow('DynamicIslandWindow', 'Dynamic Island Host'), 0x8001, [IntPtr]1, [IntPtr]0x0205)
    Start-Sleep -Milliseconds 400; [System.Windows.Forms.SendKeys]::SendWait(("{DOWN}" * $downs) + "{ENTER}"); Start-Sleep -Milliseconds 500
}
function Center { $r = New-Object B+R; [void][B]::GetWindowRect([B]::FindWindow('DynamicIslandWindow', 'Dynamic Island Input'), [ref]$r); @([int](($r.L + $r.Rt) / 2), [int](($r.T + $r.B) / 2)) }

Get-Process dynamic-island -ErrorAction SilentlyContinue | Stop-Process -Force
$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds 3
$cores = [Environment]::ProcessorCount
$dwm = Get-Process dwm | Select-Object -First 1
$pct = { param($cpu, $secs) "{0:N2}%" -f ($cpu.TotalSeconds / ($secs * $cores) * 100) }

if ($Demo -or $Transitions) {
    Tray 2                                   # Compact
    $c = Center; [void][B]::SetCursorPos($c[0], $c[1]); Start-Sleep -Milliseconds 300
    [B]::mouse_event(2, 0, 0, 0, [IntPtr]::Zero); [B]::mouse_event(4, 0, 0, 0, [IntPtr]::Zero); Start-Sleep -Milliseconds 800   # Expanded
    [void][B]::SetCursorPos(10, 600); Start-Sleep -Milliseconds 1800    # collapse back to Compact
}

if ($Transitions) {
    $c = Center
    $p.Refresh(); $dwm.Refresh(); $a0 = $p.TotalProcessorTime; $d0 = $dwm.TotalProcessorTime; $t0 = Get-Date; $n = 0
    while (((Get-Date) - $t0).TotalSeconds -lt 30) {
        [void][B]::SetCursorPos($c[0], $c[1]); Start-Sleep -Milliseconds 350
        [void][B]::SetCursorPos(10, 600); Start-Sleep -Milliseconds 350; $n += 2
    }
    $p.Refresh(); $dwm.Refresh(); $wall = ((Get-Date) - $t0).TotalSeconds
    $during = & $pct ($p.TotalProcessorTime - $a0) $wall; $dwmDuring = if ($null -eq $d0) { 'n/d (exige elevação)' } else { & $pct ($dwm.TotalProcessorTime - $d0) $wall }
    $a1 = $p.TotalProcessorTime; Start-Sleep -Seconds 2; $p.Refresh(); $after = & $pct ($p.TotalProcessorTime - $a1) 2
    $a2 = $p.TotalProcessorTime; Start-Sleep -Seconds 10; $p.Refresh(); $rest = & $pct ($p.TotalProcessorTime - $a2) 10
    Stop-Process -Id $p.Id -Force
    @"
| Data | Cenário | CPU app (transições) | CPU dwm (transições) | CPU app 0–2 s depois | CPU app 2–12 s depois |
|---|---|---|---|---|---|
| $(Get-Date -Format 'yyyy-MM-dd HH:mm') | $n transições de hover em 30 s (Compact) | $during | $dwmDuring | $after | $rest |
"@
    return
}

Start-Sleep -Seconds 2   # let everything settle; not counted as idle
$p.Refresh()
$cpu0 = $p.TotalProcessorTime; $t0 = Get-Date
$priv = @(); $ws = @(); $pws = @()
while (((Get-Date) - $t0).TotalMinutes -lt $Minutes) {
    Start-Sleep -Seconds 5
    $p.Refresh()
    $priv += $p.PrivateMemorySize64 / 1MB
    $ws += $p.WorkingSet64 / 1MB
    # Private working set = Task Manager 'Memory' column; the spec target (< 50 MB) applies to this one.
    $pws += (Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$($p.Id)").WorkingSetPrivate / 1MB
}
$p.Refresh()
# What the island looked like at the end (proves which state was measured).
$ir = New-Object B+R; [void][B]::GetWindowRect([B]::FindWindow('DynamicIslandWindow', 'Dynamic Island Input'), [ref]$ir)
$inputSize = "{0}x{1}" -f ($ir.Rt - $ir.L), ($ir.B - $ir.T)
$wall = ((Get-Date) - $t0).TotalSeconds
$cpuPct = ($p.TotalProcessorTime - $cpu0).TotalSeconds / ($wall * $cores) * 100
$stats = { param($a) "{0:N1} avg / {1:N1} peak" -f ($a | Measure-Object -Average).Average, ($a | Measure-Object -Maximum).Maximum }
Stop-Process -Id $p.Id -Force

$cpuName = (Get-CimInstance Win32_Processor).Name.Trim()
$os = (Get-CimInstance Win32_OperatingSystem)
$size = "{0:N0} KB" -f ((Get-Item $Exe).Length / 1KB)
$state = if ($State) { $State } elseif ($Demo) { 'Compact (D3D/texto carregados)' } else { 'Idle' }
$state += " (entrada no fim: $inputSize)"
@"
| Data | Estado | Duração | RAM privada (MB, meta < 50) | Commit privado (MB) | Working set (MB) | CPU média | Binário |
|---|---|---|---|---|---|---|---|
| $(Get-Date -Format 'yyyy-MM-dd HH:mm') | $state | $Minutes min | $(& $stats $pws) | $(& $stats $priv) | $(& $stats $ws) | $("{0:N3}" -f $cpuPct)% de $cores threads | $size |

Ambiente: $cpuName, $([math]::Round($os.TotalVisibleMemorySize / 1MB, 1)) GB, $($os.Caption) build $($os.BuildNumber).
"@
