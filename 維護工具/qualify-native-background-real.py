"""Independently compare original real-IMG frames with vectorized integer math."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,json,struct,sys,zipfile
import numpy as np
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
BLEND=ROOT/'驗證紀錄/current/native-background-blend-original/20261016-114436-881465'
GEO=ROOT/'驗證紀錄/current/native-background-geometry-original/20261016-110731-202085'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-real-original')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:digest(p.read_bytes())
p=read(OUT/'preparation-readback.json');r=read(OUT/'run-receipt.json');s=read(OUT/'native-summary.json')
assert r['status']=='original_real_IMG_background_ran_not_yet_independently_qualified'
assert r['owned_process'] is None and r['build.log_exit_code']==r['native-summary.json_exit_code']==0
assert s['cases']==24 and s['guards_unchanged'] and not s['resource_loader_executed']
pins=read(OUT/'compiled-inputs.json');assert len(pins)==1012
for n,h in pins.items():assert sha(ROOT/n)==h,n
assert sha(OUT/'preparation-raw.zip')==p['archive_sha256']
with zipfile.ZipFile(OUT/'preparation-raw.zip') as z:
    for n,h in p['files'].items():assert digest(z.read(n))==sha(OUT/n)==h,n
with (OUT/'private-addresses.csv').open(encoding='ascii',newline='') as f:addresses=list(csv.DictReader(f))
for name,source,size in [('init',BLEND,2030),('rgb555',BLEND,1115),('geometry',GEO,1294)]:
    old=(OUT/(name+'-original.bin')).read_bytes();assert old==(source/(name+'-original.bin')).read_bytes() and len(old)==size
    reloc=(OUT/(name+'-relocated.bin')).read_bytes();expected=bytearray(old)
    selected=[v for v in addresses if v['function']==name]
    base=int(next(v['address'] for v in selected if v['kind']=='B'))
    with (OUT/(name+'-relocations.csv')).open(encoding='ascii',newline='') as f:changes=list(csv.reader(f))
    actual=[v for v in selected if v['kind']!='B'];assert len(actual)==len(changes)
    for (kind,offset,key),record in zip(changes,actual):
        offset,key=int(offset),int(key);assert record['kind']==kind and int(record['offset'])==offset
        address=int(record['address']);value=(address-base-offset-4)&0xffffffff if kind in ['F','E','C'] else address
        struct.pack_into('<I',expected,offset,value)
    assert bytes(expected)==reloc,name
exe=(OUT/'oracle.exe').read_bytes();e=struct.unpack_from('<I',exe,60)[0]
assert exe[:2]==b'MZ' and exe[e:e+4]==b'PE\0\0' and exe[e+4:e+6]==b'\x4c\x01' and exe[e+24:e+26]==b'\x0b\x01'
dll=Path(p['host_x86_CRT']);assert sha(dll)==p['host_CRT_sha256']
physical=Path((OUT/'host-CRT-final-path.txt').read_text(encoding='ascii').strip());assert physical.samefile(dll)
assert (OUT/'host-ftol-loaded.bin').read_bytes()==(BLEND/'host-ftol-loaded.bin').read_bytes()
assert read(OUT/'host-CRT-module.json')==read(BLEND/'host-CRT-module.json')
assert s['host_ftol_calls']==1024
previous=(BLEND/'initialized-mode0.bin').read_bytes();state=(OUT/'initialized-mode0.bin').read_bytes()
for kind,bases in enumerate([[0x81089c,0x81109c,0x81189c],[0x80d89c,0x80e09c,0x80e89c],
                             [0x82109c,0x82609c,0x82b09c],[0x81209c,0x81709c,0x81c09c]]):
    for address in bases:
        off=address-0x630000;size=2048*(10 if kind>=2 else 1);assert state[off:off+size]==previous[off:off+size]
images={}
for m in p['members']:
    raw=(OUT/m['member']).read_bytes();assert digest(raw)==m['sha256'] and len(raw)==m['bytes']
    w,h,flag,key=struct.unpack_from('<4H',raw);assert [w,h,flag,key]==[m['width'],m['height'],m['flag'],m['color_key']]
    words=np.frombuffer(raw,dtype='<u2',offset=8).reshape(h,w).astype(np.uint32);assert np.all(words<32768)
    images[int(m['member'].split('.')[0])]=words
geometry_raw=(ROOT/'LiTo381-Modern/tests/fixtures/native-background-geometry-original.json').read_bytes()
assert digest(geometry_raw)=='F4B9F257B740B197CCE25F9BA2369FF5F13A1F16355BBF27E5F8F4E35BE36091'
geometry=read(ROOT/'LiTo381-Modern/tests/fixtures/native-background-geometry-original.json')['cases']
cases=read(OUT/'reference-calls.json');assert len(cases)==24
with (OUT/'results-native.csv').open(encoding='ascii',newline='') as f:results=list(csv.DictReader(f))
assert len(results)==24
yy,xx=np.indices((552,744));vectors=[];total_calls=total_changes=0
for i,(case,result) in enumerate(zip(cases,results)):
    original=next(c for c in geometry if c['input']==case['input']);assert original['calls']==case['calls']
    expected=np.full((552,744),0x1777,dtype=np.uint32)
    gate=np.zeros((552,744),dtype=np.uint8) if case['gate']==0 else np.ones((552,744),dtype=np.uint8) if case['gate']==1 else ((xx+yy)%2*7).astype(np.uint8)
    for call in case['calls']:
        base_id,overlay_id,left,top,right,bottom,x,y,ox,oy,strength,mode=call
        a=images[base_id][top:bottom,left:right]
        phase_y=oy+(192 if (ox//384)%2 else 0)
        u=384-ox%384+left;v=384-phase_y%384+top
        b=images[overlay_id][v:v+bottom-top,u:u+right-left];assert a.shape==b.shape
        output=np.zeros_like(a)
        for shift in [0,5,10]:
            ac=(a>>shift)&31;bc=(b>>shift)&31
            c=31-((31-ac)*(31-bc)+30)//31 if mode==0 else ac*bc//31
            if strength!=32:c=c*(15+strength*8//5)//31
            output|=c<<shift
        dest=expected[y:y+bottom-top,x:x+right-left]
        np.copyto(dest,output,where=gate[y:y+bottom-top,x:x+right-left]==0)
    raw=(OUT/f'frame-{i}.bin').read_bytes();assert len(raw)==744*552*2
    actual=np.frombuffer(raw,dtype='<u2').reshape(552,744)
    assert np.array_equal(actual,expected),f'case{i}'
    changes=int(np.count_nonzero(actual!=0x1777));calls=len(case['calls'])
    assert result=={'case':str(i),'calls':str(calls),'locks':str(calls),'unlocks':str(calls*2),'changed_words':str(changes)}
    vectors.append({'input':case['input'],'gate':case['gate'],'frame_sha256':digest(raw),'changed_words':changes,'pixel_calls':calls})
    total_calls+=calls;total_changes+=changes
assert s['pixel_calls']==total_calls and s['changed_words']==total_changes
save(OUT/'frame-vectors.json',{'vectors':vectors})
(OUT/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files={f.name:sha(f) for f in OUT.iterdir() if f.is_file() and f.suffix!='.zip'}
with zipfile.ZipFile(OUT/'run-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(OUT/n,n)
with zipfile.ZipFile(OUT/'run-raw.zip') as z:
    assert set(z.namelist())==set(files)
    for n,h in files.items():assert digest(z.read(n))==h,n
q={'status':'independently_verified_original_geometry_and_RGB555_pixels_with_real_warm_IMG',
    'observed_utc':datetime.now(timezone.utc).isoformat(),'path':str(OUT),'source_files_unchanged':1012,
    'original_function_bytes':4439,'real_IMG_members':5,'actual_frames':24,'pixel_calls':total_calls,'changed_words':total_changes,
    'all_frames_equal_independent_integer_scalar':True,'all_guards_unchanged':True,'original_instructions_only_allowed_relocations':True,
    'host_CRT_sha256':p['host_CRT_sha256'],'historical_CRT_DLL_identity_verified':False,'resource_loader_executed':False,
    'normal_world_adapter_verified':False,'map303_overlay_gate_complete':False,'game_sessions':0,'whole_goal_complete':False,
    'raw_archive_sha256':sha(OUT/'run-raw.zip'),'files':files,'all_archives_readback_verified':True}
assert not (OUT/'qualified.json').exists();save(OUT/'qualified.json',q);save(APP/'background-real-qualified-current.json',q)
print(json.dumps({k:q[k] for k in ['status','actual_frames','pixel_calls','changed_words','all_archives_readback_verified']},ensure_ascii=False))
