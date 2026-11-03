$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$old=Get-Content -LiteralPath (Join-Path $logs 'r461-original-source-pins.json') -Raw|ConvertFrom-Json
$text=[IO.File]::ReadAllText($old.original_disassembly.path)
if((Get-FileHash -LiteralPath $old.original_disassembly.path).Hash -ne $old.original_disassembly.sha256){throw 'Original drift'}
$root=Join-Path $project 'target\r462-source-input-reference'
if(Test-Path -LiteralPath $root){throw 'Already frozen'}
[IO.Directory]::CreateDirectory($root)|Out-Null
$ids=@($old.files|ForEach-Object{[IO.Path]::GetFileNameWithoutExtension($_.path)})+@('4062E0','47C4D0','4D6970','48AFC0')
$pins=@(foreach($id in $ids|Select-Object -Unique){$matches=[regex]::Matches($text,"(?ms)^sub_$id proc near.*?^sub_$id endp[^\r\n]*");if($matches.Count -ne 1){throw "Whole function count $id"};$path=Join-Path $root "$id.asm";[IO.File]::WriteAllText($path,$matches[0].Value,[Text.UTF8Encoding]::new($false));Pin $path})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r462-original-source-pins.json') -Value ([ordered]@{original_disassembly=$old.original_disassembly;files=$pins})
$contract=[ordered]@{
 state='SOURCE_FINITE_MOUSE_MOVE_RETARGET_AND_CURRENT_CELL_STOP_CONTRACT'; source_pins=Pin (Join-Path $logs 'r462-original-source-pins.json')
 input=[ordered]@{handler='48C080 case512 loc48C14F';guards=@('HUD intercept not reached','viewport bounds passed','popup75A838 absent','arg4 bit1 set');producer='48B9F0';plain_target_guards=@('64B8EC false','decoded831708 zero','selected75A98C absent','64B8F6 false','834EE4 present');argument='handler-dispatched/transformed point; hardware-to-handler chain and viewport resources unobserved'}
 writes=@('834EB5=1','64B8F0=0','4B90E0 converts point','64BBC8 even native halfX','64BBCC Y','834EE0=0','64BBC3=(target!=actor current cell)')
 input_side_effects=@('no 4E2150 call in selected case512','no timer cancellation','no actor cell/heading/frame/camera/roof write','no draw/wire/audio event')
 mid_step=[ordered]@{source='4E2150 actor+13 nonzero -> old heading quarter ->4E2D07';prequeued='three existing step timers retain callback4E2150 and owner0, order and due';target_adoption='only zero-frame startup/carry4E2455 onward chooses target/collision/new heading';end0='4E2E93 always commits one cell and camera center using retained old heading, then48AFC0';carry='queue74 zero,760098 false:64BBC3 true schedules4E2150(owner0);false schedules none';current_cell_stop='target equals current cell clears64BBC3; already queued steps still finish their cell; no inferred idle/carry'}
 release=[ordered]@{source='48C080 case514 loc48C57B';writes='760098=0';plain_no_popup_no_action='clock75A830 update then return;64BBC3 and queued movement unchanged';not_immediate_stop=$true}
 distinct_cancellation=[ordered]@{source='4E45F0 true local actor834EE4 and834EB3 set';call='4D6870 callback4E2150 owner0';unqualified='input/interaction route not exercised by this mouse branch';clock_correction='4D69F0 is the clock getter;4D6970 is the callback cancellation routine. Do not infer cancellation from a4D69F0 call.'}
 private_owner='offline full NAB binding plus unique Arc/revision; equal fields in a new instance do not own old tickets/preparations';retirement='local private owner clears callbacks; this is not a newly observed original world-exit producer'
 retained_terminal_pose='source final46/frame0 retained without inventing action49. Local finite CPU scene retains its pose until explicit owner retirement; no native framebuffer residency grant.'
 runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged'
}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r462-source-stop-retarget-contract.json') -Value $contract
Write-ModernEvidenceJson -Path (Join-Path $logs 'r462-minimal-runtime-producer-gaps.json') -Value ([ordered]@{state='UNOBSERVED_RUNTIME_INPUTS_REFUSE';required=@(
 [ordered]@{field='handler event512,arg4 bit1,point,viewport/HUD branch';producer='47C4D0 dispatch ->48C080';owner='actual viewport object/resource834BD4';missing='no observed input chain/object/HUD raster; conditional typed branch facts only'},
 [ordered]@{field='834EE4/75AA48,actor+8,+13,+11,+2C,+30,+74';producer='actual actor registry and movement callbacks';owner='true local player';missing='pointers and actor queue unobserved; NAB offline cannot mint them'},
 [ordered]@{field='64BBC8/CC,64BBC3,834EB5,64B8F0,834EE0';producer='48B9F0 selected plain branch';owner='original process globals';missing='values conditional, never SQL/JSON/live grant'},
 [ordered]@{field='831660..678 decoded action/class;speed/haste/variant;64B8EC/831708/64B8F6/760098';producer='original decoded state tables/input writers';owner='true player global branch';missing='neutral branch declared and unknown refused; nonplayer is separate'},
 [ordered]@{field='8 fresh48B640 outputs,final48B7B0,3 schedule clocks';producer='4E1DB0/4D65F0/4D69F0';owner='new step startup at carry';missing='actual collision/permission/clock domain unobserved; no native-map passability inference'},
 [ordered]@{field='native final-frame visible residency and GUI framebuffer';producer='original actor draw/raster/window';owner='live Work/appearance/native runtime';missing='RGB555 CPU reference only; local pose retention conditional'}
 );qualified=$false;Java_fixture_required=$false;renderer_LOADED=$false;GUI=0})
Write-Output "R462 source contract and producer gaps frozen: $($pins.Count) whole functions"
