param([string]$OutputDirectory=(Join-Path $PSScriptRoot '..\logs\continuation-20261003'))
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$server=Join-Path (Split-Path (Split-Path $project)) 'lito伺服器\LiTo381-Server'
$shared=Join-Path (Split-Path (Split-Path (Split-Path $project))) '0-客戶端'
$rulesPath=Join-Path $server 'src\main\java\local\lito381\server\BattleRules.java'
$presetPath=Join-Path $server 'src\main\java\local\lito381\server\ModernSharedDeathmatchPreset.java'
$rules=Get-Content $rulesPath -Raw
$preset=Get-Content $presetPath -Raw
$modes=[regex]::Matches($rules,'(?s)\b(PET_RACE|DEATHMATCH|QUEST_BATTLE|SIEGE_WAR|TIME_ATTACK|HONOR_MATCH)\((\d+),\s*"([^"]+)".*?List\.of\(([^)]+)\)')
if($modes.Count -ne 6){throw 'Expected exact six competitive mode declarations'}
$rows=@($modes|ForEach-Object{
    $mode=[int]$_.Groups[2].Value
    $name=$_.Groups[3].Value
    @($_.Groups[4].Value -split ','|ForEach-Object{[int]$_.Trim()})|ForEach-Object{
        $logical=$_
        $directory=Join-Path $shared "Map\$logical"
        [ordered]@{mode=$mode;mode_name=$name;logical_map=$logical;logical_shared_directory_exists=Test-Path -LiteralPath $directory;logical_shared_s32_files=@(Get-ChildItem -LiteralPath $directory -File -Filter '*.s32' -ErrorAction SilentlyContinue).Count;explicit_v47_fixture_physical_map=if($mode -eq 0 -and $logical -eq 565){5143}elseif($mode -eq 3 -and $logical -eq 547){4}elseif($mode -eq 4 -and $logical -eq 540){554}elseif(($mode -eq 1 -and $logical -eq 514)-or($mode -eq 2 -and $logical -eq 554)-or($mode -eq 5 -and $logical -eq 532)){$logical}else{$null};this_run_saved_mode_completed=$false;gui_accepted=$false;note='目錄／sector檔存在不證明合法地形、出生點、規則或可玩；不得任意別名到514'}
    }
})
if($rows.Count -ne 40){throw 'Expected exactly forty competitive mode/map pairs'}
$sections=[Collections.Generic.List[object]]::new()
$sections.Add(@{map=516;body=([regex]::Match($preset,'(?s)if\(id==516\)(.*?)if\(HUNDRED_PLAYER_MAPS')).Groups[1].Value})
foreach($case in [regex]::Matches($preset,'(?s)case\s+(\d+)\s*->\s*Map\.of\((.*?)(?=\s*case\s+\d+|\s*default\s*->)')){ $sections.Add(@{map=[int]$case.Groups[1].Value;body=$case.Groups[2].Value}) }
$conditional=[regex]::Match($preset,'(?s)"sectorPins",id==551\?\s*"([^"]+)":\s*"([^"]+)"')
$sections.Add(@{map=551;body=$conditional.Groups[1].Value})
$sections.Add(@{map=552;body=$conditional.Groups[2].Value})
$pins=@($sections|ForEach-Object{
    $map=$_.map
    [regex]::Matches($_.body,'([0-9a-f]{8}):(\d+):([0-9a-f]{64})')|ForEach-Object{
        $path=Join-Path $shared "Map\$map\$($_.Groups[1].Value).s32"
        $exists=Test-Path -LiteralPath $path
        $actual=if($exists){(Get-FileHash -LiteralPath $path).Hash}else{$null}
        [ordered]@{physical_map=$map;path=$path;expected_bytes=[long]$_.Groups[2].Value;expected_sha256=$_.Groups[3].Value;exists=$exists;actual_bytes=if($exists){(Get-Item -LiteralPath $path).Length}else{0};actual_sha256=$actual;matches=$exists -and $actual -eq $_.Groups[3].Value -and (Get-Item -LiteralPath $path).Length -eq [long]$_.Groups[2].Value}
    }
})
if(@($pins.physical_map|Sort-Object -Unique).Count -ne 9 -or $pins.Count -lt 10){throw 'Preset parser did not cover exactly nine explicitly qualified physical maps'}
$runs=Join-Path $project 'logs\continuation-20261003'
$accepted=[Collections.Generic.List[object]]::new()
$rejected=[Collections.Generic.List[object]]::new()
function Read-CompletedRun([string]$Stdout) {
    $text=Get-Content -LiteralPath $Stdout -Raw
    $summary=[regex]::Match($text,'test result: ok\. 1 passed; 0 failed; 0 ignored;.*finished in ([0-9.]+)s')
    $directory=Split-Path $Stdout
    $cleanupPath=Join-Path $directory 'cleanup.json'
    $recovered=Join-Path $directory 'recovered-completion.json'
    $cleanup=if(Test-Path -LiteralPath $cleanupPath){Get-Content -LiteralPath $cleanupPath -Raw|ConvertFrom-Json}elseif(Test-Path -LiteralPath $recovered){Get-Content -LiteralPath $recovered -Raw|ConvertFrom-Json}else{$null}
    if(-not $summary.Success -or -not $cleanup -or -not $cleanup.all_processes_retired){return $null}
    $resultPath=Join-Path $directory 'test-results.json'
    $result=if(Test-Path -LiteralPath $resultPath){@(Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json)|Where-Object{$_.stdout -eq $Stdout}}else{$null}
    if($result -and $result.exit_code -ne 0){return $null}
    if(-not $result -and -not (Test-Path -LiteralPath $recovered)){return $null}
    [ordered]@{stdout=$Stdout;stdout_sha256=(Get-FileHash -LiteralPath $Stdout).Hash;elapsed_seconds=[double]::Parse($summary.Groups[1].Value,[Globalization.CultureInfo]::InvariantCulture);cleanup=if(Test-Path -LiteralPath $cleanupPath){$cleanupPath}else{$recovered};os_exit_scope=if(Test-Path -LiteralPath $recovered){'Monitor aborted; OS exit code unobserved, passed stdout and process retirement recovered'}else{'Run test-results records OS exit code'}}
}
foreach($log in Get-ChildItem $runs -Recurse -File -Filter 'r408-six-modes-sql.stderr.log'){
    $stdout=$log.FullName.Replace('.stderr.log','.stdout.log')
    $completed=Read-CompletedRun $stdout
    if(-not $completed){$rejected.Add(@{stdout=$stdout;reason='No whole-test passed summary or retirement evidence'});continue}
    $completed.contract='modern_v47';$completed.stderr=$log.FullName
    $accepted.Add($completed)
    foreach($line in Get-Content -LiteralPath $log.FullName){if($line -match 'MIXED_MODES mode=(\d+) logical=(\d+) physical=(\d+).*loaded=true retired=true sameConnections=true'){
        $mode=[int]$Matches[1];$logical=[int]$Matches[2];$physical=[int]$Matches[3]
        foreach($row in $rows|Where-Object{$_.mode -eq $mode -and $_.logical_map -eq $logical}){
            if($row.explicit_v47_fixture_physical_map -ne $physical){throw 'Unexpected v47 logical/physical fixture mapping'}
            $row.this_run_saved_mode_completed=$true;$row.v47_program_completed=$true;$row.completed_evidence=$log.FullName
        }
    }}
}
$rotationSource=Join-Path $project 'src\modern_lito\map_rotation_interop_tests.rs'
$rotation=Get-Content -LiteralPath $rotationSource -Raw
$testDefinitions=@{}
foreach($definition in [regex]::Matches($rotation,'fn (java_shared_[a-z0-9_]+)\(\)\s*\{\s*run_map_rotation\("([^"]+)", \[([0-9, ]+)\]\);')){
    $peerMode=$definition.Groups[2].Value
    $testDefinitions[$definition.Groups[1].Value]=@{peer_mode=$peerMode;maps=@($definition.Groups[3].Value -split ','|ForEach-Object{[int]$_.Trim()});version=if($peerMode -eq 'rotation'){6}elseif($peerMode -in @('rotation-shared-516-531','rotation-shared-504-531','rotation-shared-505-531','rotation-shared-506-531')){34}else{19}}
}
if($testDefinitions.Count -ne 8){throw 'Expected eight explicit rotation test definitions'}
foreach($stdout in Get-ChildItem $runs -Recurse -File -Filter 'java_shared_*.stdout.log'){
    $name=$stdout.Name.Replace('.stdout.log','')
    if(-not $testDefinitions.ContainsKey($name)){throw ('Unknown rotation test: '+$name)}
    $completed=Read-CompletedRun $stdout.FullName
    if(-not $completed){$rejected.Add(@{stdout=$stdout.FullName;reason='No whole-test passed summary or retirement evidence'});continue}
    $definition=$testDefinitions[$name]
    $completed.contract='legacy_world_controller_v'+$definition.version;$completed.test=$name;$completed.peer_mode=$definition.peer_mode;$completed.round_maps=$definition.maps;$completed.test_definition_source=$rotationSource
    $accepted.Add($completed)
    foreach($map in $definition.maps|Sort-Object -Unique){
        $matched=@($rows|Where-Object{$_.mode -eq 1 -and $_.logical_map -eq $map})
        if($matched.Count -ne 1){throw ('Rotation map outside exact deathmatch rules: '+$map)}
        $row=$matched[0];$row.this_run_saved_mode_completed=$true
        if(-not $row.Contains('legacy_program_evidence')){$row.legacy_program_evidence=@()}
        $row.legacy_program_evidence+=@{contract=$completed.contract;physical_map=$map;stdout=$stdout.FullName}
    }
}
$canonicalSource=Join-Path $project 'src\modern_lito\canonical_map_rotation_tests.rs'
$canonical=Get-Content -LiteralPath $canonicalSource -Raw
$canonicalDefinitions=@{}
foreach($definition in [regex]::Matches($canonical,'(?s)rotation!\(\s*(java_v42_shared_map_rotation_[0-9_]+),\s*"([^"]+)",\s*\[([0-9, ]+)\]')){
    $canonicalDefinitions[$definition.Groups[1].Value]=@{peer_mode=$definition.Groups[2].Value;maps=@($definition.Groups[3].Value -split ','|ForEach-Object{[int]$_.Trim()})}
}
if($canonicalDefinitions.Count -ne 8){throw 'Expected eight explicit canonical v42 definitions'}
foreach($stdout in Get-ChildItem $runs -Recurse -File -Filter 'java_v42_shared_map_rotation_*.stdout.log'){
    $name=$stdout.Name.Replace('.stdout.log','')
    if(-not $canonicalDefinitions.ContainsKey($name)){throw ('Unknown canonical test: '+$name)}
    $completed=Read-CompletedRun $stdout.FullName
    if(-not $completed){$rejected.Add(@{stdout=$stdout.FullName;reason='No whole-test passed summary, exit0 or retirement evidence'});continue}
    $definition=$canonicalDefinitions[$name]
    $stderr=$stdout.FullName.Replace('.stdout.log','.stderr.log')
    $rounds=@([regex]::Matches((Get-Content -LiteralPath $stderr -Raw),'canonical rotation shared map (\d+): round (\d+) saved, same room (\d+), both owners returned'))
    if($rounds.Count -ne 3){throw 'Canonical success lacks three saved/returned round markers'}
    for($n=0;$n -lt 3;$n++){if([int]$rounds[$n].Groups[1].Value -ne $definition.maps[$n] -or [int]$rounds[$n].Groups[2].Value -ne $n+1){throw 'Canonical round marker disagrees with test definition'}}
    if(@($rounds|ForEach-Object{$_.Groups[3].Value}|Sort-Object -Unique).Count -ne 1){throw 'Canonical rounds changed room identity'}
    $completed.contract='canonical_controller_v42';$completed.test=$name;$completed.peer_mode=$definition.peer_mode;$completed.round_maps=$definition.maps;$completed.test_definition_source=$canonicalSource;$completed.stderr=$stderr
    $accepted.Add($completed)
    foreach($map in $definition.maps|Sort-Object -Unique){
        $matched=@($rows|Where-Object{$_.mode -eq 1 -and $_.logical_map -eq $map})
        if($matched.Count -ne 1){throw 'Canonical rotation map outside exact deathmatch rules'}
        $row=$matched[0];$row.this_run_saved_mode_completed=$true
        if(-not $row.Contains('canonical_v42_program_evidence')){$row.canonical_v42_program_evidence=@()}
        $row.canonical_v42_program_evidence+=@{contract=$completed.contract;physical_map=$map;stdout=$stdout.FullName}
    }
}
$ordinaryDefinitions=@{}
foreach($definition in [regex]::Matches($canonical,'(?s)ordinary_rotation!\(\s*(java_v47_shared_map_rotation_[0-9_]+),\s*"([^"]+)",\s*\[([0-9, ]+)\]')){
    $ordinaryDefinitions[$definition.Groups[1].Value]=@{peer_mode=$definition.Groups[2].Value;maps=@($definition.Groups[3].Value -split ','|ForEach-Object{[int]$_.Trim()})}
}
if($ordinaryDefinitions.Count -ne 8){throw 'Expected eight ordinary v47 map definitions'}
foreach($stdout in Get-ChildItem $runs -Recurse -File -Filter 'java_v47_shared_map_rotation_*.stdout.log'){
    $name=$stdout.Name.Replace('.stdout.log','')
    if(-not $ordinaryDefinitions.ContainsKey($name)){throw ('Unknown ordinary v47 test: '+$name)}
    $completed=Read-CompletedRun $stdout.FullName
    if(-not $completed){$rejected.Add(@{stdout=$stdout.FullName;reason='No whole-test passed summary, exit0 or retirement evidence'});continue}
    $definition=$ordinaryDefinitions[$name]
    $stderr=$stdout.FullName.Replace('.stdout.log','.stderr.log')
    $rounds=@([regex]::Matches((Get-Content -LiteralPath $stderr -Raw),'ordinary v47 rotation shared map (\d+): round (\d+) saved, same room (\d+), both owners returned'))
    if($rounds.Count -ne 3){throw 'Ordinary v47 success lacks three saved/returned markers'}
    for($n=0;$n -lt 3;$n++){if([int]$rounds[$n].Groups[1].Value -ne $definition.maps[$n] -or [int]$rounds[$n].Groups[2].Value -ne $n+1){throw 'Ordinary v47 round marker disagrees with test definition'}}
    if(@($rounds|ForEach-Object{$_.Groups[3].Value}|Sort-Object -Unique).Count -ne 1){throw 'Ordinary v47 rounds changed room identity'}
    $inputPath=Join-Path (Split-Path $stdout.FullName) 'inputs.json'
    $inputs=Get-Content -LiteralPath $inputPath -Raw|ConvertFrom-Json
    if($inputs.modern_prefix_manifest -ne (Join-Path $runs 'v47-map-fixture-snapshot.json')){throw 'Ordinary v47 missing explicit immutable map fixture input'}
    $completed.contract='modern_v47_ordinary_controller';$completed.test=$name;$completed.peer_mode=$definition.peer_mode;$completed.round_maps=$definition.maps;$completed.test_definition_source=$canonicalSource;$completed.stderr=$stderr;$completed.inputs=$inputPath
    $accepted.Add($completed)
    foreach($map in $definition.maps|Sort-Object -Unique){
        $matched=@($rows|Where-Object{$_.mode -eq 1 -and $_.logical_map -eq $map})
        if($matched.Count -ne 1){throw 'Ordinary v47 map outside exact deathmatch rules'}
        $row=$matched[0];$row.this_run_saved_mode_completed=$true;$row.v47_program_completed=$true
        if(-not $row.Contains('ordinary_v47_program_evidence')){$row.ordinary_v47_program_evidence=@()}
        $row.ordinary_v47_program_evidence+=@{contract=$completed.contract;physical_map=$map;stdout=$stdout.FullName}
    }
}
$null=New-Item -ItemType Directory -Path $OutputDirectory -Force
$coverage=[ordered]@{generated_at=[DateTimeOffset]::Now.ToString('o');source=@{rules=$rulesPath;rules_sha256=(Get-FileHash $rulesPath).Hash;preset=$presetPath;preset_sha256=(Get-FileHash $presetPath).Hash;rotation_tests=$rotationSource;rotation_tests_sha256=(Get-FileHash $rotationSource).Hash;canonical_tests=$canonicalSource;canonical_tests_sha256=(Get-FileHash $canonicalSource).Hash};mode_map_pairs=$rows.Count;program_completed_pairs=@($rows|Where-Object{$_.this_run_saved_mode_completed}).Count;v47_program_completed_pairs=@($rows|Where-Object{$_.Contains('v47_program_completed') -and $_.v47_program_completed}).Count;canonical_v42_program_completed_pairs=@($rows|Where-Object{$_.Contains('canonical_v42_program_evidence')}).Count;additional_legacy_pairs=@($rows|Where-Object{$_.Contains('legacy_program_evidence') -and -not $_.Contains('v47_program_completed')}).Count;pending_program_pairs=@($rows|Where-Object{-not $_.this_run_saved_mode_completed}).Count;gui_completed_pairs=0;rows=$rows;accepted_runs=$accepted.ToArray();excluded_runs=$rejected.ToArray();sector_pin_checks=$pins.Count;pin_failures=@($pins|Where-Object{-not $_.matches}).Count;sector_pins=$pins;scope='40組競技邏輯map/mode；v47、正式canonical controller v42及舊world-controller v6/v19/v34各列證據，跨契約聯集不相加重複。只有整案1passed且程序退休才計入；v42另核三場保存/同房/返回標記。共享S32 pins是位元組／長度核對，目錄存在不建立alias；GUI驗收0'}
Write-ModernEvidenceJson -Path (Join-Path $OutputDirectory 'mode-resource-coverage.json') -Value $coverage
[pscustomobject]$coverage|Select-Object mode_map_pairs,program_completed_pairs,v47_program_completed_pairs,canonical_v42_program_completed_pairs,additional_legacy_pairs,pending_program_pairs,gui_completed_pairs,sector_pin_checks,pin_failures
if(@($pins|Where-Object{-not $_.matches}).Count){throw 'Shared physical map pins drifted'}
