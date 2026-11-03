"""Read-only original call ordering and map303 source boundary observation."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
ASM=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
CLIENT=Path(r'D:\lineage開發\0-客戶端')
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
raw=ASM.read_bytes();assert digest(raw)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=raw.decode('utf-8',errors='replace').split('\n')
def function(name):
    a=next(i for i,l in enumerate(lines) if l.startswith(name+' proc near'))
    b=next(i for i in range(a+1,len(lines)) if lines[i].startswith(name+' endp'))
    return a,b
def fragment(a,b):return '\n'.join(f'{i+1}:{lines[i].rstrip(chr(13))}' for i in range(a,b+1))+'\n'
world_a,world_b=function('sub_488860')
find=lambda text:next(i for i in range(world_a,world_b+1) if lines[i].startswith(text))
floor,objects,end,background=[find(n) for n in ['loc_488C34:','loc_488D0B:','loc_488E90:','loc_488F49:']]
assert floor<objects<end<background
assert 'call    sub_487E10' in ''.join(lines[background:background+8])
assert 'call    sub_4BE000' in ''.join(lines[objects:end])
assert 'test    al, 2' in ''.join(lines[objects:end])
map_a,map_b=function('sub_483A80');map_text=''.join(lines[map_a:map_b+1])
assert 'LiTo\\\\LTMap\\\\%d\\\\%04x%04x.s32' in map_text
assert 'LiTo\\\\LTMap\\\\%d\\\\%04x%04x.seg' in map_text
assert 'jz      loc_4846D1' in map_text
root_a,root_b=function('sub_42E320')
assert 'offset byte_64787C' in ''.join(lines[root_a:root_b+1])
base=CLIENT/'map/303';native=CLIENT/'LiTo/LTmap/303'
members=sorted(p for p in base.iterdir() if p.is_file() and p.suffix.lower()=='.s32')
assert len(members)==30 and not native.exists()
out=ROOT/'驗證紀錄/current/native-background-scene-boundary'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out/'observer.py').write_bytes(Path(__file__).read_bytes())
(out/'first-pass-and-background.txt').write_text(fragment(floor,background+8),encoding='utf-8')
for name in ['sub_483A80','sub_42E320','sub_42E3B0','sub_43FEF0']:
    a,b=function(name);(out/(name+'.txt')).write_text(fragment(a,b),encoding='utf-8')
save(out/'source-inventory.json',{'native_directory':str(native),'native_exists':native.exists(),
    'BASE_directory':str(base),'BASE_s32':[{'name':p.name,'bytes':p.stat().st_size} for p in members],
    'BASE_map_used_by_product':False,'external_sources_written':False})
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(files)
    for n,h in files.items():assert digest(z.read(n))==h
proof={'status':'static_original_ordering_and_current_map303_source_boundary_observed',
    'observed_utc':datetime.now(timezone.utc).isoformat(),'path':str(out),
    'original_ASM_sha256':digest(raw),'line_domain':'raw UTF-8 LF split; CR retained in original input',
    'first_floor_LF_line':floor+1,'first_object_LF_line':objects+1,'background_branch_LF_line':background+1,
    'background_after_first_floor_and_object_pass':True,'background_before_all_bodies_assumption_valid':False,
    'plain_RLE_objects_are_possible_in_first_pass':True,'object_dispatch_executed_in_this_observation':False,
    'complete_overlay_gate_verified':False,'native_map303_directory_exists':False,'BASE_map303_s32_files':30,
    'original_map_loader_uses_LiTo_LTMap_s32_then_seg':True,
    'BASE_map_fallback_established':False,'product_map_fallback_added':False,
    'product_geometry_or_pixel_integration_changed':False,'game_sessions':0,
    'files':files,'raw_archive_sha256':digest((out/'raw.zip').read_bytes()),'archive_readback_verified':True,
    'whole_goal_complete':False}
save(out/'observation.json',proof);save(APP/'background-scene-boundary-current.json',proof)
assert digest(ASM.read_bytes())==digest(raw)
print(json.dumps({k:proof[k] for k in ['status','path','native_map303_directory_exists','BASE_map303_s32_files','archive_readback_verified']},ensure_ascii=False))
