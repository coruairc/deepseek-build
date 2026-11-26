#Requires -Version 5.1
<#
.SYNOPSIS
    deepseek-build installer for Windows (x86_64).

.DESCRIPTION
    Downloads the latest (or VERSION vX.Y.Z) release .zip and SHA256SUMS from
    GitHub Releases over HTTPS, verifies the checksum BEFORE extracting, and
    installs deepseek-build.exe to
    $env:LOCALAPPDATA\deepseek-build\bin (or INSTALL_DIR). No elevation.

    Network contract: exactly one host - github.com (the release zip and
    SHA256SUMS). Nothing else. No telemetry, no version-check API calls, no
    update checks.

    Usage:
        .\install.ps1                     install the latest release
        $env:VERSION='v0.1.0'; .\install.ps1
        $env:INSTALL_DIR='D:\bin'; .\install.ps1
        .\install.ps1 -Uninstall          remove the installed binary
        .\install.ps1 -Help

    Testing hooks (do not set in production):
        DSB_INSTALL_BASE_URL   override the GitHub releases base URL (a local
                               file:// tree for tests). Changes nothing else.
        DSB_INSTALL_TEST_ARCH  override the detected CPU architecture.
#>
[CmdletBinding()]
param(
    [switch]$Uninstall,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Repo = 'coruairc/deepseek-build'
$Target = 'x86_64-pc-windows-msvc'
$Script:TempDir = $null

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Write-Step {
    param([string]$Message)
    Write-Host "==> $Message"
}

function Write-Warn {
    param([string]$Message)
    [Console]::Error.WriteLine("!!  $Message")
}

# Throw rather than exit so the top-level finally always removes the temp dir;
# the caller turns the exception into exit 1.
function Stop-Installer {
    param([string]$Message)
    throw [System.Exception]::new($Message)
}

function Get-LocalAppData {
    if ($env:LOCALAPPDATA) {
        return $env:LOCALAPPDATA
    }
    if ($env:USERPROFILE) {
        return (Join-Path $env:USERPROFILE 'AppData\Local')
    }
    Stop-Installer 'Cannot determine %LOCALAPPDATA%; set INSTALL_DIR explicitly.'
}

# ---------------------------------------------------------------------------
# Testing hooks (no-ops in production)
# ---------------------------------------------------------------------------

$BaseUrl = $env:DSB_INSTALL_BASE_URL
if (-not $BaseUrl) {
    $BaseUrl = "https://github.com/$Repo/releases"
}
$BaseUrl = $BaseUrl.TrimEnd('/')
if (-not ($BaseUrl.StartsWith('file://', [System.StringComparison]::OrdinalIgnoreCase) -or
          $BaseUrl.StartsWith('https://github.com/', [System.StringComparison]::OrdinalIgnoreCase))) {
    Stop-Installer "DSB_INSTALL_BASE_URL must be https://github.com/... (or file:// for the test harness); refusing '$BaseUrl'."
}

# ---------------------------------------------------------------------------
# Argument handling
# ---------------------------------------------------------------------------

if ($Help) {
    $helpText = @'
install.ps1 - deepseek-build installer for Windows

Usage:
  .\install.ps1                     Install the latest release
  $env:VERSION='v0.1.0'; .\install.ps1
                                    Install a pinned version
  $env:INSTALL_DIR='D:\bin'; .\install.ps1
                                    Install elsewhere
                                    (default: %LOCALAPPDATA%\deepseek-build\bin)
  .\install.ps1 -Uninstall          Remove the installed binary
  .\install.ps1 -Help               This help

Environment:
  VERSION       Release tag to install, with or without the leading "v"
                (default: latest release)
  INSTALL_DIR   Target directory (default: %LOCALAPPDATA%\deepseek-build\bin)

Supported platform:
  Windows x86_64 (x86_64-pc-windows-msvc). Other architectures are rejected.

The script downloads the release .zip and SHA256SUMS from github.com, verifies
the checksum before extracting, and installs without elevation. It makes no
other network calls and sends no telemetry.

Independent project, not affiliated with DeepSeek.
'@
    Write-Host $helpText
    exit 0
}

# ---------------------------------------------------------------------------
# Uninstall mode (no network, no architecture check)
# ---------------------------------------------------------------------------

$InstallDir = $env:INSTALL_DIR
if (-not $InstallDir) {
    # Only resolve %LOCALAPPDATA% when it is actually needed, so an explicit
    # INSTALL_DIR works on a host where LOCALAPPDATA/USERPROFILE are unset.
    $InstallDir = Join-Path (Get-LocalAppData) 'deepseek-build\bin'
}

if ($Uninstall) {
    $installedExe = Join-Path $InstallDir 'deepseek-build.exe'
    if (Test-Path -LiteralPath $installedExe) {
        Remove-Item -LiteralPath $installedExe -Force
        Write-Step "Removed $installedExe"
    }
    else {
        Write-Step "Nothing to remove: $installedExe is absent"
    }
    exit 0
}

# ---------------------------------------------------------------------------
# Platform detection (x86_64 only)
# ---------------------------------------------------------------------------

if ([System.Environment]::OSVersion.Platform -ne [System.PlatformID]::Win32NT) {
    Stop-Installer 'This installer is for Windows only.'
}

function Get-CurrentArch {
    if ($env:DSB_INSTALL_TEST_ARCH) {
        return $env:DSB_INSTALL_TEST_ARCH
    }
    try {
        return [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    }
    catch {
        return $env:PROCESSOR_ARCHITECTURE
    }
}

$Arch = Get-CurrentArch
if ($Arch -notin @('X64', 'AMD64', 'x86_64')) {
    Write-Warn "Unsupported architecture: $Arch. Only x86_64 Windows is supported."
    exit 1
}

# ---------------------------------------------------------------------------
# Download + checksum helpers
# ---------------------------------------------------------------------------

try {
    # PowerShell 5.1 defaults to TLS 1.0; GitHub requires TLS 1.2+.
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
}
catch {
    # PowerShell 7 ignores the obsolete setting.
}

function Invoke-Download {
    param([string]$Url, [string]$Destination)
    if ($Url.StartsWith('file://', [System.StringComparison]::OrdinalIgnoreCase)) {
        # Test hook only: DSB_INSTALL_BASE_URL pointed at a local fake tree.
        $source = ([uri]$Url).LocalPath
        Copy-Item -LiteralPath $source -Destination $Destination -Force
    }
    else {
        $attempt = 0
        while ($true) {
            $attempt++
            try {
                Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing -TimeoutSec 60
                break
            }
            catch {
                if ($attempt -ge 3) {
                    throw
                }
                Start-Sleep -Seconds 2
            }
        }
    }
    if (-not (Test-Path -LiteralPath $Destination)) {
        Stop-Installer "Download failed: $Url"
    }
    if ((Get-Item -LiteralPath $Destination).Length -eq 0) {
        Stop-Installer "Downloaded file is empty: $Url"
    }
}

function Get-Sha256 {
    param([string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-ExpectedHash {
    param([string]$SumsPath, [string]$FileName)
    $pattern = '^([0-9a-fA-F]{64})  ' + [regex]::Escape($FileName) + '$'
    foreach ($line in Get-Content -LiteralPath $SumsPath) {
        if ($line -match $pattern) {
            return $Matches[1].ToLowerInvariant()
        }
    }
    return $null
}

function Get-ZipNameForLatest {
    param([string]$SumsPath)
    $pattern = '^[0-9a-fA-F]{64}  (deepseek-build-.+-' + [regex]::Escape($Target) + '\.zip)$'
    foreach ($line in Get-Content -LiteralPath $SumsPath) {
        if ($line -match $pattern) {
            $name = $Matches[1]
            if ($name -match '[\\/]' -or $name.Contains('..')) {
                Stop-Installer "refusing an unsafe asset name from SHA256SUMS: '$name'"
            }
            return $name
        }
    }
    return $null
}

function Get-VersionFromZip {
    param([string]$ZipName)
    $pattern = '^deepseek-build-(.+)-' + [regex]::Escape($Target) + '\.zip$'
    if ($ZipName -match $pattern) {
        return $Matches[1]
    }
    return $null
}

# ---------------------------------------------------------------------------
# Install
# ---------------------------------------------------------------------------

$Script:TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ('dsb-install-' + [guid]::NewGuid().ToString('N'))
$ExitCode = 0

try {
    New-Item -ItemType Directory -Force -Path $Script:TempDir | Out-Null
    $sumsPath = Join-Path $Script:TempDir 'SHA256SUMS'

    $Version = $env:VERSION
    if ($Version) {
        if ($Version.StartsWith('v')) {
            $VersionNum = $Version.Substring(1)
        }
        else {
            $VersionNum = $Version
        }
        if ($VersionNum -notmatch '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$') {
            Stop-Installer "Invalid VERSION: $Version (expected vX.Y.Z or X.Y.Z)"
        }
        $VersionTag = "v$VersionNum"
        $ZipName = "deepseek-build-$VersionNum-$Target.zip"
        Write-Step "Downloading $ZipName"
        Invoke-Download -Url "$BaseUrl/download/$VersionTag/$ZipName" -Destination (Join-Path $Script:TempDir $ZipName)
        Write-Step 'Downloading SHA256SUMS'
        Invoke-Download -Url "$BaseUrl/download/$VersionTag/SHA256SUMS" -Destination $sumsPath
    }
    else {
        Write-Step 'Resolving the latest release'
        Invoke-Download -Url "$BaseUrl/latest/download/SHA256SUMS" -Destination $sumsPath
        $ZipName = Get-ZipNameForLatest -SumsPath $sumsPath
        if (-not $ZipName) {
            Stop-Installer "The latest release has no Windows x86_64 asset (not listed in SHA256SUMS)."
        }
        $VersionNum = Get-VersionFromZip -ZipName $ZipName
        if (-not $VersionNum) {
            Stop-Installer "Cannot read the version from $ZipName."
        }
        $VersionTag = "v$VersionNum"
        Write-Step "Latest release: $VersionTag"
        Write-Step "Downloading $ZipName"
        Invoke-Download -Url "$BaseUrl/latest/download/$ZipName" -Destination (Join-Path $Script:TempDir $ZipName)
    }

    # Verify the checksum BEFORE extracting anything.
    $zipPath = Join-Path $Script:TempDir $ZipName
    $expected = Get-ExpectedHash -SumsPath $sumsPath -FileName $ZipName
    if (-not $expected) {
        Stop-Installer "$ZipName is not listed in SHA256SUMS (refusing to install an unverified zip)"
    }
    $actual = Get-Sha256 -Path $zipPath
    if ($actual -ne $expected) {
        Write-Warn "Checksum mismatch for $ZipName"
        Write-Warn "  expected: $expected"
        Write-Warn "  actual:   $actual"
        Stop-Installer 'Aborting: the downloaded zip does not match SHA256SUMS. Nothing was installed.'
    }
    Write-Step "Checksum OK ($actual)"

    $extractDir = Join-Path $Script:TempDir 'extract'
    try {
        Expand-Archive -LiteralPath $zipPath -DestinationPath $extractDir -Force
    }
    catch {
        Stop-Installer "Cannot extract ${ZipName}: $($_.Exception.Message)"
    }
    $sourceExe = Join-Path $extractDir 'deepseek-build.exe'
    if (-not (Test-Path -LiteralPath $sourceExe)) {
        Stop-Installer 'The zip does not contain deepseek-build.exe at its top level.'
    }

    try {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }
    catch {
        Stop-Installer "Cannot create $InstallDir (not writable?). Set INSTALL_DIR to a writable directory."
    }
    $destExe = Join-Path $InstallDir 'deepseek-build.exe'
    try {
        Copy-Item -LiteralPath $sourceExe -Destination $destExe -Force
    }
    catch {
        Stop-Installer "Cannot write $destExe (not writable?). Set INSTALL_DIR to a writable directory."
    }

    # PATH hint (text only; the installer never edits PATH for you).
    $normalizedInstallDir = $InstallDir.TrimEnd('\')
    $pathParts = @()
    if ($env:PATH) {
        $pathParts = $env:PATH -split ';' | ForEach-Object { $_.TrimEnd('\') }
    }
    if ($pathParts -notcontains $normalizedInstallDir) {
        Write-Warn "$InstallDir is not in your PATH."
        Write-Warn "Add it for this session:  `$env:PATH = `"$InstallDir;`$env:PATH`""
        Write-Warn "Make it permanent:  [Environment]::SetEnvironmentVariable('Path', `"$InstallDir;`" + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
    }

    Write-Step "Installed $VersionTag to $destExe"
    if (Test-Path -LiteralPath $destExe) {
        try {
            & $destExe --version
        }
        catch {
            Write-Warn 'The installed binary did not report a version (install itself succeeded).'
        }
    }
    Write-Host 'Independent project, not affiliated with DeepSeek.'
}
catch {
    Write-Warn $_.Exception.Message
    $ExitCode = 1
}
finally {
    if ($Script:TempDir -and (Test-Path -LiteralPath $Script:TempDir)) {
        Remove-Item -LiteralPath $Script:TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

exit $ExitCode
