$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$run='nw-c9afae2f2cd14da08208599b1bf066fb'
$evidence=Join-Path $logs ('r528-status-io-live-'+$run)
if(Test-Path -LiteralPath $evidence){throw 'Own run claim exists; no repeat preflight/connect'}
$exe=Join-Path $project 'target/r528-status-io-backend-consumer/r528-status-io-consumer.exe'
$ready='D:/lito-dev-runtime/20261003-supervision/server/nw-c9afae2f2cd14da08208599b1bf066fb/ready-contract.json'
$readySha='2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4'
$supply='D:/lineage開發/3-lito開發/lito伺服器/server-modern-485F-status-io-live-supply.json'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Exact($p,$bytes,$sha){$v=Pin $p;if($v.bytes -ne $bytes -or $v.sha256 -ne $sha){throw 'Authorized file pin mismatch; no execution'};$v}
$exePin=Exact $exe 3859456 'FAD83BE02142260C29DC24184580ED0283E69325AE0D6C4C4FB1DA1F22377182'
$readyPin=Exact $ready 6136 $readySha
$supplyPin=Exact $supply 10064 'B5F9174102037167BE26C7D1D912542C49439EE9B5562A471E3428F09E4117F2'
$request=Join-Path $logs 'r528-status-io-provider-supply-request.json'
if((Get-FileHash -LiteralPath $request).Hash -ne '7089029DAECDE224DE59AE1301EA1CE30D378D3D9AA60166734C717B859A2ED5'){throw 'Sealed supply request changed'}
[IO.Directory]::CreateDirectory($evidence)|Out-Null
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r528-live-c9-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
[IO.File]::Copy($ready,(Join-Path $evidence 'accepted-ready-contract.json'),$false)
[IO.File]::Copy($supply,(Join-Path $evidence 'accepted-live-supply.json'),$false)
Write-ModernEvidenceJson -Path (Join-Path $evidence 'authorized-run-claim.json') -Value ([ordered]@{schema='modern.r528.status-io.authorized-live-claim.v1';run_identity=$run;accepted_utc=[datetime]::UtcNow.ToString('o');source_supervisor='01a10207-9fe0-71f2-9e72-401a1654b4ca';consumer=$exePin;ready=$readyPin;supply=$supplyPin;request=Pin $request;preflight_limit=1;connect_limit=1;retry=$false;Modern_JavaSQL_start_stop=$false;password_print_copy_hash=$false;GUI=0;scratch_retained=$scratch})
$start=[datetime]::UtcNow.ToString('o')
& $exe --preflight $project $ready $readySha 1>(Join-Path $evidence 'preflight.stdout.log') 2>(Join-Path $evidence 'preflight.stderr.log')
$preflightExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $evidence 'preflight-accounting.json') -Value ([ordered]@{attempts=1;start_utc=$start;end_utc=[datetime]::UtcNow.ToString('o');exit_code=$preflightExit;socket_attempts=0;retry=$false;consumer=$exePin;ready=$readyPin;GUI=0;Modern_JavaSQL_start_stop=$false})
if($preflightExit -ne 0){Write-ModernEvidenceJson -Path (Join-Path $evidence 'first-failure.json') -Value ([ordered]@{stage='preflight';exit_code=$preflightExit;connect_attempts=0;retry=$false;provider_stop_required=$true;stderr=Pin (Join-Path $evidence 'preflight.stderr.log')});throw 'First preflight failure; connect not run, no retry'}
$start=[datetime]::UtcNow.ToString('o')
& $exe --connect-once $project $ready $readySha 1>(Join-Path $evidence 'connect.stdout.log') 2>(Join-Path $evidence 'connect.stderr.log')
$connectExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $evidence 'connect-accounting.json') -Value ([ordered]@{attempts=1;start_utc=$start;end_utc=[datetime]::UtcNow.ToString('o');exit_code=$connectExit;retry=$false;consumer=$exePin;ready=$readyPin;GUI=0;Modern_JavaSQL_start_stop=$false;result_path=Join-Path $logs ('r528-backend-'+$run+'/result.json');provider_stop_required=$true})
if($connectExit -ne 0){Write-ModernEvidenceJson -Path (Join-Path $evidence 'first-failure.json') -Value ([ordered]@{stage='connect-once';exit_code=$connectExit;connect_attempts=1;retry=$false;provider_stop_required=$true;stderr=Pin (Join-Path $evidence 'connect.stderr.log')});throw 'First connect failure; partial result retained, no retry'}
Write-Output 'Authorized R528 one preflight + one connect-once ended; server owner must collect and retire provider'
