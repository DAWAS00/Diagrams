$ErrorActionPreference = 'Stop'

$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCommand) {
    $cargoPath = $cargoCommand.Source
} else {
    $cargoPath = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (-not (Test-Path -LiteralPath $cargoPath)) {
        throw 'Cargo was not found. Install Rustup or add Cargo to PATH.'
    }
}

$checks = @(
    @('fmt', '--all', '--', '--check'),
    @('check', '--workspace', '--all-targets', '--locked'),
    @('clippy', '--workspace', '--all-targets', '--all-features', '--locked', '--', '-D', 'warnings'),
    @('test', '--workspace', '--locked'),
    @('doc', '--workspace', '--no-deps', '--locked'),
    @('build', '-p', 'diagram-desktop', '--release', '--locked')
)

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    foreach ($arguments in $checks) {
        Write-Host "cargo $($arguments -join ' ')"
        & $cargoPath @arguments
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo check failed: $($arguments -join ' ') (exit $LASTEXITCODE)"
        }
    }
} finally {
    Pop-Location
}
