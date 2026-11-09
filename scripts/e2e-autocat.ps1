# Pure ASCII, PowerShell 5.1 compatible
param(
  [string]$DebugPort = "9222"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$logsDir = Join-Path $repoRoot "logs"
if (-not (Test-Path $logsDir)) {
  New-Item -ItemType Directory -Path $logsDir | Out-Null
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$logPath = Join-Path $logsDir "e2e-autocat-$timestamp.json"

# Set env var for WebView2 remote debugging
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort"

Write-Host "Starting debug package with CDP on port $DebugPort..."

# Try to start debug build (cargo tauri dev)
$proc = Start-Process -FilePath "cargo" -ArgumentList "tauri", "dev" -WorkingDirectory $repoRoot -PassThru -WindowStyle Hidden

Start-Sleep -Seconds 30

# Connect to CDP via HTTP
$cdpUrl = "http://localhost:$DebugPort/json"
$pages = $null
$retries = 0
while ($retries -lt 30) {
  try {
    $pages = Invoke-RestMethod -Uri $cdpUrl -Method Get -TimeoutSec 5
    if ($pages -and $pages.Count -gt 0) { break }
  } catch {
    Start-Sleep -Seconds 2
  }
  $retries++
}

if (-not $pages -or $pages.Count -eq 0) {
  throw "Failed to connect to CDP on port $DebugPort"
}

$wsUrl = $pages[0].webSocketDebuggerUrl
Add-Type -AssemblyName System.Net.WebSockets
$ws = New-Object System.Net.WebSockets.ClientWebSocket
$cts = New-Object System.Threading.CancellationTokenSource
$ws.ConnectAsync([Uri]$wsUrl, $cts.Token).Wait()

function Send-CDP {
  param($method, $params = @{})
  $script:id++
  $msg = @{id=$script:id; method=$method; params=$params} | ConvertTo-Json -Depth 10 -Compress
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($msg)
  $seg = New-Object System.ArraySegment[byte] -ArgumentList @(,$bytes)
  $ws.SendAsync($seg, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, $cts.Token).Wait()
  return $script:id
}

function Receive-CDP {
  param($timeoutMs = 10000)
  $buffer = New-Object byte[] 4096
  $seg = New-Object System.ArraySegment[byte] -ArgumentList @(,$buffer)
  $result = $ws.ReceiveAsync($seg, $cts.Token)
  $result.Wait($timeoutMs) | Out-Null
  $msg = [System.Text.Encoding]::UTF8.GetString($buffer, 0, $result.Result.Count)
  return $msg | ConvertFrom-Json
}

$script:id = 0

# Helper to click button by text
function Click-ByText {
  param($text)
  $expr = @"
(function(){
  var buttons = Array.from(document.querySelectorAll('button'));
  for (var i=0; i<buttons.length; i++) {
    if (buttons[i].textContent.indexOf('$text') >= 0) {
      buttons[i].click();
      return true;
    }
  }
  return false;
})();
"@
  $id = Send-CDP -method "Runtime.evaluate" -params @{expression=$expr; returnByValue=$true}
  Start-Sleep -Milliseconds 500
}

# Detect Steam
Click-ByText "Detect Steam"
Start-Sleep -Seconds 5

# Open live session - need user selected, try to click Open live session
Click-ByText "Open live session"
Start-Sleep -Seconds 10

# Go to Collections tab if not already there
Click-ByText "Collections"
Start-Sleep -Seconds 3

# Run AutoCat (genre dry run)
Click-ByText "Dry-run Genre"
Start-Sleep -Seconds 5

# Get preview counts from UI
$expr = @"
(function(){
  var text = document.body.innerText;
  var m1 = text.match(/newly created\s*(\d+)/i) || text.match(/\u65b0\u5efa[^\d]*(\d+)/);
  var m2 = text.match(/add[^\d]*(\d+)/i) || text.match(/\u6dfb\u52a0\u64cd\u4f5c[^\d]*(\d+)/);
  return {raw: text.substring(0, 500), n1: m1 ? m1[1] : null, n2: m2 ? m2[1] : null};
})();
"@
$id = Send-CDP -method "Runtime.evaluate" -params @{expression=$expr; returnByValue=$true}
Start-Sleep -Milliseconds 500
$res = Receive-CDP

# Queue to pending
Click-ByText "Queue"
Start-Sleep -Seconds 3

# Get pending counts
$expr2 = @"
(function(){
  var text = document.body.innerText;
  var c1 = text.match(/(\d+)\s*\u521b\u5efa/) || text.match(/creates?\s*(\d+)/i);
  var c2 = text.match(/(\d+)\s*\u6dfb\u52a0/) || text.match(/add[^\d]*(\d+)/i);
  return {c1: c1 ? c1[1] : null, c2: c2 ? c2[1] : null};
})();
"@
$id = Send-CDP -method "Runtime.evaluate" -params @{expression=$expr2; returnByValue=$true}
Start-Sleep -Milliseconds 500
$res2 = Receive-CDP

$log = @{
  timestamp = (Get-Date).ToString("o")
  preview_new = $null
  preview_add = $null
  pending_create = $null
  pending_add = $null
}

$log | ConvertTo-Json -Depth 5 | Out-File -FilePath $logPath -Encoding ASCII

Write-Host "Log written to $logPath"
Write-Host "Done (export not executed as requested)"
