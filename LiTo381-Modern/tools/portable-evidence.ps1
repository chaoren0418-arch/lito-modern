# Static checks and version lists for the portable player package (tools/package.ps1).
# Everything here only reads files: it never starts the player, opens a network
# connection or writes outside the path it is given. Kept compatible with
# Windows PowerShell 5.1 so the checks can also run where PowerShell 7 is absent.

$script:ModernMinimumWindows = [pscustomobject]@{
    Name = 'Windows 10 version 1703 (build 15063) x64'
    Build = 15063
    Architecture = 'x64'
    Basis = 'Rust std for x86_64-pc-windows-msvc needs Windows 10; the windows use GetDpiForWindow (1607) and per-monitor DPI awareness v2 (1703).'
}
# Byte strings that only exist on the development machine. A player that still
# carries one of them was built from sources with a hardcoded development path.
# Written as code points (this file stays ASCII so Windows PowerShell 5.1 reads it
# the same way): "lineage" + U+958B U+767C, and U+5BA2 U+6236 U+7AEF (the client folder).
$script:ModernClientFolderName = -join [char[]](0x5BA2, 0x6236, 0x7AEF)
$script:ModernDevelopmentMarkers = @(
    ('lineage' + (-join [char[]](0x958B, 0x767C))),
    ('0-' + $script:ModernClientFolderName),
    ($script:ModernClientFolderName + 'xml'),
    'lito-dev-runtime')
# The player links the C runtime statically; none of these may be imported.
$script:ModernRuntimeImportPattern = '^(vcruntime|msvcp|msvcr|concrt|vccorlib|vcomp|mfc|ucrtbase|api-ms-win-crt-)'

function Get-ModernPeInfo([string]$Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    $u16 = { param($o) [int][BitConverter]::ToUInt16($bytes, $o) }
    $u32 = { param($o) [long][BitConverter]::ToUInt32($bytes, $o) }
    if ($bytes.Length -lt 0x100 -or (& $u16 0) -ne 0x5A4D) { throw 'Not a PE image' }
    $pe = [int](& $u32 0x3C)
    if ($pe -le 0 -or $pe + 24 -gt $bytes.Length -or (& $u32 $pe) -ne 0x4550) { throw 'Not a PE image' }
    $sectionCount = & $u16 ($pe + 6)
    $optional = $pe + 24
    $optionalSize = & $u16 ($pe + 20)
    if ((& $u16 $optional) -ne 0x20B) { throw 'Player must be a PE32+ (x64) image' }
    $sections = @(for ($i = 0; $i -lt $sectionCount; $i++) {
        $s = $optional + $optionalSize + 40 * $i
        [pscustomobject]@{ VirtualSize = (& $u32 ($s + 8)); VirtualAddress = (& $u32 ($s + 12)); RawSize = (& $u32 ($s + 16)); RawOffset = (& $u32 ($s + 20)) }
    })
    $offsetOf = {
        param([long]$rva)
        foreach ($s in $sections) {
            $span = [Math]::Max($s.VirtualSize, $s.RawSize)
            if ($rva -ge $s.VirtualAddress -and $rva -lt $s.VirtualAddress + $span) {
                $offset = $rva - $s.VirtualAddress + $s.RawOffset
                if ($offset -ge $bytes.Length) { throw 'PE directory points outside the file' }
                return [int]$offset
            }
        }
        throw 'PE directory RVA is not inside a section'
    }
    $nameAt = {
        param([long]$rva)
        $start = & $offsetOf $rva
        $end = $start
        while ($end -lt $bytes.Length -and $bytes[$end] -ne 0) { $end++ }
        [Text.Encoding]::ASCII.GetString($bytes, $start, $end - $start)
    }
    $directoryCount = & $u32 ($optional + 108)
    $directory = {
        param([int]$index)
        if ($index -ge $directoryCount) { return 0 }
        & $u32 ($optional + 112 + 8 * $index)
    }
    $imports = [Collections.Generic.List[string]]::new()
    $rva = & $directory 1
    if ($rva -ne 0) {
        for ($o = & $offsetOf $rva; ; $o += 20) {
            $name = & $u32 ($o + 12)
            if ($name -eq 0 -and (& $u32 $o) -eq 0 -and (& $u32 ($o + 16)) -eq 0) { break }
            $imports.Add((& $nameAt $name))
        }
    }
    $delayed = [Collections.Generic.List[string]]::new()
    $rva = & $directory 13
    if ($rva -ne 0) {
        for ($o = & $offsetOf $rva; ; $o += 32) {
            $name = & $u32 ($o + 4)
            if ($name -eq 0) { break }
            $delayed.Add((& $nameAt $name))
        }
    }
    [pscustomobject]@{
        Machine = ('0x{0:X4}' -f (& $u16 ($pe + 4)))
        LinkerVersion = ('{0}.{1}' -f $bytes[$optional + 2], $bytes[$optional + 3])
        OperatingSystemVersion = ('{0}.{1}' -f (& $u16 ($optional + 40)), (& $u16 ($optional + 42)))
        SubsystemVersion = ('{0}.{1}' -f (& $u16 ($optional + 48)), (& $u16 ($optional + 50)))
        Subsystem = (& $u16 ($optional + 68))
        Imports = @($imports | Sort-Object -Unique)
        DelayImports = @($delayed | Sort-Object -Unique)
    }
}

function Find-ModernDevelopmentMarkers([string]$Path, [string[]]$Extra = @()) {
    # ISO-8859-1 maps every byte to one char, so an ordinal text search is a byte search.
    $latin = [Text.Encoding]::GetEncoding(28591)
    $image = $latin.GetString([IO.File]::ReadAllBytes($Path))
    $found = foreach ($marker in @($script:ModernDevelopmentMarkers) + @($Extra | Where-Object { $_ })) {
        foreach ($encoding in @([Text.Encoding]::UTF8, [Text.Encoding]::Unicode)) {
            if ($image.IndexOf($latin.GetString($encoding.GetBytes($marker)), [StringComparison]::Ordinal) -ge 0) {
                '{0} ({1})' -f $marker, $encoding.WebName
            }
        }
    }
    @($found)
}

# $BuildPaths: directories of the machine that built the player (for example the
# project root); a player that embeds one still depends on that checkout.
function Assert-ModernPortableExecutable([string]$Path, [string[]]$BuildPaths = @()) {
    $info = Get-ModernPeInfo $Path
    if ($info.Machine -ne '0x8664') { throw "Player is not an x64 image: $($info.Machine)" }
    if ($info.Subsystem -ne 2) { throw "Player is not a Windows GUI image: subsystem $($info.Subsystem)" }
    $runtime = @(@($info.Imports) + @($info.DelayImports) | Where-Object { $_ -match $script:ModernRuntimeImportPattern })
    if ($runtime.Count) { throw "Player still imports a redistributable C/C++ runtime ($($runtime -join ', ')); build with tools/build.ps1 -Mode Release (static C runtime)" }
    $markers = @(Find-ModernDevelopmentMarkers $Path $BuildPaths)
    if ($markers.Count) { throw "Player contains development-machine paths: $($markers -join ', ')" }
    $info
}

function Get-ModernSha256Hex([byte[]]$Bytes) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($sha.ComputeHash($Bytes)).Replace('-', '') } finally { $sha.Dispose() }
}

function Get-ModernResourceManifest([string]$Root) {
    # Read-only inventory of the external client resources this candidate was
    # prepared against. Nothing is copied; reparse points are listed, not followed.
    if (!(Test-Path -LiteralPath $Root -PathType Container)) { throw "Resource root not found: $Root" }
    $rootItem = Get-Item -LiteralPath $Root -Force
    $prefix = $rootItem.FullName.TrimEnd('\', '/')
    $files = [Collections.Generic.List[object]]::new()
    $skipped = [Collections.Generic.List[string]]::new()
    $pending = [Collections.Generic.Stack[IO.DirectoryInfo]]::new()
    $pending.Push([IO.DirectoryInfo]$rootItem)
    $total = [long]0
    while ($pending.Count) {
        $directory = $pending.Pop()
        foreach ($entry in $directory.GetFileSystemInfos()) {
            $relative = $entry.FullName.Substring($prefix.Length + 1).Replace('\', '/')
            if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) { $skipped.Add($relative); continue }
            if ($entry -is [IO.DirectoryInfo]) { $pending.Push($entry); continue }
            $total += $entry.Length
            $files.Add([pscustomobject]@{ Path = $relative; Length = $entry.Length; SHA256 = (Get-FileHash -LiteralPath $entry.FullName -Algorithm SHA256).Hash })
        }
    }
    $ordered = [object[]]$files.ToArray()
    [Array]::Sort([string[]]@($ordered | ForEach-Object Path), $ordered, [StringComparer]::Ordinal)
    $lines = ($ordered | ForEach-Object { '{0} {1} {2}' -f $_.SHA256, $_.Length, $_.Path }) -join "`n"
    [pscustomobject]@{
        Root = $prefix
        FileCount = $ordered.Count
        TotalBytes = $total
        # SHA-256 of the UTF-8 lines "<SHA256> <Length> <Path>" joined by LF, ordinal path order.
        ListSHA256 = (Get-ModernSha256Hex ([Text.Encoding]::UTF8.GetBytes($lines)))
        SkippedReparsePoints = @($skipped)
        Files = $ordered
    }
}

function Copy-ModernPortableEntry([string]$Source, [string]$Destination) {
    # cmd.exe reads batch files in the OEM code page and needs CRLF line ends:
    # keep the entry pure ASCII so it behaves the same under any system locale.
    $text = [IO.File]::ReadAllText($Source)
    if ($text -match '[^\x09\x0A\x0D\x20-\x7E]') { throw 'Portable entry must be pure ASCII' }
    $text = ($text -replace "`r`n", "`n") -replace "`n", "`r`n"
    [IO.File]::WriteAllText($Destination, $text, [Text.Encoding]::ASCII)
}

function New-ModernPortableZip([string]$Directory, [string]$ZipPath) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    if (Test-Path -LiteralPath $ZipPath) { throw 'Portable ZIP already exists; packaging never overwrites an earlier candidate' }
    # The candidate directory is the single top-level folder of the archive.
    [IO.Compression.ZipFile]::CreateFromDirectory($Directory, $ZipPath, [IO.Compression.CompressionLevel]::Optimal, $true, [Text.Encoding]::UTF8)
    Get-Item -LiteralPath $ZipPath
}
