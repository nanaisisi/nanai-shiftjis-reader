<#
.SYNOPSIS
Build and package the nanai-shiftjis-reader app using winapp and MSIX.

.DESCRIPTION
This script supports two modes:
- Debug: build debug binary, create debug identity, and run the app.
- Package: build release binary, create a dist folder, generate certificate, pack MSIX, install cert, and install MSIX.

.EXAMPLE
.
    .\build-msix.ps1 -Mode Debug

.EXAMPLE
    .\build-msix.ps1 -Mode Package
#>

[CmdletBinding()]
param (
    [ValidateSet('Debug', 'Package')]
    [string]$Mode = 'Debug',

    [switch]$SkipCertInstall,
    [switch]$SkipMsixInstall,
    [switch]$SkipPack,
    [switch]$SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $projectRoot
try {
    $exeName = 'nanai-shiftjis-reader.exe'
    $debugPath = Join-Path $projectRoot 'target\debug\' $exeName
    $releasePath = Join-Path $projectRoot 'target\release\' $exeName
    $distDir = Join-Path $projectRoot 'dist'
    $certPath = Join-Path $projectRoot 'devcert.pfx'
    $msixName = 'nanai-shiftjis-reader.msix'
    $msixPath = Join-Path $projectRoot $msixName

    function Build-Debug {
        if (-not $SkipBuild) {
            Write-Host 'Building debug binary...' -ForegroundColor Cyan
            cargo build
        }
        if (-not (Test-Path $debugPath)) {
            throw "Debug binary not found: $debugPath"
        }
    }

    function Build-Release {
        if (-not $SkipBuild) {
            Write-Host 'Building release binary...' -ForegroundColor Cyan
            cargo build --release
        }
        if (-not (Test-Path $releasePath)) {
            throw "Release binary not found: $releasePath"
        }
    }

    function Create-DebugIdentity {
        Write-Host 'Creating debug identity...' -ForegroundColor Cyan
        winapp create-debug-identity $debugPath
    }

    function Run-DebugBinary {
        Write-Host 'Running debug binary...' -ForegroundColor Cyan
        & $debugPath
    }

    function Prepare-Dist {
        if (-not (Test-Path $distDir)) {
            New-Item -ItemType Directory -Path $distDir | Out-Null
        }
        Copy-Item -Path $releasePath -Destination $distDir -Force
    }

    function Generate-Cert {
        Write-Host 'Generating development certificate...' -ForegroundColor Cyan
        winapp cert generate --if-exists skip
        if (-not (Test-Path $certPath)) {
            throw "Certificate file not created: $certPath"
        }
    }

    function Pack-Msix {
        Write-Host 'Packing MSIX...' -ForegroundColor Cyan
        winapp pack $distDir --cert $certPath
        if (-not (Test-Path $msixPath)) {
            Write-Warning "MSIX package not found at expected path: $msixPath"
        }
    }

    function Install-Cert {
        Write-Host 'Installing certificate (administrative permissions required)...' -ForegroundColor Cyan
        winapp cert install $certPath
    }

    function Install-Msix {
        Write-Host 'Installing MSIX package...' -ForegroundColor Cyan
        Add-AppxPackage $msixPath
    }

    switch ($Mode) {
        'Debug' {
            Build-Debug
            Create-DebugIdentity
            Run-DebugBinary
        }
        'Package' {
            Build-Release
            Prepare-Dist
            Generate-Cert
            if (-not $SkipPack) {
                Pack-Msix
            }
            if (-not $SkipCertInstall) {
                Install-Cert
            }
            if (-not $SkipMsixInstall) {
                Install-Msix
            }
        }
    }
}
finally {
    Pop-Location
}
