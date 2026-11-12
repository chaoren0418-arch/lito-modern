# Windowless checks of a portable ZIP made by tools/package.ps1 -Zip.
#
# It extracts the ZIP, compares every file with SHA256.json, statically checks
# the player image, then starts ONLY the extracted LiToModern.exe with
# --version and --check-native-config (both open no window, no network and
# write no settings) and the double-click entry with --version.
# It does not show the first-run window, log in or load resources: those stay
# with the clean-host acceptance. Compatible with Windows PowerShell 5.1, so a
# clean host needs nothing installed (copy this file and portable-evidence.ps1).
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Zip,
    # New or empty directory to extract into; use one with non-ASCII characters and spaces.
    [Parameter(Mandatory)][string]$ExtractRoot,
    # Existing readable directory used as the resource root of the settings check.
    [Parameter(Mandatory)][string]$ResourceRoot
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'portable-evidence.ps1')
foreach ($path in @($Zip, $ExtractRoot, $ResourceRoot)) { if (![IO.Path]::IsPathRooted($path)) { throw "Use absolute paths: $path" } }
if (!(Test-Path -LiteralPath $Zip -PathType Leaf)) { throw "ZIP not found: $Zip" }
if (!(Test-Path -LiteralPath $ResourceRoot -PathType Container)) { throw "Resource root not found: $ResourceRoot" }
if ((Test-Path -LiteralPath $ExtractRoot) -and @(Get-ChildItem -LiteralPath $ExtractRoot -Force).Count) { throw 'ExtractRoot must be new or empty' }

function Invoke-Packaged([string]$File, [string]$Arguments, [string]$WorkingDirectory) {
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $File
    $start.Arguments = $Arguments
    $start.WorkingDirectory = $WorkingDirectory
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = [Text.Encoding]::UTF8
    $start.StandardErrorEncoding = [Text.Encoding]::UTF8
    # Only the system directories: nothing from a developer tool or SDK may be found by accident.
    $start.EnvironmentVariables['PATH'] = "$env:SystemRoot\System32;$env:SystemRoot"
    $process = [Diagnostics.Process]::Start($start)
    try {
        $process.StandardInput.Close()
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(30000)) {
            # Only the process created three lines above; never an existing player.
            $process.Kill()
            $process.WaitForExit()
            throw "Timed out: $File $Arguments"
        }
        [pscustomobject]@{ ExitCode = $process.ExitCode; Output = $stdout.Result; Errors = $stderr.Result }
    } finally { $process.Dispose() }
}

$null = New-Item -ItemType Directory -Path $ExtractRoot -Force
Expand-Archive -LiteralPath $Zip -DestinationPath $ExtractRoot
$top = @(Get-ChildItem -LiteralPath $ExtractRoot -Force)
if ($top.Count -ne 1 -or !$top[0].PSIsContainer) { throw 'ZIP must hold exactly one top-level folder' }
$package = $top[0].FullName
$exe = Join-Path $package 'LiToModern.exe'
$entry = Join-Path $package 'Start-LiToModern.cmd'

# 1. Every listed file is present and identical; nothing else was shipped.
$list = Get-Content -LiteralPath (Join-Path $package 'SHA256.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$present = @(Get-ChildItem -LiteralPath $package -File -Recurse -Force | ForEach-Object { $_.FullName.Substring($package.Length + 1).Replace('\', '/') } | Where-Object { $_ -cne 'SHA256.json' } | Sort-Object)
$listed = @($list.Files | ForEach-Object Path | Sort-Object)
if (($present -join '|') -cne ($listed -join '|')) { throw "Package files differ from SHA256.json: [$($present -join ', ')]" }
foreach ($file in $list.Files) {
    $item = Get-Item -LiteralPath (Join-Path $package $file.Path)
    if ($item.Length -ne $file.Length -or (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash -cne $file.SHA256) { throw "Hash mismatch: $($file.Path)" }
}
if (@($present | Where-Object { $_ -match '\.(dll|bin|pak|idx|spr|til)$' }).Count) { throw 'Package holds a DLL, native BIN or resource archive' }
foreach ($name in @('source', 'executable', 'resources')) {
    $null = Get-Content -LiteralPath (Join-Path $package "manifest/$name.json") -Raw -Encoding UTF8 | ConvertFrom-Json
}

# 2. Static image check: x64 GUI image, no redistributable runtime import, no development path.
$image = Assert-ModernPortableExecutable $exe

# 3. --version from an unrelated working directory with a system-only PATH.
$version = Invoke-Packaged $exe '--version' $env:SystemRoot
if ($version.ExitCode -ne 0 -or ![string]::IsNullOrWhiteSpace($version.Errors)) { throw "--version failed ($($version.ExitCode)): $($version.Errors)" }
$versionDocument = $version.Output | ConvertFrom-Json
if ($versionDocument.application -cne 'LiToModern' -or $versionDocument.build -cne 'release' -or $versionDocument.version -cne $list.Version.version -or
    $versionDocument.launches_native_bin -ne $false -or $versionDocument.pointer_bits -ne 64) { throw "Unexpected --version document: $($version.Output)" }

# 4. Settings stored under the extracted (non-ASCII) path are read back without a window or connection.
$checkDirectory = Join-Path $ExtractRoot 'settings-check'
$null = New-Item -ItemType Directory -Path $checkDirectory
$settingsPath = Join-Path $checkDirectory 'LiToModern.native.json'
$settings = [ordered]@{ schema = 1; login_endpoint = '192.0.2.10:32000'; resource_root = (Get-Item -LiteralPath $ResourceRoot).FullName.TrimEnd('\') }
[IO.File]::WriteAllText($settingsPath, ($settings | ConvertTo-Json), (New-Object Text.UTF8Encoding $false))
$check = Invoke-Packaged $exe ('--check-native-config "{0}"' -f $settingsPath) $env:SystemRoot
if ($check.ExitCode -ne 0) { throw "--check-native-config failed ($($check.ExitCode)): $($check.Output) $($check.Errors)" }
$checkDocument = $check.Output | ConvertFrom-Json
if ($checkDocument.status -cne 'valid_native_settings' -or $checkDocument.network_started -ne $false -or $checkDocument.settings_written -ne $false) { throw "Unexpected settings check: $($check.Output)" }

# 5. The double-click entry finds the player beside it; alone it reports the incomplete extraction.
$command = Join-Path $env:SystemRoot 'System32\cmd.exe'
$started = Invoke-Packaged $command ('/d /c ""{0}" --version"' -f $entry) $env:SystemRoot
if ($started.ExitCode -ne 0) { throw "Entry failed beside the player ($($started.ExitCode)): $($started.Output) $($started.Errors)" }
$aloneDirectory = Join-Path $ExtractRoot 'entry-alone'
$null = New-Item -ItemType Directory -Path $aloneDirectory
Copy-Item -LiteralPath $entry -Destination $aloneDirectory
$alone = Invoke-Packaged $command ('/d /c ""{0}""' -f (Join-Path $aloneDirectory 'Start-LiToModern.cmd')) $env:SystemRoot
if ($alone.ExitCode -ne 1 -or $alone.Output -notmatch 'LiToModern\.exe was not found') { throw "Entry without the player must stop with exit code 1 ($($alone.ExitCode)): $($alone.Output)" }

# 6. None of the checks may have created first-run settings beside the player.
if (Test-Path -LiteralPath (Join-Path $package 'LiToModern.native.json')) { throw 'A check wrote settings beside the player' }

[pscustomobject]@{
    Package = $package
    Files = $listed.Count + 1
    SourceCommit = $list.SourceCommit
    ExecutableSHA256 = $list.ExecutableSHA256
    Imports = $image.Imports
    DelayImports = $image.DelayImports
    LinkerVersion = $image.LinkerVersion
    Version = $versionDocument.version
    SettingsCheck = $checkDocument.status
    EntryBesidePlayerExitCode = $started.ExitCode
    EntryAloneExitCode = $alone.ExitCode
    NotChecked = @('first-run settings window', 'double-click from Explorer', 'login and gameplay', 'resource loading', 'a host without any Visual C++ runtime installed (only the import table proves it here)')
}
