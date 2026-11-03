$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$root=Join-Path $project 'target\r473-campaign-resource-snapshot'
if(Test-Path $root){throw 'Already snapshotted'}
[IO.Directory]::CreateDirectory($root)|Out-Null
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$sources=@(@{id='lineage381-base';root='D:\380\天堂(Lineage 3.81C) (1)';stem='Text';namespace='ORIGINAL_BASE_TEXT_V1'},@{id='existing-client-base';root='D:\lineage開發\0-客戶端';stem='Text';namespace='ORIGINAL_BASE_TEXT_V1'},@{id='lito-lttext';root='D:\380\天堂(Lineage 3.81C) (1)\LiTo';stem='LTtext';namespace='NATIVE_LITO_LTTEXT_V1'})
$pairs=@(foreach($s in $sources){$dst=Join-Path $root $s.id;[IO.Directory]::CreateDirectory($dst)|Out-Null;$pins=@(foreach($ext in @('idx','pak')){$src=Join-Path $s.root ($s.stem+'.'+$ext);if((Get-Item -LiteralPath $src).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Archive reparse rejected'};$before=Pin $src;$to=Join-Path $dst ($s.stem+'.'+$ext);[IO.File]::Copy($src,$to,$false);$copy=Pin $to;if($before.sha256 -ne $copy.sha256 -or $before.bytes -ne $copy.bytes -or (Get-FileHash -LiteralPath $src).Hash -ne $before.sha256){throw 'Copy drift'};[ordered]@{original=$before;snapshot=$copy}});[ordered]@{id=$s.id;namespace=$s.namespace;original_root=$s.root;root=$dst;stem=$s.stem;archives=$pins}})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r473-resource-snapshot.json') -Value ([ordered]@{state='THREE_COMPLETE_EXISTING_TEXT_ARCHIVE_PAIRS_READONLY_COPIED';root=$root;pairs=$pairs;R439_subset_used=$false;actual_live_loaded_selection=$false;source_roots_modified=$false;runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0})
Write-Output 'Three full Text/LTtext archive pairs fixed; 6 files; original bytes unchanged'
