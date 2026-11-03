#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly|Out-Null}
& (Join-Path $PSScriptRoot 'verify-native-world.ps1') -SkipBuild
& (Join-Path $PSScriptRoot 'verify-native-motion.ps1') -SkipBuild
$exe=Join-Path $taskRoot 'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
$sha=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
$rows=[System.Collections.Generic.List[object]]::new()
foreach($scope in @('world','motion')) {
    $receipt=Get-Content -LiteralPath (Join-Path $taskRoot "驗證紀錄/current/native-$scope/receipt.json") -Raw|ConvertFrom-Json
    if($receipt.exe_sha256 -ne $sha){throw 'Animation evidence uses stale executable'}
    foreach($case in $receipt.cases) {
        $render=$case.client.world_render
        if($render.timeline.Count -ne 6){throw 'Animation sample count'}
        $digests=[System.Collections.Generic.HashSet[string]]::new()
        $ownId=$case.client.game.world.status.object_id
        $poses=@()
        foreach($sample in $render.timeline) {
            $f=$sample.frame;$null=$digests.Add($f.rgba_sha256)
            if(!$f.continuous_animation -or $f.static_actor_frame -or @($f.actors|Where-Object{!$_.rendered}).Count -ne 0 -or $f.actors.Count -ne 12){throw 'Incomplete world actor rendering'}
            $board=@($f.actors|Where-Object{$_.logical_graphics -eq 1544})
            if($board.Count -ne 1 -or $board[0].phase -ne 'static_prop' -or $board[0].sprite_member -ne '1544-0.spr'){throw 'Board resource mapping mismatch'}
            $own=@($f.actors|Where-Object{$_.id -eq $ownId})[0]
            if(($own.screen -join ',') -ne '400,300'){throw 'Animation camera lost local actor'}
            $poses+=@{elapsed_ns=$f.elapsed_ns;phase=$own.phase;action=$own.action;source_frame=$own.source_frame;position=$own.display_position;rgba_sha256=$f.rgba_sha256;png=$sample.png}
        }
        if($digests.Count -lt 4){throw 'Animation pixels did not advance'}
        if($case.mode -eq 'room-world-move') {
            if(($poses[0..3].phase -join ',') -ne 'walk,walk,walk,walk' -or ($poses[0..3].source_frame -join ',') -ne '0,1,2,3' -or ($poses[4..5].phase -join ',') -ne 'idle,idle'){throw 'Walk to idle frame sequence mismatch'}
            $y=$case.client.motion_before.actors.($ownId.ToString()).y
            if($poses[0].position[1] -ne $y -or [Math]::Abs($poses[2].position[1]-($y-0.5)) -gt 0.00001 -or $poses[5].position[1] -ne ($y-1)){throw 'Movement interpolation mismatch'}
        }elseif(@($poses|Where-Object{$_.phase -ne 'idle'}).Count -ne 0){throw 'Stationary actor used walking animation'}
        $rows.Add(@{mode=$case.mode;verified=$true;poses=$poses;distinct_rgba=$digests.Count})
    }
}
$output=Join-Path $taskRoot '驗證紀錄/current/native-animation'
New-Item -ItemType Directory -Path $output -Force|Out-Null
@{schema=1;status='verified_continuous_native_world_animation';exe_sha256=$sha;jar_sha256=$receipt.jar_sha256;cases=$rows;tcp_cases_reused=3;gui_animation_workflow_tested=$false;events_executed=$false;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 15|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
Write-Output 'Native world idle / walk / correction animation verified'
