"""Qualify explicit TW cross-endpoint observations; never substitute default90/current14 gates."""
from pathlib import Path
import argparse, hashlib, json, zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
a=argparse.ArgumentParser();a.add_argument('case',type=Path);a.add_argument('--normal-qualification',type=Path,required=True);args=a.parse_args()
case=args.case.resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-gui')
assert not (case/'shared-G06-diagnostic-qualified.json').exists()
r=read(case/'receipt.json');npath=args.normal_qualification.resolve(strict=True);assert npath.is_relative_to(ROOT/'驗證紀錄/current')
n=read(npath);pins=read(npath.parent/'source-inputs.json');assert len(pins)==968 and n['source_manifest_sha256']==sha(npath.parent/'source-inputs.json')
for name,digest in pins.items():assert sha(ROOT/name)==digest,name
for profile,main in n['profiles'].items():
 assert main['features']==[] and main['unique_entry'] and main['machine']=='AMD64' and main['optional_header']=='PE32+'
 assert sha(main['path'])==main['sha256']
assert r['scenario']=='cross-endpoint' and r['status']=='verified_native_teleport_GUI_cross_endpoint_shared_battle'
assert r['inputs_unchanged'] and r['reference_is_current'] and r['private_desktop'] and not r['physical_input_verified']
comp=read(case/'compilation-inputs.json');profile=comp['profile'];main=n['profiles'][profile]
assert comp['files']==pins and sha(case/'compilation-inputs.json')==r['compilation_inputs_sha256']
assert comp['main_sha256']==r['exe_sha256']==main['sha256']
l=read(case/'server-launch-identity.json')
assert r['resource_profile']==l['resource_profile']=='tw13081901-v1' and r['extended_item_details'] is l['extended_item_details'] is True
assert r['common_catalog_sha256']==l['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert l['exe_sha256']==r['exe_sha256'] and l['jar_sha256']==r['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert '-Dlito.resourceProfile=tw13081901-v1' in l['command_line'] and '-Dlito.extendedItemDetails=true' in l['command_line'] and '-Dlito.commonCatalog=' in l['command_line']
s=r['server'];assert s['actual_client_sessions']==6 and s['memberships_empty'] and s['staged_loadouts_empty'] and s['authenticated_accounts_empty']
assert s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
comparisons={}
for name in ['teleport-gui-authority-comparison.json','teleport-glyph-comparison.json','world-scene-pixel-comparison.json']:
 q=read(case/name);assert q['receipt_sha256']==sha(case/'receipt.json');comparisons[name]=sha(case/name)
world=read(case/'world-scene-pixel-comparison.json');assert world['images']==4
runtime=Path(r['runtime']).resolve(strict=True);assert runtime.is_relative_to(ROOT/'LiTo381-Modern/target/current')
cap=next((runtime/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x.get('note')=='S_KEY' and x['direction']=='server_to_client' for x in frames)==6
steps=[x for x in r['steps'] if x['step']=='GUI-key-walk'];assert len(steps)==14
heading={0x68:0,0x26:1,0x66:2,0x27:3,0x62:4,0x28:5,0x64:6,0x25:7}
expected=[bytes([10])+int(x['before'][0]).to_bytes(2,'little')+int(x['before'][1]).to_bytes(2,'little')+bytes([heading[x['key']]]) for x in steps]
actual=[bytes.fromhex(x['plain']) for x in frames if x['direction']=='client_to_server' and x['plain'].startswith('0A ')]
assert actual==expected
files={'qualifier.py':Path(__file__),'normal-qualification.json':npath,'normal-source-inputs.json':npath.parent/'source-inputs.json','normal-main.exe':Path(main['path'])}
for prefix,folder in [('case',case),('runtime',runtime)]:
 for p in folder.rglob('*'):
  if p.is_file():
   assert p.resolve().is_relative_to(folder) and not p.is_symlink()
   files[prefix+'/'+p.relative_to(folder).as_posix()]=p
for name in ['verify-native-teleport-gui-authority.py','verify-teleport-glyph-pixels.py','verify-current-hud-teleport-scenes.py']:
 files['validators/'+name]=ROOT/'維護工具'/name
file_pins={name:sha(path) for name,path in files.items()};archive=case/'shared-G06-diagnostic-raw.zip';assert not archive.exists()
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for name,p in files.items():z.write(p,name)
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(file_pins)
 for name,digest in file_pins.items():
  with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
proof=dict(status='qualified_explicit_TW_G06_cross_endpoint_GUI_diagnostic_scope',profile=profile,main_sha256=main['sha256'],source_manifest_sha256=n['source_manifest_sha256'],normal_qualification_sha256=sha(npath),resource_profile='tw13081901-v1',extended_item_details=True,actual_ClientSessions=6,actual_C_MOVE=14,extra_C_MOVE=0,original_7311_7312_glyph_comparison=True,world_scene_PNGs=4,world_framebuffer_independent_pixel_oracle=False,private_desktop=True,physical_input_verified=False,current14_default90_gates_satisfied=False,full_current14_qualified=False,all_G06_recovery_scenarios_qualified=False,full_shared_profile_compatibility_verified=False,whole_goal_complete=False,comparison_pins=comparisons,raw_file_pins=file_pins,archive_sha256=sha(archive),archive_readback_verified=True)
(case/'shared-G06-diagnostic-qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','actual_ClientSessions','actual_C_MOVE','extra_C_MOVE','full_current14_qualified']}))
