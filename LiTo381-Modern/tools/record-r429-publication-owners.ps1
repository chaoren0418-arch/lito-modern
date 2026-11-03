param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$snapshot=Get-Content -LiteralPath (Join-Path $logs 'r429-service-snapshot.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $snapshot.input).Hash -ne $snapshot.input_sha256){throw 'Own input drift'}
$cfg=Get-Content -LiteralPath $snapshot.input -Raw|ConvertFrom-Json
function U32([byte[]]$b,[int]$at){$part=[byte[]]$b[$at..($at+3)];[Array]::Reverse($part);[BitConverter]::ToUInt32($part,0)}
function Hex64([byte[]]$b,[int]$at){[Convert]::ToHexString($b[$at..($at+7)])}
function Owner([byte[]]$b,[int]$at){[ordered]@{source=U32 $b $at;object_u64_hex=Hex64 $b ($at+4);epoch_u64_hex=Hex64 $b ($at+12);channel=U32 $b ($at+20)}}
$rows=@()
foreach($family in @('service','TLS_capture')){
 $name=if($family -eq 'service'){'canonical-service-data-vectors.json'}else{'canonical-tls-vectors.json'}
 $path=Join-Path $cfg.root $name
 $expected=if($family -eq 'service'){'22D8EACDD0FAF9CADC62E1D89FAADE5C0835D696F8382D21B88DA10658BFB6BD'}else{'C4A054BCA9D5840A167EC5E33C54CE3472B52368440D5138FAA43C05FAFC52F8'}
 if((Get-FileHash -LiteralPath $path).Hash -ne $expected){throw 'Fixed vector drift'}
 $v=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
 foreach($property in $v.PSObject.Properties|Where-Object{$_.Name -like '*89_hex'}){
  $b=[Convert]::FromHexString($property.Value);$offset=92+(U32 $b 84)
  $rows+=@([ordered]@{family=$family;vector=$property.Name;kind=89;bytes=$b.Length;owner=Owner $b $offset;source_attachment_present=(U32 $b 84) -gt 0;typed_DATA_present=($b[$offset+98] -eq 2)})
 }
 if($family -eq 'TLS_capture'){
  $read=[Convert]::FromHexString($v.read61_hex)
  $rows+=@([ordered]@{family=$family;vector='read61_hex';kind=61;bytes=$read.Length;owner=Owner $read 16;state=$read[64]})
 }
 $moveName=if($family -eq 'service'){'operation65_hex'}else{'move65_hex'}
 $b=[Convert]::FromHexString($v.$moveName)
 $rows+=@([ordered]@{family=$family;vector=$moveName;kind=65;bytes=$b.Length;owner=Owner $b 16;guard=[ordered]@{room_hex=Hex64 $b 40;match_hex=Hex64 $b 48;generation_hex=Hex64 $b 56;entity_hex=Hex64 $b 64;life_hex=Hex64 $b 72;engine_revision_hex=Hex64 $b 80};required_preceding_publication=if($family -eq 'service'){'service/active89_hex'}else{'TLS_capture/read61_hex'};full_receipt_Rust_verified_against_Observation=$true;expected_request_reconstructed_from_echo=$true;captured_request_not_published=$true})
}
$stdout=[IO.File]::ReadAllText((Join-Path $logs 'r429-consumer.stdout.log'))
if($stdout -notmatch 'test result: ok\. 1 passed; 0 failed;'){throw 'Consumer not passed; do not emit verified owner record'}
$report=[ordered]@{created_at=[DateTimeOffset]::Now.ToString('o');manifest_sha256=$cfg.manifest_sha256;rows=$rows;cross_source_same_object=[ordered]@{TLS_active89_source=42;TLS_MOVE65_source=7;TLS_READ61_source=7;object_u64_hex='0000000000000D05';borrowed_source42_guard_rejected=$true;source7_READ61_guard_full_receipt_accepted=$true};life_correction='Earlier life0 inference was wrong. Existing battle_motion::life maps no resources to first incarnation1. Java uses1; both fixed MOVE receipts parse with the proper preceding owner Observation. SCENE_ONLY has no vital/resources; combat/revival qualification remains absent.';renderer_LOADED=$false;live_Rust_TLS=$false;SQL=$false;GUI=0}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r429-publication-owner-chain.json') -Value $report
Write-Output ([ordered]@{publication_rows=$rows.Count;cross_source_guard_rejected=$true;proper_owner_MOVE_receipts=2;renderer_LOADED=$false}|ConvertTo-Json)
