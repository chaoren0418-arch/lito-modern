#Requires -Version 7.0
# Local provenance/staleness checks, not a signature or an adversarial trust boundary.
function Get-ModernBuildInputs([string]$Project) {
    $Project = [IO.Path]::GetFullPath($Project)
    $items = @(Get-Item -LiteralPath $Project)
    foreach ($name in @('src','windows','.cargo','config','packaging')) {
        $root = Get-Item -LiteralPath (Join-Path $Project $name)
        if (!$root.PSIsContainer -or ($root.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe build input directory' }
        $items += $root
        $items += @(Get-ChildItem -LiteralPath $root.FullName -Recurse -Force)
    }
    foreach ($name in @('Cargo.toml','Cargo.lock','build.rs','tools/build.ps1','tools/build-evidence.ps1','tools/package.ps1','tools/verify-independent.ps1')) {
        $items += Get-Item -LiteralPath (Join-Path $Project $name)
    }
    if (@($items | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw 'Reparse point in build inputs' }
    $items | Where-Object { !$_.PSIsContainer } | Sort-Object FullName -Unique | ForEach-Object {
        [pscustomobject]@{Path=[IO.Path]::GetRelativePath($Project,$_.FullName);Length=$_.Length;SHA256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
    }
}
function Get-ModernInputDigest([object[]]$Inputs) {
    $json = ConvertTo-Json -InputObject $Inputs -Depth 5 -Compress
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($json)))
}
function Assert-ModernVersion($Version, [ValidateSet('Debug','Release')][string]$Mode) {
    if ($null -eq $Version -or $Version.schema -ne 1 -or $Version.application -cne 'LiToModern' -or
        $Version.build -cne $Mode.ToLowerInvariant() -or $Version.status -cne 'development_candidate' -or
        [string]::IsNullOrWhiteSpace($Version.version) -or
        @($Version.profile_protocols).Count -ne 2 -or 'legacy_v37' -cnotin $Version.profile_protocols -or 'modern_v47' -cnotin $Version.profile_protocols -or
        @($Version.extensions).Count -ne 5 -or 'player_healing_v1' -cnotin $Version.extensions -or
        'armor_protection_v1' -cnotin $Version.extensions -or 'player_healing_armor_v1' -cnotin $Version.extensions -or
        'armor_presentation_v1' -cnotin $Version.extensions -or 'player_healing_armor_presentation_v1' -cnotin $Version.extensions -or
        $Version.launches_native_bin -isnot [bool] -or $Version.launches_native_bin -ne $false -or
        $Version.launches_tw -isnot [bool] -or $Version.launches_tw -ne $false) { throw 'Unexpected player version/contract diagnostic' }
}
function Get-ModernPlayerVersion([string]$Executable, [ValidateSet('Debug','Release')][string]$Mode) {
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Executable
    $start.ArgumentList.Add('--version')
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        if (!$process.Start()) { throw 'Cannot start player version diagnostic' }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(10000)) {
            # Only the diagnostic process just created here; never an existing player.
            $process.Kill()
            $process.WaitForExit()
            throw 'Player version diagnostic timed out'
        }
        $output = $stdout.GetAwaiter().GetResult()
        $errors = $stderr.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0 -or $output.Length -gt 4096 -or ![string]::IsNullOrWhiteSpace($errors)) { throw 'Player version diagnostic failed' }
        $version = $output | ConvertFrom-Json
        Assert-ModernVersion $version $Mode
        $version
    } finally { $process.Dispose() }
}
function Get-ModernPlayerPath([string]$Project, [ValidateSet('Debug','Release')][string]$Mode) {
    foreach($relative in @('target','target/current','target/current/x86_64-pc-windows-msvc',('target/current/x86_64-pc-windows-msvc/' + $Mode.ToLowerInvariant()))) {
        $directory = Get-Item -LiteralPath (Join-Path $Project $relative) -ErrorAction SilentlyContinue
        if ($null -ne $directory -and (!$directory.PSIsContainer -or ($directory.Attributes -band [IO.FileAttributes]::ReparsePoint))) { throw 'Unsafe player output directory' }
    }
    Join-Path $Project ('target/current/x86_64-pc-windows-msvc/' + $Mode.ToLowerInvariant() + '/LiToModern.exe')
}
function Write-ModernBuildReceipt([string]$Project, [ValidateSet('Debug','Release')][string]$Mode, [object[]]$Before, [string]$Log) {
    $after = @(Get-ModernBuildInputs $Project)
    $digest = Get-ModernInputDigest $Before
    if ($digest -cne (Get-ModernInputDigest $after)) { throw 'Build inputs changed while compiling; no receipt issued' }
    $exe = Get-Item -LiteralPath (Get-ModernPlayerPath $Project $Mode)
    if ($exe.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Player executable is a reparse point' }
    $hash = (Get-FileHash -LiteralPath $exe.FullName -Algorithm SHA256).Hash
    $version = Get-ModernPlayerVersion $exe.FullName $Mode
    if ($hash -cne (Get-FileHash -LiteralPath $exe.FullName -Algorithm SHA256).Hash -or $digest -cne (Get-ModernInputDigest @(Get-ModernBuildInputs $Project))) { throw 'Build output or inputs changed during diagnostic' }
    $receipt = [pscustomobject]@{
        Schema=1;Mode=$Mode;Target='x86_64-pc-windows-msvc';CreatedUtc=[DateTime]::UtcNow.ToString('o');BuildLog=$Log
        Executable=[IO.Path]::GetRelativePath($Project,$exe.FullName);Length=$exe.Length;SHA256=$hash
        InputSHA256=$digest;Inputs=$after;Version=$version
    }
    $path = Join-Path $Project ('target/player-' + $Mode.ToLowerInvariant() + '.build.json')
    $receipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $path -Encoding utf8
    $path
}
function Assert-ModernBuildReceipt([string]$Project, [ValidateSet('Debug','Release')][string]$Mode) {
    $path = Join-Path $Project ('target/player-' + $Mode.ToLowerInvariant() + '.build.json')
    if (!(Test-Path -LiteralPath $path -PathType Leaf)) { throw 'Missing successful player build receipt; run tools/build.ps1 first' }
    $receipt = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    $exe = Get-Item -LiteralPath (Get-ModernPlayerPath $Project $Mode)
    if ($receipt.Schema -ne 1 -or $receipt.Mode -cne $Mode -or $receipt.Target -cne 'x86_64-pc-windows-msvc' -or
        $receipt.Executable -cne [IO.Path]::GetRelativePath($Project,$exe.FullName) -or
        ($exe.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $receipt.Length -ne $exe.Length -or
        $receipt.SHA256 -cne (Get-FileHash -LiteralPath $exe.FullName -Algorithm SHA256).Hash) { throw 'Player build receipt does not match executable' }
    $inputs = @(Get-ModernBuildInputs $Project)
    if ($receipt.InputSHA256 -cne (Get-ModernInputDigest @($receipt.Inputs)) -or
        $receipt.InputSHA256 -cne (Get-ModernInputDigest $inputs)) { throw 'Player build receipt is stale; rebuild current sources before packaging' }
    Assert-ModernVersion $receipt.Version $Mode
    $receipt
}
