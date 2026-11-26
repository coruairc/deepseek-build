#Requires -Version 5.1
<#
.SYNOPSIS
    Local test harness for install.ps1.

.DESCRIPTION
    Builds a fake release tree (a fake deepseek-build.exe, the four top-level
    files, the .zip and SHA256SUMS), points install.ps1 at it with the
    DSB_INSTALL_BASE_URL test hook (a file:// URL - no server, no network), and
    runs install.ps1 through its paths:

      1. success (binary installed, non-affiliation line, no PATH hint when the
         install dir is on PATH)
      2. default "latest" resolution via latest/download/SHA256SUMS
      3. pinned stable release, with and without the leading "v"
      4. PATH hint when the install dir is not on PATH
      5. checksum mismatch (must abort, install nothing)
      6. unsupported architecture (only x86_64 is supported)
      7. uninstall (removes the binary; idempotent; no network)
      8. invalid VERSION (rejected before any download)

    install.ps1 calls `exit`, so every invocation runs in a child PowerShell
    process; running it in-process would terminate this harness.

    Run: powershell -ExecutionPolicy Bypass -File scripts/test-install.ps1
         (or: pwsh -File scripts/test-install.ps1)

    The fake binary is a text file: this harness proves the install/uninstall
    plumbing, not that a real PE reports its version. install.ps1 treats a
    binary that cannot report a version as a warning, not a failure.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$InstallPs = Join-Path $Root 'install.ps1'
$TempRoot = [System.IO.Path]::GetTempPath()
$Target = 'x86_64-pc-windows-msvc'
$FakeVersion = '0.0.0-test'
$FakeVersion2 = '0.0.1-test'
$StableVersion = '0.1.0'

$PowerShellExe = (Get-Process -Id $PID).Path
if (-not $PowerShellExe) {
    $PowerShellExe = Join-Path $PSHOME 'powershell.exe'
}

$script:Pass = 0
$script:Fail = 0
$script:FailedNames = @()
$script:Sandboxes = @()

# ---------------------------------------------------------------------------
# Harness helpers
# ---------------------------------------------------------------------------

function Say {
    param([string]$Message)
    Write-Host $Message
}

function Add-Pass {
    param([string]$Name)
    $script:Pass++
    Say "PASS: $Name"
}

function Add-Fail {
    param([string]$Name, [string]$Detail = '')
    $script:Fail++
    $script:FailedNames += $Name
    Say "FAIL: $Name"
    if ($Detail) {
        Say "      detail: $Detail"
    }
}

function Assert-Contains {
    param([string]$Text, [string]$Needle, [string]$Name)
    if ($Text -and $Text.Contains($Needle)) {
        Add-Pass $Name
    }
    else {
        Add-Fail $Name "output missing: $Needle"
    }
}

function Assert-NotContains {
    param([string]$Text, [string]$Needle, [string]$Name)
    if ($Text -and $Text.Contains($Needle)) {
        Add-Fail $Name "output unexpectedly contains: $Needle"
    }
    else {
        Add-Pass $Name
    }
}

function Assert-True {
    param([bool]$Condition, [string]$Name, [string]$Detail = '')
    if ($Condition) {
        Add-Pass $Name
    }
    else {
        Add-Fail $Name $Detail
    }
}

# ---------------------------------------------------------------------------
# Fake release tree
# ---------------------------------------------------------------------------

function New-Release {
    param([string]$Tree, [string]$Version, [switch]$Corrupt)
    $releaseDir = Join-Path $Tree (Join-Path 'download' "v$Version")
    New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null

    $stage = Join-Path $Tree "stage-$Version"
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
    Set-Content -LiteralPath (Join-Path $stage 'deepseek-build.exe') -Value "fake binary $Version" -Encoding Ascii
    Set-Content -LiteralPath (Join-Path $stage 'LICENSE') -Value 'license' -Encoding Ascii
    Set-Content -LiteralPath (Join-Path $stage 'NOTICE') -Value 'notice' -Encoding Ascii
    Set-Content -LiteralPath (Join-Path $stage 'THIRD-PARTY-NOTICES') -Value 'third-party' -Encoding Ascii
    Set-Content -LiteralPath (Join-Path $stage 'README.txt') -Value 'readme' -Encoding Ascii

    $fileName = "deepseek-build-$Version-$Target.zip"
    $zipPath = Join-Path $releaseDir $fileName
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zipPath -Force
    $hash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -LiteralPath (Join-Path $releaseDir 'SHA256SUMS') -Value "$hash  $fileName" -Encoding Ascii

    if ($Corrupt) {
        # Flip one byte after the sums file was written, so the recorded hash
        # no longer matches the file.
        $bytes = [System.IO.File]::ReadAllBytes($zipPath)
        $middle = [int]($bytes.Length / 2)
        $bytes[$middle] = $bytes[$middle] -bxor 0xFF
        [System.IO.File]::WriteAllBytes($zipPath, $bytes)
    }
}

function Build-FakeTree {
    param([string]$Tree, [switch]$Corrupt)
    New-Item -ItemType Directory -Force -Path (Join-Path $Tree 'latest\download') | Out-Null
    New-Release -Tree $Tree -Version $FakeVersion -Corrupt:$Corrupt
    New-Release -Tree $Tree -Version $FakeVersion2
    New-Release -Tree $Tree -Version $StableVersion
    # The "latest" release points at the newest version (v2), deliberately not
    # at the stable 0.1.0, so the latest assertions pin v0.0.1-test.
    Copy-Item -Path (Join-Path $Tree "download\v$FakeVersion2\*") -Destination (Join-Path $Tree 'latest\download') -Force
}

function Get-TreeUrl {
    param([string]$Tree)
    return ([System.Uri]([System.IO.Path]::GetFullPath($Tree))).AbsoluteUri
}

function New-Sandbox {
    param([string]$Name)
    $home = Join-Path $TempRoot ("dsb-test-$Name-" + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $home | Out-Null
    $script:Sandboxes += $home
    return $home
}

# ---------------------------------------------------------------------------
# install.ps1 runner (child process: install.ps1 calls `exit`)
# ---------------------------------------------------------------------------

function Invoke-Installer {
    param(
        [string]$Home,
        [string]$InstallDir,
        [string]$Version = '',
        [string]$BaseUrl = '',
        [string]$Arch = '',
        [string]$PathValue = '',
        [switch]$Uninstall
    )
    $arguments = "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$InstallPs`""
    if ($Uninstall) {
        $arguments = $arguments + ' -Uninstall'
    }

    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = $PowerShellExe
    $startInfo.Arguments = $arguments
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.CreateNoWindow = $true
    $startInfo.EnvironmentVariables['LOCALAPPDATA'] = $Home
    $startInfo.EnvironmentVariables['USERPROFILE'] = $Home
    $startInfo.EnvironmentVariables['INSTALL_DIR'] = $InstallDir
    $startInfo.EnvironmentVariables['VERSION'] = $Version
    $startInfo.EnvironmentVariables['DSB_INSTALL_BASE_URL'] = $BaseUrl
    $startInfo.EnvironmentVariables['DSB_INSTALL_TEST_ARCH'] = $Arch
    if ($PathValue) {
        $startInfo.EnvironmentVariables['Path'] = $PathValue
    }

    $process = [System.Diagnostics.Process]::Start($startInfo)
    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    return [pscustomobject]@{
        ExitCode = $process.ExitCode
        Output   = $stdout + $stderr
    }
}

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

if (-not (Test-Path -LiteralPath $InstallPs)) {
    Say "FAIL: install.ps1 not found at $InstallPs"
    exit 1
}

$Tree = Join-Path $TempRoot ('dsb-test-tree-' + [guid]::NewGuid().ToString('N'))
$BadTree = Join-Path $TempRoot ('dsb-test-treebad-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $Tree, $BadTree | Out-Null
Build-FakeTree -Tree $Tree
Build-FakeTree -Tree $BadTree -Corrupt
$TreeUrl = Get-TreeUrl $Tree
$BadTreeUrl = Get-TreeUrl $BadTree

Say "=== install.ps1 test suite (fake release tree: $TreeUrl) ==="

try {
    # -----------------------------------------------------------------------
    # Case 1: success + no PATH hint when the install dir is on PATH
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'success'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version $FakeVersion -BaseUrl $TreeUrl -PathValue $bin
    Assert-True ($result.ExitCode -eq 0) 'success: exit 0' "exit $($result.ExitCode)"
    Assert-True (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe')) 'success: binary installed'
    Assert-Contains $result.Output 'Independent project, not affiliated with DeepSeek.' 'success: non-affiliation line'
    Assert-Contains $result.Output "Installed v$FakeVersion to" 'success: installed tag reported'
    Assert-NotContains $result.Output 'not in your PATH' 'success: no PATH hint when dir is on PATH'

    # -----------------------------------------------------------------------
    # Case 2: default "latest" resolution
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'latest'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -BaseUrl $TreeUrl -PathValue $bin
    Assert-True ($result.ExitCode -eq 0) 'latest: exit 0' "exit $($result.ExitCode)"
    Assert-True (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe')) 'latest: binary installed'
    Assert-Contains $result.Output "Latest release: v$FakeVersion2" 'latest: resolved newest tag'
    Assert-Contains $result.Output "Downloading deepseek-build-$FakeVersion2-$Target.zip" 'latest: downloaded the newest zip'

    # -----------------------------------------------------------------------
    # Case 3: pinned stable release, both VERSION forms
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'stable'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version $StableVersion -BaseUrl $TreeUrl -PathValue $bin
    Assert-True (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe')) 'stable: binary installed (no v prefix)'
    Assert-Contains $result.Output "Downloading deepseek-build-$StableVersion-$Target.zip" 'stable: tarball-equivalent zip name'

    $home = New-Sandbox 'stablev'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version "v$StableVersion" -BaseUrl $TreeUrl -PathValue $bin
    Assert-True (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe')) 'stable: binary installed (v prefix)'
    Assert-Contains $result.Output "Installed v$StableVersion to" 'stable: installed tag reported'

    # -----------------------------------------------------------------------
    # Case 4: PATH hint when the install dir is not on PATH
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'hint'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version $FakeVersion -BaseUrl $TreeUrl -PathValue "$env:SystemRoot\System32"
    Assert-Contains $result.Output 'not in your PATH' 'path-hint: hint printed when dir not on PATH'

    # -----------------------------------------------------------------------
    # Case 5: checksum mismatch - abort, install nothing
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'mismatch'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version $FakeVersion -BaseUrl $BadTreeUrl -PathValue $bin
    Assert-True ($result.ExitCode -ne 0) 'checksum-mismatch: nonzero exit'
    Assert-Contains $result.Output 'Checksum mismatch' 'checksum-mismatch: mismatch message'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe'))) 'checksum-mismatch: nothing installed'

    # -----------------------------------------------------------------------
    # Case 6: unsupported architecture
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'badarch'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version $FakeVersion -BaseUrl $TreeUrl -Arch 'ARM64' -PathValue $bin
    Assert-True ($result.ExitCode -ne 0) 'unsupported-arch: nonzero exit'
    Assert-Contains $result.Output 'Unsupported architecture: ARM64' 'unsupported-arch: clear error'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe'))) 'unsupported-arch: nothing installed'

    # -----------------------------------------------------------------------
    # Case 7: uninstall
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'uninstall'
    $bin = Join-Path $home 'bin'
    $null = Invoke-Installer -Home $home -InstallDir $bin -Version $FakeVersion -BaseUrl $TreeUrl -PathValue $bin
    Assert-True (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe')) 'uninstall: setup installed the binary'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Uninstall
    Assert-True ($result.ExitCode -eq 0 -and -not (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe'))) 'uninstall: removes the binary'
    # Point the base URL at a tree that does not exist: any download attempt would fail loudly
    # (and print "Downloading"), so a clean exit proves uninstall never reaches the network.
    $result = Invoke-Installer -Home $home -InstallDir $bin -Uninstall -BaseUrl 'file:///nonexistent-deepseek-build-uninstall-probe'
    Assert-True ($result.ExitCode -eq 0) 'uninstall: idempotent when absent'
    Assert-NotContains $result.Output 'Downloading' 'uninstall: no download attempted (dead base URL)'
    Assert-NotContains $result.Output 'github.com' 'uninstall: no URL in output'

    # -----------------------------------------------------------------------
    # Case 8: invalid VERSION is rejected before any download
    # -----------------------------------------------------------------------

    $home = New-Sandbox 'invalid'
    $bin = Join-Path $home 'bin'
    $result = Invoke-Installer -Home $home -InstallDir $bin -Version 'not-a-version' -BaseUrl $TreeUrl -PathValue $bin
    Assert-True ($result.ExitCode -ne 0) 'invalid-version: nonzero exit'
    Assert-Contains $result.Output 'Invalid VERSION: not-a-version' 'invalid-version: clear message'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $bin 'deepseek-build.exe'))) 'invalid-version: nothing installed'
}
finally {
    foreach ($sandbox in $script:Sandboxes) {
        Remove-Item -LiteralPath $sandbox -Recurse -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $Tree -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $BadTree -Recurse -Force -ErrorAction SilentlyContinue
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

Say ''
if ($script:Fail -eq 0) {
    Say "ALL PASS ($($script:Pass)/$($script:Pass) assertions)"
    exit 0
}
else {
    Say "FAILED: $($script:Fail) of $($script:Pass + $script:Fail) assertions ($($script:FailedNames -join ', '))"
    exit 1
}
