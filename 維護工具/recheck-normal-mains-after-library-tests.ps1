#Requires -Version 7.0
[CmdletBinding()]param(
 [string]$SourceManifest='驗證紀錄/current/native-teleport-wire/targeted-original-tests-v4/source-inputs.json',
 [ValidatePattern('^[a-z0-9-]+$')][string]$ReceiptLabel='normal-after-library-test-fixes'
)
$ErrorActionPreference='Stop'
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
$stage=Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire';$output=Join-Path $stage $ReceiptLabel
if(Test-Path -LiteralPath $output){throw 'Existing normal build run must be preserved'}
$manifest=[IO.Path]::GetFullPath((Join-Path $taskRoot $SourceManifest));if(!$manifest.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned source manifest required'}
$pins=Get-Content -LiteralPath $manifest -Raw|ConvertFrom-Json -AsHashtable
function Check-Inputs {foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw "Source drift: $name"}}}
Check-Inputs
New-Item -ItemType Directory -Path $output|Out-Null
Copy-Item -LiteralPath $manifest -Destination (Join-Path $output 'source-inputs.json')
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $output 'runner.ps1')
$tools='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$env:PATH=$tools+';'+$env:PATH;$env:RUSTC=Join-Path $tools 'rustc.exe';$env:RUSTDOC=Join-Path $tools 'rustdoc.exe'
$receipt=@{status='building_via_unique_entry';started_utc=[DateTime]::UtcNow.ToString('o');source_files=$pins.Count;source_manifest_sha256=(Get-FileHash -LiteralPath $manifest).Hash;features=@();profiles=@{};whole_goal_complete=$false}
function Save {$receipt|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
try{
 foreach($profile in @('Debug','Release')){
  $receipt.active_profile=$profile;Save;Check-Inputs
  & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $profile *> (Join-Path $output ($profile+'-entry.log'))
  if($LASTEXITCODE -ne 0){throw 'Unique-entry normal build failed'}
  $main=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$profile.ToLowerInvariant()+'/LiToModern.exe')
  $log=Join-Path $taskRoot $(if($profile -eq 'Debug'){'驗證紀錄/current/build.log'}else{'驗證紀錄/current/build-release.log'})
  Copy-Item -LiteralPath $log -Destination (Join-Path $output ($profile+'-compiler.log'))
  Check-Inputs;$receipt.profiles[$profile]=@{path=$main;sha256=(Get-FileHash -LiteralPath $main).Hash;unique_entry=$true;features=@();GUI_revalidated=$false};Save
 }
 $receipt.status='normal_mains_rebuilt_unique_entry_pending_product_qualification';$receipt.active_profile=$null;$receipt.sources_unchanged=$true
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally{$receipt.finished_utc=[DateTime]::UtcNow.ToString('o');Save}
Write-Output ($receipt|ConvertTo-Json -Depth 10)
