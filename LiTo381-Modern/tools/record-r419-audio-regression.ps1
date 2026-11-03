$ErrorActionPreference='Stop'
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidate=Get-Content -LiteralPath (Join-Path $logs 'r419-candidate.json') -Raw|ConvertFrom-Json
$accepted=@(foreach($directory in Get-ChildItem -LiteralPath $logs -Directory -Filter 'run-Extra-*'){
    $inputs=Join-Path $directory.FullName 'inputs.json'
    if(-not(Test-Path -LiteralPath $inputs)){continue}
    $input=Get-Content -LiteralPath $inputs -Raw|ConvertFrom-Json
    if($input.test_executable_sha256 -ne $candidate.sha256){continue}
    $resultPath=Join-Path $directory.FullName 'test-results.json'
    if(-not(Test-Path -LiteralPath $resultPath)){continue}
    $results=@(Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json)
    foreach($result in $results){
        if($result.test -notlike 'java_v42_shared_map_rotation_*'){continue}
        $cleanup=Get-Content -LiteralPath (Join-Path $directory.FullName 'cleanup.json') -Raw|ConvertFrom-Json
        $text=Get-Content -LiteralPath $result.stdout -Raw
        $summary=[regex]::Match($text,'test result: ok\. 1 passed; 0 failed; 0 ignored;.*finished in ([0-9.]+)s')
        if($result.exit_code -ne 0 -or -not $cleanup.all_processes_retired -or -not $summary.Success){throw 'R419 regression did not finish successfully'}
        [ordered]@{test=$result.test;elapsed_seconds=[double]$summary.Groups[1].Value;evidence=$directory.FullName;exit_code=0;all_processes_retired=$true;remaining_files=$cleanup.remaining}
    }
})
if($accepted.Count -ne 8 -or @($accepted.test|Sort-Object -Unique).Count -ne 8){throw 'Require all eight R419 cases'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r419-audio-regression.json') -Value ([ordered]@{verified_at=[DateTimeOffset]::Now.ToString('o');candidate=$candidate;passed=$accepted;cause='TestClient input synchronously printed receipt diagnostics before resuming runtime/audio polling; blocking diagnostic I/O consumed cue lifetime. Buffered input diagnostics allow sampling original live cue.';superseded='R414 synchronous timing suggested 1195ms product delay; R417 buffered measurement showed timeline operations under1ms and live cue frames. This was diagnostic I/O, not proven product delay.';unchanged='Original cue119 assertion,500ms expiry,no history replay,server LOAD/active and spell/move deadlines; fixed R408 player not rebuilt';historical_failures='Retained and excluded from acceptance; successful replacement runs are independent full cases';gui_accepted=$false})
$statusPath=Join-Path (Split-Path $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status.blockers=@($status.blockers|Where-Object{$_ -notlike '新增v42正式控制器map rotation瞬移音效119斷言失敗*'})
$status.verified+=@('R419八案v42控制器三場共享圖輪替整案exit0+程序退休：11張DM圖。audio119原斷言保留；修正測試INPUT_ADMISSION同步診斷阻塞，非產品音效時限變更。')
$status.dependency_requests[3].status='immutable_v47_extra10_snapshot_verified_consumer_in_progress_remaining24_need_qualified_binding'
$status.next_step='v47正式constructor八案三場rotation；後續原生state-contract對照；remaining24等待Server合法binding'
$status.evidence_paths+=@((Join-Path $logs 'r419-audio-regression.json'),(Join-Path $logs 'v47-map-fixture-snapshot.json'))
$status.updated_at=[DateTimeOffset]::Now.ToString('o')
Write-ModernEvidenceJson -Path $statusPath -Value $status
