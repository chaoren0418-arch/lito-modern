"""Read back the bounded original initializer/pixels and pinned host conversion."""
from pathlib import Path
from datetime import datetime,timezone
import array,csv,hashlib,json,struct,sys,zipfile
import capstone

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-blend-original')
GB=0x630000;GE=0x832000
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
prep=read(OUT/'preparation-readback.json');run=read(OUT/'run-receipt.json');raw=read(OUT/'run-readback.json')
assert run['status']==raw['status']=='original_background_tables_and_pixels_match_independent_scalar'
assert run['owned_process'] is None and run['build.log_exit_code']==run['native-summary.json_exit_code']==0
assert len(run['process_history'])==2 and all(v['exit_code']==0 for v in run['process_history'])
assert run['process_history'][1]['arguments']==[str(OUT/'oracle.exe'),str(OUT)]
for archive,files,expected in [('preparation-raw.zip',prep['files'],prep['raw_archive_sha256']),('run-raw.zip',raw['files'],raw['raw_archive_sha256'])]:
    assert sha(OUT/archive)==expected
    with zipfile.ZipFile(OUT/archive) as z:
        assert set(z.namelist())==set(files)
        for n,h in files.items():assert digest(z.read(n))==sha(OUT/n)==h,n
pins=read(NORMAL/'source-inputs.json');nq=read(NORMAL/'qualified.json');assert len(pins)==1005
for n,h in pins.items():assert sha(ROOT/n)==h,n
for p,v in nq['profiles'].items():assert sha(v['path'])==v['sha256']==prep['normal_mains'][p]
assert sha(NORMAL/'source-inputs.json')==prep['source_manifest_sha256']
host=Path((OUT/'host-CRT-final-path.txt').read_text(encoding='ascii').strip()).resolve(strict=True)
assert host.samefile(prep['host_x86_CRT']) and sha(host)==prep['host_CRT_sha256']
module=read(OUT/'host-CRT-module.json');assert module=={'machine':332,'optional_header_magic':267,'image_size':782336,'ftol_RVA':633888,'captured_ftol_bytes':48}
dll=host.read_bytes();eh=struct.unpack_from('<I',dll,60)[0];optional=eh+24
for i in range(struct.unpack_from('<H',dll,eh+6)[0]):
    _,va,size,off=struct.unpack_from('<IIII',dll,optional+struct.unpack_from('<H',dll,eh+20)[0]+i*40+8)
    if va<=0x9ac20 and 0x9ac50<=va+size:expected_host=dll[off+0x9ac20-va:off+0x9ac50-va];break
else:raise AssertionError('host conversion section')
loaded=(OUT/'host-ftol-loaded.bin').read_bytes();assert loaded==expected_host
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);leaf=[]
for i in cs.disasm(loaded,0x9ac20):
    leaf.append(i)
    if i.mnemonic=='ret':break
assert leaf[-1].address+leaf[-1].size==0x9ac47 and all(i.mnemonic not in ['call','jmp'] and not i.mnemonic.startswith('j') for i in leaf)
assert [('or','ah, 0xc'),('fistp','qword ptr [ebp - 0xc]')] == [(i.mnemonic,i.op_str) for i in leaf if i.mnemonic in ['or','fistp']]
assert [i.op_str for i in leaf if i.mnemonic=='fldcw']==['word ptr [ebp - 4]','word ptr [ebp - 2]']
functions=read(OUT/'extraction.json')
with (OUT/'private-addresses.csv').open(encoding='ascii',newline='') as f:addresses={(r['function'],r['kind'],int(r['offset'])):int(r['address']) for r in csv.DictReader(f)}
relocation_counts={}
for name,v in functions.items():
    original=(OUT/(name+'-original.bin')).read_bytes();relocated=(OUT/(name+'-relocated.bin')).read_bytes();assert digest(original)==v['sha256'] and len(original)==len(relocated)==v['bytes']
    allowed={i for kind,offset,key in v['relocations'] for i in range(offset,offset+4)}
    assert all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
    for kind,offset,key in v['relocations']:
        value=struct.unpack_from('<I',relocated,offset)[0];target=addresses[(name,kind,offset)]
        assert value==target if kind in ['D','I'] else (addresses[(name,'B',0)]+offset+4+value)&0xffffffff==target
    relocation_counts[name]={kind:sum(k==kind for k,o,v in v['relocations']) for kind in ['D','I','F','E']}
assert sum(v['bytes'] for v in functions.values())==4246
def mix(a,b,kind):return (31-((31-a)*(31-b)+30)//31) if kind==0 else a*b//31
words=0
for mode in [0,1]:
    state=(OUT/f'initialized-mode{mode}.bin').read_bytes();assert len(state)==GE-GB
    for kind,bases in enumerate([[0x81089c,0x81109c,0x81189c],[0x80d89c,0x80e09c,0x80e89c],[0x82109c,0x82609c,0x82b09c],[0x81209c,0x81709c,0x81c09c]]):
        for address,shift in zip(bases,[0,6 if mode else 5,11 if mode else 10]):
            expected=[]
            for strength in range(10) if kind>=2 else [32]:
                for a in range(32):
                    for b in range(32):
                        v=mix(a,b,kind%2)
                        if kind>=2:v=v*(15+(strength*8)//5)//31
                        expected.append(v<<shift)
            observed=struct.unpack_from('<'+'H'*len(expected),state,address-GB)
            assert tuple(expected)==observed,('table',mode,hex(address));words+=len(expected)
assert words==135168
with (OUT/'cases.csv').open(encoding='ascii',newline='') as f:cases=[{k:int(v) for k,v in row.items()} for row in csv.DictReader(f)]
with (OUT/'results-native.csv').open(encoding='ascii',newline='') as f:results=[{k:int(v) for k,v in row.items()} for row in csv.DictReader(f)]
stream=(OUT/'pixels-native.bin').read_bytes();cursor=0;total=0
vectors=read(OUT/'full-frame-vectors.json')['vectors'];assert len(cases)==len(results)==len(vectors)==1584
for c,result,vector in zip(cases,results,vectors):
    number,count=struct.unpack_from('<II',stream,cursor);cursor+=8;assert number==c['case']
    actual=array.array('H',[0x1777])*(c['pitch']*552);seen=[]
    for i in range(count):
        index,value=struct.unpack_from('<IH',stream,cursor);cursor+=6
        assert index<len(actual) and value!=0x1777 and (not seen or index>seen[-1]);seen.append(index);actual[index]=value
    expected=array.array('H',[0x1777])*(c['pitch']*552)
    ymax=min(552,c['dest_y']+c['bottom']-c['top']);ox=32-(c['offset_x']&31);oy=32-((c['offset_y']+16*((c['offset_x']//32)&1))&31)
    for y in range(c['dest_y'],ymax):
        for x in range(c['dest_x'],c['dest_x']+c['right']-c['left']):
            if c['gate']==1 or (c['gate']==7 and (x+y)&1):continue
            sx=x-c['dest_x']+c['left'];sy=y-c['dest_y']+c['top'];u=sx+ox;v=sy+oy
            a=[sx%32,sy%32,(sx+3*sy)%32];b=[(5*u+3*v)%32,(7*u+v)%32,(u+11*v)%32]
            rgb=[mix(a,b,c['blend_mode']) for a,b in zip(a,b)]
            if c['strength']!=32:rgb=[v*(15+(c['strength']*8)//5)//31 for v in rgb]
            expected[y*c['pitch']+x]=(rgb[0]<<(11 if c['format'] else 10))|(rgb[1]<<(6 if c['format'] else 5))|rgb[2]
    assert actual==expected,('frame',number)
    bottom=c['top']+ymax-c['dest_y'];assert result=={'case':number,'bottom':bottom,'locks':1,'unlocks':2,'changes':count}
    assert vector=={'case':number,'frame_sha256':digest(actual.tobytes()),'changed_words':count,'clipped_source_bottom':bottom};total+=count
assert cursor==len(stream) and total==327376
native=read(OUT/'native-summary.json');assert native['host_ftol_calls']==2048 and native['cases']==1584 and native['guards_unchanged'] and native['changed_words']==total
invariance=read(OUT/'provenance-instrumentation-invariance.json');assert len(invariance['byte_identical_inputs_and_outputs'])==10 and not invariance['first_run_qualified']
prior=Path(invariance['first_failed_observer_run']).resolve(strict=True);assert prior.is_relative_to(ROOT/'驗證紀錄/current/native-background-blend-original')
assert sha(prior/'run-raw.zip')==invariance['first_failed_raw_archive_sha256']
for name,value in invariance['byte_identical_inputs_and_outputs'].items():assert sha(prior/name)==sha(OUT/name)==value,name
(OUT/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files={'independent-qualifier.py':OUT/'independent-qualifier.py','provenance-instrumentation-invariance.json':OUT/'provenance-instrumentation-invariance.json','normal-qualified.json':NORMAL/'qualified.json','source-inputs.json':NORMAL/'source-inputs.json'}
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(OUT/'qualification-metadata.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(OUT/'qualification-metadata.zip') as z:
    for n,h in hashes.items():assert digest(z.read(n))==h
q={'status':'independently_verified_original_background_initializer_tables_and_pixel_kernels_with_explicit_host_ftol',
 'observed_utc':datetime.now(timezone.utc).isoformat(),'path':str(OUT),'original_function_bytes':4246,'relocations':relocation_counts,
 'channel_words':135168,'pixel_cases':1584,'changed_pixel_words':total,'host_ftol_calls':2048,'lock_calls':1584,'unlock_calls':3168,
 'host_x86_CRT':str(host),'host_CRT_sha256':prep['host_CRT_sha256'],'loaded_ftol48_sha256':digest(loaded),'host_ftol_leaf_bytes':39,
 'loaded_host_conversion_matches_pinned_file':True,'historical_CRT_DLL_identity_verified':False,'private_source_and_gate_guards_unchanged':True,
 'source_files_unchanged':1005,'source_manifest_sha256':prep['source_manifest_sha256'],'normal_mains':prep['normal_mains'],
 'raw_archive_sha256':raw['raw_archive_sha256'],'qualification_metadata_files':hashes,'qualification_metadata_archive_sha256':sha(OUT/'qualification-metadata.zip'),'all_archives_readback_verified':True,
 'warm_private_surface_and_synthetic_IMG_inputs':True,'original_resource_loader_executed':False,'complete_background_geometry_and_pixels_composed':False,
 'modern_world_adapter_verified':False,'full_background_renderer_verified':False,'game_sessions':0,'whole_goal_complete':False}
dest=OUT/'qualified.json';assert not dest.exists();save(dest,q);assert read(dest)==q
save(APP/'background-blend-qualified-current.json',q)
print(json.dumps({k:q[k] for k in ['status','path','channel_words','pixel_cases','changed_pixel_words','all_archives_readback_verified']},ensure_ascii=False))
