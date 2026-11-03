param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$pointer=Get-Content -LiteralPath (Join-Path $project 'logs\continuation-20261003\r429-catalog-snapshot.json') -Raw|ConvertFrom-Json
$asm='D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm'
if((Get-FileHash -LiteralPath $asm).Hash -ne $pointer.disassembly_sha256){throw 'Original disassembly drift'}
$all=[IO.File]::ReadAllText($asm)
$names=@('sub_482D90','sub_483A80','sub_4E7A40','sub_4E02C0','sub_4E63C0','sub_4E8750','sub_4E9300','sub_4D65F0')
$files=@(foreach($name in $names){
 $m=[regex]::Match($all,'(?ms)^'+$name+' proc near\b.*?^'+$name+' endp\b')
 if(!$m.Success){throw ('Static function missing '+$name)}
 $path=Join-Path $pointer.root ($name+'.asm')
 if(!(Test-Path -LiteralPath $path)){[IO.File]::WriteAllText($path,$m.Value,[Text.UTF8Encoding]::new($false))}
 [ordered]@{function=$name;path=$path;bytes=(Get-Item -LiteralPath $path).Length;sha256=(Get-FileHash -LiteralPath $path).Hash}
})
$checks=@(
 @{function='sub_43DE10';fragments=@('"list.spr"','"LTlist.spr"','mov     dword_647E90, eax','call    sub_4D4170');meaning='Two catalogs; base count is Native global-ID offset; not same-map Shared catalog inference'},
 @{function='sub_43DDD0';fragments=@('cmp     eax, esi','sub     eax, esi','mov     eax, [ecx+eax*4]');meaning='Native appended row selection subtracts base count'},
 @{function='sub_4406C0';fragments=@('mov     esi, dword_647E90','"L%d-%d.spr"','sub     ecx, esi');meaning='Native appended graphics generate L-prefixed names'},
 @{function='sub_42EA50';fragments=@("cmp     byte ptr [edi], 4Ch ; 'L'",'lea     ecx, [edi+1]','mov     ecx, dword_647998');meaning='Base Sprite fallback strips L then reads Native sprite manager'},
 @{function='sub_482D90';fragments=@('mov     edx, dword_75A854[edi]','mov     eax, dword_647E90','add     edx, eax','call    sub_4E7A40');meaning='Map effect local sprite ID adds base count before world object creation'},
 @{function='sub_4E02C0';fragments=@('lea     ebx, [edx+3]','add     ecx, 3','call    sub_4E63C0');meaning='Idle state may choose base action+3 when catalog action exists; complete extended Layer8 route remains unqualified'}
)
$rows=@(foreach($check in $checks){
 $path=Join-Path $pointer.root ($check.function+'.asm')
 $text=[IO.File]::ReadAllText($path)
 foreach($fragment in $check.fragments){if(!$text.Contains($fragment)){throw ('Static route fragment absent: '+$check.function+' '+$fragment)}}
 [ordered]@{function=$check.function;path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash;asserted_fragments=$check.fragments;interpretation=$check.meaning}
})
$report=[ordered]@{updated_at=[DateTimeOffset]::Now.ToString('o');state='PINNED_NATIVE_CATALOG_ROUTE_STATIC_SLICE';catalog_snapshot=$pointer.root;original_pe_sha256=$pointer.pe_sha256;disassembly_sha256=$pointer.disassembly_sha256;route_checks=$rows;additional_functions=$files;contract=@{catalog='LTtext/ltlist.spr protected old28 index and per-entry L1 cipher; decoded bytes must match released extracted copy';catalog_header='285 single integer, unlike Shared three-scalar header';selected_rows=@{'38'='FirePot action3 undirected group0 subframes0..10 duration2 each';'123'='FirePot action3 undirected group0 subframes0..10 duration2 each'};sprite_members=@('LTsprite/38-0.spr','LTsprite/123-0.spr');duration_domain='22 source units, samples0..21 only; no native millisecond/FPS or wrapping assertion';depth='SPR signed A/B, depth and source insertion index; native half-cell scan and newest-first equal-depth tie';unknowns=@('Extended Layer8 complete object/action binding','Native FPS conversion, scheduling start phase and repeat behavior','Actor body/equipment catalog and archive identity','Actor/roof/full world depth insertion sequence','LTtext identity absent from authoritative SourceAttachment')};LOADED=$false;formal_PLAYER_changed=$false;GUI=0;qualified_modes=16}
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r429-native-action-route-contract.json') -Value $report
Write-Output ([ordered]@{route_checks=$rows.Count;additional_functions=$files.Count;native_ms_clock=$false;LOADED=$false}|ConvertTo-Json)
