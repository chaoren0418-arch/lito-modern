$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$old=Get-Content -LiteralPath (Join-Path $logs 'r463-original-source-pins.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $old.original_disassembly.path).Hash -ne $old.original_disassembly.sha256){throw 'Original drift'}
$text=[IO.File]::ReadAllText($old.original_disassembly.path)
$root=Join-Path $project 'target\r464-source-busy-queue-reference';if(Test-Path -LiteralPath $root){throw 'Already frozen'};[IO.Directory]::CreateDirectory($root)|Out-Null
$ids=@($old.files|ForEach-Object{[IO.Path]::GetFileNameWithoutExtension($_.path)})+@('4DFF00','4DFED0','4E82F0','4D68F0')
$pins=@(foreach($id in $ids|Select-Object -Unique){$m=[regex]::Matches($text,"(?ms)^sub_$id proc near.*?^sub_$id endp[^\r\n]*");if($m.Count -ne 1){throw "Whole function count $id"};$p=Join-Path $root "$id.asm";[IO.File]::WriteAllText($p,$m[0].Value,[Text.UTF8Encoding]::new($false));Pin $p})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r464-original-source-pins.json') -Value ([ordered]@{original_disassembly=$old.original_disassembly;files=$pins})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r464-source-busy-queue-contract.json') -Value ([ordered]@{
 state='SOURCE_BUSY_PACKET32_ACTION_QUEUE_FINISH_CELL_DEQUEUE_ATTACK_CONTRACT';source_pins=Pin (Join-Path $logs 'r464-original-source-pins.json')
 producer='4B2970 case32 ->4A5A90 ->4A2A50 dc; registry4E8160 actual local actor/object conditional. Ordinary moving46/frame>0 ->4E6AF0busy ->4E63C0. Action8 queue-clearing and currentaction8 excluded.'
 append='4E63C0 alloc8 node opcode32/argument; append atarray[count] and increment74. No overwrite. Only count>3 AND last.opcode32 AND last.arg==requested AND requested==2 returns before allocation. Duplicatedarg1 still appended, including count>3.'
 allocation='Node allocation null returns without append; no ACK implied. Count>=capacity doubles capacity78 and allocates newcapacity*4, copies count pointers and frees old array. Growth allocation success unguarded in native source; only explicit success conditional supported. Initial capacity unknown, finite model capacity2 is input assumption, never original constructor fact; private bound8 is not native capacity limit.'
 fifo='4E85F0 returns array[0], memmoves remaining count-1 pointers forward and decrementscount. FIFO follows this exact source, not timestamp inference. Null head can be removed then4E9300 returns; selected private profile only valid opcode32 nodes.'
 finish_cell='4E2150 end0 commits cell/camera and48AFC0 roof first, clears834EB3. queue74>0 schedules4E9300 with actor-pointer owner after returned166ms; does not schedule owner0 movement carry. Optional12C134 absent. Existing step prequeue not canceled by busy incoming action.'
 pop='4E9300 first4E85F0; case32 releases optionalactor68 then4E45F0(arg), frees poppednode. Selectedarg1 ordinarybase46 class734/type!=11 ->47/frame1, firstneutral trueplayer168ms4E34C0 actor-pointer. No further cell/camera/roof commit and no64BBC3/target clear.'
 attack_end='4E34C0 end0 keeps47/frame0/bank72+heading, frame0duration83 ->4E02C0. Existingqueue>0 schedules4E9300 after83ms; no584 idle added. queue0 idle49 path remains previous qualified behavior. Remaining queue is preserved throughordinary attack.'
 cleanup='4E82F0 selected actual registry actor with noopcode11 and notexcludedclass68: source draw/global unlink and registry removal ->4D68F0(actor) removes all timers matching owner, then virtualdestructor.4DFED0 ->4DFF00 frees all queued nodes, resets74/78 and frees7C. Base destructor alone does not prove timer cancellation. Owner0 callbacks not removed by4D68F0(actor); this queue profile already has none afterend0.'
 source_change='48BC80/source reload/profile swap does not by itself grant actor timer/queue cleanup. Private owner retire revokes its full queue/timer/frame state; native reload producer/order remains unknown and refused. Native removal opcode11 retention branches not implemented.'
 selected_recipe='Input clocks1200..1206 before pendingroot1254: args1,1,1,1,1,2,2; count1..6 thenlast2suppression,cap2->4->8.1498commit onecell ->1664popserial1 attack47;2549attackend ->2632popserial2. Fourremainingnodes andfuture2800 timer retired; action2 not executed.'
 no_live_ACK=$true;runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged'
})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r464-minimal-runtime-producer-gaps.json') -Value ([ordered]@{state='UNOBSERVED_BUSY_QUEUE_RUNTIME_INPUTS_REFUSE';required=@(
 [ordered]@{field='packet32/DC source session/object and native registry/global pointer';producer='4B2970/4A5A90/4E8160';missing='offline bytes and NAB binding do not prove received packet or native actor'},
 [ordered]@{field='initial capacity78/pointer7C/count74 and all successful allocations';producer='native actor construction and4E63C0 allocator';missing='initial2 conditional; native zero/failed-growth/invalidpointer branches unqualified'},
 [ordered]@{field='sameclock source order and timer removed/actor owner';producer='native event loop4D63A0/4D65F0';missing='equal clocks do not prove order; finite input uses strict-before root clocks'},
 [ordered]@{field='ordinary branch reached and decoded831590 safevalue3, neutral trueplayer clocks';producer='4E34C0/4E8750';missing='inherited R463 conditional ordinary entry; opaque runtime values unobserved'},
 [ordered]@{field='action8,opcode11 retained removal,queuedarg2 execution,source reload cleanup';producer='4A5A90/4E82F0/48BC80/4E45F0';missing='excluded branches require distinct proof and implementation; no blanket cleanup inferred'},
 [ordered]@{field='actual actor destruction/ownerall cancel and native raster';producer='4E82F0/4D68F0/4DFED0/4DFF00/runtimebridge';missing='source proved selected producer, private local retirement only; real registry/exit ordering not observed'}
 );runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0})
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json;$s.updated_at=[DateTimeOffset]::Now.ToString('o');$s.phase='R464_source_busy_queue_contract_frozen';$s.current_task='R464 source busy packet32 queue contract frozen: repeated1 appended, special repeated2 suppression onlycount>3, explicit capacity/allocation conditions; first-node dequeue sourceproven. Moving finishescell/camera/roof before actor-ownedpop, queuedattack end schedulesnextpop83 ratherthan584idle. Selected removal owner-all cancel thenqueuefree proven, native reload remainsunknown. Implement newprivateowner recipe; no R459-R463 rerun; modernonly/16of40/LOADEDfalse/GUI0.';Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output "R464 source contract frozen: $($pins.Count) whole functions"
