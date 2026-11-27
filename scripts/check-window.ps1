# Automated check of island-window behavior: click-through, hit area, focus, Alt+Tab/taskbar styles.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\check-window.ps1 [-Exe target\release\dynamic-island.exe] [-Shot out.png] [-Demo]
# Moves the mouse and clicks briefly over a test form it creates; restores the cursor afterwards.
param(
    [string]$Exe = "$PSScriptRoot\..\target\release\dynamic-island.exe",
    [string]$Shot = "",
    # Builds with the Demo menu (debug/profiling): an activity shows Compact with no gesture, and ending it hides again.
    [switch]$Demo
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint flags);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int x, int y, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int i);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static IntPtr At(int x, int y) { var p = new POINT{X=x,Y=y}; return GetAncestor(WindowFromPoint(p), 2); }
  public static void Click(int x, int y) { SetCursorPos(x, y); mouse_event(2,0,0,0,IntPtr.Zero); mouse_event(4,0,0,0,IntPtr.Zero); }
}
"@
[void][W]::SetProcessDpiAwarenessContext([IntPtr]-4)  # per-monitor v2, physical px like the app

$results = [System.Collections.Generic.List[object]]::new()
function Check($name, $ok, $detail = '') { $results.Add([pscustomobject]@{ Test = $name; Result = $(if ($ok) { 'PASS' } else { 'FAIL' }); Detail = $detail }) }
function Pump($ms) { $end = (Get-Date).AddMilliseconds($ms); while ((Get-Date) -lt $end) { [System.Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 10 } }

Get-Process dynamic-island -ErrorAction SilentlyContinue | Stop-Process -Force
$work = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea

# Test target covering the top-center of the primary work area.
$form = New-Object System.Windows.Forms.Form
$form.FormBorderStyle = 'None'; $form.StartPosition = 'Manual'; $form.BackColor = 'White'; $form.ShowInTaskbar = $false
# Briefly topmost so it lands above any maximized app (focus-steal prevention), then back to the normal band
# so the island (topmost) stays above it even after the form takes the foreground.
$form.Bounds = New-Object System.Drawing.Rectangle(($work.X + $work.Width / 2 - 400), $work.Y, 800, 320)
$script:clicks = 0
$form.Add_MouseClick({ $script:clicks++ })
$form.TopMost = $true; $form.Show(); Pump 300; $form.TopMost = $false; Pump 100

$t0 = Get-Date
$app = Start-Process -FilePath $Exe -PassThru
$inp = [IntPtr]::Zero
while ($inp -eq [IntPtr]::Zero -and ((Get-Date) - $t0).TotalSeconds -lt 5) { Pump 50; $inp = [W]::FindWindow('DynamicIslandWindow', 'Dynamic Island Input') }
$visual = [W]::FindWindow('DynamicIslandWindow', 'Dynamic Island')
Check 'app window created' ($inp -ne [IntPtr]::Zero) ("{0:N0} ms" -f ((Get-Date) - $t0).TotalMilliseconds)
if ($inp -eq [IntPtr]::Zero) { $results | Format-Table -AutoSize; exit 1 }
Pump 500
function InRect { $x = New-Object W+RECT; [void][W]::GetWindowRect($inp, [ref]$x); $x }
function Size($x) { "{0}x{1} at ({2},{3})" -f ($x.R - $x.L), ($x.B - $x.T), $x.L, $x.T }
$formH = $form.Handle
$saved = New-Object W+POINT; [void][W]::GetCursorPos([ref]$saved)
$away = @(($work.X + 40), ($work.Y + 500))
[void][W]::SetCursorPos($away[0], $away[1]); Pump 300

# Hidden on start (no activity): the input window is only the top-center reveal strip.
$r = InRect; $cx = [int](($r.L + $r.R) / 2); $idleW = $r.R - $r.L  # the strip spans the Idle width
Check 'hidden on start: input is the reveal strip' ((($r.B - $r.T) -le 3) -and ($r.T -eq $work.Y)) (Size $r)
Check 'strip centered in work area (±1px)' ([math]::Abs($cx - ($work.X + $work.Width / 2)) -le 1) "center x=$cx"
Check 'strip -> island' ([W]::At($cx, $r.T) -eq $inp)
Check 'strip: 1px left -> window below' ([W]::At($r.L - 1, $r.T) -eq $formH)
Check 'strip: 1px below -> window below' ([W]::At($cx, $r.B + 1) -eq $formH)
Check 'hidden: where the capsule would be -> window below' ([W]::At($cx, $work.Y + 30) -eq $formH)

# Extended styles: TOOLWINDOW (0x80) hides from taskbar/Alt+Tab, NOACTIVATE (0x08000000), TRANSPARENT (0x20) on visual.
$exIn = [W]::GetWindowLong($inp, -20); $exVis = [W]::GetWindowLong($visual, -20)
Check 'not in taskbar/Alt+Tab (TOOLWINDOW)' ((($exIn -band 0x80) -ne 0) -and (($exVis -band 0x80) -ne 0))
Check 'never activates (NOACTIVATE)' ((($exIn -band 0x08000000) -ne 0) -and (($exVis -band 0x08000000) -ne 0))

# Reveal: park the cursor on the strip.
[void][W]::SetCursorPos($cx, $r.T); $t0 = Get-Date
do { Pump 10; $r = InRect } while (($r.B - $r.T) -le 3 -and ((Get-Date) - $t0).TotalMilliseconds -lt 2000)
Check 'reveal after dwelling on the strip' (($r.B - $r.T) -gt 3) ("{0:N0} ms" -f ((Get-Date) - $t0).TotalMilliseconds)
Pump 100; $r = InRect
$cx = [int](($r.L + $r.R) / 2); $cy = [int](($r.T + $r.B) / 2)
Check 'capsule size (px)' $true (Size $r)
Check 'capsule centered in work area (±1px)' ([math]::Abs($cx - ($work.X + $work.Width / 2)) -le 1) "center x=$cx"
[void][W]::SetCursorPos($cx, $cy); Pump 300   # inside the 1 s grace; hover keeps it revealed
$r = InRect; $cx = [int](($r.L + $r.R) / 2); $cy = [int](($r.T + $r.B) / 2)
Check 'hit-test inside capsule -> island' ([W]::At($cx, $cy) -eq $inp)
Check 'hit-test 2px left of capsule -> window below' ([W]::At($r.L - 2, $cy) -eq $formH)
Check 'hit-test 2px above capsule (visual pad) -> window below' ([W]::At($cx, $r.T - 2) -eq $formH)
Check 'hit-test rounded corner (outside shape) -> window below' ([W]::At($r.L + 2, $r.T + 2) -eq $formH)
Check 'hit-test on shadow below capsule -> window below' ([W]::At($cx, $r.B + 4) -eq $formH)

[void][W]::SetForegroundWindow($formH); Pump 200
$before = $script:clicks
[W]::Click($r.L - 2, $cy); Pump 300
Check 'real click 2px outside reaches window below' ($script:clicks -eq $before + 1)

$before = $script:clicks
$fg = [W]::GetForegroundWindow()  # after the outside click, which legitimately activates the form
[W]::Click($cx, $cy); Pump 600     # click on revealed Idle -> Expanded (clock)
Check 'real click inside does not reach window below' ($script:clicks -eq $before)
Check 'focus unchanged after clicking capsule' ([W]::GetForegroundWindow() -eq $fg)
$e = InRect; $ex = [int](($e.L + $e.R) / 2)
Check 'click expands (clock, no activity)' (($e.B - $e.T) -gt 3 * ($r.B - $r.T)) (Size $e)
Check 'expanded: 2px below shape -> window below' ([W]::At($ex, $e.B + 2) -eq $formH)
Check 'expanded: on shadow below -> window below' ([W]::At($ex, $e.B + 8) -eq $formH)
Check 'expanded: inside -> island' ([W]::At($ex, $e.B - 10) -eq $inp)

[void][W]::SetCursorPos($away[0], $away[1]); Pump 1600
$h = InRect
Check 'hides 1 s after the cursor leaves' (($h.B - $h.T) -le 3) (Size $h)

if ($Demo) {
    # Tray menu (WM_APP+1 / WM_RBUTTONUP) -> 2nd item "Demo: iniciar atividade": Compact shows with no gesture.
    $hostWnd = [W]::FindWindow('DynamicIslandWindow', 'Dynamic Island Host')
    [void][W]::PostMessage($hostWnd, 0x8001, [IntPtr]1, [IntPtr]0x0205); Pump 400
    [System.Windows.Forms.SendKeys]::SendWait('{DOWN}{DOWN}{ENTER}'); Pump 600
    $c = InRect
    Check 'demo: activity shows Compact without gesture' ((($c.B - $c.T) -gt 3) -and (($c.R - $c.L) -gt 1.5 * $idleW)) (Size $c)
    [void][W]::PostMessage($hostWnd, 0x8001, [IntPtr]1, [IntPtr]0x0205); Pump 400
    [System.Windows.Forms.SendKeys]::SendWait('{DOWN}{DOWN}{DOWN}{ENTER}'); Pump 600
    $c = InRect
    Check 'demo: activity end hides again' (($c.B - $c.T) -le 3) (Size $c)
}
[void][W]::SetCursorPos($saved.X, $saved.Y)

if ($Shot) {
    $form.Hide(); Pump 300
    $vr = New-Object W+RECT; [void][W]::GetWindowRect($visual, [ref]$vr)
    $pad = 40
    $bmp = New-Object System.Drawing.Bitmap(($vr.R - $vr.L + 2 * $pad), ($vr.B - $vr.T + 2 * $pad))
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen(($vr.L - $pad), [math]::Max(0, $vr.T - $pad), 0, 0, $bmp.Size)
    $bmp.Save($Shot, [System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()
}

$form.Close()
Stop-Process -Id $app.Id -Force
$results | Format-Table -AutoSize
if ($results.Result -contains 'FAIL') { exit 1 }
