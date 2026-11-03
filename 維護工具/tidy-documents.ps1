#Requires -Version 7.0
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskEvidence=Join-Path $taskRoot '驗證紀錄\20261012'
$receipt=Get-Content -LiteralPath (Join-Path $taskEvidence 'archive-receipt.json') -Raw|ConvertFrom-Json
if(!$receipt.verified -or (Get-FileHash -LiteralPath $receipt.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $receipt.sha256){throw 'Verified archive required'}
$zip=[IO.Compression.ZipFile]::OpenRead($receipt.path)
try {
    $reader=[IO.StreamReader]::new($zip.GetEntry('manifest.json').Open())
    try{$manifest=$reader.ReadToEnd()|ConvertFrom-Json}finally{$reader.Dispose()}
    $pins=@{};foreach($entry in $manifest){$pins[$entry.path]=$entry}
    $project=Join-Path $taskRoot 'LiTo381-Modern'
    $files=@(Get-ChildItem -LiteralPath $project -File -Filter 'Native-R*.txt' -Force)
    $files+=@(Get-ChildItem -LiteralPath (Join-Path $project 'project') -File -Filter 'Native-R*.txt' -Force)
    # Markdown reports are preserved in the archive; JSON/raw reverse evidence stays in place.
    $files+=@(Get-ChildItem -LiteralPath (Join-Path $project 'reports') -File -Filter '*.md' -Force)
    $plan=@()
    foreach($file in $files){
        $absolute=[IO.Path]::GetFullPath($file.FullName)
        if(!$absolute.StartsWith($project+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Outside project'}
        $part=$file.Directory;while($null -ne $part){if($part.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse ancestor'};if($part.FullName -eq $taskRoot){break};$part=$part.Parent}
        if($file.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse file'}
        $relative=[IO.Path]::GetRelativePath($taskRoot,$absolute)
        $pin=$pins[$relative]
        if(!$pin -or $pin.sha256 -ne (Get-FileHash -LiteralPath $absolute -Algorithm SHA256).Hash.ToLowerInvariant()){throw "Archive mismatch: $relative"}
        $plan+=[pscustomobject]@{path=$absolute;relative=$relative;bytes=$file.Length;sha256=$pin.sha256}
    }
    $plan|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $taskEvidence 'document-cleanup-plan.json') -Encoding utf8
    foreach($file in $plan){Remove-Item -LiteralPath $file.path -Force}
    [ordered]@{files=$plan.Count;bytes=($plan|Measure-Object bytes -Sum).Sum;archive=$receipt.path;hashes_verified=$true}|
        ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskEvidence 'document-cleanup-receipt.json') -Encoding utf8
}finally{$zip.Dispose()}
Get-Content -LiteralPath (Join-Path $taskEvidence 'document-cleanup-receipt.json')
