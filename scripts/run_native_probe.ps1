# Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
# Run a workspace-owned native probe with a deadline and an artifact receipt.
# A probe may create its own GUI. Only this invocation's process tree is stopped.
# Exit 0 records program completion, not general application qualification.
# Requires PowerShell 7 (ProcessStartInfo.ArgumentList and Kill(entireProcessTree)).
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$LogPrefix,
    [Parameter(Mandatory)][string]$ProbeProfile,
    [ValidateRange(1, 600)][int]$TimeoutSeconds = 90,
    [string[]]$ProgramArguments = @()
)

$ErrorActionPreference = 'Stop'
$probeRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$probeRootPrefix = $probeRoot.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
function WorkspacePath([string]$Path) {
    $absolute = [IO.Path]::GetFullPath($Path, $probeRoot)
    if (-not $absolute.StartsWith($probeRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Probe executable and receipts must be inside the Aether workspace.'
    }
    return $absolute
}
$probeExecutable = WorkspacePath $Executable
$probeLogPrefix = WorkspacePath $LogPrefix
if (-not (Test-Path -LiteralPath $probeExecutable -PathType Leaf) -or
    [IO.Path]::GetExtension($probeExecutable) -ne '.exe') {
    throw 'Expected an existing native .exe probe inside the workspace.'
}
$null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($probeLogPrefix))
$probeHash = (Get-FileHash -LiteralPath $probeExecutable -Algorithm SHA256).Hash.ToLowerInvariant()
$startInfo = [Diagnostics.ProcessStartInfo]::new($probeExecutable)
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$startInfo.WorkingDirectory = $probeRoot
foreach ($argument in $ProgramArguments) { $startInfo.ArgumentList.Add($argument) }
$probeStarted = [DateTimeOffset]::UtcNow
$probeClock = [Diagnostics.Stopwatch]::StartNew()
$probeProcess = [Diagnostics.Process]::new()
$probeProcess.StartInfo = $startInfo
$probeTimedOut = $false
$probeStartedProcess = $false
$probeCaptureComplete = $true
$probeExit = 1
try {
    if (-not $probeProcess.Start()) { throw 'The native probe did not start.' }
    $probeStartedProcess = $true
    $probePid = $probeProcess.Id
    # Drain both pipes concurrently so a diagnostic cannot deadlock the probe.
    $stdout = $probeProcess.StandardOutput.ReadToEndAsync()
    $stderr = $probeProcess.StandardError.ReadToEndAsync()
    while (-not $probeProcess.WaitForExit(1000)) {
        if ($probeClock.Elapsed.TotalSeconds -ge $TimeoutSeconds) {
            $probeTimedOut = $true
            $probeProcess.Kill($true)
            if (-not $probeProcess.WaitForExit(5000)) {
                throw 'The timed-out probe could not be stopped; inspect the owned process.'
            }
            break
        }
    }
    $probeClock.Stop()
    $probeExit = if ($probeTimedOut) { 124 } else { $probeProcess.ExitCode }
    # A descendant can retain a pipe after the main process exits. Bound capture
    # too, and report incomplete evidence rather than waiting indefinitely.
    try {
        $probeCaptureComplete = [Threading.Tasks.Task]::WaitAll([Threading.Tasks.Task[]]@($stdout, $stderr), 5000)
    } catch { $probeCaptureComplete = $false }
    $stdoutText = if ($stdout.IsCompletedSuccessfully) { $stdout.Result } else { '[stdout capture incomplete]' }
    $stderrText = if ($stderr.IsCompletedSuccessfully) { $stderr.Result } else { '[stderr capture incomplete]' }
    if (-not $probeCaptureComplete) {
        $probeProcess.StandardOutput.Close()
        $probeProcess.StandardError.Close()
        if (-not $probeTimedOut) { $probeExit = 125 }
    }
    [IO.File]::WriteAllText("$probeLogPrefix.stdout.log", $stdoutText)
    [IO.File]::WriteAllText("$probeLogPrefix.stderr.log", $stderrText)
    $receipt = [ordered]@{
        executable = $probeExecutable
        executable_sha256 = $probeHash
        profile = $ProbeProfile
        argument_count = @($ProgramArguments).Count
        working_directory = $probeRoot
        started_utc = $probeStarted.ToString('o')
        elapsed_seconds = [math]::Round($probeClock.Elapsed.TotalSeconds, 3)
        timeout_seconds = $TimeoutSeconds
        timed_out = $probeTimedOut
        capture_complete = $probeCaptureComplete
        process_id = $probePid
        process_exited = $probeProcess.HasExited
        program_exit_code = $probeProcess.ExitCode
        runner_exit_code = $probeExit
    }
    [IO.File]::WriteAllText("$probeLogPrefix.json", ($receipt | ConvertTo-Json -Depth 4))
    if ($stdoutText) { [Console]::Out.Write($stdoutText) }
    if ($stderrText) { [Console]::Error.Write($stderrText) }
    if ($probeTimedOut) { [Console]::Error.WriteLine('Native probe exceeded its deadline and was stopped.') }
} finally {
    if ($probeStartedProcess -and -not $probeProcess.HasExited) {
        $probeProcess.Kill($true)
        $null = $probeProcess.WaitForExit(5000)
    }
    $probeProcess.Dispose()
}
exit $probeExit
