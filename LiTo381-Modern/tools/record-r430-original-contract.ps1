param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$pointer=Get-Content -LiteralPath (Join-Path $logs 'r430-original-route-snapshot.json') -Raw|ConvertFrom-Json
if($pointer.sha256 -ne '2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A' -or (Get-FileHash -LiteralPath $pointer.original_disassembly).Hash -ne $pointer.sha256){throw 'Original ASM drift'}
$all=[IO.File]::ReadAllText($pointer.original_disassembly)
$oldRoot=(Get-Content -LiteralPath (Join-Path $logs 'r429-catalog-snapshot.json') -Raw|ConvertFrom-Json).root
$names=@('sub_4D6360','sub_4D69F0','sub_4D65F0','sub_4E5B50','sub_4E1410','sub_4E1870','sub_4E34C0','sub_4E45F0','sub_4E5C00','sub_4E6AF0','sub_4E7A40','sub_4E96D0','sub_43DE10','sub_43DDD0','sub_43FFC0','sub_4406C0','sub_440010','sub_42EA50','sub_4CD9D0','sub_483770','sub_482D90','sub_483A80','sub_4E02C0')
$files=@(foreach($name in $names){$m=[regex]::Match($all,'(?ms)^'+$name+' proc near\b.*?^'+$name+' endp\b');if(-not $m.Success){throw "Original function missing $name"};$path=Join-Path $pointer.root ($name+'.asm');[IO.File]::WriteAllText($path,$m.Value,[Text.UTF8Encoding]::new($false));[ordered]@{function=$name;path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}})
$source='D:\lineage開發\1-伺服器\src\com\lineage\server\clientpackets\C_CreateChar.java'
$create=[IO.File]::ReadAllText($source)
foreach($fragment in @('new int[] { 0, 61, 138, 734, 2786, 6658, 6671 }','new int[] { 1, 48, 37, 1186, 2796, 6661, 6650 }','CLASS_LIST[pc.get_sex()][pc.getType()]','pc.setClassId(classid)')){if(-not $create.Contains($fragment)){throw 'Original class mapping mismatch'}}
$classSnapshot=Join-Path $pointer.root 'C_CreateChar.java'
if(-not (Test-Path -LiteralPath $classSnapshot)){[IO.File]::Copy($source,$classSnapshot,$false)}
if((Get-FileHash -LiteralPath $classSnapshot).Hash -ne (Get-FileHash -LiteralPath $source).Hash){throw 'Class mapping source drift'}
$checks=@(
 @{name='sub_43DE10';parts=@('mov     [ebp+var_1C], 18h','cmp     edi, 6Eh','mov     [ebp+var_1C], eax','lea     eax, [eax+eax*4]','shl     eax, 3','idiv    [ebp+var_1C]','xor     eax, 1812h','mov     [edx+edi+0Ah], ax');meaning='Native default FPS24; opcode110 override; duration ticks multiplied5*5*5*8=1000 then per-frame signed integer division and u16 XOR storage; global647EA4 modifiers separate'},
 @{name='sub_4E8750';parts=@('mov     di, [edx+eax*4+0Ah]','xor     edx, 1812h');meaning='Frame duration is decoded from original catalog, with separate speed/brave/class modifiers; local unmodified catalog study does not qualify them'},
 @{name='sub_4D69F0';parts=@('push    3E8h','call    sub_4F11F0','call    sub_4F1230');meaning='Relative timer scales counter by1000 and divides stored frequency when enabled; obfuscated external API targets not named by this evidence'},
 @{name='sub_4D65F0';parts=@('call    sub_4D69F0','add     esi, edi','mov     [esi], edx');meaning='Scheduler stores current time plus requested duration'},
 @{name='sub_4E5B50';parts=@('inc     bl','mov     [ecx+13h], bl','cmp     dl, [edi+eax*4+1]','mov     [ecx+13h], dl','call    sub_4E5C00');meaning='Frame index advances and resets0 at frame count; callback return distinguishes action end. This alone is not full repeat/lifetime/start-phase qualification'},
 @{name='sub_4E96D0';parts=@('xor     edi, 1951h','add     edi, eax','call    sub_4406C0');meaning='Directed image adds actor heading to catalog image group before original archive lookup'},
 @{name='sub_4406C0';parts=@('cmp     ecx, esi','movsx   ecx, ax','offset aDDSpr');meaning='Logical0 below basecount uses base catalog graphics alias3225, member3225-image.spr; Native LTlist route stays distinct'},
 @{name='sub_440010';parts=@('call    sub_43FFC0','mov     ecx, dword_6479A8[eax*4]','mov     ecx, dword_647984');meaning='Original hash-selected SpriteNN index first, then base Sprite on verified MISS; four selected indexes have no3225-0/1/8/9'},
 @{name='sub_4E7A40';parts=@('mov     [esi+10h], bl','mov     [esi+18h], bl','mov     byte ptr [esi+13h], 0','call    sub_4E02C0');meaning='Object action/base action originate from argument; initial frame index0 and conditional idle queue. ExtendedLayer8 complete selection and loop still unknown'},
 @{name='sub_482D90';parts=@('add     edx, eax','call    sub_4E7A40');meaning='Map effect Native local id is translated by basecount before shared object driver'}
)
# 4E8750 was already pinned by R429; copy its entire original function here.
$m=[regex]::Match($all,'(?ms)^sub_4E8750 proc near\b.*?^sub_4E8750 endp\b');[IO.File]::WriteAllText((Join-Path $pointer.root 'sub_4E8750.asm'),$m.Value,[Text.UTF8Encoding]::new($false))
$rows=@(foreach($c in $checks){$p=Join-Path $pointer.root ($c.name+'.asm');$text=[IO.File]::ReadAllText($p);foreach($part in $c.parts){if(-not $text.Contains($part)){throw ('Missing static fragment '+$c.name+' '+$part)}};[ordered]@{function=$c.name;path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;fragments=$c.parts;interpretation=$c.meaning}})
$body=Get-Content -LiteralPath (Join-Path $logs 'r430-base-body-snapshot.json') -Raw|ConvertFrom-Json
$report=[ordered]@{updated_at=(Get-Date).ToString('o');state='PINNED_ORIGINAL_CLASS_BASE_ROUTE_AND_NATIVE_CATALOG_TIME';original_disassembly_sha256=$pointer.sha256;files=$files;checks=$rows;class_source=[ordered]@{path=$classSnapshot;sha256=(Get-FileHash -LiteralPath $classSnapshot).Hash};base_body_snapshot=$body;logical=0;graphics_alias=3225;class=0;sex=0;qualified_headings=@(0,1);explicit_unarmed_source_study=$true;Native_catalog_frame_ms_rule='positive signed-word trunc(ticks*1000/FPS) per frame; native default24/opcode110 from43DE10';clock_scope='unmodified catalog only; explicit elapsed from caller phase0; one interval; no native wall-start/loop/speed/brave/global modifier qualification';catalog90_91_dependency='LTtext only; base Text/list.spr, base Sprite pair and four original MISS indexes remain additional dependencies';SourceAttachment_body_binding=$false;LOADED=$false;GUI=0}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r430-native-actor-route-contract.json') -Value $report
Write-Output 'R430 original class/route/time contract pinned'
