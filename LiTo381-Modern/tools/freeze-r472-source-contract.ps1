$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or (Get-Item -LiteralPath $p.path).Length -ne $p.bytes){throw 'Pin drift'}}
$prior=Get-Content (Join-Path $logs 'r471-original-source-pins.json') -Raw|ConvertFrom-Json
$study=Get-Content (Join-Path $logs 'r472-readonly-source-pins.json') -Raw|ConvertFrom-Json
Check $prior.original_disassembly
$text=[IO.File]::ReadAllText($prior.original_disassembly.path)
$root=Join-Path $project 'target\r472-idle-reclick-source-contract'
if(Test-Path $root){throw 'Already frozen'}
[IO.Directory]::CreateDirectory($root)|Out-Null
$seen=@{}
$files=@(foreach($p in @($prior.files)+@($study.declared_bodies)){Check $p;if(!$text.Contains([IO.File]::ReadAllText($p.path))){throw 'Source mismatch'};if(!$seen.ContainsKey($p.sha256)){$seen[$p.sha256]=$true;$dst=Join-Path $root (Split-Path $p.path -Leaf);[IO.File]::Copy($p.path,$dst,$false);Pin $dst}})
$fragments=@(foreach($p in @($prior.fragments)+@($study.exact_fragments)){Check $p;if(!$text.Contains([IO.File]::ReadAllText($p.path))){throw 'Fragment mismatch'};$dst=Join-Path $root (Split-Path $p.path -Leaf);[IO.File]::Copy($p.path,$dst,$false);Pin $dst})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r472-original-source-pins.json') -Value ([ordered]@{original_disassembly=$prior.original_disassembly;files=$files;fragments=$fragments;static_decode=$study.derived_static_decode;declared_endp_not_completeCFG=$true;native_executed=$false})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r472-source-idle-reclick-contract.json') -Value ([ordered]@{
 state='DEDICATED_SAME_STARTED_IDLE49_NORMAL513_RECLICK_CANCEL_RESET_MINIMAL_SOURCE_BRANCH'
 prior_R471=Pin (Join-Path $logs 'r471-global-success-queued-idle-handoff.json')
 readonly_boundary=Pin (Join-Path $logs 'r472-next-player-flow-boundary.json')
 source_pins=Pin (Join-Path $logs 'r472-original-source-pins.json')
 source_order='48C080 normal513 signed102ms gate uses priorcommitted3010. Busy4E6AF0 emptyqueue/frame1/actionbase+3 removes4E34C0 currentowner timer5094 before returnsfalse; frame stays1.4E2150 frame1+idle4E82B0 enters4E240F;4E2438 repeats cancellation (zero matching remaining), setsvar_E1. Neutral139FF/targetunequal/collisionqualified carry changesaction46/prequeues3fresh clocks;4E2E7C var_E1 resetsframe0 then4E5B50 increments1. Projection prepares entire conditional transaction before commit; rejectedoffline preparation leaves model unchanged, notproofnativeinvalidinput haszeroeffects.'
 action_distinction='Actor action49 is not decodedglobal831660=46. Valid49 load no831660writer; samebase46four-frame Walk table. Runtime decoded49 is rejected. Existingexpected5094 actor-memory retained; onlytimer cancelled.'
 recipe='New4721/4722 own necessarysource prefix throughglobal4502 conditionalpositive1+2 andactual4925idle once, WITHOUT oldfocused invocation orold54negative recipe. Normal5134930/currentmouse23824/currentcamera32752,32782 targets32755,32791. Cancelonlycurrentactor5094; threefresh4930/31/32 ->5096/5263/5430 owner0;46frame1bank76sub1, firstquartercameraoffset/ring changes butcameraCENTER/queuecap4/serialhistory/fullsourcebinding/owner unchanged. Nocellcommit/roofwrite/constructor/queueallocation. Endat4930 beforefirst4974keeper; no5000 callback skipped.'
 provider='4502 prefix retainsR471 requiredUNOBSERVEDreturn-only/noapplicationeffects andpositiveproviderreturns premise. Noactualsend/provider/nativeexecution; conditional success projection isnotruntimeevidence.'
 gap='47CEC0 dd64 staticdecode coveredall64bytes:647EA0=1,convert actualstring at647E18+4 via773B0440,EAX*1000,schedule47CDF0 owner0/fresh. Noactual647E18 pointer/strings/conversionresults/currentglobals witness; cannotinventdefaults or markNoOp. 5001/5002 mutuallychange647E98/9C/EA0 andstringarray indices. 5004 retained4C32A0->4C32D0 needsrealECXobjectrole/counters/bounds/lifetime; syntheticoldOwner metadata notnativeUAF proof. Consequently5096/5263/5430 andnextcell remainunexecuted.'
 runtime_inputs_observed=$false;external_provider_executed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged';overall_status='in_progress'
})
Write-Output "R472 source frozen: $($files.Count) declared bodies / $($fragments.Count) fragments"
