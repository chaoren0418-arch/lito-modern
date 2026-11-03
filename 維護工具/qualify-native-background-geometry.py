"""Read native geometry evidence and recompute clipped rectangle intersections."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,json,struct,sys,zipfile

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'floor-tests-20261016-102632-227125/normal-qualified'
OUT=Path(sys.argv[1]).resolve(strict=True)
assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-geometry-original')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def table(name):
    with (OUT/name).open(encoding='ascii',newline='') as f:return [{k:int(v) for k,v in row.items()} for row in csv.DictReader(f)]
prep=read(OUT/'preparation-readback.json');run=read(OUT/'run-receipt.json');raw=read(OUT/'run-readback.json')
assert run['status']==raw['status']=='original_background_geometry_matches_independent_scalar'
assert run['owned_process'] is None and run['build.log_exit_code']==run['native-summary.json_exit_code']==0
assert run['native_execution_started'] and run['source_inputs_unchanged'] and run['normal_mains_unchanged']
for archive,files,expected_sha in [('preparation-raw.zip',prep['files'],prep['raw_archive_sha256']),('run-raw.zip',raw['files'],raw['raw_archive_sha256'])]:
    assert sha(OUT/archive)==expected_sha
    with zipfile.ZipFile(OUT/archive) as z:
        assert set(z.namelist())==set(files)
        for name,value in files.items():assert digest(z.read(name))==sha(OUT/name)==value,name
pins=read(NORMAL/'source-inputs.json');normal=read(NORMAL/'qualified.json')
assert len(pins)==1005 and sha(NORMAL/'source-inputs.json')==prep['source_manifest_sha256']
for name,value in pins.items():assert sha(ROOT/name)==value,name
for p,v in normal['profiles'].items():assert sha(v['path'])==v['sha256']==prep['normal_mains'][p]
native=read(OUT/'native-summary.json')
assert native['cases']==2688 and native['recorded_calls']==17472 and native['guards_unchanged']
assert native['unexpected_loader_or_error_calls']==0
cases=table('cases.csv');actual=table('calls-native.csv');counts=table('counts-native.csv')
assert len(cases)==len(counts)==2688
expected=[];expected_counts=[]
for number,c in enumerate(cases):
    assert c['case']==number and c['map'] in [302,303]
    half=int(c['cell_x']/2)
    px=4*(c['cell_y']+half)+int((48-c['camera_x'])*5/6)
    py=2*(c['cell_y']-half)+int((48-c['camera_y'])*5/6)
    rx=px%384;ry=py%384;tile=int(px/384);before=len(expected)
    for column in range(3):
        parity=tile+column
        # Native signed remainder equals positive1 only for positive odd values.
        yoffset=(ry+(192 if parity>0 and parity&1 else 0))%384
        for row in range(3):
            x=column*384-rx;y=row*384-yoffset
            x0=max(0,x);y0=max(0,y);x1=min(744,x+384)
            # Original tests against552, but overflow arithmetic uses8Ah(138),
            # hence a522 bottom cap. Keep that conditional30px reduction.
            bottom_cap=522 if y+384>552 else 552
            y1=min(bottom_cap,y+384)
            if x0>=x1 or y0>=y1:continue
            strength=32 if c['level']>=15 or c['class'] in [1,2,3] else int(c['level']*9/16)
            expected.append(dict(case=number,ordinal=len(expected)-before,base_id=800 if c['map']==303 else 922,
                overlay_id=803 if c['map']==303 else 923,left=x0-x,top=y0-y,right=x1-x,bottom=y1-y,
                x=x0,y=y0,offset_x=c['offset_x'],offset_y=c['offset_y'],strength=strength,mode=0 if c['map']==303 else 1))
    expected_counts.append({'case':number,'calls':len(expected)-before})
assert actual==expected and counts==expected_counts
extraction=read(OUT/'extraction.json');original=(OUT/'geometry-original.bin').read_bytes();relocated=(OUT/'geometry-relocated.bin').read_bytes()
assembly=(OUT/'selected-assembly.txt').read_text(encoding='utf-8-sig')
assert 'cmp     ebx, 228h' in assembly and 'mov     ebx, 8Ah' in assembly
assert len(original)==len(relocated)==extraction['original_bytes']==1294 and digest(original)==extraction['original_code_sha256']
allowed={i for kind,o,key in extraction['relocations'] for i in range(o,o+4)}
assert all(a==b or i in allowed for i,(a,b) in enumerate(zip(original,relocated)))
with (OUT/'private-addresses.csv').open(encoding='ascii',newline='') as f:addresses={(r['kind'],int(r['original'])):int(r['address']) for r in csv.DictReader(f)}
for kind,offset,key in extraction['relocations']:
    value=struct.unpack_from('<I',relocated,offset)[0]
    assert value==addresses[(kind,key)] if kind=='D' else (addresses[('B',0x487e10)]+offset+4+value)&0xffffffff==addresses[(kind,key)]
assert sum(k=='D' for k,o,v in extraction['relocations'])==31 and sum(k=='C' for k,o,v in extraction['relocations'])==15
exe=(OUT/'oracle.exe').read_bytes();e=int.from_bytes(exe[60:64],'little')
assert exe[:2]==b'MZ' and exe[e:e+4]==b'PE\0\0' and exe[e+4:e+6]==b'\x4c\x01'
(OUT/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
metadata={'independent-qualifier.py':OUT/'independent-qualifier.py',
          'normal-qualified.json':NORMAL/'qualified.json','source-inputs.json':NORMAL/'source-inputs.json',
          'GUI-completion-observation.json':NORMAL/'GUI-completion-observation.json'}
metadata_hashes={n:sha(p) for n,p in metadata.items()}
with zipfile.ZipFile(OUT/'qualification-metadata.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in metadata.items():z.write(p,n)
with zipfile.ZipFile(OUT/'qualification-metadata.zip') as z:
    for n,h in metadata_hashes.items():assert digest(z.read(n))==h
q={'status':'independently_verified_original_background_geometry_and_rectangle_intersections',
   'observed_utc':datetime.now(timezone.utc).isoformat(),'path':str(OUT),'original_function_bytes':1294,
   'actual_cases':2688,'actual_recorded_pixel_calls':17472,'absolute_data_relocations':31,'forbidden_error_call_relocations':15,
   'original_instructions_unchanged_except_allowed_relocations':True,'all_private_guards_unchanged':True,
   'source_files_unchanged':1005,'source_manifest_sha256':prep['source_manifest_sha256'],'normal_mains':prep['normal_mains'],
   'helper_machine':'x86','helper_sha256':sha(OUT/'oracle.exe'),'raw_archive_sha256':raw['raw_archive_sha256'],
   'all_archives_readback_verified':True,'resource_loader_executed':False,'real_pixel_kernel_executed':False,
   'background_full_initializer_executed':False,'full_background_renderer_verified':False,
   'original_bottom_clip_policy':{'overflow_test_height':552,'overflow_cap_height':522,'purpose_verified':False},
   'source_cell_to_modern_viewport_adapter_verified':False,'game_sessions':0,'whole_goal_complete':False,
   'qualifier_sha256':sha(Path(__file__)),'qualification_metadata_files':metadata_hashes,
   'qualification_metadata_archive_sha256':sha(OUT/'qualification-metadata.zip')}
dest=OUT/'qualified.json';assert not dest.exists();dest.write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');assert read(dest)==q
(APP/'background-geometry-qualified-current.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:q[k] for k in ['status','path','actual_cases','actual_recorded_pixel_calls','all_archives_readback_verified']},ensure_ascii=False))
