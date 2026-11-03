"""Read retained successful graph vectors after the final observer name collision."""
import hashlib
import json
from pathlib import Path
import struct
import zipfile

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-lighting-mask-producer'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
assert not (OUT/'qualified.json').exists()
native=read(OUT/'native.json');cases=read(OUT/'cases.json');vectors=read(OUT/'verified-vectors.json');retained=read(OUT/'retention.json')
assert len(cases)==len(vectors)==native['cases']==864
assert native['mask_bytes']==864*65280 and native['blits']==1166
assert native['guard_bytes_checked']==864*128 and native['actor_bytes_unchanged']==864*128*128
assert read(OUT/'observer-finalization-failure.json')['native_and_scalar_comparison_completed']
assert retained['readback_verified'] and sha(OUT/'producer-evidence.zip')==retained['archive_sha256']
with zipfile.ZipFile(OUT/'producer-evidence.zip') as z:
    assert set(z.namelist())==set(retained['files'])
    for name,h in retained['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,name
data=(OUT/'native-results.bin').read_bytes();at=0;blits=0;queries=0
for i,(case,vector) in enumerate(zip(cases,vectors)):
    assert vector['case']==i and len(case['actors'])==case['header'][10]
    mask=data[at:at+65280];at+=65280;row=list(data[at:at+32]);at+=32;cached,n=struct.unpack_from('<iI',data,at);at+=8
    calls=[list(struct.unpack_from('<5i',data,at+k*20)) for k in range(n)];at+=n*20
    count=len(case['actors'])*3;values=list(data[at:at+count]);at+=count
    assert hashlib.sha256(mask).hexdigest().upper()==vector['mask_sha256']
    assert row==vector['remap'] and cached==vector['cached'] and calls==vector['blits'] and values==vector['actor_queries']
    blits+=n;queries+=count
assert at==len(data) and blits==1166
normal_dir=ROOT/'驗證紀錄/current/native-game-timers/20261016-004010-201637/normal'
normal=read(normal_dir/'qualified.json');pins=read(normal_dir/'source-inputs.json');assert len(pins)==976
for n,h in pins.items():assert sha(ROOT/n)==h,n
for v in normal['profiles'].values():assert sha(Path(v['path']))==v['sha256']
assert sha(OUT/'source.lmp')=='30CF051FE516A053AC494B8054E5A320BAB1F3F3DD7B0010E0CF0C74AACCE5AA'
functions=read(OUT/'original-functions.json')['functions'];assert set(functions)=={'producer','project','frame_count','classify','predicate','compare','remap','blend','clear'}
image=(OUT/'graph-original.bin').read_bytes()
for f in functions.values():
    address=int(f['address'],16)-0x480000
    assert hashlib.sha256(image[address:address+f['bytes']]).hexdigest().upper()==f['sha256']
assert image[0x80000:0x80000+184]==image[0x3b170:0x3b170+184]

# Preserve compact static facts needed for the later production integration.
asm_path=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm')
assert sha(asm_path)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=asm_path.read_bytes().decode('utf-8').split('\n')
selected={}
for name in ['sub_4A5E00','sub_4D0470','sub_4C51B0','sub_4E8BD0']:
    a=next(i for i,s in enumerate(lines) if s.startswith(name+' proc'))
    b=next(i for i in range(a,len(lines)) if lines[i].startswith(name+' endp'))
    selected[name]={'first_newline_line':a+1,'last_newline_line':b+1,'assembly':'\n'.join(lines[a:b+1])}
loader={}
for name,needle in [('kind102','mov     edx, dword_647E74'),('subtype108','mov     edx, dword_647E64')]:
    i=next(i for i,s in enumerate(lines) if s==needle and 132000<i<136000)
    loader[name]={'first_newline_line':i-8+1,'assembly':'\n'.join(lines[i-8:i+5])}
control={'original_ASM_sha256':sha(asm_path),'selected':selected,'catalog_loader_fragments':loader,
    'S21_flow':'4D02F0 returns the light level; 4D0470 may store it in encoded 8316A0 and returns minute-of-day; S21 then supplies that minute to 4C51B0.',
    '4C51B0_scope':'Minute-conditioned map transitions/water palette/scroll state; not the ambient level setter.',
    'catalog_kind':'647E74 is populated by attribute102; 647E64 by attribute108.',
    'packet_to_native_actor_field_mapping_verified':False,'complete_world_paint_light_gate_verified':False}
(OUT/'control-flow-supplement.json').write_text(json.dumps(control,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(OUT/'qualifier.py').write_bytes(Path(__file__).read_bytes())
(OUT/'audit-fixed-not-reexecuted.py').write_bytes((ROOT/'維護工具/audit-native-lighting-mask-producer.py').read_bytes())
supplement_names=['observer-finalization-failure.json','control-flow-supplement.json','qualifier.py','audit-fixed-not-reexecuted.py']
supplement={n:sha(OUT/n) for n in supplement_names};packed=OUT/'observer-recovery-evidence.zip'
with zipfile.ZipFile(packed,'x',zipfile.ZIP_DEFLATED) as z:
    for n in supplement:z.write(OUT/n,n)
with zipfile.ZipFile(packed) as z:
    for n,h in supplement.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
result={'status':'qualified_retained_original_mask_producer_graph_after_final_observer_recovery',
    **native,'functions_executed':list(functions),'classifier_predicate_and_frame_query_bytes_checked':queries,
    'native_and_independent_scalar_mask_cases':864,'current_product_source_files_unchanged':976,
    'current_product_source_manifest_sha256':normal['source_manifest_sha256'],
    'current_normal_mains':{p:v['sha256'] for p,v in normal['profiles'].items()},
    'raw_archive_sha256':retained['archive_sha256'],'raw_archive_readback_verified':True,
    'observer_recovery_archive_sha256':sha(packed),'native_graph_reexecuted_by_recovery':False,
    'runtime_or_raw_outputs_rewritten':False,'ASCII_casefold_and_catalog_tables_supplied_as_fixtures':True,
    'blit_dependency_entry_uses_counter_wrapper_then_unmodified_original_copy':True,
    'original_actor_source_lmp_and_guards_unchanged':True,'actual_SPR_loader_execution_verified':False,
    'packet_to_native_coordinate_and_actor_flag_mapping_verified':False,'full_world_framebuffer_verified':False,
    'original_client_calendar_UTC_or_local_resolved':False,'new_game_sessions':0,'whole_goal_complete':False,
    'qualifier_sha256':sha(Path(__file__))}
(OUT/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,ensure_ascii=False))
