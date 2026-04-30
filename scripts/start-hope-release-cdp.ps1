param(
  [string]$RepoRoot = "E:\codex\hope-desktop-shell",
  [int]$Port = 9224,
  [string]$EnvFile = "C:\Users\Administrator\.codex\.sandbox-secrets\hope-qwen.env",
  [string]$Provider = "qwen",
  [string]$Model = "qwen-plus",
  [string]$ExpectedTargetUrl = "http://tauri.localhost/#/workbench",
  [int]$WaitSeconds = 12,
  [int]$StopWaitSeconds = 10,
  [int]$FileUnlockWaitSeconds = 10,
  [int]$DiagnosticTailLines = 80,
  [string]$BaseUrl = "https://dashscope.aliyuncs.com/compatible-mode/v1",
  [ValidateSet("auto", "true", "false")]
  [string]$ModelEnabled = "auto",
  [ValidateSet("Minimal", "MinimalNoErrDialogs", "MinimalDisableBreakpad", "MinimalDisableCrashReporter", "FullDefault", "AppDefault")]
  [string]$WebView2ArgumentMode = "Minimal",
  [switch]$StopExisting,
  [switch]$StopOnly,
  [switch]$StopOnCdpFailure,
  [switch]$KeepOnCdpFailure
)

$ErrorActionPreference = "Stop"
$script:LastHopeWebView2QueryError = $null
$script:ApplicationEventBaseline = Get-Date

function Convert-ResultJson {
  param([hashtable]$Data)
  [pscustomobject]$Data | ConvertTo-Json -Depth 6
}

function Get-EnvFileMap {
  param([string]$Path)
  $map = @{}
  if (-not (Test-Path -LiteralPath $Path)) {
    return $map
  }

  foreach ($line in Get-Content -LiteralPath $Path -Encoding UTF8) {
    $trim = $line.Trim()
    if ($trim.Length -eq 0 -or $trim.StartsWith("#")) {
      continue
    }
    if ($trim -match "^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$") {
      $key = $matches[1]
      $value = $matches[2].Trim()
      if (($value.StartsWith('"') -and $value.EndsWith('"')) -or ($value.StartsWith("'") -and $value.EndsWith("'"))) {
        $value = $value.Substring(1, $value.Length - 2)
      }
      $map[$key] = $value
    }
  }
  return $map
}

function Quote-CmdArg {
  param([string]$Value)
  '"' + ($Value -replace '"', '\"') + '"'
}

function New-WebView2BrowserArguments {
  param(
    [string]$Mode,
    [int]$LocalPort
  )
  $remoteDebugging = "--remote-debugging-port=$LocalPort"
  switch ($Mode) {
    "Minimal" { return $remoteDebugging }
    "MinimalNoErrDialogs" { return "$remoteDebugging --noerrdialogs" }
    "MinimalDisableBreakpad" { return "$remoteDebugging --disable-breakpad" }
    "MinimalDisableCrashReporter" { return "$remoteDebugging --disable-crash-reporter" }
    "FullDefault" { return "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --noerrdialogs --disable-crash-reporter --disable-breakpad $remoteDebugging" }
    "AppDefault" { return $null }
    default { return $remoteDebugging }
  }
}

function Test-DirectoryHasChildren {
  param([string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    return $false
  }
  @(Get-ChildItem -LiteralPath $Path -Force -ErrorAction SilentlyContinue | Select-Object -First 1).Count -gt 0
}

function Get-ProcessState {
  param([Nullable[int]]$Id)
  if (-not $Id) {
    return [ordered]@{
      running = $false
      exit_code = $null
      path = $null
    }
  }

  $proc = Get-Process -Id $Id -ErrorAction SilentlyContinue
  if (-not $proc) {
    return [ordered]@{
      running = $false
      exit_code = $null
      path = $null
    }
  }

  $exitCode = $null
  try {
    if ($proc.HasExited) {
      $exitCode = $proc.ExitCode
    }
  } catch {
    $exitCode = $null
  }

  return [ordered]@{
    running = -not $proc.HasExited
    exit_code = $exitCode
    path = $proc.Path
  }
}

function Get-CdpTargetSummary {
  param($Targets)
  @($Targets | ForEach-Object {
    [ordered]@{
      title = $_.title
      url = $_.url
      type = $_.type
    }
  })
}

function Get-PortState {
  param([int]$LocalPort)
  $connections = @(Get-NetTCPConnection -LocalPort $LocalPort -ErrorAction SilentlyContinue)
  $listening = @($connections | Where-Object { $_.State -eq "Listen" })
  [ordered]@{
    listening = $listening.Count -gt 0
    owning_processes = @($connections | ForEach-Object { $_.OwningProcess } | Where-Object { $_ } | Sort-Object -Unique)
    states = @($connections | ForEach-Object { $_.State.ToString() } | Sort-Object -Unique)
  }
}

function Wait-PortReleased {
  param(
    [int]$LocalPort,
    [int]$TimeoutSeconds
  )
  $startedAt = Get-Date
  $deadline = $startedAt.AddSeconds($TimeoutSeconds)
  do {
    $state = Get-PortState -LocalPort $LocalPort
    if (-not $state.listening) {
      return [ordered]@{
        released = $true
        wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
        owning_processes = $state.owning_processes
        states = $state.states
      }
    }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $deadline)

  $state = Get-PortState -LocalPort $LocalPort
  return [ordered]@{
    released = -not [bool]$state.listening
    wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
    owning_processes = $state.owning_processes
    states = $state.states
  }
}

function Get-HopeAppProcessIds {
  @(
    Get-Process hope-app -ErrorAction SilentlyContinue |
      ForEach-Object { $_.Id } |
      Sort-Object -Unique
  )
}

function Get-WebView2UserDataDir {
  param([string]$CommandLine)
  if ([string]::IsNullOrWhiteSpace($CommandLine)) {
    return $null
  }
  if ($CommandLine -match '--user-data-dir="([^"]+)"') {
    return $matches[1]
  }
  if ($CommandLine -match '--user-data-dir=([^\s]+)') {
    return $matches[1]
  }
  return $null
}

function Get-WebView2RemoteDebuggingPorts {
  param([string]$CommandLine)
  if ([string]::IsNullOrWhiteSpace($CommandLine)) {
    return @()
  }
  @(
    [regex]::Matches($CommandLine, '--remote-debugging-port=(\d+)') |
      ForEach-Object { $_.Groups[1].Value } |
      Sort-Object -Unique
  )
}

function Test-PathAtOrUnder {
  param(
    [string]$Path,
    [string]$Root
  )
  if ([string]::IsNullOrWhiteSpace($Path) -or [string]::IsNullOrWhiteSpace($Root)) {
    return $false
  }
  $normalizedPath = $Path.TrimEnd('\', '/')
  $normalizedRoot = $Root.TrimEnd('\', '/')
  return $normalizedPath.Equals($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
    $normalizedPath.StartsWith($normalizedRoot + "\", [System.StringComparison]::OrdinalIgnoreCase) -or
    $normalizedPath.StartsWith($normalizedRoot + "/", [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-RelevantApplicationEvents {
  param([datetime]$Since)
  try {
    @(
      Get-WinEvent -FilterHashtable @{LogName = "Application"; StartTime = $Since} -ErrorAction Stop |
        Where-Object {
          $_.ProviderName -in @("Application Error", "Application Popup", "Windows Error Reporting") -or
          $_.Message -match "msedgewebview2|hope-app|WebView2"
        } |
        Sort-Object TimeCreated -Descending |
        Select-Object -First 20 |
        ForEach-Object {
          $summary = ($_.Message -split "`r?`n" | Where-Object { $_.Trim().Length -gt 0 } | Select-Object -First 4) -join " | "
          [ordered]@{
            time = $_.TimeCreated.ToString("o")
            provider = $_.ProviderName
            id = $_.Id
            level = $_.LevelDisplayName
            summary = $summary
          }
        }
    )
  } catch {
    if ($_.Exception.Message -like "*No events were found*") {
      return @()
    }
    @(
      [ordered]@{
        time = (Get-Date).ToString("o")
        provider = "query_error"
        id = $null
        level = "Error"
        summary = $_.Exception.Message
      }
    )
  }
}

function Test-DirectoryWritable {
  param([string]$Path)
  try {
    New-Item -ItemType Directory -Force -Path $Path | Out-Null
    $probe = Join-Path $Path (".lock-probe-" + [guid]::NewGuid().ToString("N") + ".tmp")
    Set-Content -LiteralPath $probe -Value "probe" -NoNewline -Encoding ASCII
    Remove-Item -LiteralPath $probe -Force
    return [ordered]@{
      writable = $true
      error = $null
    }
  } catch {
    return [ordered]@{
      writable = $false
      error = $_.Exception.Message
    }
  }
}

function Test-FileUnlocked {
  param([string]$Path)
  try {
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None)
    $stream.Close()
    return [ordered]@{
      unlocked = $true
      error = $null
    }
  } catch {
    return [ordered]@{
      unlocked = $false
      error = $_.Exception.Message
    }
  }
}

function Wait-FileUnlocked {
  param(
    [string]$Path,
    [int]$TimeoutSeconds
  )
  $startedAt = Get-Date
  $deadline = $startedAt.AddSeconds($TimeoutSeconds)
  do {
    $state = Test-FileUnlocked -Path $Path
    if ($state.unlocked) {
      return [ordered]@{
        unlocked = $true
        wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
        error = $null
      }
    }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $deadline)

  $state = Test-FileUnlocked -Path $Path
  return [ordered]@{
    unlocked = [bool]$state.unlocked
    wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
    error = $state.error
  }
}

function Wait-HopeAppStopped {
  param([int]$TimeoutSeconds)
  $startedAt = Get-Date
  $deadline = $startedAt.AddSeconds($TimeoutSeconds)
  do {
    $remaining = @(Get-Process hope-app -ErrorAction SilentlyContinue)
    if ($remaining.Count -eq 0) {
      return [ordered]@{
        stopped = $true
        wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
        remaining_pids = @()
      }
    }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $deadline)

  $remaining = @(Get-Process hope-app -ErrorAction SilentlyContinue)
  return [ordered]@{
    stopped = $remaining.Count -eq 0
    wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
    remaining_pids = @($remaining | ForEach-Object { $_.Id })
  }
}

function Get-HopeWebView2Processes {
  try {
    $script:LastHopeWebView2QueryError = $null
    return @(
      Get-CimInstance Win32_Process -Filter "name='msedgewebview2.exe'" -ErrorAction Stop |
        Where-Object {
          $_.CommandLine -like "*--webview-exe-name=hope-app.exe*" -or
          $_.CommandLine -like "*\hope-webview2-cdp\*"
        } |
        ForEach-Object {
          $commandLine = [string]$_.CommandLine
          $remoteDebuggingPorts = Get-WebView2RemoteDebuggingPorts -CommandLine $commandLine
          [ordered]@{
            id = [int]$_.ProcessId
            parent_id = [int]$_.ParentProcessId
            path = $_.ExecutablePath
            user_data_dir = Get-WebView2UserDataDir -CommandLine $commandLine
            has_remote_debugging_port = $remoteDebuggingPorts.Count -gt 0
            remote_debugging_ports = $remoteDebuggingPorts
            has_noerrdialogs = $commandLine.Contains("--noerrdialogs")
            has_disable_breakpad = $commandLine.Contains("--disable-breakpad")
            has_disable_crash_reporter = $commandLine.Contains("--disable-crash-reporter")
          }
        }
    )
  } catch {
    $script:LastHopeWebView2QueryError = $_.Exception.Message
    @()
  }
}

function Close-HopeCdpTargets {
  param(
    [int]$LocalPort,
    [string]$ExpectedUrl
  )

  $detail = [ordered]@{
    attempted = $false
    closed_target_ids = @()
    target_urls = @()
    errors = @()
  }

  try {
    $targets = @(Invoke-RestMethod -Uri "http://127.0.0.1:$LocalPort/json/list" -TimeoutSec 2)
  } catch {
    $detail.errors += $_.Exception.Message
    return $detail
  }

  $hopeTargets = @(
    $targets |
      Where-Object {
        $_.url -eq $ExpectedUrl -or $_.url -like "http://tauri.localhost/*"
      }
  )
  if ($hopeTargets.Count -eq 0) {
    return $detail
  }

  $detail.attempted = $true
  foreach ($target in $hopeTargets) {
    $detail.target_urls += $target.url
    if ([string]::IsNullOrWhiteSpace($target.id)) {
      continue
    }
    try {
      Invoke-RestMethod -Uri "http://127.0.0.1:$LocalPort/json/close/$($target.id)" -TimeoutSec 2 | Out-Null
      $detail.closed_target_ids += $target.id
    } catch {
      $detail.errors += $_.Exception.Message
    }
  }

  return $detail
}

function Wait-HopeWebView2Stopped {
  param([int]$TimeoutSeconds)
  $startedAt = Get-Date
  $deadline = $startedAt.AddSeconds($TimeoutSeconds)
  do {
    $remaining = @(Get-HopeWebView2Processes)
    if ($remaining.Count -eq 0) {
      return [ordered]@{
        stopped = $true
        wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
        remaining_pids = @()
      }
    }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $deadline)

  $remaining = @(Get-HopeWebView2Processes)
  return [ordered]@{
    stopped = $remaining.Count -eq 0
    wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
    remaining_pids = @($remaining | ForEach-Object { $_.id })
  }
}

function Stop-HopeProcessTree {
  param(
    [int[]]$ProcessIds,
    [int]$TimeoutSeconds,
    [int]$CdpPort,
    [string]$ExpectedTargetUrl
  )

  $detail = [ordered]@{
    requested_pids = @($ProcessIds | Where-Object { $_ } | Sort-Object -Unique)
    close_main_window_pids = @()
    force_stop_pids = @()
    webview2_pids_before_stop = @()
    webview2_query_error = $null
    cdp_close_detail = $null
    webview2_force_stop_pids = @()
    webview2_remaining_pids = @()
    port_released = $false
    port_wait_ms = 0
    port_owning_processes_after_stop = @()
    port_states_after_stop = @()
    stopped = $false
    wait_ms = 0
  }

  $startedAt = Get-Date
  $uniquePids = @($detail.requested_pids)
  $webviewBefore = @(Get-HopeWebView2Processes)
  $detail.webview2_query_error = $script:LastHopeWebView2QueryError
  $detail.webview2_pids_before_stop = @($webviewBefore | ForEach-Object { $_.id })
  if ($uniquePids.Count -eq 0 -and $webviewBefore.Count -eq 0) {
    $portState = Wait-PortReleased -LocalPort $CdpPort -TimeoutSeconds 2
    $detail.port_released = [bool]$portState.released
    $detail.port_wait_ms = $portState.wait_ms
    $detail.port_owning_processes_after_stop = $portState.owning_processes
    $detail.port_states_after_stop = $portState.states
    $detail.stopped = $true
    return $detail
  }

  $detail.cdp_close_detail = Close-HopeCdpTargets -LocalPort $CdpPort -ExpectedUrl $ExpectedTargetUrl
  if ($detail.cdp_close_detail.attempted) {
    Start-Sleep -Milliseconds 500
  }

  foreach ($procId in $uniquePids) {
    $proc = Get-Process -Id $procId -ErrorAction SilentlyContinue
    if (-not $proc) {
      continue
    }

    try {
      if ($proc.MainWindowHandle -ne 0 -and $proc.CloseMainWindow()) {
        $detail.close_main_window_pids += $procId
      }
    } catch {
    }
  }

  $graceSeconds = [Math]::Min([Math]::Max([int]($TimeoutSeconds / 2), 2), 6)
  $graceState = Wait-HopeAppStopped -TimeoutSeconds $graceSeconds
  if (-not $graceState.stopped) {
    foreach ($remainingPid in $graceState.remaining_pids) {
      Stop-Process -Id $remainingPid -Force -ErrorAction SilentlyContinue
      $detail.force_stop_pids += $remainingPid
    }
  }

  $remainingSeconds = [Math]::Max($TimeoutSeconds - $graceSeconds, 2)
  $hostState = Wait-HopeAppStopped -TimeoutSeconds $remainingSeconds
  $webviewState = Wait-HopeWebView2Stopped -TimeoutSeconds 3
  if (-not $detail.webview2_query_error) {
    $detail.webview2_query_error = $script:LastHopeWebView2QueryError
  }
  if (-not $webviewState.stopped) {
    foreach ($webviewPid in $webviewState.remaining_pids) {
      Stop-Process -Id $webviewPid -Force -ErrorAction SilentlyContinue
      $detail.webview2_force_stop_pids += $webviewPid
    }
    $webviewState = Wait-HopeWebView2Stopped -TimeoutSeconds 2
  }

  $portState = Wait-PortReleased -LocalPort $CdpPort -TimeoutSeconds 3
  $detail.webview2_remaining_pids = @($webviewState.remaining_pids)
  $detail.port_released = [bool]$portState.released
  $detail.port_wait_ms = $portState.wait_ms
  $detail.port_owning_processes_after_stop = $portState.owning_processes
  $detail.port_states_after_stop = $portState.states
  $detail.stopped = [bool]($hostState.stopped -and $webviewState.stopped -and $portState.released)
  $detail.wait_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
  return $detail
}

function Get-SanitizedFileTail {
  param(
    [string]$Path,
    [int]$TailLines
  )
  if (-not (Test-Path -LiteralPath $Path)) {
    return @()
  }

  @(
    Get-Content -LiteralPath $Path -Tail $TailLines -ErrorAction SilentlyContinue |
      ForEach-Object {
        ($_ -replace '(?i)(api[_-]?key|token|secret|authorization)\s*=\s*[^,\s]+', '$1=<redacted>') `
           -replace '(?i)(api[_-]?key|token|secret|authorization)["'']?\s*:\s*["''][^"'']+["'']', '$1:<redacted>'
      }
  )
}

$exe = Join-Path $RepoRoot "target\release\hope-app.exe"
$workDir = Join-Path $RepoRoot "target\release"
$launchId = "hope-cdp-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + $PID
$diagnosticLog = Join-Path $workDir "hope-shell-diagnostic.log"
$profileRoot = Join-Path $env:TEMP "hope-webview2-cdp"
$profileDir = Join-Path $profileRoot $launchId
$effectiveModelEnabled = if ($ModelEnabled -eq "auto") {
  $Provider -eq "qwen" -and $Model -eq "qwen-plus"
} else {
  $ModelEnabled -eq "true"
}
$result = [ordered]@{
  ok = $false
  launch_id = $launchId
  repo_root = $RepoRoot
  exe = $exe
  exe_exists = Test-Path -LiteralPath $exe
  exe_last_write_time = $null
  exe_size = $null
  exe_unlocked_before_launch = $false
  exe_unlocked = $false
  exe_unlock_wait_ms = $null
  exe_unlock_error = $null
  port = $Port
  port_released = $false
  provider = $Provider
  model = $Model
  model_enabled = [bool]$effectiveModelEnabled
  webview2_argument_mode = $WebView2ArgumentMode
  api_key_present = $false
  base_url_present = $false
  expected_target_url = $ExpectedTargetUrl
  cdp_args = New-WebView2BrowserArguments -Mode $WebView2ArgumentMode -LocalPort $Port
  launch_args = @("--hope-qa-cdp-port=$Port")
  webview2_user_data_folder = $null
  stdout_log = $null
  stderr_log = $null
  stdout_tail_sanitized = @()
  stderr_tail_sanitized = @()
  diagnostic_log = $diagnosticLog
  diagnostic_log_tail_sanitized = @()
  stopped_existing_pids = @()
  stop_existing_detail = $null
  stop_existing_complete = $null
  stop_existing_wait_ms = $null
  remaining_existing_pids = @()
  hope_app_remaining_pids = @()
  webview2_remaining_pids = @()
  webview2_query_error = $null
  webview2_processes_after_wait = @()
  webview2_remote_debugging_port_seen = $false
  webview2_expected_user_data_folder_seen = $false
  webview2_noerrdialogs_seen = $false
  webview2_disable_breakpad_seen = $false
  webview2_disable_crash_reporter_seen = $false
  stop_only = [bool]$StopOnly
  prelaunch_gate = $null
  events_since_script_start = @()
  failure_diagnostics = $null
  webview2_environment_not_created_or_crashed = $false
  started_pid = $null
  pid_file_started_pid = $null
  port_listening = $false
  port_owning_processes = @()
  port_states = @()
  cdp_ready = $false
  cdp_version_ready = $false
  cdp_browser = $null
  cdp_websocket_present = $false
  json_version_status = "not_checked"
  json_list_status = "not_checked"
  target_url = $null
  target_title = $null
  target_count = 0
  tauri_target_seen = $false
  last_target_url = $null
  last_target_title = $null
  last_targets = @()
  poll_attempts = 0
  elapsed_ms = $null
  process_running_after_wait = $false
  process_exit_code = $null
  process_path = $null
  stopped_started_pid_on_failure = $false
  stop_started_pid_on_failure_detail = $null
  error = $null
  last_version_error = $null
  last_list_error = $null
}

if (-not $result.exe_exists) {
  $result.error = "release exe not found"
  Convert-ResultJson $result
  exit 1
}

$exeItem = Get-Item -LiteralPath $exe
$result.exe_last_write_time = $exeItem.LastWriteTime.ToString("o")
$result.exe_size = $exeItem.Length

if ($StopExisting -or $StopOnly) {
  $existingPids = @(Get-HopeAppProcessIds)
  $result.stopped_existing_pids = @($existingPids)
  $stopDetail = Stop-HopeProcessTree -ProcessIds $existingPids -TimeoutSeconds $StopWaitSeconds -CdpPort $Port -ExpectedTargetUrl $ExpectedTargetUrl
  $result.stop_existing_detail = $stopDetail
  $result.stop_existing_complete = [bool]$stopDetail.stopped
  $result.stop_existing_wait_ms = $stopDetail.wait_ms
  $result.remaining_existing_pids = @($stopDetail.requested_pids | Where-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
  $result.hope_app_remaining_pids = @(Get-HopeAppProcessIds)
  $remainingWebView2 = @(Get-HopeWebView2Processes)
  $result.webview2_query_error = $script:LastHopeWebView2QueryError
  $result.webview2_remaining_pids = @($remainingWebView2 | ForEach-Object { $_.id })
  $portStateAfterStop = Get-PortState -LocalPort $Port
  $result.port_listening = [bool]$portStateAfterStop.listening
  $result.port_released = -not [bool]$portStateAfterStop.listening
  $result.port_owning_processes = $portStateAfterStop.owning_processes
  $result.port_states = $portStateAfterStop.states
  if (-not $stopDetail.stopped) {
    $result.error = "existing hope-app processes did not stop before timeout"
    $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
    Convert-ResultJson $result
    exit 4
  }

  if ($StopOnly) {
    $unlockState = Wait-FileUnlocked -Path $exe -TimeoutSeconds $FileUnlockWaitSeconds
    $result.exe_unlocked_before_launch = [bool]$unlockState.unlocked
    $result.exe_unlocked = [bool]$unlockState.unlocked
    $result.exe_unlock_wait_ms = $unlockState.wait_ms
    $result.exe_unlock_error = $unlockState.error
    $result.hope_app_remaining_pids = @(Get-HopeAppProcessIds)
    $remainingWebView2 = @(Get-HopeWebView2Processes)
    $result.webview2_query_error = $script:LastHopeWebView2QueryError
    $result.webview2_remaining_pids = @($remainingWebView2 | ForEach-Object { $_.id })
    $portStateAfterStopOnly = Get-PortState -LocalPort $Port
    $result.port_listening = [bool]$portStateAfterStopOnly.listening
    $result.port_released = -not [bool]$portStateAfterStopOnly.listening
    $result.port_owning_processes = $portStateAfterStopOnly.owning_processes
    $result.port_states = $portStateAfterStopOnly.states
    $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
    $result.ok = [bool]($unlockState.unlocked -and $result.hope_app_remaining_pids.Count -eq 0 -and $result.webview2_remaining_pids.Count -eq 0 -and $result.port_released)
    if (-not $unlockState.unlocked) {
      $result.error = "release exe is still locked after stop-only cleanup"
      Convert-ResultJson $result
      exit 5
    }
    if ($result.hope_app_remaining_pids.Count -gt 0 -or $result.webview2_remaining_pids.Count -gt 0 -or -not $result.port_released) {
      $result.error = "release shell cleanup is incomplete"
      Convert-ResultJson $result
      exit 4
    }
    Convert-ResultJson $result
    exit 0
  }
} else {
  $existingPids = @(Get-HopeAppProcessIds)
  if ($existingPids.Count -gt 0) {
    $result.error = "hope-app already running; rerun with -StopExisting or stop it before launch"
    $result.existing_pids = $existingPids
    $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
    Convert-ResultJson $result
    exit 2
  }
}

$unlockState = Wait-FileUnlocked -Path $exe -TimeoutSeconds $FileUnlockWaitSeconds
$result.exe_unlocked_before_launch = [bool]$unlockState.unlocked
$result.exe_unlocked = [bool]$unlockState.unlocked
$result.exe_unlock_wait_ms = $unlockState.wait_ms
$result.exe_unlock_error = $unlockState.error
if (-not $unlockState.unlocked) {
  $result.error = "release exe is still locked before launch"
  $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
  Convert-ResultJson $result
  exit 5
}

$result.webview2_user_data_folder = $profileDir
$prelaunchHopePids = @(Get-HopeAppProcessIds)
$prelaunchWebView2 = @(Get-HopeWebView2Processes)
$result.webview2_query_error = $script:LastHopeWebView2QueryError
$prelaunchPortState = Get-PortState -LocalPort $Port
$profileState = Test-DirectoryWritable -Path $profileDir
$profileOwnedByOldProcess = @(
  $prelaunchWebView2 |
    Where-Object { Test-PathAtOrUnder -Path $_.user_data_dir -Root $profileDir } |
    ForEach-Object { $_.id }
)
$result.prelaunch_gate = [ordered]@{
  hope_app_clear = $prelaunchHopePids.Count -eq 0
  hope_app_pids = $prelaunchHopePids
  webview2_clear = $prelaunchWebView2.Count -eq 0
  webview2_pids = @($prelaunchWebView2 | ForEach-Object { $_.id })
  webview2_query_error = $script:LastHopeWebView2QueryError
  port_released = -not [bool]$prelaunchPortState.listening
  port_owning_processes = $prelaunchPortState.owning_processes
  port_states = $prelaunchPortState.states
  exe_unlocked = [bool]$unlockState.unlocked
  user_data_folder = $profileDir
  user_data_folder_writable = [bool]$profileState.writable
  user_data_folder_error = $profileState.error
  user_data_folder_owned_by_old_process = $profileOwnedByOldProcess.Count -gt 0
  user_data_folder_old_process_pids = $profileOwnedByOldProcess
}
if (-not $result.prelaunch_gate.hope_app_clear -or -not $result.prelaunch_gate.webview2_clear -or -not $result.prelaunch_gate.port_released -or -not $result.prelaunch_gate.exe_unlocked -or -not $result.prelaunch_gate.user_data_folder_writable -or $result.prelaunch_gate.user_data_folder_owned_by_old_process) {
  $result.hope_app_remaining_pids = $prelaunchHopePids
  $result.webview2_remaining_pids = @($prelaunchWebView2 | ForEach-Object { $_.id })
  $result.port_listening = [bool]$prelaunchPortState.listening
  $result.port_released = -not [bool]$prelaunchPortState.listening
  $result.port_owning_processes = $prelaunchPortState.owning_processes
  $result.port_states = $prelaunchPortState.states
  $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
  $result.error = "prelaunch gate failed"
  Convert-ResultJson $result
  exit 6
}

$envMap = Get-EnvFileMap -Path $EnvFile
$apiKey = $envMap["HOPE_TEXT_MODEL_API_KEY"]
$envFileBaseUrl = $envMap["HOPE_TEXT_MODEL_BASE_URL"]
$effectiveBaseUrl = if (-not [string]::IsNullOrWhiteSpace($envFileBaseUrl)) {
  $envFileBaseUrl.Trim()
} elseif ($Provider -eq "qwen" -and -not [string]::IsNullOrWhiteSpace($BaseUrl)) {
  $BaseUrl.Trim()
} else {
  $null
}
$result.api_key_present = -not [string]::IsNullOrWhiteSpace($apiKey)
$result.base_url_present = -not [string]::IsNullOrWhiteSpace($effectiveBaseUrl)

New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
$result.webview2_user_data_folder = $profileDir
$pidFile = Join-Path $profileDir "hope-app.pid"
$stdoutLog = Join-Path $profileDir "hope-app.stdout.log"
$stderrLog = Join-Path $profileDir "hope-app.stderr.log"
$result.stdout_log = $stdoutLog
$result.stderr_log = $stderrLog
New-Item -ItemType File -Force -Path $stdoutLog | Out-Null
New-Item -ItemType File -Force -Path $stderrLog | Out-Null

try {
  $browserArgs = New-WebView2BrowserArguments -Mode $WebView2ArgumentMode -LocalPort $Port
  $result.cdp_args = $browserArgs
  $launchArgs = @(
    "--hope-qa-env-file=$EnvFile",
    "--hope-qa-provider=$Provider",
    "--hope-qa-model=$Model",
    "--hope-qa-model-enabled=$($effectiveModelEnabled.ToString().ToLowerInvariant())",
    "--hope-qa-launch-id=$launchId",
    "--hope-qa-webview2-user-data-folder=$profileDir",
    "--hope-qa-pid-file=$pidFile"
  )
  if (-not [string]::IsNullOrWhiteSpace($effectiveBaseUrl)) {
    $launchArgs = @($launchArgs + "--hope-qa-base-url=$effectiveBaseUrl")
  }
  if ($WebView2ArgumentMode -eq "AppDefault") {
    $launchArgs = @($launchArgs + "--hope-qa-cdp-port=$Port")
  } else {
    $previousBrowserArgs = [Environment]::GetEnvironmentVariable("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "Process")
    [Environment]::SetEnvironmentVariable("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", $browserArgs, "Process")
  }
  $result.launch_args = $launchArgs
  try {
    $appProc = Start-Process -FilePath $exe -ArgumentList $launchArgs -WorkingDirectory $workDir -PassThru
  } finally {
    if ($WebView2ArgumentMode -ne "AppDefault") {
      [Environment]::SetEnvironmentVariable("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", $previousBrowserArgs, "Process")
    }
  }
  if ($appProc -and $appProc.Id) {
    $result.started_pid = $appProc.Id
  }

  $pidDeadline = (Get-Date).AddSeconds(5)
  do {
    if (Test-Path -LiteralPath $pidFile) {
      $pidValue = (Get-Content -LiteralPath $pidFile -Raw).Trim()
      if ($pidValue -match '^\d+$') {
        $result.pid_file_started_pid = [int]$pidValue
        if (-not $result.started_pid) {
          $result.started_pid = [int]$pidValue
        }
        break
      }
    }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $pidDeadline)

  $startedAt = Get-Date
  $deadline = $startedAt.AddSeconds($WaitSeconds)
  do {
    Start-Sleep -Milliseconds 700
    $result.poll_attempts += 1

    $processState = Get-ProcessState -Id $result.started_pid
    $result.process_running_after_wait = [bool]$processState.running
    $result.process_exit_code = $processState.exit_code
    $result.process_path = $processState.path

    try {
      $version = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/version" -TimeoutSec 2
      $result.cdp_version_ready = $true
      $result.cdp_browser = $version.Browser
      $result.cdp_websocket_present = -not [string]::IsNullOrWhiteSpace($version.webSocketDebuggerUrl)
      $result.last_version_error = $null
      $result.json_version_status = "ok"
    } catch {
      $result.last_version_error = $_.Exception.Message
      $result.json_version_status = "error"
    }

    try {
      $targets = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/list" -TimeoutSec 2
      $result.json_list_status = "ok"
      $targetList = @($targets)
      $result.target_count = $targetList.Count
      $result.last_targets = Get-CdpTargetSummary -Targets $targetList
      $tauriTargets = @($targetList | Where-Object { $_.url -like "http://tauri.localhost/*" })
      if ($tauriTargets.Count -gt 0) {
        $result.tauri_target_seen = $true
        $result.last_target_url = $tauriTargets[0].url
        $result.last_target_title = $tauriTargets[0].title
      }

      $target = @($targetList | Where-Object { $_.url -eq $ExpectedTargetUrl } | Select-Object -First 1)
      if ($target.Count -gt 0) {
        $processState = Get-ProcessState -Id $result.started_pid
        if (-not $processState.running) {
          $result.error = "expected CDP target appeared, but hope-app process is not running"
          break
        }
        $result.ok = $true
        $result.cdp_ready = $true
        $result.target_url = $target[0].url
        $result.target_title = $target[0].title
        $result.process_running_after_wait = [bool]$processState.running
        $result.process_exit_code = $processState.exit_code
        $result.process_path = $processState.path
        $result.error = $null
        break
      }
      $result.last_list_error = $null
    } catch {
      $result.last_list_error = $_.Exception.Message
      $result.json_list_status = "error"
    }
  } while ((Get-Date) -lt $deadline)

  $result.elapsed_ms = [int]((Get-Date) - $startedAt).TotalMilliseconds
  $portState = Get-PortState -LocalPort $Port
  $result.port_listening = [bool]$portState.listening
  $result.port_released = -not [bool]$portState.listening
  $result.port_owning_processes = $portState.owning_processes
  $result.port_states = $portState.states
  $webview2AfterWait = @(Get-HopeWebView2Processes)
  $result.webview2_query_error = $script:LastHopeWebView2QueryError
  $result.webview2_processes_after_wait = $webview2AfterWait
  $webviewProfileCreated = Test-DirectoryHasChildren -Path (Join-Path $profileDir "EBWebView")
  $result.webview2_remote_debugging_port_seen = @($webview2AfterWait | Where-Object { $_.has_remote_debugging_port }).Count -gt 0
  $result.webview2_expected_user_data_folder_seen = @($webview2AfterWait | Where-Object { Test-PathAtOrUnder -Path $_.user_data_dir -Root $profileDir }).Count -gt 0
  $result.webview2_noerrdialogs_seen = @($webview2AfterWait | Where-Object { $_.has_noerrdialogs }).Count -gt 0
  $result.webview2_disable_breakpad_seen = @($webview2AfterWait | Where-Object { $_.has_disable_breakpad }).Count -gt 0
  $result.webview2_disable_crash_reporter_seen = @($webview2AfterWait | Where-Object { $_.has_disable_crash_reporter }).Count -gt 0
  $result.webview2_environment_not_created_or_crashed = [bool]((-not $result.cdp_ready) -and $webview2AfterWait.Count -eq 0 -and $webviewProfileCreated)
  $result.stdout_tail_sanitized = Get-SanitizedFileTail -Path $stdoutLog -TailLines 20
  $result.stderr_tail_sanitized = Get-SanitizedFileTail -Path $stderrLog -TailLines 20
  $result.diagnostic_log_tail_sanitized = Get-SanitizedFileTail -Path $diagnosticLog -TailLines $DiagnosticTailLines
  $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline

  if (-not $result.cdp_ready -and [string]::IsNullOrWhiteSpace($result.error)) {
    if ($result.cdp_version_ready -and $result.tauri_target_seen) {
      $result.error = "expected target url was not reached before timeout"
    } elseif ($result.cdp_version_ready) {
      $result.error = "CDP version endpoint was ready, but no tauri page target was found before timeout"
    } elseif (-not [string]::IsNullOrWhiteSpace($result.last_list_error)) {
      $result.error = $result.last_list_error
    } elseif (-not [string]::IsNullOrWhiteSpace($result.last_version_error)) {
      $result.error = $result.last_version_error
    } else {
      $result.error = "CDP target was not ready before timeout"
    }
  }

  if (-not $result.cdp_ready -and $StopOnCdpFailure -and -not $KeepOnCdpFailure -and $result.started_pid) {
    $processStateBeforeStop = Get-ProcessState -Id $result.started_pid
    $webview2BeforeFailureStop = @(Get-HopeWebView2Processes)
    $result.webview2_query_error = $script:LastHopeWebView2QueryError
    $portStateBeforeFailureStop = Get-PortState -LocalPort $Port
    $failureProfileCreated = Test-DirectoryHasChildren -Path (Join-Path $profileDir "EBWebView")
    $result.failure_diagnostics = [ordered]@{
      hope_app_pid = $result.started_pid
      hope_app_running = [bool]$processStateBeforeStop.running
      hope_app_exit_code = $processStateBeforeStop.exit_code
      hope_app_path = $processStateBeforeStop.path
      webview2_created = $webview2BeforeFailureStop.Count -gt 0
      webview2_profile_created = [bool]$failureProfileCreated
      webview2_environment_not_created_or_crashed = [bool]($webview2BeforeFailureStop.Count -eq 0 -and $failureProfileCreated)
      webview2_processes = $webview2BeforeFailureStop
      webview2_has_remote_debugging_port = @($webview2BeforeFailureStop | Where-Object { $_.has_remote_debugging_port }).Count -gt 0
      webview2_remote_debugging_ports = @($webview2BeforeFailureStop | ForEach-Object { $_.remote_debugging_ports } | Sort-Object -Unique)
      expected_user_data_folder = $profileDir
      webview2_expected_user_data_folder_seen = @($webview2BeforeFailureStop | Where-Object { Test-PathAtOrUnder -Path $_.user_data_dir -Root $profileDir }).Count -gt 0
      webview2_user_data_folders = @($webview2BeforeFailureStop | ForEach-Object { $_.user_data_dir } | Where-Object { $_ } | Sort-Object -Unique)
      port_listening = [bool]$portStateBeforeFailureStop.listening
      port_owning_processes = $portStateBeforeFailureStop.owning_processes
      port_states = $portStateBeforeFailureStop.states
      diagnostic_log_tail_sanitized = $result.diagnostic_log_tail_sanitized
      events_since_script_start = $result.events_since_script_start
    }
    $stopDetail = Stop-HopeProcessTree -ProcessIds @($result.started_pid) -TimeoutSeconds $StopWaitSeconds -CdpPort $Port -ExpectedTargetUrl $ExpectedTargetUrl
    $result.stop_started_pid_on_failure_detail = $stopDetail
    $result.stopped_started_pid_on_failure = $true
    $result.hope_app_remaining_pids = @(Get-HopeAppProcessIds)
    $remainingWebView2AfterFailureStop = @(Get-HopeWebView2Processes)
    $result.webview2_query_error = $script:LastHopeWebView2QueryError
    $result.webview2_remaining_pids = @($remainingWebView2AfterFailureStop | ForEach-Object { $_.id })
    $portStateAfterFailureStop = Get-PortState -LocalPort $Port
    $result.port_listening = [bool]$portStateAfterFailureStop.listening
    $result.port_released = -not [bool]$portStateAfterFailureStop.listening
    $result.port_owning_processes = $portStateAfterFailureStop.owning_processes
    $result.port_states = $portStateAfterFailureStop.states
  }
} catch {
  $result.error = $_.Exception.Message
  $portState = Get-PortState -LocalPort $Port
  $result.port_listening = [bool]$portState.listening
  $result.port_released = -not [bool]$portState.listening
  $result.port_owning_processes = $portState.owning_processes
  $result.port_states = $portState.states
  $webview2AfterCatch = @(Get-HopeWebView2Processes)
  $result.webview2_query_error = $script:LastHopeWebView2QueryError
  $result.webview2_processes_after_wait = $webview2AfterCatch
  $result.stdout_tail_sanitized = Get-SanitizedFileTail -Path $stdoutLog -TailLines 20
  $result.stderr_tail_sanitized = Get-SanitizedFileTail -Path $stderrLog -TailLines 20
  $result.diagnostic_log_tail_sanitized = Get-SanitizedFileTail -Path $diagnosticLog -TailLines $DiagnosticTailLines
  $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
}

Convert-ResultJson $result
if ($result.ok) {
  exit 0
}
exit 3
