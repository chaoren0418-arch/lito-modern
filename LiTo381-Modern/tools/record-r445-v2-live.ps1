param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function CheckPin($p,$sha){if((Get-FileHash -LiteralPath $p).Hash -ne $sha){throw "Drift: $p"}}
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$cp=Join-Path $logs 'r445-candidate.json';$c=Get-Content $cp -Raw|ConvertFrom-Json;CheckPin $c.path $c.sha256
if($c.source_pins.Count -ne 37){throw 'Source count'}
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
foreach($name in @('r445c-v2guard.stdout.log','r445c-v1Ready-regression.stdout.log','r445d-live.stdout.log')){if([IO.File]::ReadAllText((Join-Path $logs $name)) -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'Focused tests'}}
$build=@(Get-Content (Join-Path $logs 'r445d-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build'}
$libs=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($libs.Count -ne 1){throw 'Library count'}
$lib=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r445d.rlib';if(!(Test-Path $lib)){[IO.File]::Copy($libs[0],$lib,$false)};CheckPin $lib (Get-FileHash $libs[0]).Hash
$c|Add-Member production_library ([ordered]@{path=$lib;sha256=(Get-FileHash $lib).Hash;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$w=Get-Content (Join-Path $logs 'r445d-live-interface-witness.json') -Raw|ConvertFrom-Json
$runtime=$w.runtime;$ready=Get-Content (Join-Path $runtime 'ready.json') -Raw|ConvertFrom-Json
$m=Get-Content (Join-Path $runtime 'modern-rust-v2-report.json') -Raw|ConvertFrom-Json
if($m.schema -ne 2 -or $m.state -ne 'RUST_V2_SAME_WORK_PASSED_PENDING_RETIREMENT' -or $m.failure -or $m.cleanup_errors.Count -or $m.TLS_opened -ne 1 -or $m.TLS_close_notify -ne 1 -or $m.SQL_timer_armed -or $m.full_recipe_rerun -or $m.R442_matrix_rerun -or $m.new_owner_admitted_during_hold -or $m.renderer_LOADED -or $m.events.Count -ne 19){throw 'Limited live result'}
$holds=@($m.events|Where-Object event -eq ACTUAL_SAME_WORK_RECONCILED_SQL_HOLD);$term=@($m.events|Where-Object event -eq ACTUAL_SAME_WORK_TERMINAL);$releases=@($m.events|Where-Object event -eq ACTUAL_SAME_WORK_RELEASE)
if($holds.Count -ne 4 -or $term.Count -ne 5 -or $releases.Count -ne 4 -or @($term|Where-Object{$_.data.job.state -eq 'COMPLETE'}).Count -ne 3 -or @($term|Where-Object{$_.data.job.state -eq 'REJECTED'}).Count -ne 2){throw 'Work counts'}
$keys=@('job_id','source','object','selected_epoch_hex','channel_id','private_registration_generation','effective_gear_revision_at_entry','work_deadline_monotonic_nanos','work_operation_id','correlation_origin')
$correlations=@(foreach($h in $holds){
 $entry=$h.data.hold;$terminal=@($term|Where-Object{$_.data.job.job_id -eq $entry.job_id});$release=@($releases|Where-Object{$_.data.hold.hold_id -eq $entry.hold_id})
 if($terminal.Count -ne 1 -or $release.Count -ne 1 -or !$terminal[0].data.job.work_observed -or $entry.correlation_origin -ne 'ACTUAL_REGISTERED_WORK_OPERATION_SQL_AND_RECONCILED_STORE' -or $h.data.provider_after -ne $h.data.provider_before+1 -or $entry.sql_cutoff_exclusive_epoch_millis -ne -1){throw 'Actual Work correlation'}
 foreach($k in $keys){if($null -eq $entry.$k -or $entry.$k -ne $terminal[0].data.job.$k -or $entry.$k -ne $release[0].data.hold.$k){throw "Immutable witness changed: $k"}}
 if($terminal[0].data.job.epoch_hex -ne $entry.selected_epoch_hex -or $release[0].data.hold.backend_remaining_nanos_at_release -gt $entry.remaining_nanos_at_entry){throw 'Scope or deadline domain'}
 [ordered]@{job_id=$entry.job_id;source=$entry.source;object=$entry.object;epoch=$entry.selected_epoch_hex;channel=$entry.channel_id;private_generation=$entry.private_registration_generation;gear_revision=$entry.effective_gear_revision_at_entry;work_operation_id=$entry.work_operation_id;work_deadline_monotonic_nanos=$entry.work_deadline_monotonic_nanos;backend_remaining_at_entry=$entry.remaining_nanos_at_entry;backend_remaining_at_release=$release[0].data.hold.backend_remaining_nanos_at_release;terminal=$terminal[0].data.job.state;all_same_Work_fields_equal=$true}
})
$CAS=@($m.events|Where-Object event -eq ACTUAL_TLS_CAS)
if($CAS.Count -ne 2 -or $CAS[0].data.revision -ne 1 -or $CAS[1].data.revision -ne 2 -or $CAS[0].data.allflags.Count -ne 29 -or $CAS[1].data.allflags.Count -ne 29 -or $term[0].data.job.revision -ne 1 -or $term[0].data.job.weapon_object -ne 0 -or $term[1].data.job.revision -ne 2 -or $term[1].data.job.weapon_object -ne 10304){throw 'Actual CAS rev1/rev2 evidence'}
$busy=@($m.events|Where-Object event -eq ACTUAL_BACK_HELD_SESSION_GATE_BUSY);$timeout=@($m.events|Where-Object event -eq ACTUAL_WORK_BACKEND_DEADLINE_BEFORE_RELEASE)
if($busy.Count -ne 1 -or $busy[0].data.status -ne 2 -or $busy[0].data.new_owner_admitted -or $timeout.Count -ne 1 -or $timeout[0].data.SQL_timer -or $timeout[0].data.hold.released -or !$timeout[0].data.hold.finished -or $timeout[0].data.hold.result -ne 'REJECTED' -or @($releases|Where-Object{$_.data.backend_timeout -and $_.data.hold.backend_remaining_nanos_at_release -le 0}).Count -ne 1){throw 'BUSY/backend deadline'}
$attempts=@(foreach($prefix in @('r445c','r445d')){
 $aw=Get-Content (Join-Path $logs "$prefix-live-interface-witness.json") -Raw|ConvertFrom-Json;$ar=Get-Content (Join-Path $aw.runtime 'ready.json') -Raw|ConvertFrom-Json
 $sql=Get-Content (Join-Path $aw.runtime 'external-sql-probe.json') -Raw|ConvertFrom-Json;$ret=Get-Content (Join-Path $aw.runtime 'retirement.json') -Raw|ConvertFrom-Json;$owned=Get-Content (Join-Path $aw.runtime 'owned-process-retirement.json') -Raw|ConvertFrom-Json
 if($ar.schema -ne 2 -or $ar.state -ne 'RETIRED' -or $ret.schema -ne 2 -or $ret.state -ne 'TLS_CONTROL_AND_BACKEND_RETIRED' -or !$ret.timer_restored -or !$ret.world_store_quiescent -or $sql.state -ne 'PASSED_EXTERNAL_FIXTURE_RETIREMENT' -or !$sql.mysql_process_retired -or !$sql.mysql_listener_released -or !$owned.helper_retired -or $owned.exit_code -ne 0 -or $owned.remaining_owned_ports.Count){throw 'Attempt retirement'}
 if(@($sql.dynamic_before.PSObject.Properties).Count -ne 172 -or @($sql.dynamic_after.PSObject.Properties).Count -ne 172 -or @($sql.dynamic_changed_tables).Count){throw 'Table count'}
 foreach($e in $sql.dynamic_before.PSObject.Properties){$after=$sql.dynamic_after.PSObject.Properties[$e.Name];if(!$after -or (ConvertTo-Json $e.Value -Depth 10 -Compress) -ne (ConvertTo-Json $after.Value -Depth 10 -Compress)){throw 'Source table drift'}}
 if(@(Get-NetTCPConnection -LocalPort @($ar.mysql.port,$ar.tls.port,$ar.control.port) -State Listen -ErrorAction SilentlyContinue).Count){throw 'Listener remains'}
 foreach($pair in @(@($aw.helper_pid,$aw.helper_FILETIME),@($aw.mysql_pid,$aw.mysql_FILETIME))){$p=Get-Process -Id $pair[0] -ErrorAction SilentlyContinue;if($p -and $p.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -eq $pair[1]){throw 'Same owned PID FILETIME remains'}}
 [ordered]@{prefix=$prefix;runtime=$aw.runtime;fixture_id=$aw.fixture_id;helper_PID=$aw.helper_pid;helper_FILETIME=$aw.helper_FILETIME;mysql_PID=$aw.mysql_pid;mysql_FILETIME=$aw.mysql_FILETIME;source_tables_unchanged=172;three_ports_released=$true;all_retired=$true}
})
$root=Join-Path $project ('target\r445d-safe-live-evidence-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$safe=@(foreach($name in @('modern-rust-v2-report.json','control-final-status.json','provider-reads.json','external-sql-probe.json','retirement.json','owned-process-live.json','owned-tls-control-live.json','owned-process-retirement.json')){
 $src=Join-Path $runtime $name;if([IO.File]::ReadAllText($src).Contains($ready.control.token)){throw 'Control secret'};$dst=Join-Path $root $name;[IO.File]::Copy($src,$dst,$false);CheckPin $dst (Get-FileHash $src).Hash;Pin $dst
})
$reads=Get-Content (Join-Path $root 'provider-reads.json') -Raw|ConvertFrom-Json;if($reads.Count -ne 5 -or @($reads|Where-Object{-not $_.backend_scope_bound -or $_.source -ne 7}).Count){throw 'Actual SQL reads'}
$history=@(foreach($name in @('r441','r442','r443','r444','r445-first-attempt')){
 $p=Join-Path $logs "$name-candidate.json";$o=Get-Content $p -Raw|ConvertFrom-Json;CheckPin $o.path $o.sha256;foreach($pin in $o.source_pins){CheckPin $pin.snapshot $pin.sha256};if($o.production_library){CheckPin $o.production_library.path $o.production_library.sha256};[ordered]@{record=Pin $p;source_copies=$o.source_pins.Count;preserved=$true}
})
$failedPath=Join-Path $logs 'r445-first-attempt.json';$failed=Get-Content $failedPath -Raw|ConvertFrom-Json;foreach($p in $failed.safe_evidence){CheckPin $p.path $p.sha256}
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$scope=Get-Content (Join-Path $logs 'source-change-scope.json') -Raw|ConvertFrom-Json;if($scope.source_files -ne 508 -or $scope.new_source_files.Count -ne 29){throw 'Source scope'}
$c.validation='PASSED1newv2guard+1affectedv1Readyguard+1limitedactualRustSQLTLS1.90sec;19events/5SQL/4holds/3COMPLETE2REJECTED;CASrev1/rev2 immutable sameWork witness;BACK single-slotBusy intentional, serial reselect succeeds;500ms backend timeout noSQLtimer;2attempts172tables/PID FILETIME/3ports retired. First wrongunarmedtest expectation retained;v1/R443/R444/R408 preserved.'
Write-ModernEvidenceJson -Path $cp -Value $c
$resolution=[ordered]@{state='R442_DIRECT_WORK_FIELDS_RESOLVED_BY_EXPLICIT_V2_NOT_OVERLAP_ADMISSION';direct_job_epoch_channel_private_generation_gear_revision_operation_deadline=$true;correlation_origin='ACTUAL_REGISTERED_WORK_OPERATION_SQL_AND_RECONCILED_STORE';overlap_reselect_intentionally_unavailable=$true;new_owner_admitted_before_old_Work_retirement=$false;new_backend_lane=$false;source_trace='R445 fixed producer evidence/source-qualified-overlap-trace.json plus actual Rust CHAR_RESULT status2';v1_historical_records_preserved=$true;remaining='Live Class/appearance ownership and full renderer/GUI qualification not granted by Work witness'}
$resolutionPath=Join-Path $logs 'r445-gap-resolution.json';Write-ModernEvidenceJson -Path $resolutionPath -Value $resolution
$resultPath=Join-Path $logs 'r445-v2-same-work-result.json'
Write-ModernEvidenceJson -Path $resultPath -Value ([ordered]@{state='PASSED_LIMITED_ACTUAL_RUST_V2_SAME_WORK_CAS_AND_LIFECYCLE';updated_at=[DateTimeOffset]::Now.ToString('o');candidate=$c;fixture_manifest='C88CBCC80A512A2F17766B72872D388EFC71D66C1CDBC3A46E5CDA69C7B1C1BE';fixture_files=3665;producer_evidence_files=17;new_v2_guard_passed=1;affected_v1_Ready_guard_passed=1;actual_live_passed=1;actual_live_seconds=1.90;events=19;actual_PhysicalReader_SQL=5;actual_held_Work=4;terminal_COMPLETE=3;terminal_REJECTED=2;CAS_revisions=@(1,2);CAS_flags_each=29;same_Work_correlations=$correlations;BACK_actual_CHAR_RESULT='BackendUnavailable';BACK_actual_status=2;overlap_intentionally_unavailable=$true;new_owner_admitted_during_hold=$false;serial_reselect_passed=$true;backend_deadline_ms=500;backend_timeout_evidence=$timeout;SQL_timer_armed=$false;SQL_expiry_evidence_remains_R441=$true;TLS_opened=1;TLS_close_notify=1;TLS_retired=1;two_attempt_retirements=$attempts;safe_live_evidence=$safe;first_wrong_expectation_attempt=Pin $failedPath;first_attempt_safe_files=8;historical_preserved=$history;gap_resolution=Pin $resolutionPath;v1_strict_contract_preserved=$true;source_files=508;baseline=479;unchanged=469;allowed_changed=10;new_sources=29;Rust_live_SQL_TLS=$true;no_private_lease_created=$true;full_recipe_rerun=$false;R442_matrix_rerun=$false;live_player_appearance_qualified=$false;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';formal_R408_changed=$false})
$handoff=Join-Path $logs 'Native-control-v2-R445同Work實際Rust驗收交接.txt'
[IO.File]::WriteAllText($handoff,@"
R445d37source/host/lib固定：1new strictv2guard、1shared Ready v1 regression，及1限定actual Rust SQL/TLS test1.90sec pass。C88C固定candidate3665/evidence17各SHA/bytes/完整集複製到Modern；只stock Invoke-NativeExternalControlV2Frozen RuntimeNamespace modern。v1 schema1獨立保持，v2Ready要求control_contract/hold_point/overlap_policy，strict status/jobs無duplicate key，reply schema只等於已綁ready，typed retirement2。
19safeevents/5actualSQL/4actual SQL+sameWork reconciled store-beforecommit Holds/3COMPLETE2REJECTED。真TLS AUTH/actualroster/select/channel/完整29flags CASrev1(unarmedobject0)和rev2(weapon10304)，Hold及terminal同job/source/object/opaque epoch/channel/private registration generation/gear revision/work operationID/absolute monotonic deadline/correlation origin逐項相同。gen2在10275/E200，gen7在10163/E201是實際觀察、不得從epoch推算或重建lease。
BACK held Work仍佔唯一SessionGate槽，真CHAR_RESULT status2 BackendUnavailable，新owner未admit。RELEASE→舊terminalREJECTED→真正退休後read-only roster與循序select10163/E201 fresh COMPLETE weapon10252。設計性overlap不可用，不報overlap通過、不新增lane。
backend500ms hold保留sameWork witness，未SQLtimer、SQLcutoff=-1；provider先finishedREJECTED且未release，RELEASE的同absolute deadline剩餘<=0，terminal同Work REJECTED；與R441 SQLexpiry/backendactive案分離。close_notify1/1TLS retired，cleanup0。
首C attempt因本次測試錯期待UNARMEDobject-1而停，實際schema值0；其rev1COMPLETE同Work witness及8safe檔/host/lib37source保留。修正只testexpectation後D重跑新有限live；沒有重跑v1 recipe或R442整套/舊guards。兩輪各172source表逐鍵相同、helper/MySQL exact PID/native FILETIME退休、3ports空，所有runtime檔案保留無刪除。
R443/R444原版正tick/zero首步交接均封存保持；508source=479baseline469unchanged10allowedchanged+29new，R408不動。v2同Work欄位解決R442直接關聯缺口，並未授予live Class/外觀lease、player runtime zeroTick visible residency/events/shadow105/worlddepth/full Native renderer。LOADEDfalse/GUI0/16of40。ready含token留runtime，不複製safe，不存AUTHraw。
Result: $resultPath
"@,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r445-final-verification.json') -Value ([ordered]@{result=Pin $resultPath;handoff=Pin $handoff;candidate=Pin $cp;host=Pin $c.path;library=Pin $lib;source_copies=37;safe_latest_files=8;safe_first_attempt_files=8;two_attempts_each_172tables_unchanged=$true;all_owned_PID_FILETIME_and_ports_retired=$true;source_files=508;Rust_live_SQL_TLS=$true;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='native_control_v2_actual_same_Work_passed_renderer_qualification_pending'
$status.current_task='R443/R444已收尾固定；R445d1newv2guard/1affectedv1Readyguard/1actualRustlive1.90sec通過，19events5SQL4holds3COMPLETE2REJECTED，CASrev1/2同Work job/epoch/channel/privategen/gearrev/operation/deadline直接一致。BACK-held真Busy status2為唯一槽設計，release後退休循序reselect通過；500ms backend無SQLtimer。兩attempt各172表同/exact PID FILETIME3ports退休、first unarmed-1錯期待保留8safe/source/lib；37source新host/lib/8finalsafe固定，508source/R408保持。v1與R443/R444保持、無R441/R442整套重跑；liveClass/appearance/zeroTick可見phase/event/shadow105/worlddepth/fullrenderer待資格，LOADEDfalse/GUI0/16of40。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R445 frozen:guards2/live1.90sec/5SQL/4sameWork holds/2attempts172tables andowned retirement;LOADEDfalse'
