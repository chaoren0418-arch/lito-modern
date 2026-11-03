$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$old=Get-Content -LiteralPath (Join-Path $logs 'r462-original-source-pins.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $old.original_disassembly.path).Hash -ne $old.original_disassembly.sha256){throw 'Original drift'}
$text=[IO.File]::ReadAllText($old.original_disassembly.path)
$root=Join-Path $project 'target\r463-source-action-cancel-reference';if(Test-Path -LiteralPath $root){throw 'Already frozen'};[IO.Directory]::CreateDirectory($root)|Out-Null
$ids=@($old.files|ForEach-Object{[IO.Path]::GetFileNameWithoutExtension($_.path)})+@('4A5A90','4B2970','4A2A50','4E6AF0','4E9800','4E01A0','4E0200','4E8160','40B6F0','406CE0','4A5ED0','4E0C50','4E6900','4E6520','4A3910','4E09D0')
$pins=@(foreach($id in $ids|Select-Object -Unique){$m=[regex]::Matches($text,"(?ms)^sub_$id proc near.*?^sub_$id endp[^\r\n]*");if($m.Count -ne 1){throw "Whole function count $id"};$p=Join-Path $root "$id.asm";[IO.File]::WriteAllText($p,$m[0].Value,[Text.UTF8Encoding]::new($false));Pin $p})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r463-original-source-pins.json') -Value ([ordered]@{original_disassembly=$old.original_disassembly;files=$pins})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r463-source-action-cancel-contract.json') -Value ([ordered]@{
 state='SOURCE_RECEIVED_ACTION32_GAP_CANCEL_OWNER0_SWITCH_ATTACK47_CONTRACT';source_pins=Pin (Join-Path $logs 'r463-original-source-pins.json')
 producer='4B2970 selected case32 ->4A5A90; ecx+1 parsed4A2A50 dc = objectDWORD little endian and signed actionByte'
 actual_actor='4E8160 registry lookup; pointer must equal834EE4, actor+8 must equal75AA48; fixture object value alone grants neither pointer nor live admission'
 busy_gate='4E6AF0: queue74>0/non-idle queue or nonzero frame/current46!=base46+3 ->true;4A5A90 busy route queues opcode32 via4E63C0; does not immediately4E45F0'
 local_UI='40B6F0/406CE0 and48C080 plain interaction also reject busy/834EB3;4E09D0 sends conditional attack wire, not an unconditional midstep4E45F0 producer'
 selected_gap='after movement end0 actor.frame0/current46,queue74zero,carry4E2150(owner0) pending;4E6AF0false. Before due dispatch source order must be explicit, never inferred from same timestamp.'
 immediate='4A5A90 nonbusy ->if834EB4 then4E9800 (selected actor12C/134 absent returns) ->actor68 absent ->4E45F0(arg1)'
 cancel='4E46AF true834EE4 and834EB3 set clears834EB3/EB4,4D6870(fn4E2150,owner0) removes all matching; other functions/owners retained'
 cancellation_distinction=@('4D6870 matches callback and owner, removes all and rechecks shifted position','4D6970 matches callback regardless owner','4D69F0 gets clock; it does not cancel')
 position='4E45F0 selected ordinary/class734/type!=11/base46/action!=8 branch unlinks prior draw, setsactor10=47, resets13=0 then advances1, sameheading4; no actorcell/camera/roof write. This cancels pending carry only after the prior cell already committed, not midstep quarter/camera.'
 flag='64BBC3/destination unchanged by selected4A5A90/4E45F0; cancelled movement does not automatically restart from this flag'
 first_attack='class734/base46 arg1 ->47; trueplayer neutral4E8750 first166ms;4D65F0 fresh+168 schedules4E34C0(actor pointer),expected=fresh+168'
 ordinary_attack='4E34C0 ordinary4E5B50 frames2..6,4D69F0 lateness;max(duration-lateness,1)+2 via4D66F0 with explicit samplebase. End0 retains action47 butcatalogframe0 usesbase46image72;queue0/actor50zero/neutralplayer4E02C0 createsidle49 opcode32+584 fresh timer; first idle49+168.'
 neutral_trueplayer='actual pointer/object/global relation conditional; speed831680/haste831718 zero,specialspeed false,counter/variant0; do not reuse earlier nonplayer20000 branch as trueplayer evidence'
 equal_clock='conditional packet-first at1664 can remove not-yet-dispatched carry; timer already removed/started must refuse this immediate profile; no guessed message-loop order'
 no_midstep_cancel_grant=$true;runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged'
})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r463-minimal-runtime-producer-gaps.json') -Value ([ordered]@{state='UNOBSERVED_RECEIVED_ACTION_AND_RUNTIME_ORDER_REFUSE';required=@(
 [ordered]@{field='receivedpacket opcode32/DC/object/action1 and dispatcher branch';producer='network receive4B2970/4A5A90/4A2A50';owner='actual selected native session';missing='only bounded offline bytes, no actual incoming message/source session/order'},
 [ordered]@{field='registry pointer==834EE4;object==75AA48;frame0/queue0/base46/type647E74!=11';producer='4E8160 actor registry and4E6AF0';owner='true local actor';missing='opaque pointer and class metadata not observed; explicit conditional branch facts'},
 [ordered]@{field='834EB3/EB4,12C/134/68 absence;pendingfn4E2150 owner0';producer='end0 carry schedule and original actor queue';owner='original process timer table';missing='native timer records unobserved; only private source-typed pending carry'},
 [ordered]@{field='sameclock packet-before-carry,dispatch not started';producer='original event/message/scheduler loop';owner='runtime thread clock';missing='equal timestamp alone insufficient; explicit observation requirement; no GUI/hooks execution'},
 [ordered]@{field='trueplayer speed/haste/action47duration,newordinaryowner and expectedclock';producer='4E8750/4E5B50/4D65F0/4D66F0';owner='actual actor pointer';missing='neutral values and fresh samples conditional; not nonplayer clock grant'},
 [ordered]@{field='busy midstep action queue,action8 and optional effects';producer='4E63C0/4E6900/4E9300,4E45F0 special branches';owner='actor queued message/resources';missing='profile deferred; no immediate-cancel or position rollback invented'},
 [ordered]@{field='live appearance/Work/native framebuffer';producer='real runtime bridge';owner='actual runtime epoch';missing='all unqualified; CPU RGB555 reference only'}
 );runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0})
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json;$s.updated_at=[DateTimeOffset]::Now.ToString('o');$s.phase='R463_source_received_action_gap_cancel_contract_frozen';$s.current_task="R463 $($pins.Count)wholeASM frozen source contract/gaps. Actualpacket32/DC action1 immediate switch only atframe0 queue0 gap;midstep4E6AF0busy queues action. After1498 cell commit pending1664 owner0 carry can be canceled4D6870 under explicit packet-before-timer order. Implement trueplayer neutral attack with private scene/preflight, no R459-R462 rerun or runtime grant.";Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output "R463 source contract frozen: $($pins.Count) whole functions"
