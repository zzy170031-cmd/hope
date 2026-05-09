param(
  [string]$RepoRoot = "E:\codex\hope-desktop-shell",
  [int]$Port = 9224,
  [string]$EnvFile = "C:\Users\Administrator\.codex\.sandbox-secrets\hope-qwen.env",
  [string]$Provider = "qwen",
  [string]$Model = "qwen3.6-plus",
  [string]$ExpectedTargetUrl = "http://tauri.localhost/#/workbench",
  [int]$WaitSeconds = 12,
  [int]$StopWaitSeconds = 10,
  [int]$FileUnlockWaitSeconds = 10,
  [int]$DiagnosticTailLines = 80,
  [string]$BaseUrl = "https://dashscope.aliyuncs.com/compatible-mode/v1",
  [ValidateSet("auto", "true", "false")]
  [string]$ModelEnabled = "auto",
  [ValidateSet("Minimal", "MinimalNoErrDialogs", "MinimalDisableBreakpad", "MinimalDisableCrashReporter", "FullDefault", "AppDefault")]
  [string]$WebView2ArgumentMode = "AppDefault",
  [switch]$StopExisting,
  [switch]$StopOnly,
  [switch]$StopOnCdpFailure,
  [switch]$KeepOnCdpFailure,
  [switch]$LaunchDiagnosticOnly,
  [switch]$NoProxy,
  [switch]$QaProviderHardFail
)

$ErrorActionPreference = "Stop"
$script:LastHopeWebView2QueryError = $null
$script:ApplicationEventBaseline = Get-Date
$script:HopeWebView2HardenedBrowserArgs = "--no-first-run --no-default-browser-check --disable-background-networking --disable-component-update --disable-domain-reliability --disable-sync --disable-gpu --disable-gpu-compositing --disable-client-side-phishing-detection --disable-component-extensions-with-background-pages --metrics-recording-only --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,OptimizationHints,AutofillServerCommunication,CertificateTransparencyComponentUpdater"
$AllowedQwenTextModels = @(
  "qwen3.6-plus",
  "qwen3.6-plus-2026-04-02"
)

function Convert-ResultJson {
  param([hashtable]$Data)
  [pscustomobject]$Data | ConvertTo-Json -Depth 6
}

function Write-JsonArtifact {
  param(
    $Value,
    [string]$Path,
    [int]$Depth = 10
  )
  $parent = Split-Path -Parent $Path
  if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
  $Value | ConvertTo-Json -Depth $Depth | Set-Content -LiteralPath $Path -Encoding UTF8
  return $Path
}

function Get-ComparablePath {
  param([string]$Path)
  if ([string]::IsNullOrWhiteSpace($Path)) {
    return $null
  }
  try {
    return ([System.IO.Path]::GetFullPath($Path).TrimEnd('\')).ToLowerInvariant()
  } catch {
    return ($Path.TrimEnd('\')).ToLowerInvariant()
  }
}

function Get-FileFingerprint {
  param([string]$Path)
  if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path)) {
    return [ordered]@{
      path = $Path
      exists = $false
      size = $null
      last_write_time = $null
      sha256 = $null
      error = $null
      captured_at = (Get-Date).ToString("o")
    }
  }

  try {
    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256
    return [ordered]@{
      path = $item.FullName
      exists = $true
      size = $item.Length
      last_write_time = $item.LastWriteTime.ToString("o")
      sha256 = $hash.Hash
      error = $null
      captured_at = (Get-Date).ToString("o")
    }
  } catch {
    return [ordered]@{
      path = $Path
      exists = Test-Path -LiteralPath $Path
      size = $null
      last_write_time = $null
      sha256 = $null
      error = $_.Exception.Message
      captured_at = (Get-Date).ToString("o")
    }
  }
}

function Update-ProcessImageProvenance {
  param(
    [System.Collections.IDictionary]$Result,
    [string]$ExpectedExe
  )

  $expectedFingerprint = if ($Result.Contains("exe_fingerprint") -and $null -ne $Result["exe_fingerprint"]) {
    $Result["exe_fingerprint"]
  } else {
    Get-FileFingerprint -Path $ExpectedExe
  }
  $state = Get-ProcessState -Id $Result.started_pid
  $processPath = if (-not [string]::IsNullOrWhiteSpace($state.path)) { $state.path } else { $Result.process_path }
  $processFingerprint = if (-not [string]::IsNullOrWhiteSpace($processPath)) {
    Get-FileFingerprint -Path $processPath
  } else {
    [ordered]@{
      path = $processPath
      exists = $false
      size = $null
      last_write_time = $null
      sha256 = $null
      error = "process path unavailable"
      captured_at = (Get-Date).ToString("o")
    }
  }

  $expectedComparable = Get-ComparablePath -Path $ExpectedExe
  $processComparable = Get-ComparablePath -Path $processPath
  $pathMatches = [bool](
    -not [string]::IsNullOrWhiteSpace($expectedComparable) -and
    $expectedComparable -eq $processComparable
  )
  $fingerprintMatches = [bool](
    $pathMatches -and
    [bool]$expectedFingerprint.exists -and
    [bool]$processFingerprint.exists -and
    $expectedFingerprint.size -eq $processFingerprint.size -and
    $expectedFingerprint.last_write_time -eq $processFingerprint.last_write_time -and
    -not [string]::IsNullOrWhiteSpace([string]$expectedFingerprint.sha256) -and
    $expectedFingerprint.sha256 -eq $processFingerprint.sha256
  )

  if (-not [string]::IsNullOrWhiteSpace($state.path)) {
    $Result.process_path = $state.path
  }
  $Result.process_path_matches_expected_exe = $pathMatches
  $Result.process_image_matches_expected_exe = $fingerprintMatches
  $Result.current_exe_provenance_status = if ($fingerprintMatches) {
    "process_path_hash_mtime_size_match"
  } elseif ($pathMatches) {
    "process_path_match_fingerprint_gap"
  } else {
    "process_path_mismatch_or_unavailable"
  }
  $Result.current_exe_provenance = [ordered]@{
    expected_exe = $ExpectedExe
    expected_exe_fingerprint = $expectedFingerprint
    process_path = $processPath
    process_running = [bool]$state.running
    process_exit_code = $state.exit_code
    process_path_matches_expected_exe = $pathMatches
    process_image_fingerprint = $processFingerprint
    process_image_matches_expected_exe = $fingerprintMatches
    status = $Result.current_exe_provenance_status
    captured_at = (Get-Date).ToString("o")
  }
}

function Write-StopOnlyCleanupArtifact {
  param([System.Collections.IDictionary]$Result)
  if (-not [bool]$Result.stop_only) {
    return $null
  }
  $Result.cleanup_artifact_written_at = (Get-Date).ToString("o")
  $Result.cleanup_artifact_written = $true
  Write-JsonArtifact -Value ([pscustomobject]$Result) -Path $Result.cleanup_artifact_path -Depth 16 | Out-Null
  return $Result.cleanup_artifact_path
}

function Add-ResultArrayValue {
  param(
    [System.Collections.IDictionary]$Result,
    [string]$Name,
    $Value
  )
  $current = @()
  if ($Result.Contains($Name) -and $null -ne $Result[$Name]) {
    $current = @($Result[$Name])
  }
  $Result[$Name] = @($current + $Value)
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

function Set-ProcessEnvValue {
  param(
    [string]$Name,
    [AllowNull()][string]$Value
  )
  [Environment]::SetEnvironmentVariable($Name, $Value, "Process")
  if ($null -eq $Value) {
    Remove-Item -LiteralPath ("Env:{0}" -f $Name) -ErrorAction SilentlyContinue
  } else {
    Set-Item -LiteralPath ("Env:{0}" -f $Name) -Value $Value
  }
}

function Test-ProcessEnvPresent {
  param([string]$Name)
  -not [string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($Name, "Process"))
}

function Get-ProxyEnvPresence {
  $httpProxyPresent = Test-ProcessEnvPresent -Name "HTTP_PROXY"
  $httpsProxyPresent = Test-ProcessEnvPresent -Name "HTTPS_PROXY"
  $allProxyPresent = Test-ProcessEnvPresent -Name "ALL_PROXY"
  $noProxyPresent = Test-ProcessEnvPresent -Name "NO_PROXY"
  $lowerHttpProxyPresent = Test-ProcessEnvPresent -Name "http_proxy"
  $lowerHttpsProxyPresent = Test-ProcessEnvPresent -Name "https_proxy"
  $lowerAllProxyPresent = Test-ProcessEnvPresent -Name "all_proxy"
  $lowerNoProxyPresent = Test-ProcessEnvPresent -Name "no_proxy"
  $uppercaseProxyPresent = [bool]($httpProxyPresent -or $httpsProxyPresent -or $allProxyPresent)
  $lowercaseProxyPresent = [bool]($lowerHttpProxyPresent -or $lowerHttpsProxyPresent -or $lowerAllProxyPresent)
  [ordered]@{
    http_proxy_present = [bool]($httpProxyPresent -or $lowerHttpProxyPresent)
    https_proxy_present = [bool]($httpsProxyPresent -or $lowerHttpsProxyPresent)
    all_proxy_present = [bool]($allProxyPresent -or $lowerAllProxyPresent)
    no_proxy_present = [bool]($noProxyPresent -or $lowerNoProxyPresent)
    uppercase_proxy_present = $uppercaseProxyPresent
    lowercase_proxy_present = $lowercaseProxyPresent
    process_env_proxy_present = [bool]($uppercaseProxyPresent -or $lowercaseProxyPresent)
    raw_values_redacted = $true
  }
}

function Test-NoProxyLoopbackOnly {
  $values = @(
    [Environment]::GetEnvironmentVariable("NO_PROXY", "Process"),
    [Environment]::GetEnvironmentVariable("no_proxy", "Process")
  ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
  if ($values.Count -eq 0) {
    return $true
  }
  $allowed = @("localhost", "127.0.0.1", "::1")
  $entries = @(
    $values |
      ForEach-Object { $_.Split(",") } |
      ForEach-Object { $_.Trim().ToLowerInvariant() } |
      Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
  )
  if ($entries.Count -eq 0) {
    return $true
  }
  @($entries | Where-Object { $allowed -notcontains $_ }).Count -eq 0
}

function Clear-QaProxyEnv {
  foreach ($name in @("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY", "http_proxy", "https_proxy", "all_proxy", "no_proxy")) {
    [Environment]::SetEnvironmentVariable($name, $null, "Process")
    Remove-Item -LiteralPath ("Env:{0}" -f $name) -ErrorAction SilentlyContinue
  }
}

function Get-SanitizedLaunchArgs {
  param([string[]]$ArgList)
  @(
    foreach ($arg in $ArgList) {
      if ($arg -like "--hope-qa-env-file=*") {
        "--hope-qa-env-file=<redacted-path>"
      } elseif ($arg -like "--hope-qa-base-url=*") {
        "--hope-qa-base-url=<redacted-url-present>"
      } elseif ($arg -like "--hope-qa-webview2-user-data-folder=*") {
        "--hope-qa-webview2-user-data-folder=<redacted-path>"
      } elseif ($arg -like "--hope-qa-pid-file=*") {
        "--hope-qa-pid-file=<redacted-path>"
      } else {
        $arg
      }
    }
  )
}

function New-WebView2BrowserArguments {
  param(
    [string]$Mode,
    [int]$LocalPort
  )
  $remoteDebugging = "--remote-debugging-port=$LocalPort"
  $hardened = "$script:HopeWebView2HardenedBrowserArgs $remoteDebugging"
  switch ($Mode) {
    "Minimal" { return $remoteDebugging }
    "MinimalNoErrDialogs" { return "$remoteDebugging --noerrdialogs" }
    "MinimalDisableBreakpad" { return "$remoteDebugging --disable-breakpad" }
    "MinimalDisableCrashReporter" { return "$remoteDebugging --disable-crash-reporter" }
    "FullDefault" { return "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --noerrdialogs --disable-crash-reporter --disable-breakpad $remoteDebugging" }
    "AppDefault" { return $hardened }
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

function Get-SanitizedCdpVersionArtifactValue {
  param($Version)
  if ($null -eq $Version) {
    return $null
  }
  [ordered]@{
    Browser = $Version.Browser
    Protocol_Version = $Version.'Protocol-Version'
    webSocketDebuggerUrl_present = -not [string]::IsNullOrWhiteSpace($Version.webSocketDebuggerUrl)
    userAgent_present = -not [string]::IsNullOrWhiteSpace($Version.'User-Agent')
    raw_field_names = @($Version.PSObject.Properties.Name)
  }
}

function Get-SanitizedCdpListArtifactValue {
  param($Targets)
  if ($null -eq $Targets) {
    return @()
  }
  @(
    @($Targets) | ForEach-Object {
      [ordered]@{
        id = $_.id
        type = $_.type
        title = $_.title
        url = $_.url
        description = $_.description
        webSocketDebuggerUrl_present = -not [string]::IsNullOrWhiteSpace($_.webSocketDebuggerUrl)
        devtoolsFrontendUrl_present = -not [string]::IsNullOrWhiteSpace($_.devtoolsFrontendUrl)
        raw_field_names = @($_.PSObject.Properties.Name)
      }
    }
  )
}

function Write-CdpJsonArtifact {
  param(
    [string]$Path,
    [string]$ArtifactClass,
    [string]$Endpoint,
    [string]$Status,
    $RawSanitized,
    [AllowNull()][string]$ErrorMessage,
    [int]$PollAttempt
  )
  $artifact = [ordered]@{
    artifact_class = $ArtifactClass
    captured_at = (Get-Date).ToString("o")
    poll_attempt = $PollAttempt
    endpoint = $Endpoint
    status = $Status
    error = $ErrorMessage
    raw_sanitized = $RawSanitized
  }
  Write-JsonArtifact -Value $artifact -Path $Path -Depth 12 | Out-Null
  return $Path
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

function Get-HopeProcessArgValue {
  param(
    [string]$CommandLine,
    [string]$ArgPrefix
  )
  if ([string]::IsNullOrWhiteSpace($CommandLine) -or [string]::IsNullOrWhiteSpace($ArgPrefix)) {
    return $null
  }
  $escapedPrefix = [regex]::Escape($ArgPrefix)
  $quotedPattern = $escapedPrefix + '"([^"]+)"'
  $barePattern = $escapedPrefix + '([^\s]+)'
  if ($CommandLine -match $quotedPattern) {
    return $matches[1]
  }
  if ($CommandLine -match $barePattern) {
    return $matches[1]
  }
  return $null
}

function Get-HopeAppProcesses {
  try {
    @(
      Get-CimInstance Win32_Process -Filter "name='hope-app.exe'" -ErrorAction Stop |
        ForEach-Object {
          $commandLine = [string]$_.CommandLine
          [ordered]@{
            id = [int]$_.ProcessId
            parent_id = [int]$_.ParentProcessId
            path = $_.ExecutablePath
            command_line_present = -not [string]::IsNullOrWhiteSpace($commandLine)
            pid_file = Get-HopeProcessArgValue -CommandLine $commandLine -ArgPrefix '--hope-qa-pid-file='
            user_data_dir = Get-HopeProcessArgValue -CommandLine $commandLine -ArgPrefix '--hope-qa-webview2-user-data-folder='
            shutdown_signal_file = Get-HopeProcessArgValue -CommandLine $commandLine -ArgPrefix '--hope-qa-shutdown-signal-file='
          }
        }
    )
  } catch {
    @()
  }
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
          $_.Message -match "msedgewebview2|hope-app|WebView2|127\.0\.0\.1:5173|tauri\.localhost"
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
            has_no_first_run = $commandLine.Contains("--no-first-run")
            has_no_default_browser_check = $commandLine.Contains("--no-default-browser-check")
            has_remote_allow_tauri_localhost = $commandLine.Contains("--remote-allow-origins=http://tauri.localhost")
            has_disable_web_ooui_features = $commandLine.Contains("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection")
            has_disable_background_networking = $commandLine.Contains("--disable-background-networking")
            has_disable_component_update = $commandLine.Contains("--disable-component-update")
            has_disable_gpu = $commandLine.Contains("--disable-gpu")
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
        $_.url -eq $ExpectedUrl -or
        $_.url -like "http://tauri.localhost/*" -or
        $_.url -like "http://127.0.0.1:5173/*"
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
    graceful_signal_paths = @()
    graceful_signal_errors = @()
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
  $hostProcesses = @(Get-HopeAppProcesses)
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

  foreach ($procInfo in @($hostProcesses | Where-Object { $uniquePids -contains $_.id })) {
    if ([string]::IsNullOrWhiteSpace($procInfo.shutdown_signal_file)) {
      continue
    }
    try {
      $shutdownParent = Split-Path -Parent $procInfo.shutdown_signal_file
      if (-not [string]::IsNullOrWhiteSpace($shutdownParent)) {
        New-Item -ItemType Directory -Force -Path $shutdownParent | Out-Null
      }
      Set-Content -LiteralPath $procInfo.shutdown_signal_file -Value "shutdown" -NoNewline -Encoding ASCII
      $detail.graceful_signal_paths += $procInfo.shutdown_signal_file
    } catch {
      $detail.graceful_signal_errors += [ordered]@{
        pid = $procInfo.id
        path = $procInfo.shutdown_signal_file
        error = $_.Exception.Message
      }
    }
  }

  if ($detail.graceful_signal_paths.Count -gt 0) {
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

  $graceSeconds = [Math]::Max([Math]::Min($TimeoutSeconds - 2, 8), 3)
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

function Get-SanitizedTextLine {
  param([string]$Line)
  ($Line -replace '(?i)(api[_-]?key|token|secret|authorization)\s*=\s*[^,\s]+', '$1=<redacted>') `
    -replace '(?i)(api[_-]?key|token|secret|authorization)["'']?\s*:\s*["''][^"'']+["'']', '$1:<redacted>'
}

function Get-SanitizedFileTail {
  param(
    [string]$Path,
    [int]$TailLines
  )
  if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path)) {
    return @()
  }

  @(
    Get-Content -LiteralPath $Path -Tail $TailLines -ErrorAction SilentlyContinue |
      ForEach-Object { Get-SanitizedTextLine -Line $_ }
  )
}

function Get-SanitizedFileLines {
  param(
    [string]$Path,
    [int]$MaxLines = 400
  )
  if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path)) {
    return @()
  }

  @(
    Get-Content -LiteralPath $Path -ErrorAction SilentlyContinue |
      Select-Object -First $MaxLines |
      ForEach-Object { Get-SanitizedTextLine -Line $_ }
  )
}

function Get-LaunchDiagnosticEvidence {
  param(
    [string]$Path,
    [string]$LaunchId
  )

  if ([string]::IsNullOrWhiteSpace($Path)) {
    return [ordered]@{
      diagnostic_log_exists = $false
      diagnostic_log_path = $Path
      line_count = 0
      launch_id_anchor_seen = $false
      process_start_seen = $false
      before_configure_seen = $false
      after_configure_seen = $false
      before_builder_run_seen = $false
      setup_enter_seen = $false
      before_main_window_get_seen = $false
      before_main_window_build_seen = $false
      main_window_created_seen = $false
      main_window_found_seen = $false
      setup_exit_seen = $false
      post_setup_window_probe_seen = $false
      post_setup_window_probe_lines = @()
      builder_run_returned_ok_seen = $false
      builder_run_error_line = $null
      main_window_error_line = $null
      main_window_build_error_line = $null
      panic_line = $null
      webview2_additional_browser_args_line = $null
      webview2_user_data_folder_line = $null
      diagnostic_log_path_line = $null
      app_process_env_proxy_present = $null
      app_process_env_proxy_present_line = $null
      app_process_no_proxy_loopback_only = $null
      app_process_no_proxy_loopback_only_line = $null
      last_line = $null
      lines_tail = @()
    }
  }

  $lines = @(Get-SanitizedFileLines -Path $Path -MaxLines 800)
  $launchAnchor = if ([string]::IsNullOrWhiteSpace($LaunchId)) { $null } else { "launch_id=$LaunchId" }

  [ordered]@{
    diagnostic_log_exists = Test-Path -LiteralPath $Path
    diagnostic_log_path = $Path
    line_count = $lines.Count
    launch_id_anchor_seen = [bool]($launchAnchor -and @($lines | Where-Object { $_ -like "*$launchAnchor*" }).Count -gt 0)
    process_start_seen = @($lines | Where-Object { $_ -like "*process_start*" }).Count -gt 0
    before_configure_seen = @($lines | Where-Object { $_ -like "*before_configure_qa_webview2_from_args*" }).Count -gt 0
    after_configure_seen = @($lines | Where-Object { $_ -like "*after_configure_qa_webview2_from_args*" }).Count -gt 0
    before_builder_run_seen = @($lines | Where-Object { $_ -like "*before_builder_run*" }).Count -gt 0
    setup_enter_seen = @($lines | Where-Object { $_ -like "*setup_enter*" }).Count -gt 0
    before_main_window_get_seen = @($lines | Where-Object { $_ -like "*before_main_window_get*" }).Count -gt 0
    before_main_window_build_seen = @($lines | Where-Object { $_ -like "*before_main_window_build*" }).Count -gt 0
    main_window_created_seen = @($lines | Where-Object { $_ -like "*main_window=created_in_setup*" }).Count -gt 0
    main_window_found_seen = @($lines | Where-Object { $_ -like "*main_window=found*" }).Count -gt 0
    setup_exit_seen = @($lines | Where-Object { $_ -like "*setup_exit*" }).Count -gt 0
    post_setup_window_probe_seen = @($lines | Where-Object { $_ -like "*post_setup_window_probe*" }).Count -gt 0
    post_setup_window_probe_lines = @($lines | Where-Object { $_ -like "*post_setup_window_probe*" })
    builder_run_returned_ok_seen = @($lines | Where-Object { $_ -like "*builder_run_returned_ok*" }).Count -gt 0
    builder_run_error_line = @($lines | Where-Object { $_ -like "*builder_run_error=*" } | Select-Object -First 1)
    main_window_error_line = @($lines | Where-Object { $_ -like "*main_window_error=*" } | Select-Object -First 1)
    main_window_build_error_line = @($lines | Where-Object { $_ -like "*main_window_build_error=*" } | Select-Object -First 1)
    panic_line = @($lines | Where-Object { $_ -like "*panic=*" } | Select-Object -First 1)
    webview2_additional_browser_args_line = @($lines | Where-Object { $_ -like "*webview2_additional_browser_args=*" -or $_ -like "*webview2_additional_browser_args_runtime=*" } | Select-Object -Last 1)
    webview2_user_data_folder_line = @($lines | Where-Object { $_ -like "*webview2_user_data_folder=*" -or $_ -like "*webview2_user_data_folder_runtime=*" } | Select-Object -Last 1)
    diagnostic_log_path_line = @($lines | Where-Object { $_ -like "*diagnostic_log_path=*" } | Select-Object -First 1)
    app_process_env_proxy_present = @($lines | Where-Object { $_ -like "*app_process_env_proxy_present=true*" }).Count -gt 0
    app_process_env_proxy_present_line = @($lines | Where-Object { $_ -like "*app_process_env_proxy_present=*" } | Select-Object -Last 1)
    app_process_no_proxy_loopback_only = @($lines | Where-Object { $_ -like "*app_process_no_proxy_loopback_only=true*" }).Count -gt 0
    app_process_no_proxy_loopback_only_line = @($lines | Where-Object { $_ -like "*app_process_no_proxy_loopback_only=*" } | Select-Object -Last 1)
    last_line = if ($lines.Count -gt 0) { $lines[-1] } else { $null }
    lines_tail = @($lines | Select-Object -Last 20)
  }
}

function Get-WebView2ProfileStageEvidence {
  param([string]$ProfileDir)

  $profileExists = Test-Path -LiteralPath $ProfileDir
  $ebWebViewDir = Join-Path $ProfileDir "EBWebView"
  $ebExists = Test-Path -LiteralPath $ebWebViewDir
  $localStatePath = Join-Path $ebWebViewDir "Local State"
  $lastVersionPath = Join-Path $ebWebViewDir "Last Version"
  $defaultDir = Join-Path $ebWebViewDir "Default"
  $crashpadDir = Join-Path $ebWebViewDir "Crashpad"
  $edgeLocalStateTmp = @()
  $lockEntries = @()
  if ($ebExists) {
    $edgeLocalStateTmp = @(
      Get-ChildItem -LiteralPath $ebWebViewDir -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "Edge-Local-State-Tmp*" } |
        ForEach-Object { $_.Name }
    )
    $lockEntries = @(
      Get-ChildItem -LiteralPath $ebWebViewDir -Recurse -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match 'LOCK|Singleton|Crashpad|\.tmp$' } |
        Select-Object -First 40 |
        ForEach-Object {
          [ordered]@{
            name = $_.Name
            relative_path = $_.FullName.Replace($ProfileDir, "<profile>")
          }
        }
    )
  }

  [ordered]@{
    profile_dir = $ProfileDir
    profile_dir_exists = $profileExists
    ebwebview_dir = $ebWebViewDir
    ebwebview_exists = $ebExists
    root_entries = if ($profileExists) {
      @(
        Get-ChildItem -LiteralPath $ProfileDir -Force -ErrorAction SilentlyContinue |
          Select-Object -First 20 |
          ForEach-Object { $_.Name }
      )
    } else {
      @()
    }
    eb_entries = if ($ebExists) {
      @(
        Get-ChildItem -LiteralPath $ebWebViewDir -Force -ErrorAction SilentlyContinue |
          Select-Object -First 30 |
          ForEach-Object { $_.Name }
      )
    } else {
      @()
    }
    has_default = Test-Path -LiteralPath $defaultDir
    has_local_state = Test-Path -LiteralPath $localStatePath
    local_state_length = if (Test-Path -LiteralPath $localStatePath) { (Get-Item -LiteralPath $localStatePath).Length } else { $null }
    has_last_version = Test-Path -LiteralPath $lastVersionPath
    last_version_text = if (Test-Path -LiteralPath $lastVersionPath) { (Get-Content -LiteralPath $lastVersionPath -Raw -ErrorAction SilentlyContinue).Trim() } else { $null }
    has_crashpad = Test-Path -LiteralPath $crashpadDir
    edge_local_state_tmp = $edgeLocalStateTmp
    lock_like_entries = $lockEntries
    lock_like_count = $lockEntries.Count
  }
}

function Get-WebView2ProcessQueryCapability {
  try {
    $processes = @(Get-CimInstance Win32_Process -Filter "name='msedgewebview2.exe'" -ErrorAction Stop)
    $hopeProcesses = @(
      $processes |
        Where-Object {
          $_.CommandLine -like "*--webview-exe-name=hope-app.exe*" -or
          $_.CommandLine -like "*\hope-webview2-cdp\*"
        }
    )
    $nonHopeProcesses = @(
      $processes |
        Where-Object {
          -not (
            $_.CommandLine -like "*--webview-exe-name=hope-app.exe*" -or
            $_.CommandLine -like "*\hope-webview2-cdp\*"
          )
        }
    )
    $nonHopeOwners = @(
      $nonHopeProcesses |
        ForEach-Object {
          $commandLine = [string]$_.CommandLine
          if ($commandLine -match '--webview-exe-name=([^\s"]+)') {
            $matches[1]
          } else {
            "unknown"
          }
        } |
        Sort-Object -Unique
    )
    return [ordered]@{
      query_ok = $true
      query_error = $null
      process_count = $processes.Count
      hope_process_count = $hopeProcesses.Count
      non_hope_process_count = $nonHopeProcesses.Count
      non_hope_webview_exe_names = $nonHopeOwners
      scope = "hope-app-and-hope-webview2-profile-only"
    }
  } catch {
    return [ordered]@{
      query_ok = $false
      query_error = $_.Exception.Message
      process_count = $null
      hope_process_count = $null
      non_hope_process_count = $null
      non_hope_webview_exe_names = @()
      scope = "query-failed"
    }
  }
}

function Get-EventSummaryFromProvider {
  param(
    [string]$Provider,
    [datetime]$Since,
    [int]$Limit = 10
  )

  try {
    $events = @(
      Get-WinEvent -FilterHashtable @{ LogName = "Application"; ProviderName = $Provider; StartTime = $Since } -ErrorAction Stop |
        Sort-Object TimeCreated -Descending |
        Select-Object -First $Limit
    )
    return @(
      $events | ForEach-Object {
        [ordered]@{
          time = $_.TimeCreated.ToString("o")
          provider = $_.ProviderName
          id = $_.Id
          level = $_.LevelDisplayName
          summary = (($_.Message -split "`r?`n" | Where-Object { $_.Trim().Length -gt 0 } | Select-Object -First 3) -join " | ")
        }
      }
    )
  } catch {
    if ($_.Exception.Message -like "*No events were found*" -or $_.Exception.Message -like "*do not write events*") {
      return @()
    }
    return @(
      [ordered]@{
        time = (Get-Date).ToString("o")
        provider = $Provider
        id = $null
        level = "Error"
        summary = $_.Exception.Message
      }
    )
  }
}

function Get-LaunchEventLogEvidence {
  param([datetime]$Since)

  $messageMatches = @()
  try {
    $messageMatches = @(
      Get-WinEvent -FilterHashtable @{ LogName = "Application"; StartTime = $Since } -ErrorAction Stop |
      Where-Object { $_.Message -match "hope-app|msedgewebview2|WebView2|127\.0\.0\.1:5173|tauri\.localhost" } |
        Sort-Object TimeCreated -Descending |
        Select-Object -First 20 |
        ForEach-Object {
          [ordered]@{
            time = $_.TimeCreated.ToString("o")
            provider = $_.ProviderName
            id = $_.Id
            level = $_.LevelDisplayName
            summary = (($_.Message -split "`r?`n" | Where-Object { $_.Trim().Length -gt 0 } | Select-Object -First 3) -join " | ")
          }
        }
    )
  } catch {
    $messageMatches = @(
      [ordered]@{
        time = (Get-Date).ToString("o")
        provider = "message_match"
        id = $null
        level = "Error"
        summary = $_.Exception.Message
      }
    )
  }

  [ordered]@{
    application_error = @(Get-EventSummaryFromProvider -Provider "Application Error" -Since $Since)
    application_popup = @(Get-EventSummaryFromProvider -Provider "Application Popup" -Since $Since)
    windows_error_reporting = @(Get-EventSummaryFromProvider -Provider "Windows Error Reporting" -Since $Since)
    dotnet_runtime = @(Get-EventSummaryFromProvider -Provider ".NET Runtime" -Since $Since)
    message_match = $messageMatches
  }
}

function Get-WebView2RuntimeStatus {
  $registryEntries = @()
  foreach ($root in @(
      "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients",
      "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients",
      "HKCU:\Software\Microsoft\EdgeUpdate\Clients"
    )) {
    if (-not (Test-Path -LiteralPath $root)) {
      continue
    }
    foreach ($key in @(Get-ChildItem -LiteralPath $root -ErrorAction SilentlyContinue)) {
      $props = Get-ItemProperty -LiteralPath $key.PSPath -ErrorAction SilentlyContinue
      if (-not $props) {
        continue
      }
      $name = [string]$props.name
      if ($name -match "WebView2|Edge WebView2|Microsoft Edge WebView2 Runtime") {
        $registryEntries += [ordered]@{
          root = $root
          key = $key.PSChildName
          name = $name
          version = $props.pv
          location = $props.location
        }
      }
    }
  }

  $installEntries = @()
  foreach ($path in @(
      "C:\Program Files (x86)\Microsoft\EdgeWebView\Application",
      "C:\Program Files\Microsoft\EdgeWebView\Application",
      (Join-Path $env:LOCALAPPDATA "Microsoft\EdgeWebView\Application")
    )) {
    if (-not (Test-Path -LiteralPath $path)) {
      continue
    }
    foreach ($dir in @(Get-ChildItem -LiteralPath $path -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 5)) {
      $exePath = Join-Path $dir.FullName "msedgewebview2.exe"
      $installEntries += [ordered]@{
        root = $path
        version_dir = $dir.Name
        exe_exists = Test-Path -LiteralPath $exePath
        exe_version = if (Test-Path -LiteralPath $exePath) { (Get-Item -LiteralPath $exePath).VersionInfo.FileVersion } else { $null }
      }
    }
  }

  [ordered]@{
    registry_entries = $registryEntries
    install_entries = $installEntries
  }
}

function Get-TargetMaterializationStage {
  param(
    [System.Collections.IDictionary]$Result,
    [bool]$SetupExitSeen
  )
  if ([bool]$Result.cdp_ready) {
    return "expected-target-ready"
  }
  if ($Result.json_list_status -eq "ok" -and [int]$Result.target_count -gt 0) {
    if ([string]::IsNullOrWhiteSpace([string]$Result.last_target_url)) {
      return "cdp-list-target-without-url"
    }
    return "cdp-list-target-url-mismatch"
  }
  if ($Result.json_list_status -eq "ok" -and [int]$Result.target_count -eq 0) {
    return "cdp-list-zero-target"
  }
  if ($Result.cdp_version_ready) {
    return "cdp-version-ready-list-not-ready"
  }
  if ([bool]$Result.port_listening) {
    return "port-listening-cdp-not-ready"
  }
  if ($SetupExitSeen) {
    return "post-setup-no-cdp"
  }
  return "pre-setup-or-window-build"
}

function Save-PostSetupProbe {
  param(
    [System.Collections.IDictionary]$Result,
    [string]$Path,
    [string]$ProfileDir,
    [string]$DiagnosticLog,
    [int]$Port,
    [string]$ExpectedTargetUrl,
    [Nullable[int]]$StartedPid,
    [int]$PollAttempt
  )

  $diagnosticEvidence = Get-LaunchDiagnosticEvidence -Path $DiagnosticLog -LaunchId $Result.launch_id
  $portState = Get-PortState -LocalPort $Port
  $processState = Get-ProcessState -Id $StartedPid
  $appProcess = if ($StartedPid) { Get-Process -Id $StartedPid -ErrorAction SilentlyContinue } else { $null }
  $webView2Processes = @(Get-HopeWebView2Processes)
  $webView2QueryError = $script:LastHopeWebView2QueryError
  $profileEvidence = Get-WebView2ProfileStageEvidence -ProfileDir $ProfileDir
  $expectedProfileProcesses = @(
    $webView2Processes | Where-Object { Test-PathAtOrUnder -Path $_.user_data_dir -Root $ProfileDir }
  )
  $mainWindowPresent = [bool]($appProcess -and $appProcess.MainWindowHandle -ne 0)
  $setupExitSeen = [bool]$diagnosticEvidence.setup_exit_seen
  $materializationStage = Get-TargetMaterializationStage -Result $Result -SetupExitSeen $setupExitSeen
  $profileUsed = [bool](
    $profileEvidence.ebwebview_exists -or
    @($expectedProfileProcesses).Count -gt 0 -or
    @($diagnosticEvidence.webview2_user_data_folder_line).Count -gt 0
  )

  $probe = [ordered]@{
    artifact_class = "hope-post-setup-probe"
    captured_at = (Get-Date).ToString("o")
    launch_id = $Result.launch_id
    poll_attempt = $PollAttempt
    expected_target_url = $ExpectedTargetUrl
    process_running = [bool]$processState.running
    process_exit_code = $processState.exit_code
    main_window_present = $mainWindowPresent
    main_window_title = if ($appProcess) { $appProcess.MainWindowTitle } else { $null }
    cdp_port_listening = [bool]$portState.listening
    cdp_port_owning_processes = @($portState.owning_processes)
    cdp_port_states = @($portState.states)
    cdp_version_ready = [bool]$Result.cdp_version_ready
    json_list_status = $Result.json_list_status
    target_count = [int]$Result.target_count
    last_target_url = $Result.last_target_url
    last_target_title = $Result.last_target_title
    target_materialization_stage = $materializationStage
    diagnostic_last_line = $diagnosticEvidence.last_line
    diagnostic_setup_exit_seen = $setupExitSeen
    diagnostic_main_window_created_seen = [bool]$diagnosticEvidence.main_window_created_seen
    diagnostic_post_setup_window_probe_seen = [bool]$diagnosticEvidence.post_setup_window_probe_seen
    diagnostic_post_setup_window_probe_lines = @($diagnosticEvidence.post_setup_window_probe_lines)
    webview2_query_error = $webView2QueryError
    webview2_process_count = @($webView2Processes).Count
    webview2_expected_profile_process_count = @($expectedProfileProcesses).Count
    webview2_expected_user_data_folder_seen = @($expectedProfileProcesses).Count -gt 0
    expected_profile_used = $profileUsed
    profile_dir = $ProfileDir
    profile_ebwebview_exists = [bool]$profileEvidence.ebwebview_exists
    profile_has_default = [bool]$profileEvidence.has_default
    profile_has_local_state = [bool]$profileEvidence.has_local_state
    profile_has_last_version = [bool]$profileEvidence.has_last_version
    profile_lock_like_count = $profileEvidence.lock_like_count
    artifact_path = $Path
  }
  Write-JsonArtifact -Value $probe -Path $Path -Depth 12 | Out-Null
  $Result.post_setup_probe_latest = $probe
  $Result.post_setup_probe_latest_artifact_path = $Path
  $Result.target_materialization_stage = $materializationStage
  Add-ResultArrayValue -Result $Result -Name "post_setup_probe_artifact_paths" -Value $Path
  return $probe
}

function Get-ReleaseBuildProvenance {
  param(
    [string]$RepoRoot,
    [datetime]$ExeLastWriteTime
  )

  $releaseBuildRoot = Join-Path $RepoRoot "target\release\build"
  $uiDistRoot = Join-Path $RepoRoot "ui\dist"
  $uiDistIndex = Join-Path $uiDistRoot "index.html"

  $latestHopeAppOutput = @(
    if (Test-Path -LiteralPath $releaseBuildRoot) {
      Get-ChildItem -LiteralPath $releaseBuildRoot -Directory -Filter "hope-app-*" -ErrorAction SilentlyContinue |
        ForEach-Object {
          $outputPath = Join-Path $_.FullName "output"
          if (Test-Path -LiteralPath $outputPath) {
            Get-Item -LiteralPath $outputPath
          }
        } |
        Sort-Object LastWriteTime -Descending |
      Select-Object -First 1
    }
  )

  $tauriCandidates = @()
  if (Test-Path -LiteralPath $releaseBuildRoot) {
    foreach ($dir in @(Get-ChildItem -LiteralPath $releaseBuildRoot -Directory -Filter "tauri-*" -ErrorAction SilentlyContinue)) {
      $outputPath = Join-Path $dir.FullName "output"
      if (-not (Test-Path -LiteralPath $outputPath)) {
        continue
      }
      $outputItem = Get-Item -LiteralPath $outputPath
      $lines = @(Get-Content -LiteralPath $outputPath -ErrorAction SilentlyContinue)
      $devLine = @($lines | Where-Object { $_ -like "cargo:dev=*" } | Select-Object -First 1)
      $devValue = if ($devLine.Count -gt 0) {
        $devLine[0].Substring("cargo:dev=".Length)
      } else {
        $null
      }
      $cargoDev = if ($null -eq $devValue) { $null } else { $devValue -eq "true" }
      $tauriCandidates += [ordered]@{
        path = $outputItem.FullName
        last_write_time = $outputItem.LastWriteTime.ToString("o")
        cargo_dev = $cargoDev
        rustc_cfg_dev = @($lines | Where-Object { $_ -eq "cargo:rustc-cfg=dev" }).Count -gt 0
        rustc_cfg_custom_protocol = @($lines | Where-Object { $_ -eq "cargo:rustc-cfg=custom_protocol" }).Count -gt 0
      }
    }
  }
  $tauriCandidates = @($tauriCandidates | Sort-Object { [datetime]$_.last_write_time } -Descending)
  $latestTauriOutput = @($tauriCandidates | Select-Object -First 1)
  $latestDevTauriOutput = @($tauriCandidates | Where-Object { $_.cargo_dev -eq $true } | Select-Object -First 1)
  $latestReleaseLikeTauriOutput = @($tauriCandidates | Where-Object { $_.cargo_dev -eq $false -and $_.rustc_cfg_custom_protocol } | Select-Object -First 1)

  $hopeLines = if ($latestHopeAppOutput.Count -gt 0) {
    @(Get-Content -LiteralPath $latestHopeAppOutput[0].FullName -ErrorAction SilentlyContinue)
  } else {
    @()
  }
  $uiDistAssetsCount = if (Test-Path -LiteralPath $uiDistRoot) {
    @(Get-ChildItem -LiteralPath $uiDistRoot -Recurse -File -ErrorAction SilentlyContinue).Count
  } else {
    0
  }

  $latestTauriDev = if ($latestTauriOutput.Count -gt 0) { $latestTauriOutput[0].cargo_dev } else { $null }
  $latestTauriCustomProtocol = if ($latestTauriOutput.Count -gt 0) { [bool]$latestTauriOutput[0].rustc_cfg_custom_protocol } else { $false }

  $provenanceStatus = if ($latestTauriDev -eq $true) {
    "dev_mode_build_output"
  } elseif ($latestTauriDev -eq $false -and $latestTauriCustomProtocol -and (Test-Path -LiteralPath $uiDistIndex)) {
    "release_like_build_output"
  } else {
    "unknown"
  }

  [ordered]@{
    release_build_root = $releaseBuildRoot
    ui_dist_root = $uiDistRoot
    ui_dist_exists = Test-Path -LiteralPath $uiDistRoot
    ui_dist_index_exists = Test-Path -LiteralPath $uiDistIndex
    ui_dist_index_last_write_time = if (Test-Path -LiteralPath $uiDistIndex) { (Get-Item -LiteralPath $uiDistIndex).LastWriteTime.ToString("o") } else { $null }
    ui_dist_assets_count = $uiDistAssetsCount
    exe_last_write_time = $ExeLastWriteTime.ToString("o")
    tauri_output_candidates = @($tauriCandidates | Select-Object -First 6)
    latest_tauri_output = if ($latestTauriOutput.Count -gt 0) {
      $latestTauriOutput[0]
    } else {
      $null
    }
    latest_dev_tauri_output = if ($latestDevTauriOutput.Count -gt 0) {
      $latestDevTauriOutput[0]
    } else {
      $null
    }
    latest_release_like_tauri_output = if ($latestReleaseLikeTauriOutput.Count -gt 0) {
      $latestReleaseLikeTauriOutput[0]
    } else {
      $null
    }
    latest_hope_app_output = if ($latestHopeAppOutput.Count -gt 0) {
      [ordered]@{
        path = $latestHopeAppOutput[0].FullName
        last_write_time = $latestHopeAppOutput[0].LastWriteTime.ToString("o")
        tauri_config_rerun_seen = @($hopeLines | Where-Object { $_ -like "cargo:rerun-if-changed=*tauri.conf.json" }).Count -gt 0
        target_triple_seen = @($hopeLines | Where-Object { $_ -like "cargo:rustc-env=TAURI_ENV_TARGET_TRIPLE=*" }).Count -gt 0
      }
    } else {
      $null
    }
    likely_current_exe_matches_latest_build_output = if ($latestTauriOutput.Count -gt 0) {
      [Math]::Abs(($ExeLastWriteTime - ([datetime]$latestTauriOutput[0].last_write_time)).TotalMinutes) -le 30
    } else {
      $null
    }
    likely_current_exe_matches_latest_dev_output = if ($latestDevTauriOutput.Count -gt 0) {
      [Math]::Abs(($ExeLastWriteTime - ([datetime]$latestDevTauriOutput[0].last_write_time)).TotalMinutes) -le 30
    } else {
      $null
    }
    provenance_status = $provenanceStatus
    direct_cargo_release_shell_acceptable = [bool](
      $provenanceStatus -eq "release_like_build_output" -and
      $latestTauriOutput.Count -gt 0 -and
      [Math]::Abs(($ExeLastWriteTime - ([datetime]$latestTauriOutput[0].last_write_time)).TotalMinutes) -le 30
    )
  }
}

function Update-LaunchResultEvidence {
  param(
    [System.Collections.IDictionary]$Result,
    [string]$ProfileDir,
    [string]$StdoutLog,
    [string]$StderrLog,
    [string]$DiagnosticLog,
    [string]$GlobalDiagnosticLog,
    [string]$LaunchId,
    [int]$TailLines,
    [datetime]$Since
  )

  $Result.stdout_tail_sanitized = @(Get-SanitizedFileTail -Path $StdoutLog -TailLines 20)
  $Result.stderr_tail_sanitized = @(Get-SanitizedFileTail -Path $StderrLog -TailLines 20)
  $Result.diagnostic_log_tail_sanitized = @(Get-SanitizedFileTail -Path $DiagnosticLog -TailLines $TailLines)
  $Result.diagnostic_log_evidence = Get-LaunchDiagnosticEvidence -Path $DiagnosticLog -LaunchId $LaunchId
  $Result.webview2_profile_stage_evidence = Get-WebView2ProfileStageEvidence -ProfileDir $ProfileDir
  $Result.webview2_process_query_capability = Get-WebView2ProcessQueryCapability
  $Result.webview2_runtime_status = Get-WebView2RuntimeStatus
  $Result.app_process_env_proxy_present = $Result.diagnostic_log_evidence.app_process_env_proxy_present
  $Result.no_proxy_loopback_only = $Result.diagnostic_log_evidence.app_process_no_proxy_loopback_only
  $Result.event_log_evidence = Get-LaunchEventLogEvidence -Since $Since
  $Result.events_since_script_start = Get-RelevantApplicationEvents -Since $Since
  $Result.global_diagnostic_log = $GlobalDiagnosticLog
  $Result.global_diagnostic_log_exists = if ([string]::IsNullOrWhiteSpace($GlobalDiagnosticLog)) {
    $false
  } else {
    Test-Path -LiteralPath $GlobalDiagnosticLog
  }
}

$exe = Join-Path $RepoRoot "target\release\hope-app.exe"
$workDir = Join-Path $RepoRoot "target\release"
$launchId = "hope-cdp-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + $PID
$profileRoot = Join-Path $env:TEMP "hope-webview2-cdp"
$profileDir = Join-Path $profileRoot $launchId
$diagnosticLog = Join-Path $profileDir "hope-shell-diagnostic.log"
$globalDiagnosticLog = Join-Path $workDir "hope-shell-diagnostic.log"
$launchTextModelConfigured = -not [bool]$LaunchDiagnosticOnly
$effectiveModelEnabled = if ($LaunchDiagnosticOnly) {
  $false
} elseif ($ModelEnabled -eq "auto") {
  $Provider -eq "qwen" -and $AllowedQwenTextModels -contains $Model
} else {
  $ModelEnabled -eq "true"
}
$effectiveProviderTimeoutSeconds = if ($QaProviderHardFail) { 120 } else { 30 }
$proxyEnvBefore = Get-ProxyEnvPresence
Clear-QaProxyEnv
Set-ProcessEnvValue -Name "HOPE_QA_WEBVIEW2_PROXY_CLEARED" -Value "true"
if ($NoProxy) {
  Set-ProcessEnvValue -Name "HOPE_QA_PROXY_CLEARED" -Value "true"
  Set-ProcessEnvValue -Name "HOPE_QA_PROXY_EVIDENCE" -Value "true"
} else {
  Set-ProcessEnvValue -Name "HOPE_QA_PROXY_CLEARED" -Value $null
  Set-ProcessEnvValue -Name "HOPE_QA_PROXY_EVIDENCE" -Value $null
}
if ($LaunchDiagnosticOnly) {
  Set-ProcessEnvValue -Name "HOPE_QA_NO_LOCAL_FALLBACK" -Value $null
  Set-ProcessEnvValue -Name "HOPE_QA_PROVIDER_TIMEOUT_SECONDS" -Value $null
} elseif ($QaProviderHardFail) {
  Set-ProcessEnvValue -Name "HOPE_QA_NO_LOCAL_FALLBACK" -Value "true"
  Set-ProcessEnvValue -Name "HOPE_QA_PROVIDER_TIMEOUT_SECONDS" -Value ([string]$effectiveProviderTimeoutSeconds)
} else {
  Set-ProcessEnvValue -Name "HOPE_QA_NO_LOCAL_FALLBACK" -Value $null
  Set-ProcessEnvValue -Name "HOPE_QA_PROVIDER_TIMEOUT_SECONDS" -Value $null
}
$effectiveProviderTimeoutEnv = [Environment]::GetEnvironmentVariable("HOPE_QA_PROVIDER_TIMEOUT_SECONDS", "Process")
$proxyEnvAfter = Get-ProxyEnvPresence
$result = [ordered]@{
  ok = $false
  launch_id = $launchId
  repo_root = $RepoRoot
  exe = $exe
  exe_exists = Test-Path -LiteralPath $exe
  exe_last_write_time = $null
  exe_size = $null
  exe_sha256 = $null
  exe_fingerprint = $null
  exe_unlocked_before_launch = $false
  exe_unlocked = $false
  exe_unlock_wait_ms = $null
  exe_unlock_error = $null
  process_path_matches_expected_exe = $false
  process_image_matches_expected_exe = $false
  current_exe_provenance_status = "not_checked"
  current_exe_provenance = $null
  port = $Port
  port_released = $false
  provider = if ($launchTextModelConfigured) { $Provider } else { $null }
  model = if ($launchTextModelConfigured) { $Model } else { $null }
  model_enabled = if ($launchTextModelConfigured) { [bool]$effectiveModelEnabled } else { $false }
  launch_diagnostic_only = [bool]$LaunchDiagnosticOnly
  launch_text_model_configured = [bool]$launchTextModelConfigured
  launch_provider_model_args_included = [bool]$launchTextModelConfigured
  qa_no_proxy = [bool]$NoProxy
  qa_provider_hard_fail = [bool]$QaProviderHardFail
  qa_provider_timeout_seconds = if ($QaProviderHardFail) { $effectiveProviderTimeoutSeconds } else { $null }
  effective_provider_timeout_seconds = $effectiveProviderTimeoutSeconds
  qa_provider_timeout_env_value = $effectiveProviderTimeoutEnv
  launcher_process_env_proxy_present_before = [bool]$proxyEnvBefore.process_env_proxy_present
  proxy_env_before = $proxyEnvBefore
  proxy_env_after = $proxyEnvAfter
  process_env_proxy_present = [bool]$proxyEnvAfter.process_env_proxy_present
  webview2_launch_env_proxy_cleared = -not [bool]$proxyEnvAfter.process_env_proxy_present
  webview2_proxy_cleared_in_launcher = -not [bool]$proxyEnvAfter.process_env_proxy_present
  no_proxy_loopback_only = Test-NoProxyLoopbackOnly
  app_process_env_proxy_present = $null
  qa_proxy_cleared_in_launcher = [bool]($NoProxy -and -not $proxyEnvAfter.process_env_proxy_present)
  webview2_argument_mode = $WebView2ArgumentMode
  api_key_present = $false
  base_url_present = $false
  expected_target_url = $ExpectedTargetUrl
  cdp_args = New-WebView2BrowserArguments -Mode $WebView2ArgumentMode -LocalPort $Port
  launch_args = @()
  launch_args_sanitized = @()
  launcher_expected_env = [ordered]@{
    hope_shell_diagnostic_log_path = $diagnosticLog
    hope_qa_launch_id = $launchId
    webview2_user_data_folder = $profileDir
    webview2_additional_browser_arguments = New-WebView2BrowserArguments -Mode $WebView2ArgumentMode -LocalPort $Port
  }
  build_provenance = $null
  release_target_mismatch_blocked = $false
  dev_target_detected = $false
  dev_target_urls = @()
  webview2_user_data_folder = $null
  stdout_log = $null
  stderr_log = $null
  stdout_tail_sanitized = @()
  stderr_tail_sanitized = @()
  diagnostic_log = $diagnosticLog
  global_diagnostic_log = $globalDiagnosticLog
  global_diagnostic_log_exists = $false
  diagnostic_log_tail_sanitized = @()
  diagnostic_log_evidence = $null
  webview2_profile_stage_evidence = $null
  webview2_process_query_capability = $null
  webview2_process_query_capability_after_stop = $null
  webview2_cleanup_scope = "hope-app-and-hope-webview2-profile-only"
  webview2_total_process_count_after_stop = $null
  webview2_hope_process_count_after_stop = $null
  webview2_non_hope_process_count_after_stop = $null
  webview2_non_hope_process_owners_after_stop = @()
  webview2_runtime_status = $null
  event_log_evidence = $null
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
  webview2_hardened_args_expected = [bool]($WebView2ArgumentMode -eq "AppDefault")
  webview2_hardened_args_seen = $false
  webview2_app_popup_suppression_arg_present = $false
  webview2_runtime_noerrdialogs_seen = $false
  webview2_noerrdialogs_seen = $false
  webview2_disable_breakpad_seen = $false
  webview2_disable_crash_reporter_seen = $false
  webview2_popup_suppression_detected = $false
  webview2_popup_suppression_forbidden = [bool]($WebView2ArgumentMode -eq "AppDefault")
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
  cdp_json_version_artifact_paths = @()
  cdp_json_version_latest_artifact_path = $null
  cdp_json_list_artifact_paths = @()
  cdp_json_list_latest_artifact_path = $null
  post_setup_probe_artifact_paths = @()
  post_setup_probe_latest_artifact_path = $null
  post_setup_probe_latest = $null
  cleanup_artifact_class = "hope-stoponly-cleanup"
  cleanup_artifact_path = Join-Path $profileDir "hope-stoponly-cleanup-result.json"
  stop_only_cleanup_artifact_path = Join-Path $profileDir "hope-stoponly-cleanup-result.json"
  cleanup_artifact_written = $false
  cleanup_artifact_written_at = $null
  target_materialization_stage = "not-started"
  result_artifact_path = Join-Path $profileDir "hope-launch-result.json"
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
$result.exe_fingerprint = Get-FileFingerprint -Path $exe
$result.exe_sha256 = $result.exe_fingerprint.sha256
$result.build_provenance = Get-ReleaseBuildProvenance -RepoRoot $RepoRoot -ExeLastWriteTime $exeItem.LastWriteTime
$result.webview2_process_query_capability = Get-WebView2ProcessQueryCapability

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
  $result.webview2_process_query_capability_after_stop = Get-WebView2ProcessQueryCapability
  $result.webview2_total_process_count_after_stop = $result.webview2_process_query_capability_after_stop.process_count
  $result.webview2_hope_process_count_after_stop = $result.webview2_process_query_capability_after_stop.hope_process_count
  $result.webview2_non_hope_process_count_after_stop = $result.webview2_process_query_capability_after_stop.non_hope_process_count
  $result.webview2_non_hope_process_owners_after_stop = $result.webview2_process_query_capability_after_stop.non_hope_webview_exe_names
  $portStateAfterStop = Get-PortState -LocalPort $Port
  $result.port_listening = [bool]$portStateAfterStop.listening
  $result.port_released = -not [bool]$portStateAfterStop.listening
  $result.port_owning_processes = $portStateAfterStop.owning_processes
  $result.port_states = $portStateAfterStop.states
  if (-not $stopDetail.stopped) {
    $result.error = "existing hope-app processes did not stop before timeout"
    $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
    Write-StopOnlyCleanupArtifact -Result $result | Out-Null
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
    $result.webview2_process_query_capability_after_stop = Get-WebView2ProcessQueryCapability
    $result.webview2_total_process_count_after_stop = $result.webview2_process_query_capability_after_stop.process_count
    $result.webview2_hope_process_count_after_stop = $result.webview2_process_query_capability_after_stop.hope_process_count
    $result.webview2_non_hope_process_count_after_stop = $result.webview2_process_query_capability_after_stop.non_hope_process_count
    $result.webview2_non_hope_process_owners_after_stop = $result.webview2_process_query_capability_after_stop.non_hope_webview_exe_names
    $portStateAfterStopOnly = Get-PortState -LocalPort $Port
    $result.port_listening = [bool]$portStateAfterStopOnly.listening
    $result.port_released = -not [bool]$portStateAfterStopOnly.listening
    $result.port_owning_processes = $portStateAfterStopOnly.owning_processes
    $result.port_states = $portStateAfterStopOnly.states
    $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
    $webview2QueryOk = [bool]($result.webview2_process_query_capability -and $result.webview2_process_query_capability.query_ok)
    $result.ok = [bool]($webview2QueryOk -and $unlockState.unlocked -and $result.hope_app_remaining_pids.Count -eq 0 -and $result.webview2_remaining_pids.Count -eq 0 -and $result.port_released)
    if (-not $webview2QueryOk) {
      $result.error = "WebView2 process query is unavailable; StopOnly cannot prove cleanup"
      Write-StopOnlyCleanupArtifact -Result $result | Out-Null
      Convert-ResultJson $result
      exit 4
    }
    if (-not $unlockState.unlocked) {
      $result.error = "release exe is still locked after stop-only cleanup"
      Write-StopOnlyCleanupArtifact -Result $result | Out-Null
      Convert-ResultJson $result
      exit 5
    }
    if ($result.hope_app_remaining_pids.Count -gt 0 -or $result.webview2_remaining_pids.Count -gt 0 -or -not $result.port_released) {
      $result.error = "release shell cleanup is incomplete"
      Write-StopOnlyCleanupArtifact -Result $result | Out-Null
      Convert-ResultJson $result
      exit 4
    }
    Write-StopOnlyCleanupArtifact -Result $result | Out-Null
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

if (-not $StopOnly -and [bool]$result.process_env_proxy_present) {
  $result.error = "proxy environment was not cleared before WebView2 launch"
  $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
  Convert-ResultJson $result
  exit 6
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
$result.webview2_process_query_capability = Get-WebView2ProcessQueryCapability
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
  webview2_query_ok = [bool]($result.webview2_process_query_capability -and $result.webview2_process_query_capability.query_ok)
  webview2_clear = $prelaunchWebView2.Count -eq 0
  webview2_pids = @($prelaunchWebView2 | ForEach-Object { $_.id })
  webview2_total_process_count = $result.webview2_process_query_capability.process_count
  webview2_hope_process_count = $result.webview2_process_query_capability.hope_process_count
  webview2_non_hope_process_count = $result.webview2_process_query_capability.non_hope_process_count
  webview2_non_hope_process_owners = $result.webview2_process_query_capability.non_hope_webview_exe_names
  webview2_cleanup_scope = $result.webview2_cleanup_scope
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
if (-not $result.prelaunch_gate.webview2_query_ok -or -not $result.prelaunch_gate.hope_app_clear -or -not $result.prelaunch_gate.webview2_clear -or -not $result.prelaunch_gate.port_released -or -not $result.prelaunch_gate.exe_unlocked -or -not $result.prelaunch_gate.user_data_folder_writable -or $result.prelaunch_gate.user_data_folder_owned_by_old_process) {
  $result.hope_app_remaining_pids = $prelaunchHopePids
  $result.webview2_remaining_pids = @($prelaunchWebView2 | ForEach-Object { $_.id })
  $result.port_listening = [bool]$prelaunchPortState.listening
  $result.port_released = -not [bool]$prelaunchPortState.listening
  $result.port_owning_processes = $prelaunchPortState.owning_processes
  $result.port_states = $prelaunchPortState.states
  $result.events_since_script_start = Get-RelevantApplicationEvents -Since $script:ApplicationEventBaseline
  if (-not $result.prelaunch_gate.webview2_query_ok) {
    $result.error = "WebView2 process query is unavailable before launch; refusing to start release shell"
  } else {
    $result.error = "prelaunch gate failed"
  }
  Convert-ResultJson $result
  exit 6
}

$apiKey = $null
$envFileBaseUrl = $null
$effectiveBaseUrl = $null
if (-not $LaunchDiagnosticOnly) {
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
}
$result.api_key_present = -not [string]::IsNullOrWhiteSpace($apiKey)
$result.base_url_present = -not [string]::IsNullOrWhiteSpace($effectiveBaseUrl)

New-Item -ItemType Directory -Force -Path $profileDir | Out-Null
$result.webview2_user_data_folder = $profileDir
$pidFile = Join-Path $profileDir "hope-app.pid"
$shutdownSignalFile = Join-Path $profileDir "hope-app.shutdown"
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
    "--hope-qa-launch-id=$launchId",
    "--hope-qa-webview2-user-data-folder=$profileDir",
    "--hope-qa-pid-file=$pidFile",
    "--hope-qa-shutdown-signal-file=$shutdownSignalFile",
    "--hope-qa-no-proxy=$($NoProxy.ToString().ToLowerInvariant())"
  )
  if ($launchTextModelConfigured) {
    $launchArgs = @(
      $launchArgs +
      @(
        "--hope-qa-env-file=$EnvFile",
        "--hope-qa-provider=$Provider",
        "--hope-qa-model=$Model",
        "--hope-qa-model-enabled=$($effectiveModelEnabled.ToString().ToLowerInvariant())",
        "--hope-qa-no-local-fallback=$($QaProviderHardFail.ToString().ToLowerInvariant())",
        "--hope-qa-provider-timeout-seconds=$effectiveProviderTimeoutSeconds"
      )
    )
    if (-not [string]::IsNullOrWhiteSpace($effectiveBaseUrl)) {
      $launchArgs = @($launchArgs + "--hope-qa-base-url=$effectiveBaseUrl")
    }
  }
  $previousBrowserArgs = [Environment]::GetEnvironmentVariable("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "Process")
  $previousDiagnosticLogPath = [Environment]::GetEnvironmentVariable("HOPE_SHELL_DIAGNOSTIC_LOG_PATH", "Process")
  $previousLaunchId = [Environment]::GetEnvironmentVariable("HOPE_QA_LAUNCH_ID", "Process")
  $textModelEnvNames = @(
    "HOPE_TEXT_MODEL_PROVIDER",
    "HOPE_TEXT_MODEL_MODEL",
    "HOPE_TEXT_MODEL_ENABLED",
    "HOPE_TEXT_MODEL_API_KEY",
    "HOPE_TEXT_MODEL_BASE_URL",
    "HOPE_QA_NO_LOCAL_FALLBACK",
    "HOPE_QA_PROVIDER_TIMEOUT_SECONDS"
  )
  $previousTextModelEnv = @{}
  if ($WebView2ArgumentMode -eq "AppDefault") {
    $launchArgs = @($launchArgs + "--hope-qa-cdp-port=$Port")
  }
  $result.launch_args = Get-SanitizedLaunchArgs -ArgList $launchArgs
  $result.launch_args_sanitized = $result.launch_args
  try {
    Set-ProcessEnvValue -Name "HOPE_SHELL_DIAGNOSTIC_LOG_PATH" -Value $diagnosticLog
    Set-ProcessEnvValue -Name "HOPE_QA_LAUNCH_ID" -Value $launchId
    if ($LaunchDiagnosticOnly) {
      foreach ($name in $textModelEnvNames) {
        $previousTextModelEnv[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
        Set-ProcessEnvValue -Name $name -Value $null
      }
    }
    if ($WebView2ArgumentMode -eq "AppDefault") {
      Set-ProcessEnvValue -Name "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS" -Value $null
    } else {
      Set-ProcessEnvValue -Name "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS" -Value $browserArgs
    }
    $appProc = Start-Process -FilePath $exe -ArgumentList $launchArgs -WorkingDirectory $workDir -PassThru
  } finally {
    Set-ProcessEnvValue -Name "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS" -Value $previousBrowserArgs
    Set-ProcessEnvValue -Name "HOPE_SHELL_DIAGNOSTIC_LOG_PATH" -Value $previousDiagnosticLogPath
    Set-ProcessEnvValue -Name "HOPE_QA_LAUNCH_ID" -Value $previousLaunchId
    if ($LaunchDiagnosticOnly) {
      foreach ($name in $textModelEnvNames) {
        Set-ProcessEnvValue -Name $name -Value $previousTextModelEnv[$name]
      }
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

  Update-ProcessImageProvenance -Result $result -ExpectedExe $exe

  $startedAt = Get-Date
  $deadline = $startedAt.AddSeconds($WaitSeconds)
  do {
    Start-Sleep -Milliseconds 700
    $result.poll_attempts += 1
    $pollProbeSaved = $false

    $processState = Get-ProcessState -Id $result.started_pid
    $result.process_running_after_wait = [bool]$processState.running
    $result.process_exit_code = $processState.exit_code
    $result.process_path = $processState.path

    $versionArtifactPath = Join-Path $profileDir ("cdp-json-version-poll-{0:000}.json" -f $result.poll_attempts)
    try {
      $version = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/version" -TimeoutSec 2
      $result.cdp_version_ready = $true
      $result.cdp_browser = $version.Browser
      $result.cdp_websocket_present = -not [string]::IsNullOrWhiteSpace($version.webSocketDebuggerUrl)
      $result.last_version_error = $null
      $result.json_version_status = "ok"
      Write-CdpJsonArtifact -Path $versionArtifactPath -ArtifactClass "cdp-json-version" -Endpoint "http://127.0.0.1:$Port/json/version" -Status "ok" -RawSanitized (Get-SanitizedCdpVersionArtifactValue -Version $version) -ErrorMessage $null -PollAttempt $result.poll_attempts | Out-Null
    } catch {
      $result.last_version_error = $_.Exception.Message
      $result.json_version_status = "error"
      Write-CdpJsonArtifact -Path $versionArtifactPath -ArtifactClass "cdp-json-version" -Endpoint "http://127.0.0.1:$Port/json/version" -Status "error" -RawSanitized $null -ErrorMessage $_.Exception.Message -PollAttempt $result.poll_attempts | Out-Null
    }
    Add-ResultArrayValue -Result $result -Name "cdp_json_version_artifact_paths" -Value $versionArtifactPath
    $result.cdp_json_version_latest_artifact_path = $versionArtifactPath

    $listArtifactPath = Join-Path $profileDir ("cdp-json-list-poll-{0:000}.json" -f $result.poll_attempts)
    try {
      $targets = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/json/list" -TimeoutSec 2
      $result.json_list_status = "ok"
      $targetList = @($targets)
      $result.target_count = $targetList.Count
      $result.last_targets = Get-CdpTargetSummary -Targets $targetList
      Write-CdpJsonArtifact -Path $listArtifactPath -ArtifactClass "cdp-json-list" -Endpoint "http://127.0.0.1:$Port/json/list" -Status "ok" -RawSanitized (Get-SanitizedCdpListArtifactValue -Targets $targetList) -ErrorMessage $null -PollAttempt $result.poll_attempts | Out-Null
      if ($targetList.Count -gt 0) {
        $result.last_target_url = $targetList[0].url
        $result.last_target_title = $targetList[0].title
      }
      $devTargets = @(
        $targetList |
          Where-Object {
            $_.url -eq "http://127.0.0.1:5173/" -or
            $_.url -like "http://127.0.0.1:5173/*"
          }
      )
      $result.dev_target_detected = $devTargets.Count -gt 0
      $result.dev_target_urls = @($devTargets | ForEach-Object { $_.url } | Sort-Object -Unique)
      $tauriTargets = @($targetList | Where-Object { $_.url -like "http://tauri.localhost/*" })
      if ($tauriTargets.Count -gt 0) {
        $result.tauri_target_seen = $true
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
        Add-ResultArrayValue -Result $result -Name "cdp_json_list_artifact_paths" -Value $listArtifactPath
        $result.cdp_json_list_latest_artifact_path = $listArtifactPath
        $probePath = Join-Path $profileDir ("post-setup-probe-poll-{0:000}.json" -f $result.poll_attempts)
        Save-PostSetupProbe -Result $result -Path $probePath -ProfileDir $profileDir -DiagnosticLog $diagnosticLog -Port $Port -ExpectedTargetUrl $ExpectedTargetUrl -StartedPid $result.started_pid -PollAttempt $result.poll_attempts | Out-Null
        $pollProbeSaved = $true
        break
      }
      $result.last_list_error = $null
    } catch {
      $result.last_list_error = $_.Exception.Message
      $result.json_list_status = "error"
      Write-CdpJsonArtifact -Path $listArtifactPath -ArtifactClass "cdp-json-list" -Endpoint "http://127.0.0.1:$Port/json/list" -Status "error" -RawSanitized $null -ErrorMessage $_.Exception.Message -PollAttempt $result.poll_attempts | Out-Null
    }
    if (-not $pollProbeSaved) {
      Add-ResultArrayValue -Result $result -Name "cdp_json_list_artifact_paths" -Value $listArtifactPath
      $result.cdp_json_list_latest_artifact_path = $listArtifactPath
      $probePath = Join-Path $profileDir ("post-setup-probe-poll-{0:000}.json" -f $result.poll_attempts)
      Save-PostSetupProbe -Result $result -Path $probePath -ProfileDir $profileDir -DiagnosticLog $diagnosticLog -Port $Port -ExpectedTargetUrl $ExpectedTargetUrl -StartedPid $result.started_pid -PollAttempt $result.poll_attempts | Out-Null
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
  Update-LaunchResultEvidence -Result $result -ProfileDir $profileDir -StdoutLog $stdoutLog -StderrLog $stderrLog -DiagnosticLog $diagnosticLog -GlobalDiagnosticLog $globalDiagnosticLog -LaunchId $launchId -TailLines $DiagnosticTailLines -Since $script:ApplicationEventBaseline
  $webviewProfileEvidence = $result.webview2_profile_stage_evidence
  $webviewProfileCreated = [bool](
    $webviewProfileEvidence -and
    $webviewProfileEvidence.ebwebview_exists -and
    (
      $webviewProfileEvidence.eb_entries.Count -gt 0 -or
      $webviewProfileEvidence.has_default -or
      $webviewProfileEvidence.has_local_state -or
      $webviewProfileEvidence.has_last_version -or
      $webviewProfileEvidence.has_crashpad -or
      $webviewProfileEvidence.edge_local_state_tmp.Count -gt 0
    )
  )
  $result.webview2_remote_debugging_port_seen = @($webview2AfterWait | Where-Object { $_.has_remote_debugging_port }).Count -gt 0
  $result.webview2_expected_user_data_folder_seen = @($webview2AfterWait | Where-Object { Test-PathAtOrUnder -Path $_.user_data_dir -Root $profileDir }).Count -gt 0
  $result.webview2_hardened_args_seen = @($webview2AfterWait | Where-Object { $_.has_no_first_run -and $_.has_disable_background_networking -and $_.has_disable_component_update -and $_.has_disable_gpu }).Count -gt 0
  $result.webview2_noerrdialogs_seen = @($webview2AfterWait | Where-Object { $_.has_noerrdialogs }).Count -gt 0
  $result.webview2_runtime_noerrdialogs_seen = [bool]$result.webview2_noerrdialogs_seen
  $result.webview2_disable_breakpad_seen = @($webview2AfterWait | Where-Object { $_.has_disable_breakpad }).Count -gt 0
  $result.webview2_disable_crash_reporter_seen = @($webview2AfterWait | Where-Object { $_.has_disable_crash_reporter }).Count -gt 0
  $result.webview2_app_popup_suppression_arg_present = @($result.diagnostic_log_tail_sanitized | Where-Object { $_ -like "*webview2_popup_suppression_arg_present=true*" }).Count -gt 0
  $result.webview2_popup_suppression_detected = [bool]($result.webview2_app_popup_suppression_arg_present -or $result.webview2_disable_breakpad_seen -or $result.webview2_disable_crash_reporter_seen)
  $webview2ProcessQueryOk = [bool]($result.webview2_process_query_capability -and $result.webview2_process_query_capability.query_ok)
  $result.webview2_environment_not_created_or_crashed = [bool](
    (-not $result.cdp_ready) -and
    $webviewProfileCreated -and
    (
      ($webview2ProcessQueryOk -and $webview2AfterWait.Count -eq 0) -or
      ((-not $webview2ProcessQueryOk) -and (-not $result.port_listening))
    )
  )

  if (-not $result.cdp_ready -and [string]::IsNullOrWhiteSpace($result.error)) {
    if ($result.dev_target_detected) {
      $result.release_target_mismatch_blocked = $true
      $result.error = "release shell opened devUrl target http://127.0.0.1:5173; stale/dev build blocker"
    } elseif ($result.cdp_version_ready -and $result.tauri_target_seen) {
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
    $failureProfileCreated = [bool](
      $result.webview2_profile_stage_evidence -and
      $result.webview2_profile_stage_evidence.ebwebview_exists -and
      (
        $result.webview2_profile_stage_evidence.eb_entries.Count -gt 0 -or
        $result.webview2_profile_stage_evidence.has_default -or
        $result.webview2_profile_stage_evidence.has_local_state -or
        $result.webview2_profile_stage_evidence.has_last_version -or
        $result.webview2_profile_stage_evidence.has_crashpad -or
        $result.webview2_profile_stage_evidence.edge_local_state_tmp.Count -gt 0
      )
    )
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
      webview2_hardened_args_seen = @($webview2BeforeFailureStop | Where-Object { $_.has_no_first_run -and $_.has_disable_background_networking -and $_.has_disable_component_update -and $_.has_disable_gpu }).Count -gt 0
      webview2_runtime_noerrdialogs_seen = @($webview2BeforeFailureStop | Where-Object { $_.has_noerrdialogs }).Count -gt 0
      webview2_app_popup_suppression_arg_present = @($result.diagnostic_log_tail_sanitized | Where-Object { $_ -like "*webview2_popup_suppression_arg_present=true*" }).Count -gt 0
      webview2_popup_suppression_detected = @($result.diagnostic_log_tail_sanitized | Where-Object { $_ -like "*webview2_popup_suppression_arg_present=true*" }).Count -gt 0 -or @($webview2BeforeFailureStop | Where-Object { $_.has_disable_breakpad -or $_.has_disable_crash_reporter }).Count -gt 0
      expected_user_data_folder = $profileDir
      webview2_expected_user_data_folder_seen = @($webview2BeforeFailureStop | Where-Object { Test-PathAtOrUnder -Path $_.user_data_dir -Root $profileDir }).Count -gt 0
      webview2_user_data_folders = @($webview2BeforeFailureStop | ForEach-Object { $_.user_data_dir } | Where-Object { $_ } | Sort-Object -Unique)
      port_listening = [bool]$portStateBeforeFailureStop.listening
      port_owning_processes = $portStateBeforeFailureStop.owning_processes
      port_states = $portStateBeforeFailureStop.states
      build_provenance = $result.build_provenance
      release_target_mismatch_blocked = [bool]$result.release_target_mismatch_blocked
      dev_target_detected = [bool]$result.dev_target_detected
      dev_target_urls = $result.dev_target_urls
      diagnostic_log_evidence = $result.diagnostic_log_evidence
      diagnostic_log_tail_sanitized = $result.diagnostic_log_tail_sanitized
      webview2_profile_stage_evidence = $result.webview2_profile_stage_evidence
      webview2_process_query_capability = $result.webview2_process_query_capability
      webview2_runtime_status = $result.webview2_runtime_status
      event_log_evidence = $result.event_log_evidence
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
    Update-LaunchResultEvidence -Result $result -ProfileDir $profileDir -StdoutLog $stdoutLog -StderrLog $stderrLog -DiagnosticLog $diagnosticLog -GlobalDiagnosticLog $globalDiagnosticLog -LaunchId $launchId -TailLines $DiagnosticTailLines -Since $script:ApplicationEventBaseline
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
  Update-LaunchResultEvidence -Result $result -ProfileDir $profileDir -StdoutLog $stdoutLog -StderrLog $stderrLog -DiagnosticLog $diagnosticLog -GlobalDiagnosticLog $globalDiagnosticLog -LaunchId $launchId -TailLines $DiagnosticTailLines -Since $script:ApplicationEventBaseline
}

Update-ProcessImageProvenance -Result $result -ExpectedExe $exe
$result.result_artifact_path = Write-JsonArtifact -Value ([pscustomobject]$result) -Path $result.result_artifact_path -Depth 16
Convert-ResultJson $result
if ($result.ok) {
  exit 0
}
exit 3
