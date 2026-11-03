$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or (Get-Item -LiteralPath $p.path).Length -ne $p.bytes){throw ('T40 source drift '+$p.path)}}
$handoffPath='D:\lineage開發\3-lito開發\lito登入器\Native-World與Actor建立順序及操作門檻交接.json'
$handoff=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
if($handoff.contract.sha256 -ne '452A230681A47F4CDAA7F85D47631898BEF22A898DE0E940693CCA80D01D0E4F'){throw 'Unexpected T40 contract'}
Check $handoff.contract;Check $handoff.operation_boundary_contract;Check $handoff.validation
$contract=Get-Content -LiteralPath $handoff.contract.path -Raw|ConvertFrom-Json;Check $contract.native_index
$index=Get-Content -LiteralPath $contract.native_index.path -Raw|ConvertFrom-Json;Check $index.PE
$matches=[Collections.Generic.List[object]]::new()
function Seek($x){if($null -eq $x -or $x -is [string] -or $x -is [ValueType]){return};if($x -is [array]){foreach($a in $x){Seek $a};return};if($x.path -like '*new-aligned-constructor-pe-4DFDD0-4E01A0.asm'){$matches.Add($x)};foreach($prop in $x.PSObject.Properties){Seek $prop.Value}}
Seek $index;if($matches.Count -ne 1){throw 'Constructor dependency pin count'};$constructor=$matches[0];Check $constructor
$body=[IO.File]::ReadAllText($constructor.path);foreach($fact in @('004DFE05 mov dword ptr [esi + 0x7c], eax','004DFE0D mov dword ptr [esi], 0x4fabe4','004DFEAA mov eax, esi','004DFEB8 ret')){if(!$body.Contains($fact)){throw 'Missing aligned source fact'}}
$root=Join-Path $project 'target\r465-source-reload-reference\T40-dependency';if(Test-Path -LiteralPath $root){throw 'Already frozen'};[IO.Directory]::CreateDirectory($root)|Out-Null
$copies=@(foreach($p in @((Pin $handoffPath),$handoff.contract,$handoff.operation_boundary_contract,$handoff.validation,$contract.native_index,$constructor)){Check $p;$dst=Join-Path $root (Split-Path $p.path -Leaf);[IO.File]::Copy($p.path,$dst,$false);$pin=Pin $dst;if($pin.sha256 -ne $p.sha256){throw 'Dependency copy drift'};[ordered]@{external_original=$p;snapshot=$pin}})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r465-T40-bootstrap-source-dependency.json') -Value ([ordered]@{state='T40_FIXED_ALIGNED_CONSTRUCTOR_AND_LOCAL_KEY_ORDER_SOURCE_ADOPTED_CONDITIONALLY';snapshots=$copies;original_PE=$index.PE;expected_contract_sha256=$handoff.contract.sha256;proved_new_normal_path=@('encoded continuation fills actor7C allocation pointer, vtable4FABE4 and base fields','normalpath through4DFEB8 returns this; rawIDA endp was a prefix only','STATUS0C writes75AA48;PUT_OBJECT0B/4E7A40 missingactor ->constructor ->actor8key ->4E77E0 signedkey registry ->matching75AA48 assigns834EE4','HP0D updates protected8315F0 and8348BC; does not grant actor/input ready');model_boundary='R465 uses explicitsuccessful13C/16byteallocation/remaininginitialization/registry/pointer/camera/roof branch certificates. Additional decodednormalpath narrows source gap; no exception/wholeCFG/OOM/providerABI/nativeallocation/liveness grant. Source and scripts only read; originalPE not run.';supersedes='r465 frozen initial contract constructor truncated gap: normal continuation now source-supplemented; full safety/runtime still unqualified';runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0})
Write-Output 'R465 T40 dependency: six source snapshots checked; aligned normal constructor continuation adopted; runtime unqualified'
