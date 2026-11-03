param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidatePath=Join-Path $logs 'r440-candidate.json'
$c=Get-Content $candidatePath -Raw|ConvertFrom-Json
if((Get-FileHash $c.path).Hash -ne $c.sha256){throw 'Host drift'}
foreach($pin in $c.source_pins){if((Get-FileHash $pin.path).Hash -ne $pin.sha256 -or (Get-FileHash $pin.snapshot).Hash -ne $pin.sha256){throw 'Source drift'}}
$stdout=Get-Content (Join-Path $logs 'r440c-consumer.stdout.log') -Raw
if($stdout -notmatch '6 passed; 0 failed; 1 ignored;' -or $stdout -notmatch 'actual_Rust_SQL_TLS_external_navigation.*ignored'){throw 'Test evidence'}
$build=@(Get-Content (Join-Path $logs 'r440c-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build failed'}
$lib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($lib.Count -ne 1){throw 'Library count'}
$dest=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r440.rlib'
if(Test-Path $dest){throw 'Library already frozen'}
[IO.File]::Copy($lib[0],$dest,$false)
if((Get-FileHash $lib[0]).Hash -ne (Get-FileHash $dest).Hash){throw 'Library copy'}
$c|Add-Member -NotePropertyName production_library -NotePropertyValue ([ordered]@{path=$dest;sha256=(Get-FileHash $dest).Hash;offline_locked=$true;target='i686-pc-windows-msvc'})
$c.validation='PASSED6 adapter local guards/1 explicit live test ignored. Ready/control default-off binding compiled; no Rust live fixture execution. Pending new namespace-authorized Server candidate.'
Write-ModernEvidenceJson -Path $candidatePath -Value $c
$old=@(foreach($r in 435..439){$p=Join-Path $logs "r$r-candidate.json";if(Test-Path $p){$o=Get-Content $p -Raw|ConvertFrom-Json;if((Get-FileHash $o.path).Hash -ne $o.sha256){throw 'Historical host drift'};foreach($pin in $o.source_pins){if((Get-FileHash $pin.snapshot).Hash -ne $pin.sha256){throw 'Historical source drift'}};if($o.production_library -and (Get-FileHash $o.production_library.path).Hash -ne $o.production_library.sha256){throw 'Historical library drift'};[ordered]@{revision=$r;host_sha256=$o.sha256;source_copies_verified=$o.source_pins.Count;preserved=$true}}})
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$snapshot=Get-Content (Join-Path $logs 'r440-external-fixture-snapshot.json') -Raw|ConvertFrom-Json
if((Get-FileHash (Join-Path $snapshot.candidate.root 'manifest.json')).Hash -ne $snapshot.candidate.manifest_sha256){throw 'External manifest drift'}
$result=[ordered]@{updated_at=[DateTimeOffset]::Now.ToString('o');state='FROZEN_RUST_EXTERNAL_ADAPTER_LOCAL_GUARDS_PASSED_PENDING_MODERN_NAMESPACE_READY';candidate=$c;focused_passed=6;focused_failed=0;live_test_ignored=1;copied_external_files=3596;external_manifest_sha256=$snapshot.candidate.manifest_sha256;ready_control_schema=1;strict_typed_ready=$true;owner_PID_native_FILETIME=$true;pinned_CA_and_server_certificate_file=$true;explicit_loopback_only=$true;control_request_UUID_exact_echo=$true;control_HTTP_body_and_deadline_bounded=$true;expiry_query_started_before_source_conservative_cutoff=$true;explicit_hold_release_after_cutoff=$true;raw_SQL_timestamp_equals_conservative_cutoff_assumption=$false;timer_cleanup_attempted_even_if_hold_release_fails=$true;secrets_in_reports=$false;local_guards_are_not_live=$true;historical_preserved=$old;Rust_live_SQL_TLS=$false;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';formal_R408_changed=$false;pending='Server launcher and Java main currently restrict runtime to server namespace; supervisor requested a new fixed candidate accepting Modern-owned runtime. No permission to bypass namespace guard or write server runtime.'}
$resultPath=Join-Path $logs 'r440-external-adapter-result.json'
Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-external-R440嚴格ready-control接線與namespace待接交接.txt'
$text=@'
R440：Rust default-off external ready/control adapter 已編譯並封存；6 new focused local guards passed /0failed /1 explicit live test ignored。
Ready 只接受明確 ready.json 與 runtime，32KiB bound/strict unknown及duplicate拒絕，fixture UUID/manifest/baseline/protocol26 CAPS65791/source7+42/selection10275及10163/Map564/limits/expiry45sec檢查。Native PID+FILETIME查驗仍存活；DER檔案hash與路徑confinement核對，TLS仍使用既有正式PKI/DNS connect API。
Control 限loopback/http schema1，request UUID+fixture echo、header8KiB/body120KiB、ContentLength exact與bounded total I/O，無retry或guess ready。Bearer token與HTTP request buffer Zeroizing，無Debug/Serialize，安全Report不含AUTHraw、control token或SQL credentials。
R438 runner 已接 Control::progress_expiry：真实SQL hold entered後檢查 queryStarted<保守source cutoff，在wallclock>cutoff明確RELEASE，backend remaining>500ms；PREPARE應Aborted/PreparationFailed。不是以8秒hold timeout代替expiry證據；也不再假設秒級SQL Timestamp等於保守query-start cutoff。Control cleanup 即使RELEASE失敗仍嘗試還原唯一owned timer，錯誤進cleanup_errors；TLS finally逐一close/drop。
現有Server固定DF463855...3596files已自有immutable拷貝，handoff位於logs，未多塞入candidate而破壞stock launcher exact file-set。Producer SQL/TLS replay證據不當作Rust live。
待接：Server launcher與Java main只接受server namespace；監製正在請Server提供有namespace驗證的新版候選。Modern尚未啟動helper/MySQL/listener，也未繞過guard或寫server runtime。
新版manifest需先獨立copy/hash後更新r440 snapshot；不可套用舊DF463855 ready。Explicit live測試入口 modern_lito::native_external_fixture::tests::actual_Rust_SQL_TLS_external_navigation --ignored --exact，僅在LITO_R440_READY提供Modern-owned専用runtime ready.json後使用。會寫redacted modern-rust-report.json並SHUTDOWN；測試成功仍須獨立核對helper/MySQL/two listeners退休、172來源表與timer還原，不能只看recipe PASS。
尚未補齊Rust liveheld FRESH_READ/BACK/reselect/disconnect/timeout/old-reader preSQL cases；R437 producer fixed evidence與R440 local guards不能代替這些真Rust驗收。LOADED=false/GUI0/16of40；Native appearance profile未擴權，R408保持。
'@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r440-final-verification.json') -Value ([ordered]@{result=$resultPath;result_sha256=(Get-FileHash $resultPath).Hash;handoff=$handoff;handoff_sha256=(Get-FileHash $handoff).Hash;host_sha256=$c.sha256;production_library_sha256=$c.production_library.sha256;source_copies=$c.source_pins.Count;source_files=503;Rust_live_SQL_TLS=$false;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json'
$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o')
$status.phase='native_external_adapter_frozen_pending_modern_namespace_ready'
$status.current_task='R439 5focused/432SPR/160plans/3視閱PNG/4054pins/31source+lib完成；R440嚴格ready/control/expiry進度与cleanup，6focused pass/1live ignored/32source+lib凍結，DF4638553596files自有copy。等待Server允許Modern-owned runtime的新固定候選；Rust-live/held FRESH_READ生命週期仍未驗，未啟動Server namespace。LOADEDfalse/GUI0/16of40/R408保持。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R4406localpass/1liveignored/32source+lib frozen; no fixture launched'
