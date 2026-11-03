param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidatePath=Join-Path $logs 'r441-candidate.json'
$c=Get-Content $candidatePath -Raw|ConvertFrom-Json
if((Get-FileHash $c.path).Hash -ne $c.sha256){throw 'Host drift'}
foreach($pin in $c.source_pins){if((Get-FileHash $pin.path).Hash -ne $pin.sha256 -or (Get-FileHash $pin.snapshot).Hash -ne $pin.sha256){throw 'Source drift'}}
$local=Get-Content (Join-Path $logs 'r441b-consumer.stdout.log') -Raw
$live=Get-Content (Join-Path $logs 'r441-live.stdout.log') -Raw
if($local -notmatch '6 passed; 0 failed; 1 ignored;' -or $live -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'Test evidence'}
$build=@(Get-Content (Join-Path $logs 'r441b-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build'}
$lib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($lib.Count -ne 1){throw 'Library count'}
$dest=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r441.rlib'
if(Test-Path $dest){throw 'Library already frozen'}
[IO.File]::Copy($lib[0],$dest,$false)
if((Get-FileHash $lib[0]).Hash -ne (Get-FileHash $dest).Hash){throw 'Library copy'}
$c|Add-Member -NotePropertyName production_library -NotePropertyValue ([ordered]@{path=$dest;sha256=(Get-FileHash $dest).Hash;offline_locked=$true;target='i686-pc-windows-msvc'})
$w=Get-Content (Join-Path $logs 'r441-live-interface-witness.json') -Raw|ConvertFrom-Json
$runtime=$w.runtime
if(-not $runtime.StartsWith('D:\lito-dev-runtime\20261003-supervision\modern\native-external-navigation-runtime-') -or $runtime -notmatch 'runtime-[0-9a-f]{32}$'){throw 'Runtime scope'}
$ready=Get-Content (Join-Path $runtime 'ready.json') -Raw|ConvertFrom-Json
$report=Get-Content (Join-Path $runtime 'modern-rust-report.json') -Raw|ConvertFrom-Json
$sql=Get-Content (Join-Path $runtime 'external-sql-probe.json') -Raw|ConvertFrom-Json
$ret=Get-Content (Join-Path $runtime 'retirement.json') -Raw|ConvertFrom-Json
$owned=Get-Content (Join-Path $runtime 'owned-process-retirement.json') -Raw|ConvertFrom-Json
if($report.state -ne 'RUST_SQL_TLS_RECIPE_PASSED_PENDING_PROCESS_RETIREMENT' -or !$report.report.completed -or $report.report.failure -or $report.report.cleanup_errors.Count -or $report.report.TLS_connections_opened -ne 2 -or $report.report.TLS_connections_closed -ne 2 -or $report.report.TLS_sockets_retired -ne 2 -or $report.report.dto_acknowledgements -ne 2 -or !$report.shutdown_requested){throw 'Rust recipe'}
$expiry=$report.report.actual_expiry_observation
if($expiry.scope.source -ne 7 -or $expiry.scope.object -ne 10275 -or $expiry.scope.epoch_u64_hex -ne '8000000000000002' -or $expiry.gear_revision -ne 4 -or $expiry.item -ne 10304 -or $expiry.query_started_millis -ge $expiry.cutoff_exclusive -or $expiry.provider_finished_millis -le $expiry.cutoff_exclusive -or $expiry.backend_remaining_nanos -le 500000000){throw 'Expiry actual times'}
if($ready.state -ne 'RETIRED' -or $ready.fixture_id -ne $w.fixture_id -or $report.fixture_id -ne $w.fixture_id -or $sql.fixture_id -ne $w.fixture_id -or $ret.fixture_id -ne $w.fixture_id -or $ret.state -ne 'TLS_CONTROL_AND_BACKEND_RETIRED' -or !$ret.timer_restored -or !$ret.world_store_quiescent -or $sql.state -ne 'PASSED_EXTERNAL_FIXTURE_RETIREMENT' -or !$sql.mysql_process_retired -or !$sql.mysql_listener_released -or !$owned.helper_retired -or $owned.exit_code -ne 0 -or $owned.remaining_owned_ports.Count){throw 'Fixture retirement'}
$before=@($sql.dynamic_before.PSObject.Properties);$after=@($sql.dynamic_after.PSObject.Properties)
if($before.Count -ne 172 -or $after.Count -ne 172 -or @($sql.dynamic_changed_tables).Count){throw 'Dynamic table counts'}
foreach($entry in $before){$a=$sql.dynamic_after.PSObject.Properties[$entry.Name];if(!$a -or (ConvertTo-Json $entry.Value -Depth 10 -Compress) -ne (ConvertTo-Json $a.Value -Depth 10 -Compress)){throw 'Dynamic table drift'}}
$listeners=@(Get-NetTCPConnection -LocalPort @($ready.mysql.port,$ready.tls.port,$ready.control.port) -State Listen -ErrorAction SilentlyContinue)
if($listeners.Count){throw 'Listener remains'}
foreach($id in @($w.helper_pid,$w.mysql_pid)){if(Get-Process -Id $id -ErrorAction SilentlyContinue){throw 'Owned PID remains'}}
$snapshot=Get-Content (Join-Path $logs 'r441b-external-fixture-snapshot.json') -Raw|ConvertFrom-Json
if($snapshot.candidate.manifest_sha256 -ne $ready.candidate_manifest_sha256 -or (Get-FileHash (Join-Path $snapshot.candidate.root 'manifest.json')).Hash -ne $snapshot.candidate.manifest_sha256){throw 'Actual candidate binding'}
$manifest=Get-Content (Join-Path $snapshot.candidate.root 'manifest.json') -Raw|ConvertFrom-Json
$files=@(Get-ChildItem $snapshot.candidate.root -Recurse -File|Where-Object{$_.FullName -ne (Join-Path $snapshot.candidate.root 'manifest.json')})
if($files.Count -ne 3594 -or $manifest.files.Count -ne 3594){throw 'Candidate exact file set'}
foreach($entry in $manifest.files){$p=Join-Path $snapshot.candidate.root $entry.path;if((Get-Item $p).Length -ne $entry.bytes -or (Get-FileHash $p).Hash -ne $entry.sha256){throw 'Candidate payload drift'}}
$evidenceRoot=Join-Path $project ('target\r441-live-evidence-'+[Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($evidenceRoot)|Out-Null
$evidence=@(foreach($name in @('modern-rust-report.json','provider-reads.json','external-sql-probe.json','retirement.json','owned-process-live.json','owned-tls-control-live.json','owned-process-retirement.json','control-final-status.json')){
 $src=Join-Path $runtime $name;$text=[IO.File]::ReadAllText($src);if($text.Contains($ready.control.token)){throw 'Control token in report'};$dst=Join-Path $evidenceRoot $name;[IO.File]::Copy($src,$dst,$false);$hash=(Get-FileHash $src).Hash;if((Get-FileHash $dst).Hash -ne $hash -or (Get-FileHash $src).Hash -ne $hash){throw 'Live evidence copy drift'};[ordered]@{path=$dst;sha256=$hash;bytes=(Get-Item $dst).Length}
})
$reads=Get-Content (Join-Path $evidenceRoot 'provider-reads.json') -Raw|ConvertFrom-Json
if($reads.Count -ne 3 -or @($reads|Where-Object{-not $_.backend_scope_bound -or $_.object -ne 10275 -or $_.items -ne 29}).Count -or @($reads|Where-Object source -eq 7).Count -ne 2 -or @($reads|Where-Object source -eq 42).Count -ne 1){throw 'Actual Native SQL calls'}
$old=@(foreach($r in 435..440){$p=Join-Path $logs "r$r-candidate.json";if(Test-Path $p){$o=Get-Content $p -Raw|ConvertFrom-Json;if((Get-FileHash $o.path).Hash -ne $o.sha256){throw 'Historical host drift'};foreach($pin in $o.source_pins){if((Get-FileHash $pin.snapshot).Hash -ne $pin.sha256){throw 'Historical source drift'}};if($o.production_library -and (Get-FileHash $o.production_library.path).Hash -ne $o.production_library.sha256){throw 'Historical library drift'};[ordered]@{revision=$r;host_sha256=$o.sha256;source_copies_verified=$o.source_pins.Count;preserved=$true}}})
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$c.validation='PASSED6 namespace/adapter guards plus1 actual Rust SQL/TLS bounded recipe; both TLS clients closed/dropped, three actual Native SQL reads, expiry release cutoff+5ms with backend active6.9816144sec;172 dynamic tables unchanged and helper/MySQL/three listeners retired. DTO-only Loaded ack; renderer LOADEDfalse/GUI0.'
Write-ModernEvidenceJson -Path $candidatePath -Value $c
$result=[ordered]@{updated_at=[DateTimeOffset]::Now.ToString('o');state='PASSED_ACTUAL_RUST_EXTERNAL_NATIVE_SQL_TLS_NAVIGATION_AND_RETIREMENT';candidate=$c;namespace='modern';external_snapshot=$snapshot;candidate_files_reverified=3594;local_guard_passed=6;actual_live_passed=1;actual_live_seconds=5.09;actual_fixture_id=$w.fixture_id;actual_runtime=$runtime;live_evidence=$evidence;Rust_live_SQL_TLS=$true;Native_SQL_provider_reads=3;actual_sources=@(7,42);same_selected_object=10275;reselected_object=10163;gear_CAS_allflags_revisions=@(1,2,3,4);other_source_isolation_checks=4;report_steps=14;TLS_clients_opened=2;TLS_clients_closed=2;TLS_sockets_retired=2;DTO_only_Loaded_acknowledgements=2;actual_expiry=$expiry;dynamic_source_tables_unchanged=172;timer_restored=$true;owned_helper_retired=$true;owned_mysql_retired=$true;owned_listeners_retired=3;cleanup_errors=0;secrets_in_reports=$false;AUTHraw_captured=$false;historical_preserved=$old;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';formal_R408_changed=$false;remaining='Rust held FRESH_READ BACK/reselect/disconnect/timeout and old-reader preSQL cases are not executed by this recipe. No live Native appearance/render/GUI acceptance.'}
$resultPath=Join-Path $logs 'r441-live-navigation-result.json'
Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-external-R441實際Rust-SQL-TLS與退休交接.txt'
$text=@'
R441：真正Modern Rust client 經既有TLS1.2/1.3+PKI/DNS/auth API接fixed Native external fixture；不是讀producer capture或synthetic模型。6namespace/adapter guards及1 actual live test passed（live5.09sec）。
固定Server候選37096A23161C0358786C6039C6BA790278CD1594A111294B9FC6D6AC12FA8E01/3594files，自有source-copy-source snapshot，stock launcher明確RuntimeNamespace modern。正式R408/Server Java/SQL來源均未修改，歷史候選保存。Producer pending replay的狀態與本次獨立Rust live結果分開。
Modern唯一runtime：native-external-navigation-runtime-0d9645863318459ca220916a8ead6ec6，fixture190be4c2-f2c4-4af7-b113-3e8928d78d8c；helper20548/nativeFILETIME134355558545327303、MySQL2116/134355558599090062；port50316/50398/50399。preconnect native身份+三listener ownership由Modern核對，strict ready/control再查PIDFILETIME。
兩Rust連線選同object10275於source7/42，不同owner；实际roster/select/channel/whole gear CAS weaponOFF/armorON/armorOFF/weaponON rev1/2/3/4與allflags echo，4次source42隔離READ，room create/join/ready/PREPARE map564/COMBAT-only world owner/generation/revision/phase，2個顯式DTO-only Loaded ACK後combat READ，離world/room/BACK。
真正reselect object10163 opaqueepoch8000000000000001，再回object10275 epoch8000000000000002，重新建room/ready/第二PREPARE。ARM_HOLD由同實際SQL provider在真查詢後進入，TIMER_ARM秒級原timestamp未拿來強制等於source保守cutoff；以queryStarted1791082391270<sourcecutoff1791082394270，實際wallclock1791082394275 (+5ms) RELEASE，backend remaining6981614400ns（6.9816144sec），PREPARE Aborted/PreparationFailed。gear revision4/desired10304沒有默默變成UNARMED發布。這是query-valid/publication-expired；不是backend timeout。
14個safe report steps，NativeSQLprovider真讀3次source7兩次/source42一次，each29items/backend_scope_bound；不是所有equipment/roster SQL呼叫總數。final兩BACK釋放owner，TLS2close/drop，control timer清理/SHUTDOWN。helper/MySQL自然退出、exit0、三listeners現查0，172dynamic source表before/after獨立逐鍵核對相同、ownedtimer還原、worldstorequiescent。保存8份安全live檔copy+SHA，未copy含token的ready.json；無AUTHraw或controltoken入report。
Rust held FRESH_READ/BACK/reselect/disconnect/timeout/old-reader preSQL矩陣尚未執行；R437 producer歷史證據不能冒作此Rustlive矩陣。Native外觀、完整renderer、GUI/真人可玩仍未驗，LOADEDfalse/GUI0/16of40。
'@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r441-final-verification.json') -Value ([ordered]@{result=$resultPath;result_sha256=(Get-FileHash $resultPath).Hash;handoff=$handoff;handoff_sha256=(Get-FileHash $handoff).Hash;host_sha256=$c.sha256;production_library_sha256=$c.production_library.sha256;source_copies=$c.source_pins.Count;source_files=503;safe_live_artifact_pins=8;candidate_files_reverified=3594;Rust_live_SQL_TLS=$true;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json'
$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='native_Rust_SQL_TLS_navigation_recipe_passed_and_retired'
$status.current_task='R441新namespace strict ready/control6guards+1真Rust SQL/TLS recipe5.09sec完成；source7/42同object、4CAS/allflags/isolation、COMBAT prepare/DTOACK/read/leave/reselect、真query-valid+cutoff5ms RELEASE拒絕/6.98sec activebackend；172tables相同、timer還原、helper/MySQL/3ports退出，32source+lib/8safe live evidence/3594pins封存。尚缺Rust held FRESH_READ生命週期矩陣與Native完整renderer/GUI。R439靜態appearance保持、LOADEDfalse/GUI0/16of40/R408不動。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R4416local+1actualRustSQLTLS passed;172tables unchanged;helper/MySQL/3ports retired;frozen'
