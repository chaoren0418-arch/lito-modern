#Requires -Version 7.0
[CmdletBinding()]param(
 [Parameter(Mandatory)][string]$BuildRun,
 [ValidatePattern('^[a-z0-9-]+$')][string]$RunLabel='full',
 [string]$Filter='',
 [switch]$PetFixtures
)
$ErrorActionPreference='Stop'
$taskRoot='D:\lineage開發\3-lito開發\lito重建版'
$project=Join-Path $taskRoot 'LiTo381-Modern'
$run=[IO.Path]::GetFullPath($BuildRun)
if(!$run.StartsWith($taskRoot+'\驗證紀錄\current\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned build run required'}
$build=Get-Content -LiteralPath (Join-Path $run 'receipt.json') -Raw|ConvertFrom-Json
$product=Get-Content -LiteralPath (Join-Path $run 'product-retention.json') -Raw|ConvertFrom-Json
$pins=Get-Content -LiteralPath (Join-Path $run 'source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
$fixtures=$null
if($PetFixtures){$fixtures=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json') -Raw|ConvertFrom-Json -AsHashtable}
if($build.exit_code -ne 0 -or !$build.sources_unchanged -or !$product.archive_readback_verified){throw 'Unqualified library build'}
if(![IO.Path]::GetFullPath($product.product).StartsWith($project+'\target\current\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned test executable required'}
function Check-Inputs {
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw "Compilation input drift: $name"}}
 if((Get-FileHash -LiteralPath $product.product).Hash -ne $product.product_sha256){throw 'Test executable drift'}
 if($fixtures){foreach($name in $fixtures.files.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $fixtures.files[$name]){throw "Pet fixture input drift: $name"}}}
}
Check-Inputs
$output=Join-Path $run ('runtime-'+$RunLabel)
if(Test-Path -LiteralPath $output){throw 'Run already exists; preserve and inspect it'}
New-Item -ItemType Directory -Path $output|Out-Null
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $output 'runner.ps1')
if(!$Filter){
 & python -X utf8 (Join-Path $taskRoot '維護工具/preserve-legacy-library-reports.py') snapshot $output
 if($LASTEXITCODE -ne 0){throw 'Historical report preservation failed; tests not started'}
}
$info=[Diagnostics.ProcessStartInfo]::new($product.product)
$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
$info.Environment['TEMP']=Join-Path $project 'target/current/scratch';$info.Environment['TMP']=$info.Environment['TEMP']
if($fixtures){
 foreach($name in $fixtures.inputs.Keys){$info.Environment[$name]=$fixtures.inputs[$name]}
 foreach($name in $fixtures.outputs.Keys){
  $destination=Join-Path $output $fixtures.outputs[$name].name
  if($fixtures.outputs[$name].kind -eq 'directory'){New-Item -ItemType Directory -Path $destination|Out-Null}
  $info.Environment[$name]=$destination
 }
}
if($Filter){$info.ArgumentList.Add($Filter)}
$info.ArgumentList.Add('--test-threads=1')
$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
$receipt=@{status='running';started_utc=[DateTime]::UtcNow.ToString('o');source_files=$pins.Count;source_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $run 'source-inputs.json')).Hash;product_sha256=$product.product_sha256;filter=$Filter;test_threads=1;whole_goal_complete=$false}
$receipt.test_target=if($product.test_target){$product.test_target}else{'lib'}
if($fixtures){$receipt.historical_pet_fixture_file_count=$fixtures.files.Count;$receipt.historical_pet_fixture_map_sha256=(Get-FileHash -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json')).Hash;$receipt.current_formal_pet_race_verified=$false}
function Save {$receipt|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
$process=[Diagnostics.Process]::Start($info);$receipt.pid=$process.Id;$receipt.creation_utc=$process.StartTime.ToUniversalTime().ToString('o');Save
$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
while(!$process.HasExited){Start-Sleep -Milliseconds 250}
$process.WaitForExit()
$log=$stdout.GetAwaiter().GetResult();$errors=$stderr.GetAwaiter().GetResult()
$log|Set-Content -LiteralPath (Join-Path $output 'tests.log') -Encoding utf8
$errors|Set-Content -LiteralPath (Join-Path $output 'tests.stderr.txt') -Encoding utf8
if(!$Filter){
 & python -X utf8 (Join-Path $taskRoot '維護工具/preserve-legacy-library-reports.py') collect-restore $output
 if($LASTEXITCODE -ne 0){throw 'Inspect retained historical-report restoration before continuing'}
}
$receipt.exit_code=$process.ExitCode;$receipt.finished_utc=[DateTime]::UtcNow.ToString('o')
Check-Inputs;$receipt.sources_unchanged=$true
$result=[regex]::Match($log,'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;')
if(!$result.Success){$receipt.status='terminal_result_missing';Save;throw 'Missing terminal test result'}
$receipt.passed=[int]$result.Groups[2].Value;$receipt.failed=[int]$result.Groups[3].Value;$receipt.ignored=[int]$result.Groups[4].Value;$receipt.filtered_out=[int]$result.Groups[6].Value
if(!$Filter -and $receipt.filtered_out -ne 0){$receipt.status='unexpected_filtered_tests';Save;throw 'Full library run filtered tests'}
$receipt.status=if($process.ExitCode -eq 0 -and $receipt.failed -eq 0){'passed_pinned_library_tests'}else{'failed_pinned_library_tests'}
Save
Write-Output ($receipt|ConvertTo-Json -Depth 8)
if($process.ExitCode -ne 0){exit $process.ExitCode}
