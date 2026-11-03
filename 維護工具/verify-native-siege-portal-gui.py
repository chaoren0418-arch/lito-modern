"""Independent GUI P-key siege entrance, original wire, authority and pixel checks."""
from pathlib import Path
import collections,hashlib,json,re,sys
from PIL import Image
p=Path(sys.argv[1]).resolve();read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'));sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');root=Path(__file__).resolve().parents[1];runtime=Path(r['runtime']);mapid=r['map_id']
assert r['status']=='verified_native_siege_portal_GUI_pending_independent_comparison' and r['inputs_unchanged'] and r['reference_is_current']
assert r['private_desktop'] and not r['physical_input_verified']
assert r['server']['actual_client_sessions']==60 and r['server']['memberships_empty'] and r['server']['authority_mutations']==0
assert not r['server']['sql_access'] and not r['server']['state_persistent']
build=read(p/'build-inputs.json');assert build['status']=='completed_build' and build['profile']==r['profile'];assert sha(p/'build-inputs.json')==r['build_manifest_sha256']
assert build['main']['sha256']==r['exe_sha256'] and build['products']['native_arena_portal']['sha256']==r['support_binary_sha256']
assert sha(runtime/'immutable-siege-support.exe')==r['support_binary_sha256']
for n,h in build['files'].items():assert sha(root/n)==h,n
for n,h in r['source_pins'].items():assert sha(p/'source-snapshot'/n)==h,n
cat=root/'驗證紀錄/current/native-world-integration/C02-ArenaPortalCatalog.javap.txt';assert sha(cat)=='6AD4EC7B9A985052C5EB642DABE61E649E0E71E077B06AD5353661F21A8DE2F4'
portals={}
for match in re.finditer(r'// String ([0-9, -]+)',cat.read_text(encoding='utf-8-sig')):
 for word in match[1].split():
  if re.fullmatch(r'\d+,\d+,\d+,\d+,\d+,-?\d+',word):
   a=list(map(int,word.split(',')))
   if a[2]==mapid:portals[tuple(a[:2])]=a[2:]
assert portals
plan=read(p/'gui-routes.json');assert sha(p/'gui-routes.json')==r['route_plan_sha256'];assert plan['map_id']==mapid
sourcepins=read(root/'驗證紀錄/current/native-arena-portals/original-map-inputs.json');pins=[x for x in sourcepins if x['map']==mapid];assert len(pins)==(19 if mapid==563 else 4)
for x in pins:assert sha(Path(x['path']))==x['sha256']
assert len(plan['sectors'])==len(pins)
for row in plan['sectors']:
 pin=next(x for x in pins if Path(x['path']).name==row['file']);assert pin['sha256']==row['sha256'].upper()
cap=next((runtime/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==60
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN'];client=[b for x,b in packets if x['direction']=='client_to_server'];server=[b for x,b in packets if x['direction']=='server_to_client'];decode=lambda b:tuple(int.from_bytes(b[i:i+2],'little') for i in range(1,len(b),2))
assert len([b for b in client if b[0]==0x73])==4 and len([b for b in server if b[0]==0x5a])==2 and client.count(b'\x08')==2
assert not any(b[0]==0x14 for b in client),'readiness skill was cast'
assert [int.from_bytes(b[2:],'little') for b in client if b[:2]==b'\xd1\x03']==([563] if mapid==563 else [])
trace=[json.loads(l)['state'] for l in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
expected_accounts={'PeerFixture','PeerFixture2'}|{f'Siege{n:02}' for n in range(3,31)};identities={}
for a in expected_accounts:
 ids={t['object_id'] for t in trace if t['test_account']==a and t['phase']=='BATTLE'};assert len(ids)==1,(a,ids);identities[a]=ids.pop()
assert len(set(identities.values()))==30
extras=list(p.glob('e*-siege-member.json'));assert len(extras)==28
for f in extras:
 d=read(f);s=d['battle'];assert s['world']['map_id']==mapid and s['world']['hp'][0]>0 and s['world']['status']['object_id']==identities[d['account']]
 room=s['joined_room'];assert room['capacity']==30 and len(room['members'])==30
 assert collections.Counter(x['force'] for x in room['members'])=={0:15,1:15}
 for a,force in [('PeerFixture',0),('PeerFixture2',1)]:assert next(x for x in room['members'] if x['name']==a+'(測試乙)')['force']==force
expected_sources=collections.Counter();expected_destinations=collections.Counter();roles=[];walk_count=0
def world_shot(name):return name in ['battle','near-entrance','post-portal-walk'] or name.startswith('sector-') or 'entry-' in name or name=='status-message-bottom'
selection_before=next(x for x in r['steps'] if x['role']=='b' and x['step']=='lobby-selection-before-update')
selection_after=next(x for x in r['steps'] if x['role']=='b' and x['step']=='lobby-selection-updated')
room_id=lambda x:int(re.match(r'#(\d+) ',x['selected_room_row'])[1])
assert room_id(selection_before)==room_id(selection_after) and '28/30' in selection_before['selected_room_row'] and '29/30' in selection_after['selected_room_row']
for role,account,force in [('a','PeerFixture',0),('b','PeerFixture2',1)]:
 transitions=r[role+'-portals'];assert len(transitions)==2;walks=[s for s in r['steps'] if s['role']==role and s['step']=='GUI-key-walk'];assert len(walks)==len(plan[role]['steps'])+1;walk_count+=len(walks)
 for actual,planned in zip(walks[:-1],plan[role]['steps']):assert all(actual[k]==planned[k] for k in ['before','after','key'])
 all_states=[t for t in trace if t['test_account']==account and t['object_id']==identities[account] and t['phase']=='BATTLE'];assert all_states
 states=[t for t in all_states if len((t.get('room_state') or {}).get('members',[]))==30];assert states
 attacker=states[0]['siege_attacker_force'];assert attacker in (0,1) and all(t['siege_attacker_force']==attacker for t in states);admitted=force!=attacker
 for shot in [x for x in r['steps'] if x['role']==role and 'authority' in x and world_shot(x['step'])]:
  authority=shot['authority'];assert authority['phase']=='BATTLE' and len(authority['room_state']['members'])==30 and authority['siege_attacker_force']==attacker
 initial_items=transitions[0]['before']['native_inventory'];assert len(initial_items)==28
 for row in transitions:
  a=row['before'];z=row['after'];assert row['admitted']==admitted
  assert len(a['room_state']['members'])==len(z['room_state']['members'])==30 and a['siege_attacker_force']==z['siege_attacker_force']==attacker
  assert a['test_account']==z['test_account']==account and a['object_id']==z['object_id']==identities[account]
  assert a['character']==z['character'] and a['session_identity']==z['session_identity'] and a['source']==z['source']=='測試乙'
  assert a['native_inventory']==z['native_inventory']==initial_items
  src=min(portals,key=lambda s:max(abs(a['x']-s[0]),abs(a['y']-s[1])));assert max(abs(a['x']-src[0]),abs(a['y']-src[1]))<=3;expected_sources[src]+=1
  if admitted:
   dest=portals[src];expected_destinations[tuple(dest[:3])]+=1;assert [z['x'],z['y']]==dest[1:3]
  else:assert [a['x'],a['y']]==[z['x'],z['y']]
  assert any(t['session_identity']==a['session_identity'] and [t['x'],t['y']]==[z['x'],z['y']] for t in states)
 for w in walks:assert any([t['x'],t['y']]==w['after'] for t in states)
 post=next(s for s in r['steps'] if s['role']==role and s['step']=='post-portal-walk');assert post['authority']['native_inventory']==initial_items and walks[-1]['after']==[post['authority']['x'],post['authority']['y']]
 worldshots=[s for s in r['steps'] if s['role']==role and world_shot(s['step'])]
 centers=set();imgs=[]
 for s in worldshots:
  f=Path(s['png']);assert sha(f)==s['png_sha256'];assert s['terrain_paint_probe'][-1]>=10
  im=Image.open(f).convert('RGB');hist=im.getcolors(im.width*im.height);assert hist and len(hist)>1024
  colored=0
  for gy in range(8):
   for gx in range(16):
    pixel=im.getpixel((round(im.width*(.25+.5*gx/15)),round(im.height*(.25+.25*gy/7))))
    colored+=max(pixel)>20 and max(pixel)-min(pixel)>15
  assert colored>=10
  if s['step'].startswith('sector-'):centers.add(((s['authority']['x']-32704)//64,(s['authority']['y']-32704)//64))
  imgs.append({'file':f.name,'sha256':sha(f),'colored_probe':colored})
 if mapid==563:assert len(centers)>=2,'both-role real sector crossing not observed'
 roles.append({'role':role,'force':force,'attacker_force':attacker,'admitted':admitted,'P_key_entries':2,'source_path_steps':len(walks)-1,'post_portal_walk':walks[-1]['after'],'original_inventory_items':len(initial_items),'walk_sector_centers':sorted(centers),'images':imgs,'cleanup_trace_samples_outside_full_roster':len(all_states)-len(states)})
status_pixel_proofs=[]
def status_scroll_pixels(shot,proof):
 im=Image.open(shot['png']).convert('RGB');width,height=im.size
 left,right=round(width*.05),round(width*.925)
 sample=im.crop((left,round(height*.75),right,round(height*.88)))
 bg=collections.Counter(sample.get_flattened_data()).most_common(1)[0][0]
 assert min(bg)>=200 and max(bg)-min(bg)<=2,'status background not identified'
 runs=[]
 for y in range(round(height*.6),round(height*.92)):
  fraction=sum(im.getpixel((x,y))==bg for x in range(left,right))/(right-left)
  if fraction>=.55:
   if not runs or y!=runs[-1][-1]+1:runs.append([])
   runs[-1].append(y)
 assert runs,'status text region absent'
 rows=max(runs,key=len);top,bottom=rows[0],rows[-1]+1
 assert bottom-top==proof['client_height'],'status pixel region does not match control height'
 groups=[]
 for y in range(top,bottom):
  if sum(max(im.getpixel((x,y)))<100 for x in range(left,right))>=3:
   if not groups or y-groups[-1][-1]>4:groups.append([])
   groups[-1].append(y)
 assert len(groups)>=3,'last status lines not visible'
 pitches=[groups[i+1][0]-groups[i][0] for i in range(len(groups)-1)]
 assert max(pitches)-min(pitches)<=2 and min(pitches)>10,'irregular or clipped status rows'
 assert len(groups[-1])>=max(len(x) for x in groups)-3,'last status row ink is incomplete'
 assert groups[-1][-1]+1<=bottom-3,'last status row touches clipped edge'
 if proof.get('glyph_height_basis')=='native_edit_line_pitch':assert all(abs(x-proof['line_pitch'])<=2 for x in pitches)
 return {'png':Path(shot['png']).name,'sha256':sha(Path(shot['png'])),'status_top':top,'status_bottom':bottom,'background_RGB':bg,'text_row_bounds':[[x[0],x[-1]+1] for x in groups],'physical_line_pitches':pitches,'bottom_ink_margin':bottom-groups[-1][-1]-1,'remote_metric_basis':proof.get('glyph_height_basis','observer_GDI_diagnostic_not_acceptance_basis')}
if r.get('verify_status_scroll'):
 proofs=[x for x in r['steps'] if x['step']=='status-scroll-proof'];assert len(proofs)==1
 proof=proofs[0];assert not next(x for x in roles if x['role']==proof['role'])['admitted']
 assert proof['class_name'].lower()=='edit' and all(proof[k] for k in ['readonly','vertical_scroll','character_input_rejected'])
 assert '入口：沒有回覆' in proof['status_text'] and '方向鍵移動' in proof['status_text']
 assert proof['first_visible_top']==0 and proof['first_visible_bottom']>0 and proof['wrapped_lines']>=5
 assert 0<=proof['last_character_y'] and proof['last_character_y']+proof['glyph_height']<=proof['client_height']
 a,z=proof['authority_before'],proof['authority_after']
 for k in ['x','y','object_id','character','session_identity','native_inventory','source']:assert a[k]==z[k],k
 assert len(a['room_state']['members'])==len(z['room_state']['members'])==30
 shot=next(x for x in r['steps'] if x['role']==proof['role'] and x['step']=='status-message-bottom')
 status_pixel_proofs.append(status_scroll_pixels(shot,proof))
assert sum(x['admitted'] for x in roles)==1
assert collections.Counter(decode(b) for b in client if b[0]==0x73)==expected_sources
assert collections.Counter(decode(b) for b in server if b[0]==0x5a)==expected_destinations
assert sum(b[0]==0x0a for b in client)==walk_count
out={'status':'verified_native_siege_P_key_GUI_source_portal_and_current_scene','map':mapid,'checks':roles,'actual_sessions':60,'players':30,'players_per_team':15,'C_MOVE_requests':walk_count,'C73_requests':4,'S5A_replies':2,'C08_acks':2,'readiness_skill_casts':0,'lobby_room_selection_survived_real_peer_update':True,'exe_sha256':r['exe_sha256'],'support_binary_sha256':r['support_binary_sha256'],'jar_sha256':r['jar_sha256'],'receipt_sha256':sha(p/'receipt.json'),'capture_sha256':sha(cap),'trace_sha256':sha(p/'server-trace.jsonl'),'validator_sha256':sha(Path(__file__)),'full_build_pins_verified':True,'private_desktop':True,'physical_input_verified':False,'full_game_complete':False,'status_scroll_verified':bool(r.get('verify_status_scroll')),'status_scroll_pixel_proofs':status_pixel_proofs}
verifier=p/'validation-sources'/(sha(Path(__file__))[:16]+'-verify-native-siege-portal-gui.py');verifier.parent.mkdir(exist_ok=True)
if verifier.exists():assert sha(verifier)==sha(Path(__file__))
else:verifier.write_bytes(Path(__file__).read_bytes())
out['validator_source']=str(verifier.relative_to(p)).replace('\\','/')
(p/'siege-portal-gui-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:out[k] for k in ['status','map','actual_sessions','C_MOVE_requests','C73_requests']},ensure_ascii=False))
