$ErrorActionPreference='Stop'
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidate=Get-Content -LiteralPath (Join-Path $logs 'r420-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'Frozen R420 drift'}
& (Join-Path $PSScriptRoot 'audit-mode-resource-coverage.ps1')
$coverage=Get-Content -LiteralPath (Join-Path $logs 'mode-resource-coverage.json') -Raw|ConvertFrom-Json
$cases=@(foreach($completed in $coverage.accepted_runs|Where-Object{$_.contract -eq 'modern_v47_ordinary_controller'}){
    $inputs=Get-Content -LiteralPath $completed.inputs -Raw|ConvertFrom-Json
    if($inputs.test_executable_sha256 -ne $candidate.sha256){throw 'Ordinary v47 candidate differs'}
    $directory=Split-Path $completed.stdout
    $cleanup=Get-Content -LiteralPath $completed.cleanup -Raw|ConvertFrom-Json
    [ordered]@{test=$completed.test;round_maps=$completed.round_maps;elapsed_seconds=$completed.elapsed_seconds;evidence=$directory;exit_code=0;all_processes_retired=$cleanup.all_processes_retired;remaining_files=$cleanup.remaining}
})
if($cases.Count -ne 8 -or @($cases.test|Sort-Object -Unique).Count -ne 8 -or $coverage.v47_program_completed_pairs -ne 16){throw 'Require all eight whole cases and16v47keys'}
$maps=@($cases.round_maps|ForEach-Object{$_}|Sort-Object -Unique)
if($maps.Count -ne 11){throw 'Require exactly eleven qualified deathmatch maps'}
$result=[ordered]@{verified_at=[DateTimeOffset]::Now.ToString('o');candidate=$candidate;fixture_manifest=(Join-Path $logs 'v47-map-fixture-snapshot.json');fixture_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $logs 'v47-map-fixture-snapshot.json')).Hash;passed_cases=$cases;deathmatch_maps=$maps;saved_rounds=24;v47_completed_pairs=16;remaining_pairs=24;gui_completed_pairs=0;constructor='SessionController::new(configured_modern_profile), strict schema2 modern_v47, no downgrade';observed_flow='Two TLS owners/source-qualified characters/lobby/shared room, three pinned shared maps, LOAD/combat/teleport119 cadence/unique Saved result/return/durable totals, same room and owners, stale attachment rejected, final character roster and quiescent shutdown';unchanged='Player EXE, original500ms cue expiry,15sec LOAD,45sec active and spell/move cooldowns; isolated candidate JAR only';historical_failures='Excluded; successful independent whole runs replace test coverage, not historical evidence'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r420-v47-map-consumer-result.json') -Value $result
$statusPath=Join-Path (Split-Path $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status.verified+=@('R420同固定host八案正式constructor/schema2/v47輪替24個Saved回合全pass/exit0/觀測程序退休，涵蓋11DM maps；v47聯集16/40、remaining24、GUI0。','remaining24 immutable evidence exact keys與425 sector byte checks相符；Native/Shared不同namespace，新增binding0；原生state-contract來源SHA核對與階段/退出條件對照已更新。')
$status.pending_acceptance=@($status.pending_acceptance|ForEach-Object{if($_ -like '40組模式*'){'40組模式／邏輯地圖：v47已驗16組、剩餘24組待合法模式binding與全流程；GUI0'}elseif($_ -like '本輪LiTo.bin新原生*'){'本輪LiTo.bin只版本握手；歷史manual與static閉合條件已對照，當輪登入／自然關閉與真人GUI仍待驗'}else{$_}})
$status.dependency_requests[3].status='extra10_v47_consumer_full_three_rounds_passed_remaining24_binding_needed'
$status.dependency_requests[3].request='Server immutable v47 DM11 fixture consumed:8whole cases/24Saved rounds/same owners and room/normal retirement passed; onlyremaining24 need explicit geometry/spawn/objective/encounter/race bindings, no Native-to-Shared alias.'
$status.evidence_paths+=@((Join-Path $logs 'r420-v47-map-consumer-result.json'),(Join-Path $logs 'remaining-map-handoff-consumer.json'),(Join-Path $logs 'source-change-scope.json'))
$status.current_task='完成v47 DM11消費端輪替；核對九张Shared geometry Rust解析及持續可進行項目'
$status.phase='remaining_resource_qualification'
$status.overall_status='in_progress'
$status.next_step='九張Shared geometry獨立Modern解析；remaining24需要Server模式binding；完整正式PLAYER GUI/capture/input仍待工具或真人可驗證證據'
$status.updated_at=[DateTimeOffset]::Now.ToString('o')
Write-ModernEvidenceJson -Path $statusPath -Value $status
