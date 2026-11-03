param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidatePath=Join-Path $logs 'r442-candidate.json';$c=Get-Content $candidatePath -Raw|ConvertFrom-Json
if((Get-FileHash $c.path).Hash -ne $c.sha256){throw 'Host drift'}
foreach($pin in $c.source_pins){if((Get-FileHash $pin.path).Hash -ne $pin.sha256 -or (Get-FileHash $pin.snapshot).Hash -ne $pin.sha256){throw 'Source drift'}}
foreach($name in @('r442d-consumer.stdout.log','r442b-HTTP-regression.stdout.log','r442d-live.stdout.log')){if((Get-Content (Join-Path $logs $name) -Raw) -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'Focused test evidence'}}
$build=@(Get-Content (Join-Path $logs 'r442d-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build'}
$lib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($lib.Count -ne 1){throw 'Library count'}
$dest=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r442.rlib';if(Test-Path $dest){throw 'Library already frozen'};[IO.File]::Copy($lib[0],$dest,$false)
if((Get-FileHash $dest).Hash -ne (Get-FileHash $lib[0]).Hash){throw 'Library copy'}
$c|Add-Member production_library ([ordered]@{path=$dest;sha256=(Get-FileHash $dest).Hash;offline_locked=$true;target='i686-pc-windows-msvc'})
$w=Get-Content (Join-Path $logs 'r442d-live-interface-witness.json') -Raw|ConvertFrom-Json
$runtime=$w.runtime;$ready=Get-Content (Join-Path $runtime 'ready.json') -Raw|ConvertFrom-Json
$m=Get-Content (Join-Path $runtime 'modern-rust-lifecycle-report.json') -Raw|ConvertFrom-Json
if($m.state -ne 'RUST_SELECTED_READER_MATRIX_PASSED_PENDING_RETIREMENT' -or $m.failure -or $m.cleanup_errors.Count -or $m.TLS_opened -ne 2 -or $m.TLS_close_notify_completed -ne 1 -or $m.TLS_hard_socket_drop -ne 1 -or $m.TLS_sockets_retired -ne 2 -or $m.SQL_expiry_timer_armed -or $m.full_recipe_rerun){throw 'Matrix result'}
$terminal=@($m.events|Where-Object event -eq JOB_TERMINAL);$pre=@($m.events|Where-Object event -eq OLD_READER_PRE_SQL_REJECTED);$holds=@($m.events|Where-Object event -eq REAL_SQL_HOLD_ENTERED);$overlap=@($m.events|Where-Object event -eq OVERLAP_RESELECT_ROSTER_BACKEND_UNAVAILABLE)
if($terminal.Count -ne 10 -or @($terminal|Where-Object{$_.data.job.state -eq 'COMPLETE'}).Count -ne 6 -or @($terminal|Where-Object{$_.data.job.state -eq 'REJECTED'}).Count -ne 4 -or $pre.Count -ne 4 -or $holds.Count -ne 4 -or $overlap.Count -ne 1 -or $overlap[0].data.new_owner_admitted){throw 'Case counts or overlap qualification'}
foreach($p in $pre){if($p.data.provider_calls_before -ne $p.data.provider_calls_after -or !$p.data.no_new_job -or !$p.data.no_running_job_before_request -or $p.data.HTTP -ne 400){throw 'Old reader preSQL'}}
$timeout=@($m.events|Where-Object event -eq BACKEND_DEADLINE_FINISHED_BEFORE_RELEASE)
$releases=@($m.events|Where-Object event -eq RELEASE)
if($timeout.Count -ne 1 -or $timeout[0].data.requested_backend_deadline_ms -ne 500 -or $timeout[0].data.finished.result -ne 'REJECTED' -or $timeout[0].data.finished.released -or $releases.Count -ne 4 -or @($releases|Where-Object{$_.data.result.backend_remaining_nanos_at_release -le 0}).Count -ne 1){throw 'Backend deadline evidence'}
$attempts=@(foreach($prefix in @('r442','r442c','r442d')){
 $aw=Get-Content (Join-Path $logs "$prefix-live-interface-witness.json") -Raw|ConvertFrom-Json
 $ar=Get-Content (Join-Path $aw.runtime 'ready.json') -Raw|ConvertFrom-Json
 $sql=Get-Content (Join-Path $aw.runtime 'external-sql-probe.json') -Raw|ConvertFrom-Json
 $ret=Get-Content (Join-Path $aw.runtime 'retirement.json') -Raw|ConvertFrom-Json
 $owned=Get-Content (Join-Path $aw.runtime 'owned-process-retirement.json') -Raw|ConvertFrom-Json
 if($ar.state -ne 'RETIRED' -or $ar.fixture_id -ne $aw.fixture_id -or $sql.state -ne 'PASSED_EXTERNAL_FIXTURE_RETIREMENT' -or $ret.state -ne 'TLS_CONTROL_AND_BACKEND_RETIRED' -or !$ret.timer_restored -or !$ret.world_store_quiescent -or !$owned.helper_retired -or $owned.exit_code -ne 0 -or $owned.remaining_owned_ports.Count -or !$sql.mysql_process_retired -or !$sql.mysql_listener_released){throw 'Attempt retirement'}
 if(@($sql.dynamic_before.PSObject.Properties).Count -ne 172 -or @($sql.dynamic_after.PSObject.Properties).Count -ne 172 -or @($sql.dynamic_changed_tables).Count){throw 'Tables count'}
 foreach($entry in $sql.dynamic_before.PSObject.Properties){$a=$sql.dynamic_after.PSObject.Properties[$entry.Name];if(!$a -or (ConvertTo-Json $entry.Value -Depth 10 -Compress) -ne (ConvertTo-Json $a.Value -Depth 10 -Compress)){throw 'Table drift'}}
 if(@(Get-NetTCPConnection -LocalPort @($ar.mysql.port,$ar.tls.port,$ar.control.port) -State Listen -ErrorAction SilentlyContinue).Count){throw 'Owned listener remains'}
 foreach($id in @($aw.helper_pid,$aw.mysql_pid)){if(Get-Process -Id $id -ErrorAction SilentlyContinue){throw 'Owned PID remains'}}
 [ordered]@{prefix=$prefix;runtime=$aw.runtime;fixture_id=$aw.fixture_id;helper_PID=$aw.helper_pid;helper_FILETIME=$aw.helper_FILETIME;mysql_PID=$aw.mysql_pid;mysql_FILETIME=$aw.mysql_FILETIME;source_tables_unchanged=172;all_retired=$true}
})
$root=Join-Path $project ('target\r442-live-evidence-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$safe=@(foreach($name in @('modern-rust-lifecycle-report.json','provider-reads.json','external-sql-probe.json','retirement.json','owned-process-live.json','owned-tls-control-live.json','owned-process-retirement.json','control-final-status.json')){
 $src=Join-Path $runtime $name;if([IO.File]::ReadAllText($src).Contains($ready.control.token)){throw 'Control secret in report'};$dst=Join-Path $root $name;[IO.File]::Copy($src,$dst,$false);$sha=(Get-FileHash $src).Hash;if((Get-FileHash $dst).Hash -ne $sha -or (Get-FileHash $src).Hash -ne $sha){throw 'Evidence copy drift'};[ordered]@{path=$dst;sha256=$sha;bytes=(Get-Item $dst).Length}
})
$reads=Get-Content (Join-Path $root 'provider-reads.json') -Raw|ConvertFrom-Json
if($reads.Count -ne 10 -or @($reads|Where-Object{-not $_.backend_scope_bound}).Count -or @($reads|Where-Object source -eq 7).Count -ne 8 -or @($reads|Where-Object source -eq 42).Count -ne 2){throw 'Actual SQL provider count'}
$history=@(foreach($name in @('r435-candidate.json','r436-candidate.json','r437-candidate.json','r438-candidate.json','r439-candidate.json','r440-candidate.json','r441-candidate.json','r442-first-attempt-candidate.json','r442-second-attempt-candidate.json')){
 $p=Join-Path $logs $name;if(Test-Path $p){$o=Get-Content $p -Raw|ConvertFrom-Json;if((Get-FileHash $o.path).Hash -ne $o.sha256){throw 'Historical host drift'};foreach($pin in $o.source_pins){if((Get-FileHash $pin.snapshot).Hash -ne $pin.sha256){throw 'Historical source drift'}};if($o.production_library -and (Get-FileHash $o.production_library.path).Hash -ne $o.production_library.sha256){throw 'Historical library drift'};[ordered]@{record=$name;host_sha256=$o.sha256;source_copies_verified=$o.source_pins.Count;preserved=$true}}
})
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$c.validation='1new targeted job/HTTP400/permanent-cutoff guard+1normalHTTP regression+1actual live matrix2.54sec:10real SQL reads/6COMPLETE/4REJECTED/4preSQL rejects. Overlap reselect got actual BackendUnavailable; serial reselect passed. Missing direct generation/Work attribution fields remain explicit. All3attempts/172tables/timers/PID FILETIME/ports retired.'
Write-ModernEvidenceJson -Path $candidatePath -Value $c
$needs=[ordered]@{state='EXACT_MINIMUM_PRODUCER_GAPS_NOT_A_READY_OR_PERMISSION_REQUEST';fixture_manifest=$w.manifest;overlap_case='After BACK while old selected Work remains held, CHAR_RESULT is BackendUnavailable; no new owner admitted. Need an explicit default-off fixture lane that can serve actual roster/select while exactly one retired Work is still held, or source-qualified confirmation this overlap is intentionally unavailable.';safe_control_fields=@('Hold.job_id correlating the sole FRESH_READ','Hold.selected_epoch_hex','Hold.channel_id','Hold.private_registration_generation opaque u64 hex','Hold.effective_gear_revision_at_entry from same Work','Hold.Work_deadline_monotonic_nanos or bounded remaining at RELEASE','terminal Job same owner/generation/gear revision and Work operation ID');constraints='No minted lease, no test issuer, no Native profile1 widening, no formal wire change required. Schema version must explicitly change if adding fields; Rust currently denies unknown fields.';current_correlation='Exact source/object/epoch/channel request; actual TLS selection/full gear rev0; sole RUNNING FRESH_READ/one-shot ARM_HOLD/provider count delta1; direct private generation remains absent.'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r442-minimum-producer-requirements.json') -Value $needs
$result=[ordered]@{state='PASSED_AVAILABLE_RUST_LIFECYCLE_CASES_OVERLAP_RESELECT_NOT_ADMITTED';updated_at=[DateTimeOffset]::Now.ToString('o');candidate=$c;fixture_manifest_sha256=$w.manifest;fixture_file_count=3594;reused_R441_immutable_snapshot=$true;targeted_new_guard_passed=1;targeted_normal_HTTP_regression_passed=1;actual_live_passed=1;actual_live_seconds=2.54;events=72;actual_PhysicalReader_SQL=10;source7_SQL=8;source42_SQL=2;actual_jobs_complete=6;actual_jobs_rejected=4;actual_holds=4;actual_old_reader_preSQL_rejected=4;gear_revision_before_Work=0;opaque_selected_epochs=@('8000000000000100','8000000000000101','8000000000000102');BACK_held_work_rejected=$true;serial_reselection_after_held_work_retirement_passed=$true;overlap_reselection_before_old_Work_release_passed=$false;overlap_actual_CHAR_RESULT='BackendUnavailable';new_owner_admitted_during_overlap=$false;backend_deadline_without_SQL_timer_passed=$true;backend_deadline_ms=500;timeout_evidence=$timeout;hard_disconnect_held_work_rejected=$true;other_source_fresh_read_after_disconnect_passed=$true;normal_fresh_recovery_after_timeout_passed=$true;SQL_expiry_timer_armed=$false;R441_SQL_expiry_evidence_separate=$true;no_private_lease_created=$true;direct_private_registration_generation=$null;missing_same_Work_fields=$needs.safe_control_fields;world_match_id=$null;world_generation=$null;world_scope_reason='Lobby-only actual selected PhysicalReader; no PREPARE/full recipe rerun';TLS_opened=2;TLS_close_notify=1;TLS_hard_socket_drop=1;TLS_sockets_retired=2;safe_live_evidence=$safe;three_attempt_retirements=$attempts;source_tables_unchanged_each_attempt=172;cleanup_errors=0;historical_preserved=$history;Rust_live_SQL_TLS=$true;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';formal_R408_changed=$false}
$resultPath=Join-Path $logs 'r442-selected-reader-result.json';Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-selected-reader-R442實際生命週期與精確缺口交接.txt'
$text=@'
R442固定r442d host/lib/33source：1新focused job-owner/HTTP400/永久cutoff守門、1受影響normal HTTP regression及1 actual Rust live矩陣（2.54sec）通過。沒有重跑R441 recipe/6guards；同37096A...3594自有固定fixture，stock launcher RuntimeNamespace modern。
最終72safe events、10实际private-selected PhysicalReader SQL（source7=8/source42=2）/10jobs（6COMPLETE/4REJECTED）/4真实held SQL/4old-reader preSQL拒絕。2真TLS AUTH/實際roster-selectedLobby/fullGear rev0；opaque epoch8000000000000100→0101→0102，objects10275/10163/10275，source42同object獨立。
BACK在trueSQL hold entered後執行、查驗Characters/selectednull，再RELEASE，job ACTUAL_READER_REJECTED，後續FRESH_READ HTTP400，provider calls與jobs皆無新增且before無RUNNING job；不是busy-job誤算preSQL。
10163/epoch0101 fresh COMPLETE weapon10252；其Work held後BACK，嘗試新10275/epoch0102 roster實際回覆CHAR_RESULT BackendUnavailable，新owner未admit。舊Work RELEASE→REJECTED/preSQL拒絕後，同TLS再取得真roster/select/channel/fullgear，newepoch fresh COMPLETE weapon10304。因此循序reselect通過，old-work跨新owner已admit的overlap reselect尚未通過，沒有用serial結果替代。精確producer需求在r442-minimum-producer-requirements.json。
500ms backenddeadline案未arm SQL timer，真queryEntered/remaining441791500ns；520191500ns client觀察後provider已REJECTED/finished且未RELEASE，隨後RELEASE backend剩餘<=0；terminal job拒絕。SQLcutoff=-1永久，與R441 query-active/SQLcutoff+5ms/backend6.98secactive expiry案分開。相同selected reader之後fresh COMPLETE恢復。
真Rust Drop TCP ShutdownBoth，不發close_notify，已有held Work RELEASE後拒絕；之後old FRESH_READ preSQL拒絕，source42正常fresh COMPLETE。TLS2 opened/1close_notify/1hardDrop/2retired，cleanup0。三輪owned helper/MySQL/3ports全部退休，各172dynamic source表逐鍵相同，timer恢復/worldstorequiescent。
前兩輪不擦除：r442b僅因測試錯把valid_until=-1永久當有限cutoff而fail；r442c partial已passBACK/preSQL與10163正常fresh，未保存typed roster理由而停止。b/c host/lib/source與各6safe evidence凍結，後續d新增-1精確支持/finiteexclusivecutoff/typed CHAR_RESULT及read-only roster未admit時可安全續測獨立案。不得將b/c視作產品故障或整案pass。
Control schema1缺Hold.job_id/selected_epoch/channel/private registration generation/same Work gear revision/absolute backend deadline與terminal同owner generation字段；目前僅由唯一RUNNING job、單次ARM_HOLD消耗、provider delta1、實際TLS scope/fullgearrev0做明確有限關聯。World match/generation=null因沒有PREPARE，不能聲稱完整renderer/actor READY。8safe filescopy+SHA，不copy含token的ready，不存AUTHraw/controltoken。
後續轉原版behavior/renderer獨立工作；overlap和直接generation同Work歸屬是可驗證待producer缺口。LOADEDfalse/GUI0/16of40/R408原樣。
'@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r442-final-verification.json') -Value ([ordered]@{result=$resultPath;result_sha256=(Get-FileHash $resultPath).Hash;handoff=$handoff;handoff_sha256=(Get-FileHash $handoff).Hash;producer_needs_sha256=(Get-FileHash (Join-Path $logs 'r442-minimum-producer-requirements.json')).Hash;source_files=504;source_copies=33;safe_live_files=8;failed_or_partial_safe_files=12;all_three_attempts_retired=$true;Rust_live_SQL_TLS=$true;overlap_reselect_passed=$false;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='native_available_reader_lifecycle_passed_renderer_study_next'
$status.current_task='R442d 1newguard/1HTTPregression/1actual live2.54sec完成，72events/10SQL(8source7+2source42)/6COMPLETE4REJECTED/4held/4preSQL，BACK/backend500ms(noSQLtimer)/hardDrop及serialreselect恢復成功。Overlap新roster真BackendUnavailable未admit，直接privategen/sameWorkgear欄位缺口明列。3attempts各172tables同/owned PID FILETIME3ports退休，b/cfailpartial保留，33source+lib/8safe封存。續原版behavior/renderer可獨立工作；LOADEDfalse/GUI0/16of40/R408不動。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R442available cases frozen;overlap not admitted;3attempts/172tables/owned ports retired'
