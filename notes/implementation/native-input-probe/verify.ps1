param(
    [string]$CargoHome,
    [string]$TargetDirectory
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../../..')).Path
$manifestPath = Join-Path $PSScriptRoot 'Cargo.toml'
if (-not $TargetDirectory) {
    $TargetDirectory = Join-Path $repoRoot 'target/production-audit/native-input-probe'
}
$previousCargoHome = $env:CARGO_HOME
try {
    if ($CargoHome) {
        $env:CARGO_HOME = (Resolve-Path -LiteralPath $CargoHome).Path
    }
    & cargo test --manifest-path $manifestPath --locked --offline --lib --target-dir $TargetDirectory
    if ($LASTEXITCODE -ne 0) {
        throw "Native input source tests failed with exit code $LASTEXITCODE"
    }
}
finally {
    $env:CARGO_HOME = $previousCargoHome
}
