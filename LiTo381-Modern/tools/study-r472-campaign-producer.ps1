$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$asm=Get-Content (Join-Path $logs 'r472-original-source-pins.json') -Raw|ConvertFrom-Json
if((Get-FileHash $asm.original_disassembly.path).Hash -ne $asm.original_disassembly.sha256){throw 'Original drift'}
$text=[IO.File]::ReadAllText($asm.original_disassembly.path)
$root=Join-Path $project 'target\r472-campaign-global-producer-boundary'
if(Test-Path $root){throw 'Already studied'}
[IO.Directory]::CreateDirectory($root)|Out-Null
$files=@(foreach($name in @('47C8A0','47EB50','492270','440770','4408D0','4D43D0')){$m=[regex]::Matches($text,'(?ms)^sub_'+$name+' proc near.*?^sub_'+$name+' endp[^\r\n]*');if($m.Count -ne 1){throw 'Declared count'};$p=Join-Path $root ("sub_$name.asm");[IO.File]::WriteAllText($p,$m[0].Value,[Text.UTF8Encoding]::new($false));Pin $p})
$fragments=@(foreach($spec in @(@('campaign-load-selected-branch','^loc_47C916:.*?(?=^loc_47C92A:)'),@('campaign-static-initial-pointer','^dword_647E18 dd 0[^\r\n]*'),@('campaign-static-initial-count','^dword_647E94 dd 0[^\r\n]*'))){$m=[regex]::Matches($text,'(?ms)'+$spec[1]);if($m.Count -ne 1){throw 'Fragment count'};$p=Join-Path $root ($spec[0]+'.asm-fragment');[IO.File]::WriteAllText($p,$m[0].Value,[Text.UTF8Encoding]::new($false));Pin $p})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r472-campaign-global-producer-gap.json') -Value ([ordered]@{
 state='5000_GLOBAL_CALLBACK_INPUT_PRODUCER_AND_LIFETIME_PINNED_NOT_EXECUTED'
 original_disassembly=$asm.original_disassembly;declared_bodies=$files;exact_fragments=$fragments;declared_endp_not_completeCFG=$true
 producer='Readableselected47C916 under64B6DCfalse and636584=3B5h or4E4h calls440770("campaignDesc", &647E94) thenstoresreturnedpointer647E18. Thisselectedregion occursbeyond47C8A0 declaredendp inIDA; exactfragment is separatelypinned, notclaimedwholedeclaredCFG.'
 loader='440770 obtainslanguagechar4D43D0 andformats%s-%c.tbl, opensvia647980/42E710, readsfirstline/773B0440 integer count with0<count<8000h, allocatescount*4 array andzeroesentries;43DCC0 parsesindices, duplicate/stringalloc77406880 assignsentry, readsuntilEOF. Returnsallocatedtable andwrites647E94. Source facts donotprovewhichlanguage/tablebytes/indices/non-nullstrings/allocations areactuallyloaded.'
 initializer='492270 undercampaigngate readsactual[647E18+10h] integer, forms647E94-value-7 divisor for773DC650 returnmodulo, sets647E98=remainder+value+6,647E9C0/647EA0false,cancels47CEC0/47CE50/47CDF0 owner0 thenconvertsactual[647E18] and[647E18+10h] toscheduleoriginaltimers. Random/providerinput andnonzero divisor/pointer/stringbounds areunobserved; no inventedindex/conversion.'
 unload='47EB50 selectedsamegate calls4408D0 tofreeallocatedstringentries/tablethenzeros647E18; timer/globalcallbackeffects/lifetimeordering needactualwitness. Static647E18/647E94 datazero isimageinitializationonly, notvalidcurrentstateat5000 anddoesnotjustifyNoOp/null dereference.'
 callback='Staticdd64decode47CEC0 writes647EA0true, dereferences[647E18+4],calls773B0440 andschedules47CDF0 delayconverted*1000 withfreshclock. 47CE50 increments/wraps647E98 andindexesstringarray;47CDF0 computesconverted[+4]-converted[0],increments647E9C/clearsEA0/reschedules47CEC0. No actualstringvalues/lifetime/counterclockcomplete snapshot means cannotcommitany5000branch.'
 retained_owner='4C32A0->4C32D0 countdown/index/length/reschedule ECXobject stillneedsnativeobjectrole/bounds/lifetime. Oldsyntheticownermetadata isnotproof nativefreedactor; do notreattachnewactor orclaimUAF.'
 minimum_missing=@('Current636584/64B6DC campaigngate and4D43D0 language/647980 table-reader witness','SelectedcampaignDesc-%c.tbl bytes and440770 loadedcount/indexednon-nullstringallocationresults','Sameprocesslive647E18pointer/647E94count/647E98index/647E9Ccounter/647EA0flag withunload/cancellationordering','Required773B0440 conversionreturns and773DC650initializer result/nonzero divisionbound','Fresh4D69F0/timerarraycapacity/order snapshots for5000/5001/5002','Retained4C32A0 ECXobjectrole/counters/bounds/lifetime andremainingwhitelisteffects')
 resource_data_missing_not_inferred_from_R439_subset=$true;new_Rust_change=$false;new_focused_test_run=$false;native_callback_executed=$false;external_provider_executed=$false;runtime_inputs_observed=$false;no_default_strings_or_noop=$true;next_cell5430_unexecuted=$true;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged';overall_status='in_progress'
})
Write-Output 'Campaign producer boundary pinned: 6 declared bodies + 3 exact fragments; no input invented'
