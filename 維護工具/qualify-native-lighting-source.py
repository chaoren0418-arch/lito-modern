"""Seal extracted x86 lighting primitives; this is not a renderer qualification."""
from pathlib import Path
import hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1];S=ROOT/'驗證紀錄/current/native-lighting-source'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
assert not (S/'qualified.json').exists()
meta=read(S/'extracted-code.json');pe=Path(meta['original_PE_path'])
assert sha(pe)==meta['original_PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
for name,v in meta['functions'].items():
    original=(S/(name+'-original.bin')).read_bytes();relocated=(S/(name+'-dependency-relocated.bin')).read_bytes()
    assert hashlib.sha256(original).hexdigest().upper()==v['original_sha256']
    assert hashlib.sha256(relocated).hexdigest().upper()==v['relocated_sha256']
    at=int(v['only_relocation']['call_address'],16)-int(v['start'],16)
    assert len(original)==len(relocated)==v['bytes'] and original[at]==relocated[at]==0xe8
    assert original[:at+1]==relocated[:at+1] and original[at+5:]==relocated[at+5:]
    addresses=[0x75a7f0,0x75a7ec,0x75ab18] if name=='daylight' else [0x834ee4]
    for address in addresses:assert original.count(address.to_bytes(4,'little'))==1
summary=read(S/'relocatable-oracle.stdout')
assert summary['status']=='extracted_x86_primitive_matches_scalar'
assert (summary['curve_cases'],summary['boundary_vectors'],summary['signed_actor_light_cases'])==(4216320,43920,1024)
assert read(S/'build-command.json')['oracle_exit_code']==0
seen=set();levels=set()
for line in (S/'boundary-vectors.jsonl').read_text(encoding='utf-8').splitlines():
    v=json.loads(line);d,m,f=v['day_of_year_zero_based'],v['minute'],v['flags']
    assert 0<=d<=365 and 0<=m<1440 and 0<=f<8 and (d,m,f) not in seen;seen.add((d,m,f))
    trunc=lambda a,b:(-1 if a<0 else 1)*(abs(a)//b)
    rise=452-d*192//172 if d<=172 else 260+d-173
    sunset=986+d*197//172 if d<=172 else 1183+trunc((173-d)*197,192)
    k=1+(m-rise+40)*15//40 if rise-40<m<rise else 15 if rise<=m<=sunset else 1+(sunset-m+40)*15//40 if sunset<m<sunset+40 else 2
    level=min(15,5+2*k//3);level=level-1 if level>=10 and f else level
    assert (v['sunrise'],v['sunset'],v['level'])==(rise,sunset,max(5,level))
    levels.add(level)
assert len(seen)==43920 and levels=={5,6,14,15}
helper=ROOT/'LiTo381-Modern/target/current/native-lighting-source-oracle/oracle.exe'
raw=helper.read_bytes();at=struct.unpack_from('<I',raw,60)[0]
assert struct.unpack_from('<H',raw,at+4)[0]==0x14c and struct.unpack_from('<H',raw,at+24)[0]==0x10b
normal_path=ROOT/'驗證紀錄/current/native-light-spell/normal/qualified.json';normal=read(normal_path)
pins=read(normal_path.parent/'source-inputs.json');assert len(pins)==970
for n,h in pins.items():assert sha(ROOT/n)==h,n
for v in normal['profiles'].values():assert sha(Path(v['path']))==v['sha256']
audit={'status':'native_x86_annual_daylight_and_signed_light_receiver_primitives_verified_only',
 'original_PE_sha256':sha(pe),'native_function_cases':summary,'boundary_vectors_Python_rechecked':43920,
 'retained_boundary_levels_observed':sorted(levels),'helper_machine':'I386 PE32 isolated test helper; client remains AMD64 PE32+',
 'curve_contract':{'first_season':'day0..172: sunrise=452-trunc(day*192/172); sunset=986+trunc(day*197/172)',
 'second_season':'day173..365: sunrise=260+day-173; sunset=1183+trunc((173-day)*197/192)',
 'twilight':'strictly inside40 minutes before sunrise or after sunset v=1+trunc(remaining*15/40); sunrise..sunset inclusive v15; otherwise v2',
 'level':'min15,5+trunc(2*v/3); subtract1 if level>=10 and any of three source environment flags nonzero; lower bound5',
 'actor_light':'original S1B uses signed range; own actor clamps below5; absent actor lookup does not create an actor; raw wire authority must stay separate from presentation'},
 'dependency_relocations':{'code':'two external CALL dependencies stubbed; three weather flag addresses and local actor address moved to helper fixtures; original arithmetic/constants/relative branches unchanged',
 'calendar':'tm-shaped calendar input tuple; gmtime is a stub label, not proof of original imported API identity or UTC/local semantics'},
 'current_normal_source_files_unchanged':970,'current_normal_source_manifest_sha256':normal['source_manifest_sha256'],
 'current_normal_mains':{k:v['sha256'] for k,v in normal['profiles'].items()},'actual_new_game_sessions':0,
 'product_source_changed':False,'original_client_executed':False,'calendar_dependency_UTC_or_local_resolved':False,
 'clock_advance_scale_verified':False,'spatial_light_falloff_verified':False,'RGB_pixel_shading_verified':False,
 'night_illumination_implemented':False,'whole_goal_complete':False}
(S/'audit.json').write_text(json.dumps(audit,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
base=pe.parents[1]
files={p.name:p for p in S.iterdir() if p.is_file() and p.suffix!='.zip'}
files.update({'qualifier.py':Path(__file__),'NativeLightingPrimitiveOracle.cpp':ROOT/'維護工具/oracle/NativeLightingPrimitiveOracle.cpp',
 'native-x86-oracle.exe':helper,'current-normal-qualification.json':normal_path,
 'readonly-light-status-clue.py':base/'tools/native_light_status_audit.py',
 'readonly-light-spell-clue.py':base/'tools/native_light_spell_audit.py'})
fp={n:sha(p) for n,p in files.items()};archive=S/'source-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(fp)
    for n,h in fp.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
q={k:v for k,v in audit.items() if k not in ['curve_contract','dependency_relocations']}
q.update({'audit_sha256':sha(S/'audit.json'),'archive_sha256':sha(archive),'archive_file_pins':fp,'archive_readback_verified':True})
(S/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:q[k] for k in ['status','native_function_cases','boundary_vectors_Python_rechecked','actual_new_game_sessions','product_source_changed','archive_readback_verified']}))
