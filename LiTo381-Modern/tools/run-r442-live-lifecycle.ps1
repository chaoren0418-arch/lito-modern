param([ValidatePattern('^r442[a-z]$')][string]$Revision='r442c')
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$c=Get-Content (Join-Path $logs 'r442-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash $c.path).Hash -ne $c.sha256){throw 'Host drift'}
$snapshot=Get-Content (Join-Path $logs 'r441b-external-fixture-snapshot.json') -Raw|ConvertFrom-Json
$control=@(Get-Content (Join-Path $logs "$Revision-launcher.stdout.log")|Where-Object{$_ -like 'CONTROL_ROOT=*'})
if($control.Count -ne 1){throw 'Control root'}
$runtime=Get-Content -LiteralPath (Join-Path $control[0].Substring(13) 'current-attempt.txt') -Raw
if(-not $runtime.StartsWith('D:\lito-dev-runtime\20261003-supervision\modern\native-external-navigation-runtime-') -or $runtime -notmatch 'runtime-[0-9a-f]{32}$'){throw 'Runtime scope'}
$path=Join-Path $runtime 'ready.json'
$r=Get-Content $path -Raw|ConvertFrom-Json
if($r.state -ne 'READY' -or $r.runtime_namespace -ne 'modern' -or $r.candidate_manifest_sha256 -ne $snapshot.candidate.manifest_sha256){throw 'Ready binding'}
$helper=Get-Process -Id $r.owner.pid
$mysql=Get-Process -Id $r.mysql.pid
if($helper.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -ne $r.owner.creation_filetime_utc -or $mysql.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -ne $r.mysql.creation_filetime_utc){throw 'Native identity'}
$listeners=@(Get-NetTCPConnection -LocalPort @($r.tls.port,$r.control.port,$r.mysql.port) -State Listen -ErrorAction Stop|Select-Object LocalAddress,LocalPort,OwningProcess)
if($listeners.Count -ne 3 -or @($listeners|Where-Object{($_.LocalPort -eq $r.mysql.port -and $_.OwningProcess -ne $mysql.Id) -or ($_.LocalPort -ne $r.mysql.port -and $_.OwningProcess -ne $helper.Id) -or $_.LocalAddress -ne '127.0.0.1'}).Count){throw 'Listener ownership'}
Write-ModernEvidenceJson -Path (Join-Path $logs "$Revision-live-interface-witness.json") -Value ([ordered]@{state='ACTUAL_OWNED_MODERN_READY_WITNESSED_BEFORE_RUST';at=[DateTimeOffset]::Now.ToString('o');runtime=$runtime;ready_path=$path;ready_sha256=(Get-FileHash $path).Hash;fixture_id=$r.fixture_id;manifest=$r.candidate_manifest_sha256;helper_pid=$helper.Id;helper_FILETIME=$r.owner.creation_filetime_utc;mysql_pid=$mysql.Id;mysql_FILETIME=$r.mysql.creation_filetime_utc;listeners=$listeners;ready_control_secrets_serialized=$false})
try{
 $env:LITO_R442_READY=$path
 & $c.path 'modern_lito::native_external_fixture::lifecycle::actual_selected_reader_lifecycle_matrix' '--exact' '--ignored' '--nocapture' 1> (Join-Path $logs "$Revision-live.stdout.log") 2> (Join-Path $logs "$Revision-live.stderr.log")
 $exitCode=$LASTEXITCODE
}finally{Remove-Item Env:\LITO_R442_READY -ErrorAction SilentlyContinue}
Get-Content (Join-Path $logs "$Revision-live.stdout.log") -Tail 8
Get-Content (Join-Path $logs "$Revision-live.stderr.log") -Tail 8
if($exitCode -ne 0){throw 'Actual lifecycle matrix failed; safe report retained'}
