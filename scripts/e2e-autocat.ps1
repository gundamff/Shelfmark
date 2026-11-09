# Pure ASCII, PowerShell 5.1 compatible.
#
# End-to-end acceptance for AutoCat (genre rules) on Shelfmark.
# Boots the dev build with WebView2 remote debugging, drives the real UI over CDP,
# and asserts that the AutoCat preview counts match the queued changeset counts.
#
# It deliberately does NOT export: writing to Steam requires the game client to be
# fully closed and is left to a human.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\e2e-autocat.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\e2e-autocat.ps1 -KeepRunning
#
# Requires: npm deps installed (npm install), Steam installed with at least one user,
# and metadata already fetched once so the genre dry-run has something to match.

param(
  [string]$DebugPort = "9222",
  [int]$BootTimeoutSec = 240,
  [switch]$KeepRunning
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$logsDir = Join-Path $repoRoot "logs"
if (-not (Test-Path $logsDir)) { New-Item -ItemType Directory -Path $logsDir | Out-Null }
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$logPath = Join-Path $logsDir "e2e-autocat-$stamp.json"

# UI strings, escaped so this file stays pure ASCII (PS 5.1 reads BOM-less
# files as ANSI and would mangle raw CJK literals).
$S_DETECT   = '\u68c0\u6d4b Steam'                                  # Detect Steam
$S_LIVE     = '\u6253\u5f00\u5b9e\u65f6\u4f1a\u8bdd'                # Open live session
$S_DRYRUN   = '\u8bd5\u8fd0\u884c Genre'                            # Dry-run genre rules
$S_QUEUE    = '\u52a0\u5165\u5f85\u5bfc\u51fa'                      # Queue changeset to pending
$S_NEEDSESSION = '\u8bf7\u5148\u5728\u8bbe\u7f6e\u4e2d\u6253\u5f00' # 'open a demo or live session first' banner

$proc = $null
$ws = $null
$script:msgId = 0
$result = @{
  timestamp          = (Get-Date).ToString("o")
  ok                 = $false
  error              = $null
  preview_matched    = $null
  preview_skipped    = $null
  preview_new        = $null
  preview_add        = $null
  pending_create     = $null
  pending_rename     = $null
  pending_delete     = $null
  pending_add        = $null
  pending_remove     = $null
  assertion_passed   = $false
  export_executed    = $false
}

function Save-Result {
  $result | ConvertTo-Json -Depth 6 | Out-File -FilePath $logPath -Encoding UTF8
}

function Stop-DevStack {
  if ($script:proc) {
    try { Stop-Process -Id $script:proc.Id -Force -ErrorAction SilentlyContinue } catch { }
  }
  Get-Process shelfmark -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  $listener = Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue
  if ($listener) {
    $listener.OwningProcess | Select-Object -Unique | ForEach-Object {
      Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue
    }
  }
}

function Send-CdpEval {
  param([string]$Expression, [int]$TimeoutMs = 60000)

  $script:msgId++
  $id = $script:msgId
  $payload = @{
    id     = $id
    method = "Runtime.evaluate"
    params = @{ expression = $Expression; returnByValue = $true; awaitPromise = $true }
  } | ConvertTo-Json -Depth 8 -Compress

  $bytes = [System.Text.Encoding]::UTF8.GetBytes($payload)
  $seg = New-Object System.ArraySegment[byte] -ArgumentList @(, $bytes)
  $ws.SendAsync($seg, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, [System.Threading.CancellationToken]::None).Wait()

  $deadline = (Get-Date).AddMilliseconds($TimeoutMs)
  $sb = New-Object System.Text.StringBuilder
  while ((Get-Date) -lt $deadline) {
    $buf = New-Object byte[] 65536
    $s = New-Object System.ArraySegment[byte] -ArgumentList @(, $buf)
    $task = $ws.ReceiveAsync($s, [System.Threading.CancellationToken]::None)
    if (-not $task.Wait($TimeoutMs)) { break }
    if ($task.Result.MessageType -eq [System.Net.WebSockets.WebSocketMessageType]::Close) {
      throw "CDP socket closed by peer"
    }
    $null = $sb.Append([System.Text.Encoding]::UTF8.GetString($buf, 0, $task.Result.Count))
    if (-not $task.Result.EndOfMessage) { continue }

    $text = $sb.ToString()
    $null = $sb.Clear()
    if (-not $text) { continue }
    try { $msg = $text | ConvertFrom-Json } catch { continue }
    # Event notifications carry no id; skip anything that is not our reply.
    if ($null -eq $msg.id -or $msg.id -ne $id) { continue }
    if ($msg.error) { throw ("CDP error: " + ($msg.error | ConvertTo-Json -Compress)) }
    if ($msg.result.exceptionDetails) {
      throw ("JS error: " + $msg.result.exceptionDetails.text)
    }
    return $msg.result.result.value
  }
  throw "CDP timeout waiting for id $id"
}

function Click-Button {
  param([string]$Label)
  $expr = @'
(function(){
  var want = '__LABEL__';
  var bs = Array.from(document.querySelectorAll('button'));
  for (var i = 0; i < bs.length; i++) {
    var b = bs[i];
    if ((b.innerText || '').trim() === want) {
      if (b.disabled) { return 'disabled'; }
      b.click();
      return 'ok';
    }
  }
  return 'notfound';
})()
'@
  $expr = $expr.Replace("__LABEL__", $Label)
  return (Send-CdpEval -Expression $expr -TimeoutMs 30000)
}

function Read-State {
  $expr = @'
(function(){
  var t = document.body.innerText || '';
  var out = { session_open: t.indexOf('__NEEDSESSION__') < 0 };

  // AutoCat block: "matched N, skipped M, new X, add operations Y"
  var pm = t.match(/__MATCHED__\s*(\d+)[\s\S]{0,80}?__NEW__\s*(\d+)[\s\S]{0,80}?__ADDOP__\s*(\d+)/);
  out.preview_matched = pm ? parseInt(pm[1], 10) : null;
  out.preview_new     = pm ? parseInt(pm[2], 10) : null;
  out.preview_add     = pm ? parseInt(pm[3], 10) : null;

  var ps = t.match(/__SKIPPED__\s*(\d+)/);
  out.preview_skipped = ps ? parseInt(ps[1], 10) : null;

  // Pending block: "A created . B renamed . C deleted . D added . E removed"
  var qm = t.match(/(\d+)\s*__CREATE__[\s\S]{0,60}?(\d+)\s*__RENAME__[\s\S]{0,60}?(\d+)\s*__DELETE__[\s\S]{0,60}?(\d+)\s*__ADD__[\s\S]{0,60}?(\d+)\s*__REMOVE__/);
  if (qm) {
    out.pending_create = parseInt(qm[1], 10);
    out.pending_rename = parseInt(qm[2], 10);
    out.pending_delete = parseInt(qm[3], 10);
    out.pending_add    = parseInt(qm[4], 10);
    out.pending_remove = parseInt(qm[5], 10);
  } else {
    out.pending_create = null; out.pending_rename = null; out.pending_delete = null;
    out.pending_add = null; out.pending_remove = null;
  }
  return out;
})()
'@
  $expr = $expr.Replace("__NEEDSESSION__", $S_NEEDSESSION)
  $expr = $expr.Replace("__MATCHED__", '\u5339\u914d')
  $expr = $expr.Replace("__SKIPPED__", '\u8df3\u8fc7')
  $expr = $expr.Replace("__NEW__", '\u65b0\u5efa')
  $expr = $expr.Replace("__ADDOP__", '\u6dfb\u52a0\u64cd\u4f5c')
  $expr = $expr.Replace("__CREATE__", '\u521b\u5efa')
  $expr = $expr.Replace("__RENAME__", '\u91cd\u547d\u540d')
  $expr = $expr.Replace("__DELETE__", '\u5220\u9664')
  $expr = $expr.Replace("__ADD__", '\u6dfb\u52a0')
  $expr = $expr.Replace("__REMOVE__", '\u79fb\u9664')
  return (Send-CdpEval -Expression $expr -TimeoutMs 30000)
}

try {
  # ---- 0. clean slate -------------------------------------------------------
  Get-Process shelfmark -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 1

  # ---- 1. boot dev build with WebView2 CDP --------------------------------
  Write-Host "Starting dev build (npm run tauri dev) with CDP on port $DebugPort ..."
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort"
  $npm = (Get-Command npm.cmd -ErrorAction SilentlyContinue).Source
  if (-not $npm) { throw "npm.cmd not found on PATH" }
  $script:proc = Start-Process -FilePath $npm -ArgumentList "run", "tauri", "dev" `
    -WorkingDirectory $repoRoot -PassThru -WindowStyle Hidden

  # ---- 2. wait for the CDP endpoint ---------------------------------------
  $target = $null
  $deadline = (Get-Date).AddSeconds($BootTimeoutSec)
  while ((Get-Date) -lt $deadline) {
    try {
      $pages = Invoke-RestMethod -Uri "http://localhost:$DebugPort/json" -Method Get -TimeoutSec 5
      $cand = $pages | Where-Object { $_.type -eq 'page' -and $_.url -like '*1420*' } | Select-Object -First 1
      if (-not $cand) { $cand = $pages | Where-Object { $_.type -eq 'page' } | Select-Object -First 1 }
      if ($cand -and $cand.webSocketDebuggerUrl) { $target = $cand; break }
    } catch { }
    Start-Sleep -Seconds 3
  }
  if (-not $target) { throw "CDP endpoint did not come up within $BootTimeoutSec seconds" }
  Write-Host "CDP target: $($target.url)"

  # ---- 3. attach -----------------------------------------------------------
  # ClientWebSocket ships in System.dll on .NET Framework 4.5+; Add-Type is not needed.
  $ws = New-Object System.Net.WebSockets.ClientWebSocket
  $cts = [System.Threading.CancellationToken]::None
  $ws.ConnectAsync([Uri]$target.webSocketDebuggerUrl, $cts).Wait()
  $null = Send-CdpEval -Expression "'ready'" -TimeoutMs 15000
  Write-Host "Attached."

  # ---- 4. detect Steam -----------------------------------------------------
  $r = Click-Button -Label $S_DETECT
  if ($r -ne 'ok') { throw "button 'detect steam' not clickable: $r" }
  Start-Sleep -Seconds 6
  Write-Host "Steam detected."

  # ---- 5. open live session ------------------------------------------------
  $r = Click-Button -Label $S_LIVE
  if ($r -ne 'ok') { throw "button 'open live session' not clickable: $r" }

  $state = $null
  $deadline = (Get-Date).AddSeconds(90)
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 3
    $state = Read-State
    if ($state.session_open) { break }
  }
  if (-not $state -or -not $state.session_open) {
    throw "live session did not open (no Steam user or library unreadable)"
  }
  Write-Host "Live session open."

  # ---- 6. genre dry run ----------------------------------------------------
  $r = Click-Button -Label $S_DRYRUN
  if ($r -ne 'ok') { throw "button 'dry-run genre' not clickable: $r" }

  $preview = $null
  $deadline = (Get-Date).AddSeconds(120)
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 3
    $state = Read-State
    if ($null -ne $state.preview_new) { $preview = $state; break }
  }
  if (-not $preview) { throw "dry run produced no preview counts" }
  $result.preview_matched = $preview.preview_matched
  $result.preview_skipped = $preview.preview_skipped
  $result.preview_new     = $preview.preview_new
  $result.preview_add     = $preview.preview_add
  Write-Host ("Preview: matched={0} skipped={1} new={2} add_ops={3}" -f `
    $preview.preview_matched, $preview.preview_skipped, $preview.preview_new, $preview.preview_add)

  # ---- 7. queue the changeset ---------------------------------------------
  $r = Click-Button -Label $S_QUEUE
  if ($r -ne 'ok') { throw "button 'queue to pending' not clickable: $r" }
  Start-Sleep -Seconds 4

  $pending = Read-State
  $result.pending_create = $pending.pending_create
  $result.pending_rename = $pending.pending_rename
  $result.pending_delete = $pending.pending_delete
  $result.pending_add    = $pending.pending_add
  $result.pending_remove = $pending.pending_remove
  Write-Host ("Pending: create={0} rename={1} delete={2} add={3} remove={4}" -f `
    $pending.pending_create, $pending.pending_rename, $pending.pending_delete, `
    $pending.pending_add, $pending.pending_remove)

  # ---- 8. assert -----------------------------------------------------------
  $problems = @()
  if ($pending.pending_create -ne $preview.preview_new) {
    $problems += "create mismatch: preview=$($preview.preview_new) pending=$($pending.pending_create)"
  }
  if ($pending.pending_add -ne $preview.preview_add) {
    $problems += "add mismatch: preview=$($preview.preview_add) pending=$($pending.pending_add)"
  }
  if ($pending.pending_delete -ne 0) {
    $problems += "unexpected deletes: $($pending.pending_delete)"
  }
  if ($problems.Count -gt 0) { throw ($problems -join "; ") }

  $result.assertion_passed = $true
  $result.ok = $true
  Write-Host "PASS: preview counts match the queued changeset."
  Write-Host "Export was NOT executed (by design)."
}
catch {
  $result.error = $_.Exception.Message
  Write-Host ("FAIL: " + $_.Exception.Message)
}
finally {
  if ($ws) { try { $ws.Dispose() } catch { } }
  Save-Result
  if (-not $KeepRunning) { Stop-DevStack }
  Write-Host "Log written to $logPath"
}

if ($result.ok) { exit 0 } else { exit 1 }
