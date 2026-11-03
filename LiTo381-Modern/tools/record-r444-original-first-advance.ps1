param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function CheckPin($p,$sha){if((Get-FileHash -LiteralPath $p).Hash -ne $sha){throw "Drift: $p"}}
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$old=Get-Content (Join-Path $logs 'r443-candidate.json') -Raw|ConvertFrom-Json
CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256
foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256}
$asm='D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm'
CheckPin $asm '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
$original=[IO.File]::ReadAllText($asm).Replace("`r`n","`n")
$route=Get-Content (Join-Path $logs 'r430-native-actor-route-contract.json') -Raw|ConvertFrom-Json
$runtime=Get-Content (Join-Path $logs 'r433-original-runtime-clock-contract.json') -Raw|ConvertFrom-Json
$functions=@($route.files|Where-Object{$_.function -in @('sub_43DE10','sub_4E5B50','sub_4E45F0','sub_4E5C00','sub_4E7A40','sub_4E34C0','sub_4D65F0')})+@($runtime.new_source_pins)
$root=Join-Path $project ('target\r444-original-first-advance-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$source=@(foreach($p in $functions){CheckPin $p.path $p.sha256;$body=[IO.File]::ReadAllText($p.path).Replace("`r`n","`n").Trim();if(!$original.Contains($body)){throw 'Fragment differs from original disassembly'};$dst=Join-Path $root ($p.function+'.asm');[IO.File]::Copy($p.path,$dst,$false);CheckPin $dst $p.sha256;[ordered]@{function=$p.function;path=$dst;sha256=$p.sha256}})
function Body($name){[IO.File]::ReadAllText(($source|Where-Object function -eq $name).path)}
function Require($name,$pattern){if((Body $name) -notmatch $pattern){throw "Missing source fact $name"}}
Require 'sub_43DE10' 'idiv\s+\[ebp\+var_1C\]'
Require 'sub_43DE10' 'xor\s+eax, 1812h[\s\S]*?mov\s+\[edx\+edi\+0Ah\], ax'
Require 'sub_4E45F0' 'mov\s+byte ptr \[esi\+13h\], 0\s+call\s+sub_4E5B50'
Require 'sub_4E5B50' 'inc\s+bl[\s\S]*?mov\s+\[ecx\+13h\], bl[\s\S]*?cmp\s+dl, \[edi\+eax\*4\+1\]'
Require 'sub_4E5B50' 'loc_4E5BDF:[\s\S]*?push\s+1\s+call\s+sub_4E5C00'
Require 'sub_4E45F0' 'loc_4E4952:[\s\S]*?add\s+ecx, 2\s+call\s+sub_4D65F0'
Require 'sub_4E7A40' 'mov\s+byte ptr \[esi\+13h\], 0'
Require 'sub_4E7A40' 'push\s+0\s+call\s+sub_4E5C00'
Require 'sub_4E5C00' 'test\s+al, al[\s\S]*?jz\s+loc_4E61D6'
Require 'sub_4E34C0' 'sub\s+ecx, \[ebp\+var_44\]\s+cmp\s+ecx, 1'
$contract=[ordered]@{state='ORIGINAL_STATIC_ORDINARY_TRANSITION_FIRST_ADVANCE_QUALIFIED_CONDITIONALLY';original_disassembly=Pin $asm;source=$source;facts=@(
 '43DE10 computes ticks*1000/FPS and stores XOR1812 word; zero is not universally clamped positive. Global647EA4 may scale selected actions; positive-duration helper remains conservative.',
 '4E7A40 initializes actor+13 frame0 and calls4E5C00(arg0=0): initial member/subframe loaded; this is separate from ordinary action transition and proves no visible residency duration.',
 '4E45F0 ordinary branch sets actor+13=0 then calls4E5B50;4E5B50 increments BEFORE count comparison or loading4E5C00. With count>1, selected frame is1 regardless of initial tick0 or positive tick.',
 '4E5C00 updates member/subframe and reads raw events; arg0=0 bypasses a guarded event block, whereas4E5B50 passes1. Additional globals/guards remain; events are not dispatched by this study.',
 '4E5C00 returns4E8750 effective duration for selected frame. Explicit nonplayer/queue0/modifiers0/action21counter0/no catalog scaling gives catalog duration. Original player equality selects different encoded globals; not qualified.',
 '4E45F0 schedules next4E34C0 callback with returned frame1 duration+2. This +2 applies after selected frame1, not invented positive residency for frame0.',
 '4E34C0 later advances first, computes max(1,effectiveDuration-lateness)+2 and expected+=effectiveDuration+2. At end4E5B50 resetsframe0/returnsfalse and continuation is state dependent; no loop grant.');conditional_implementation='Only first ordinary attack transition with explicit branch reached, frozen external state, different playerglobal, queue-after-pop0, modifiers0, action21counter0, scaling-disabled and sampledclock. Original runtime inputs are NOT observed.';unqualified=@('Visible frame0 residency during load or between rendering passes','Which transition/player/special branch is actually used for a live player','Zero ticks in later selected frames; source may schedule+2 or lateness floor but this helper rejects them','Runtime event effects, draw ordering, end/requeue/loop, shadow/105 and world/depth');original_BIN_executed=$false;renderer_LOADED=$false;GUI=0}
$contractPath=Join-Path $logs 'r444-original-first-advance-contract.json';Write-ModernEvidenceJson -Path $contractPath -Value $contract
$build=@(Get-Content (Join-Path $logs 'r444b-host-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Host build'}
$exe=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.executable}|ForEach-Object{$_.executable});if($exe.Count -ne 1){throw 'Host count'}
$hostRoot=Join-Path $project 'target\r444b-fixed-ordinary-first-advance-host';if(Test-Path $hostRoot){throw 'Already frozen'};[IO.Directory]::CreateDirectory((Join-Path $hostRoot 'source'))|Out-Null
$hostPath=Join-Path $hostRoot 'lito-modern-r444b-host-tests.exe';[IO.File]::Copy($exe[0],$hostPath,$false)
$pins=@(foreach($p in $old.source_pins){$dst=Join-Path $hostRoot ('source\'+(Split-Path $p.path -Leaf));[IO.File]::Copy($p.path,$dst,$false);$sha=(Get-FileHash $p.path).Hash;CheckPin $dst $sha;[ordered]@{path=$p.path;snapshot=$dst;sha256=$sha}})
$candidate=[ordered]@{path=$hostPath;sha256=(Get-FileHash $hostPath).Hash;source_pins=$pins;validation='PENDING targeted original first-advance consumer';original_source_contract=Pin $contractPath;LOADED=$false}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r444-candidate.json') -Value $candidate
& $hostPath 'modern_lito::native_appearance_clock_study_tests::original_ordinary_first_advance_zero_tick_is_not_a_visible_clock' '--exact' '--nocapture' 1> (Join-Path $logs 'r444b-consumer.stdout.log') 2> (Join-Path $logs 'r444b-consumer.stderr.log')
if($LASTEXITCODE -ne 0){Get-Content (Join-Path $logs 'r444b-consumer.stdout.log');throw 'Consumer failed'}
Get-Content (Join-Path $logs 'r444b-consumer.stdout.log') -Tail 4
