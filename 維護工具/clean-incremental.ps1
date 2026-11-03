#Requires -Version 7.0
[CmdletBinding()]param()
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskEvidence=Join-Path $taskRoot '驗證紀錄\20261012'
$taskTarget=[IO.Path]::GetFullPath((Join-Path $taskRoot 'LiTo381-Modern\target'))
function Assert-Confined([string]$Path) {
    $absolute=[IO.Path]::GetFullPath($Path)
    if(!$absolute.StartsWith($taskTarget+'\',[StringComparison]::OrdinalIgnoreCase)) {throw "Outside target: $absolute"}
    $part=Get-Item -LiteralPath $absolute -Force
    while($null -ne $part) {
        if($part.Attributes -band [IO.FileAttributes]::ReparsePoint){throw "Reparse refused: $($part.FullName)"}
        if($part.FullName -eq $taskRoot){break}
        $part=$part.Parent
    }
    return $absolute
}
$receipt=Get-Content -LiteralPath (Join-Path $taskEvidence 'archive-receipt.json') -Raw|ConvertFrom-Json
if(!$receipt.verified -or !(Test-Path -LiteralPath $receipt.path)){throw 'Verified source archive required'}
if((Get-FileHash -LiteralPath $receipt.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $receipt.sha256){throw 'Archive hash drift'}
$inventory=Get-Content -LiteralPath (Join-Path $taskEvidence 'inventory-before.json') -Raw|ConvertFrom-Json
$plan=@()
foreach($property in $inventory.incremental_candidates.PSObject.Properties) {
    $path=Assert-Confined $property.Name
    if($path.StartsWith((Join-Path $taskTarget 'current')+'\',[StringComparison]::OrdinalIgnoreCase)){continue}
    if([IO.Path]::GetFileName($path) -ne 'incremental'){throw 'Not an incremental cache'}
    $plan+=[pscustomobject]@{path=$path;before_bytes=$property.Value.bytes;before_files=$property.Value.files}
}
$running=@(Get-CimInstance Win32_Process|Where-Object { $_.Name -in @('cargo.exe','rustc.exe') -and $_.CommandLine -like '*lito重建版*' })
foreach($candidate in $plan){
    foreach($process in $running){if($process.CommandLine -like "*$($candidate.path)*"){throw 'Active build cache'}}
    $items=@(Get-ChildItem -LiteralPath $candidate.path -Recurse -Force)
    if(@($items|Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'Nested reparse refused'}
    $unexpected=@($items|Where-Object {!$_.PSIsContainer -and $_.Name -notmatch '\.(bin|o|obj|lock|rmeta)$'})
    if($unexpected.Count){throw "Unexpected incremental content: $($unexpected[0].FullName)"}
    $candidate|ConvertTo-Json -Compress|Add-Content -LiteralPath (Join-Path $taskEvidence 'incremental-cleanup-plan.jsonl') -Encoding utf8
}
$bytes=[long]0;$files=[long]0
foreach($candidate in $plan){
    $path=Assert-Confined $candidate.path
    # All absolute targets and their complete contents were checked above.
    Remove-Item -LiteralPath $path -Recurse -Force
    if(Test-Path -LiteralPath $path){throw "Removal incomplete: $path"}
    $bytes+=$candidate.before_bytes;$files+=$candidate.before_files
    $candidate|ConvertTo-Json -Compress|Add-Content -LiteralPath (Join-Path $taskEvidence 'incremental-cleanup-done.jsonl') -Encoding utf8
}
[ordered]@{root=$taskRoot;archive=$receipt.path;cache_directories=$plan.Count;deleted_bytes=$bytes;deleted_files=$files;current_cache_preserved=$true;resources_deleted=0;database_files_deleted=0}|
    ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskEvidence 'incremental-cleanup-receipt.json') -Encoding utf8
Get-Content -LiteralPath (Join-Path $taskEvidence 'incremental-cleanup-receipt.json')
