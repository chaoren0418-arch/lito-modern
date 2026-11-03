"""Compose qualified original init/geometry/pixels with actual selected IMG bytes."""
from pathlib import Path
from datetime import datetime,timezone
import csv,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
BLEND=ROOT/'驗證紀錄/current/native-background-blend-original/20261016-114436-881465'
GEO=ROOT/'驗證紀錄/current/native-background-geometry-original/20261016-110731-202085'
CURRENT=APP/'background-geometry-tests-20261016-122035-301179'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pins=read(CURRENT/'compiled-inputs.json');assert len(pins)==1012
for n,h in pins.items():assert digest((ROOT/n).read_bytes())==h,n
bq=read(BLEND/'qualified.json');gq=read(GEO/'qualified.json')
assert bq['all_archives_readback_verified'] and gq['all_archives_readback_verified']
out=ROOT/'驗證紀錄/current/native-background-real-original'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
for name in ['init-original.bin','init-relocations.csv','rgb555-original.bin','rgb555-relocations.csv','data-initial.bin','screen-constants.bin']:
    (out/name).write_bytes((BLEND/name).read_bytes())
(out/'geometry-original.bin').write_bytes((GEO/'geometry-original.bin').read_bytes())
(out/'geometry-relocations.csv').write_bytes((GEO/'relocations.csv').read_bytes())
cpp=(BLEND/'oracle.cpp').read_text(encoding='utf-8')
helpers=cpp[:cpp.index('int wmain(')]
def replace(s,a,b):assert s.count(a)==1,a;return s.replace(a,b)
helpers=replace(helpers,'static unsigned char* global(U address){',
    'static constexpr std::array<U,6> external_keys{0x4f7430,0x62f324,0x62f4c0,0x62f4c4,0x834bd4,0x834ee4};\nstatic std::array<U,6> external_values{};\nstatic unsigned char* global(U address){\n    for(size_t i=0;i<external_keys.size();++i)if(external_keys[i]==address)return reinterpret_cast<unsigned char*>(&external_values[i]);')
helpers=replace(helpers,'else {need(kind=="E"&&key==0x5091863,"error call target");',
    'else {need((kind=="E"&&key==0x5091863)||(kind=="C"&&key==0x497c60),"error call target");')
helpers=helpers.replace('kind=="F"||kind=="E"','kind=="F"||kind=="E"||kind=="C"')
helpers=replace(helpers,':0x4bf500;',':std::string(name)=="geometry"?0x487e10:0x4bf500;')
helpers=replace(helpers,'kind=="E"?addr(reinterpret_cast<void*>(&forbidden))','(kind=="E"||kind=="C")?addr(reinterpret_cast<void*>(&forbidden))')
crt_setup=cpp[cpp.index('    HMODULE crt='):cpp.index('    std::ofstream addresses(')]
tail=(ROOT/'維護工具/oracle/NativeBackgroundRealTail.cpp').read_text(encoding='utf-8')
tail=replace(tail,'    /* CRT_SETUP */',crt_setup)
(out/'oracle.cpp').write_text(helpers+tail,encoding='utf-8')
inventory=read(APP/'map303-background-BASE-members-readonly.json')
client=Path(r'D:\lineage開發\0-客戶端')
assert digest((client/'Sprite.idx').read_bytes())==inventory['index_sha256']
pak=client/'Sprite.pak';before=pak.stat();assert before.st_size==inventory['pak_size']
with pak.open('rb') as f:
    for member in inventory['members']:
        f.seek(member['offset']);raw=f.read(member['bytes']);assert digest(raw)==member['sha256']
        (out/member['member']).write_bytes(raw)
after=pak.stat();assert (before.st_size,before.st_mtime_ns)==(after.st_size,after.st_mtime_ns)
fixture=read(ROOT/'LiTo381-Modern/tests/fixtures/native-background-geometry-original.json')
selected=[]
for mapid in [302,303]:
    for j,(level,klass) in enumerate([(0,0),(14,0),(15,0),(0,2)]):
        pose=[0,0,48,48] if j%2==0 else [-32725,32784,-25,99]
        offset=[65484,65534] if j%2==0 else [384,768]
        params=[mapid,level,klass,*pose,*offset]
        case=next(c for c in fixture['cases'] if c['input']==params)
        for gate in [0,1,7]:selected.append({'input':params,'gate':gate,'calls':case['calls']})
assert len(selected)==24
with (out/'cases.csv').open('w',encoding='ascii',newline='') as f:
    w=csv.writer(f);w.writerow(['case','map','level','class','cell_x','cell_y','camera_x','camera_y','offset_x','offset_y','gate'])
    for i,c in enumerate(selected):w.writerow([i,*c['input'],c['gate']])
save(out/'reference-calls.json',selected)
cmd=read(BLEND/'build-command.json');cmd['args']=[s.replace(str(BLEND),str(out)) for s in cmd['args']];cmd['cwd']=str(out);save(out/'build-command.json',cmd)
save(out/'compiled-inputs.json',pins);(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
proof={'status':'prepared_original_real_IMG_background_not_executed','path':str(out),'source_files':1012,
    'source_manifest_sha256':digest((CURRENT/'compiled-inputs.json').read_bytes()),'cases':24,'original_function_bytes':4439,
    'original_geometry_qualification_sha256':digest((GEO/'qualified.json').read_bytes()),
    'original_blend_qualification_sha256':digest((BLEND/'qualified.json').read_bytes()),
    'host_CRT_sha256':bq['host_CRT_sha256'],'host_x86_CRT':bq['host_x86_CRT'] if 'host_x86_CRT' in bq else r'C:\Windows\SysWOW64\msvcrt.dll',
    'members':inventory['members'],'resource_loader_executed':False,'historical_CRT_DLL_identity_verified':False,
    'game_sessions':0,'whole_goal_complete':False}
save(out/'preparation.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'preparation-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'preparation-raw.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h
save(out/'preparation-readback.json',{**proof,'files':files,'archive_sha256':digest((out/'preparation-raw.zip').read_bytes())})
save(APP/'background-real-preparation-current.json',{'path':str(out)})
print(json.dumps({'path':str(out),'prepared_cases':24,'original_bytes':4439},ensure_ascii=False))
