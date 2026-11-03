"""Independent wire, passive authority and original GUI evidence comparison."""
from pathlib import Path
import argparse,hashlib,json,re,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser();parser.add_argument('case');args=parser.parse_args()
p=Path(args.case).resolve(strict=True)
assert p.is_relative_to(ROOT/'驗證紀錄/current/native-light-gui')
read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
def sha(f):
    with f.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='completed_native_light_GUI_pending_independent_wire_qualification'
assert not (p/'qualified.json').exists()
assert r['private_desktop'] and not r['physical_input_verified']
assert r['inputs_unchanged'] and r['reference_is_current']
assert r['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
normal=read(p/'normal-qualification.json')
profile=next(k for k,v in normal['profiles'].items() if v['sha256']==r['exe_sha256'])
assert sha(Path(normal['profiles'][profile]['path']))==r['exe_sha256']
manifest=p/'source-inputs.json';pins=read(manifest)
assert len(pins)==r['source_files']==970
assert sha(manifest)==r['source_manifest_sha256']==normal['source_manifest_sha256']
for n,h in pins.items():assert sha(ROOT/n)==h,n
for n,h in r['source_pins'].items():
    assert sha(ROOT/n)==sha(p/'source-snapshot'/n)==h,n
command=r['server_launch']['command']
assert '-Dlito.resourceProfile='+r['resource_profile'] in command
assert '-Dlito.extendedItemDetails='+str(r['extended_item_details']).lower() in command
if r['resource_profile']=='tw13081901-v1':
    assert r['extended_item_details'] and '-Dlito.commonCatalog=' in command
    assert r['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
server=r['server'];assert server['status']=='sessions_closed'
assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty']
assert server['authority_mutations']==server['classic_world_members_after_disconnect']==server['active_hotel_leases_after_disconnect']==0
assert not server['sql_access'] and not server['state_persistent']
captures=list((p/'raw-capture').glob('*/frames.jsonl'));assert len(captures)==1
wire=[json.loads(l) for l in captures[0].read_text(encoding='utf-8-sig').splitlines()]
def payload(row):
    if row['plain']=='MASKED_C_LOGIN':
        assert row['direction']=='client_to_server' and row['opcode']=='0x01'
        return b''
    assert re.fullmatch('[0-9a-fA-F ]*',row['plain'])
    return bytes.fromhex(row['plain'])
assert sum(row['direction']=='server_to_client' and payload(row)[:1]==b'\x01' for row in wire)==4
requests=[(i,row) for i,row in enumerate(wire) if row['direction']=='client_to_server' and payload(row)[:1]==b'\x14']
rounds=2 if r['scenario']=='recast' else 1;refused=r['scenario']=='refused'
assert len(requests)==len(r['casts'])==rounds
assert all(payload(row)==b'\x14\x00\x01' for _,row in requests)
assert not any(row['direction']=='client_to_server' and payload(row)[:1]==b'\x0a' for row in wire)
trace=[json.loads(l)['state'] for l in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
log=(p/'server.log').read_text(encoding='utf-8-sig')
commits=re.findall(r'C_USE_SPELL light caster=TestKnight phase=BATTLE duration=3600 range=14 mp=(\d+)/(\d+)',log)
assert len(commits)==(0 if refused else rounds)
counts={'C14':rounds,'S37':0,'S1B':0,'S4A':0}
for n,((start,request),cast) in enumerate(zip(requests,r['casts'])):
    end=requests[n+1][0] if n+1<len(requests) else len(wire)
    replies=[row for row in wire[start+1:end] if row['direction']=='server_to_client']
    effects=[row for row in replies if payload(row)[:1]==b'\x37']
    lights=[row for row in replies if row['note'].endswith('reason=light-spell')]
    sounds=[row for row in replies if payload(row)==b'\x4a\x00\x91\x00']
    actions=[row for row in replies if 'S_ACTION light-cast' in row['note']]
    mana=[row for row in replies if row['note'].startswith('S_MANA_POINT light=')]
    assert all(payload(row)==b'\x37\x01\0\0\0\xce\x09' for row in effects)
    assert all(payload(row)==b'\x1b\x01\0\0\0\x0e' for row in lights)
    assert all(payload(row)==b'\x20\x01\0\0\0\x13' for row in actions)
    before,after,passive=cast['before'],cast['after_a'],cast['after_b']
    assert before['object_id']==after['object_id']==1 and passive['object_id']==3
    assert before['hp']==after['hp'] and before['native_inventory']==after['native_inventory']
    assert passive['mp']==r['before_b']['mp'] and passive['hp']==r['before_b']['hp'] and passive['light_expires_at_nanos']==0
    assert before['x']==after['x'] and before['y']==after['y'] and after['phase']=='BATTLE'
    if refused:
        assert not effects and not lights and not sounds and not actions and not mana
        assert before['mp']==after['mp']==2 and before['light_expires_at_nanos']==after['light_expires_at_nanos']==0
        assert '未收到施法回覆，可重新施放' in cast['GUI_status']
    else:
        assert len(effects)==len(lights)==len(actions)==2 and len(sounds)==len(mana)==1
        committed,maximum=map(int,commits[n]);assert before['mp']-committed==4 and after['mp']==committed
        assert payload(mana[0])==b'\x4d'+committed.to_bytes(2,'little')+maximum.to_bytes(2,'little')
        assert after['light_expires_at_nanos']>before['light_expires_at_nanos']
        assert any(v['object_id']==1 and v['mp']==committed and v['light_expires_at_nanos']==after['light_expires_at_nanos'] for v in trace)
        assert '收到施法動作' in cast['GUI_status'] and f'MP {committed}/{maximum}' in cast['GUI_status']
    for key,rows in [('S37',effects),('S1B',lights),('S4A',sounds)]:counts[key]+=len(rows)
world_pngs=0
for step in r['steps']:
    file=Path(step['png']).resolve(strict=True);assert file.parent==p and sha(file)==step['png_sha256']
    with Image.open(file) as im:assert im.width>=800 and im.height>=600
    if step['step']!='lobby':
        assert step['terrain_paint_probe'] and step['terrain_paint_probe'][-1]>=10
        assert '地圖 564' in step['status'] and '音效：' not in step['status']
        world_pngs+=1
assert world_pngs==2+rounds*2
files={str(f.relative_to(p)).replace('\\','/'):f for f in p.rglob('*') if f.is_file()}
assert not any(n in files for n in ['qualified.json','raw-evidence.zip'])
files['qualifier.py']=Path(__file__);fp={n:sha(f) for n,f in files.items()}
archive=p/'raw-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,f in files.items():z.write(f,n)
with zipfile.ZipFile(archive) as z:
    for n,h in fp.items():
        with z.open(n) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,n
q={'status':'qualified_actual_native_light_GUI_defined_scenario','profile':profile,'scenario':r['scenario'],
    'main_sha256':r['exe_sha256'],'source_files':970,'source_manifest_sha256':sha(manifest),
    'resource_profile':r['resource_profile'],'extended_item_details':r['extended_item_details'],
    'actual_sessions':4,'LOGIN_sessions':2,'GAME_sessions':2,'wire_counts':counts,'world_PNGs':world_pngs,
    'passive_light_activation_or_refusal_verified':True,'original_MP_cost_or_MP2_refusal_verified':True,
    'private_desktop':True,'physical_input_verified':False,'multi_DPI_verified':False,
    'night_illumination_verified':False,'natural3600s_expiry_verified':False,'physical_audio_verified':False,
    'GUI_device_playback_event_count_verified':False,'full_world_pixel_oracle':False,
    'raw_archive_sha256':sha(archive),'raw_archive_files':fp,'raw_archive_readback_verified':True,'whole_goal_complete':False}
(p/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:q[k] for k in ['status','profile','scenario','resource_profile','actual_sessions','wire_counts','world_PNGs','raw_archive_readback_verified']}))
