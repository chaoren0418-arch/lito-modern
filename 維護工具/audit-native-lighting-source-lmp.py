"""Bounded original source.lmp catalogue and static mask-pipeline provenance."""
import hashlib
import json
from pathlib import Path
import re
import struct
import zipfile

ROOT=Path(__file__).resolve().parents[1]
ASSETS=Path(r'D:/lineage開發/0-客戶端')
OUT=ROOT/'驗證紀錄/current/native-lighting-source-lmp'
BASE=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
indices=sorted(list(ASSETS.glob('*.idx'))+list((ASSETS/'LiTo').glob('*.idx')))
assert len(indices)==24
matches=[];index_pins={}
for index in indices:
    data=index.read_bytes();assert len(data)>=4
    count=struct.unpack_from('<I',data)[0];assert len(data)==4+count*28
    index_pins[str(index)]=hashlib.sha256(data).hexdigest().upper()
    for ordinal in range(count):
        record=data[4+ordinal*28:4+(ordinal+1)*28]
        offset,name,size=struct.unpack('<I20sI',record)
        if name.split(b'\0')[0].lower()!=b'source.lmp':continue
        pak=index.with_suffix('.pak');assert 0<size<=4*1024*1024 and offset+size<=pak.stat().st_size
        with pak.open('rb') as stream:stream.seek(offset);payload=stream.read(size)
        assert len(payload)==size
        matches.append((payload,{'index':str(index),'index_sha256':index_pins[str(index)],
            'entry_ordinal':ordinal,'entry_record_hex':record.hex(),'pak':str(pak),
            'pak_bytes':pak.stat().st_size,'entry_offset':offset,'entry_bytes':size}))
assert len(matches)==1
payload,source=matches[0]
digest=hashlib.sha256(payload).hexdigest().upper()
assert len(payload)==152736 and digest=='30CF051FE516A053AC494B8054E5A320BAB1F3F3DD7B0010E0CF0C74AACCE5AA'
assert len(payload)>=192
offsets=struct.unpack_from('<16I',payload)
dimensions=[struct.unpack_from('<ii',payload,64+i*8) for i in range(16)]
slots=[];spans={}
for radius,(offset,(width,height)) in enumerate(zip(offsets,dimensions)):
    assert 0<=width<=320 and 0<=height<=204
    if width==0 or height==0:
        assert (radius,offset,width,height)==(15,0,0,0)
        slots.append({'range':radius,'offset':offset,'width':width,'height':height,'empty':True})
        continue
    assert 192<=offset and offset+width*height<=len(payload)
    pixels=payload[offset:offset+width*height]
    spans[(offset,offset+len(pixels))]=pixels
    slots.append({'range':radius,'offset':offset,'width':width,'height':height,'empty':False,
        'pixels':len(pixels),'minimum':min(pixels),'maximum':max(pixels),
        'nonzero_pixels':sum(v!=0 for v in pixels),'center':pixels[(height//2)*width+width//2],
        'raw_mask_sha256':hashlib.sha256(pixels).hexdigest().upper()})
assert len(spans)==8
end=192
for a,b in sorted(spans):assert a==end;end=b
assert end==len(payload)
for first in range(0,14,2):
    assert (slots[first]['offset'],slots[first]['width'],slots[first]['height'])==(slots[first+1]['offset'],slots[first+1]['width'],slots[first+1]['height'])
selected_path=ROOT/'驗證紀錄/current/native-lighting-raster-bindings/selected-assembly.json'
binding=read(selected_path.parent/'qualified.json')
assert sha(selected_path)==binding['file_pins']['selected-assembly.json']
selected=read(selected_path)
initializer=selected['sub_4831B0']['assembly'];producer=selected['sub_485A40']['assembly']
for text in ['aSourceLmp','push    0FF00h','lea     ecx, [esi+40h]','add     ecx, 8','cmp     eax, offset dword_75AB60']:
    assert text in initializer,text
for text in ['mov     al, [esi+12h]','cmp     al, 0Fh','movsx   eax, al','call    sub_4BB170','cmp     eax, 2BCh']:
    assert text in producer,text
assembly_path=BASE/'full-disassembly.asm'
assert sha(assembly_path)==binding['original_ASM_sha256']
text=assembly_path.read_text(encoding='utf-8')
for name in ['sub_4BB170','sub_4BB230','sub_4E8EA0','sub_4E89F0','sub_4B8FC0']:
    match=re.search(r'(?ms)^'+name+r' proc\b.*?^'+name+r' endp\b',text);assert match
    selected[name]={'assembly':match.group(0)}
compositor=selected['sub_4BB170']['assembly'];clear=selected['sub_4BB230']['assembly']
assert 'cmp     eax, 140h' in compositor and 'cmp     esi, 0CCh' in compositor
assert 'cmp     [eax], cl' in compositor and 'jnb     short loc_4BB203' in compositor
assert 'mov     [eax], cl' in compositor and 'add     esi, 140h' in compositor
assert 'mov     ecx, 3FC0h' in clear and 'rep stosd' in clear and 'xor     eax, eax' in clear
normal_path=ROOT/'驗證紀錄/current/native-light-spell/normal/qualified.json'
normal=read(normal_path);pins=read(normal_path.parent/'source-inputs.json');assert len(pins)==970
for name,h in pins.items():assert sha(ROOT/name)==h,name
OUT.mkdir(exist_ok=False)
(OUT/'source.lmp').write_bytes(payload)
(OUT/'selected-assembly.json').write_text(json.dumps(selected,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
proof={'status':'original_source_lmp_slots_and_mask_pipeline_pinned_static_only',
    'source':source,'source_lmp_sha256':digest,'indices_checked':24,'index_pins':index_pins,
    'matches':1,'header_bytes':192,'slots':slots,'unique_nonempty_spans':8,
    'complete_payload_extent_accounted':True,
    'original_mask_working_extent':[320,204],'original_mask_bytes':65280,
    'source_observations':{'clear_function':'sub_4BB230','clear_value':0,
        'compositor':'sub_4BB170','composition':'unsigned per-byte maximum with rectangle clipping',
        'producer':'sub_485A40','actor_light_offset':'0x12',
        'positive_signed_light_range_max15':True,'source_range_sum_break_at':700,
        'source_center_alignment':'original x is halved then low2 bits cleared before compositor'},
    'runtime_stencil_selection_and_projection_verified':False,
    'native_compositor_execution_verified':False,'full_light_mask_generation_verified':False,
    'mode_flag_semantics_resolved':False,'full_world_framebuffer_verified':False,
    'client_source_files_unchanged':970,'product_source_changes':0,'game_sessions':0,'whole_goal_complete':False}
(OUT/'audit.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files={p.name:p for p in OUT.iterdir() if p.is_file()};files.update({'auditor.py':Path(__file__),'normal-qualified.json':normal_path})
fp={n:sha(p) for n,p in files.items()};archive=OUT/'source-lmp-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
    for n,h in fp.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
(OUT/'qualified.json').write_text(json.dumps({**proof,'archive_sha256':sha(archive),'file_pins':fp,'archive_readback_verified':True},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','indices_checked','matches','source_lmp_sha256','unique_nonempty_spans','game_sessions']},ensure_ascii=False))
