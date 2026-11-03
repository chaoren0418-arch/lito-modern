#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile='Release')
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$stage=Join-Path $taskRoot ('LiTo381-Modern/target/current/diagnostic-output/'+$stamp+'/'+$BuildProfile)
$out=Join-Path $taskRoot ('驗證紀錄/current/diagnostic-output/'+$stamp)
New-Item -ItemType Directory -Path $stage,$out | Out-Null
$source=Join-Path $taskRoot ('LiTo381-Modern/target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
$exe=Join-Path $stage 'LiToModern.exe'
Copy-Item -LiteralPath $source -Destination $exe
$digest=(Get-FileHash -LiteralPath $exe).Hash
$pins=@{}
foreach($name in @('LiTo381-Modern/src/main.rs','LiTo381-Modern/src/modern_lito/rebuild_verification.rs','啟動現代客戶端.ps1','維護工具/verify-relocated-diagnostic-output.ps1')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}
$result=[ordered]@{status='running';profile=$BuildProfile;exe_sha256=$digest;retained_exe=$exe;output=$out;stage=$stage;source_pins=$pins;cases=@();clean_machine_verified=$false;native_GUI_retested=$false;full_game_complete=$false}
function Save {$result|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8}
function RunDiagnostic([string]$Name,[string[]]$Arguments,[int]$ExpectedExit) {
 $stdout=Join-Path $out ($Name+'.json');$stderr=Join-Path $out ($Name+'.stderr.txt')
 $process=Start-Process -FilePath $exe -ArgumentList $Arguments -WorkingDirectory $stage -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
 try {
  if(!$process.WaitForExit(45000)) {
   $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
   if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Contains($exe)){throw 'Owned diagnostic identity mismatch'}
   $process.Kill();$process.WaitForExit();throw 'Owned diagnostic deadline exceeded'
  }
  if($process.ExitCode -ne $ExpectedExit){throw ($Name+' unexpected exit '+$process.ExitCode)}
  $case=@{name=$Name;exit_code=$process.ExitCode;stdout_sha256=(Get-FileHash -LiteralPath $stdout).Hash;stderr_sha256=(Get-FileHash -LiteralPath $stderr).Hash}
  if($ExpectedExit -eq 0){$case.document=Get-Content -LiteralPath $stdout -Raw|ConvertFrom-Json}
  $result.cases+=@($case);Save
  return $case
 }finally{$process.Dispose()}
}
Save
try {
 $resources='"D:\lineage開發\0-客戶端"'
 $default=RunDiagnostic 'default' @('--verify-resources',$resources) 0
 $expected=Join-Path $stage 'verification/resource-input-slice.png'
 if([IO.Path]::GetFullPath($default.document.png) -ne [IO.Path]::GetFullPath($expected) -or !(Test-Path -LiteralPath $expected)){throw 'Default output did not follow relocated working directory'}
 $explicitRoot=Join-Path $stage 'writable diagnostics/new parent/leaf'
 $explicit=RunDiagnostic 'explicit' @('--verify-resources',$resources,'--output',('"'+$explicitRoot+'"')) 0
 if([IO.Path]::GetFullPath($explicit.document.png) -ne [IO.Path]::GetFullPath((Join-Path $explicitRoot 'resource-input-slice.png'))){throw 'Explicit nested output ignored'}
 if($default.document.png_rgba_sha256 -ne $explicit.document.png_rgba_sha256 -or $default.document.png_rgba_sha256 -ne '3281d489f6ae62620c1630b5878114f585f0b08a1a46440a1a0b9996157fcb63'){throw 'Original resource/input raster changed'}
 $missingOutput=Join-Path $stage 'missing diagnostic'
 $null=RunDiagnostic 'missing-resources' @('--verify-resources',('"'+(Join-Path $stage 'missing-resources')+'"'),'--output',('"'+$missingOutput+'"')) 2
 if(Test-Path -LiteralPath $missingOutput){throw 'Missing resources wrote an output artifact'}
 $blocked=Join-Path $stage 'blocked-output';Set-Content -LiteralPath $blocked -Value 'preserve existing file' -Encoding utf8
 $blockedHash=(Get-FileHash -LiteralPath $blocked).Hash
 $null=RunDiagnostic 'blocked-output' @('--verify-resources',$resources,'--output',('"'+$blocked+'"')) 2
 if((Get-FileHash -LiteralPath $blocked).Hash -ne $blockedHash){throw 'Blocked output overwrote existing file'}
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw 'Diagnostic source drift'}}
 if((Get-FileHash -LiteralPath $exe).Hash -ne $digest){throw 'Retained diagnostic binary drift'}
 $result.default_output_follows_runtime_directory=$true;$result.explicit_nested_output_verified=$true;$result.original_raster_unchanged=$true;$result.missing_resources_emit_no_artifact=$true;$result.blocked_output_preserves_existing_file=$true;$result.owned_processes_stopped=$true
 $result.status='verified_relocated_diagnostic_runtime_output_and_failure_paths';Save
}catch{$result.status='failed';$result.error=$_.Exception.Message;Save;throw}
Write-Output $out
