from pathlib import Path
import json,hashlib,sys
from PIL import Image
p=Path(sys.argv[1]).resolve();read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'));sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_observer_bootstrap_preflight' and r['test_exit_code']==r['server_exit_code']==0
if r.get('detached_review_candidate'):
 assert r['jar_sha256']=='A1F119F5BCBF0159E8CFB3396B7B601BF0930983B375F374704B81874591E27C'
else:assert r['jar_sha256']=='9719F45DB324206617726CE06D1A9E2184E9A3B3AEFAB3D50F3EBBDFBF09CFAC'
assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
s=r['server'];assert s['actual_client_sessions']==6 and s['memberships_empty'] and not s['sql_access'] and not s['state_persistent'] and s['authority_mutations']==0
trace=[json.loads(x) for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
reports={role:read(p/f'{role}-observer-bootstrap.json') for role in ['a','b','o']}
for role,id in [('a',1),('b',3),('o',4)]:
 d=reports[role];state=d['snapshot'];assert state['phase']=='battle_pending' and state['world']['map_id']==564 and state['world']['status']['object_id']==id
 assert state['observer']==(role=='o') and d['own_actor_present']==(role!='o') and d['world_ready']==(role!='o')
 rows=[x for x in trace if x['object_id']==id and x['phase']=='BATTLE'];assert rows and all(set(x['combatant_ids'])=={1,3} for x in rows)
assert len(reports['o']['snapshot']['joined_room']['members'])==3
room=read(p/'observer-room-preflight.json');assert room['phase']=='room_ready' and room['observer'] and all(m['state']!=1 for m in room['joined_room']['members'] if m['name']==room['selected_character'])
world=reports['o']['snapshot']['world'];assert '4' not in world['actors'] and {'1','3'}<=set(world['actors'])
view=read(p/'observer-view.json');b=world['observer_bounds'];before=view['before'];after=view['after']
assert before['view_mode']==after['view_mode']=='observer'
assert before['view_position']==[(a+b)//2 for a,b in zip(b['min'],b['max_exclusive'])]
assert after['view_position']==[before['view_position'][0],before['view_position'][1]-1]
assert view['final_camera']==[before['view_position'][0],b['min'][1]] and view['pans']==before['view_position'][1]-b['min'][1]
assert all(view[x] for x in ['camera_sent_no_wire','actors_unchanged','player_actions_rejected','old_frame_rejected']) and not view['own_actor_created']
for label,frame in [('before',before),('after',after)]:
 assert frame['minimap']['visible'] and frame['minimap']['observer_camera'] and frame['minimap']['own_id'] is None
 assert frame['minimap']['center']==frame['view_position'] and not any(a['id']==4 for a in frame['actors'])
 for id in [1,3]:assert next(a for a in frame['actors'] if a['id']==id)['rendered']
 with Image.open(p/f'o-observer-{label}.png') as img:assert img.size==(800,600) and hashlib.sha256(img.convert('RGBA').tobytes()).hexdigest().upper()==frame['rgba_sha256'].upper()
for id in [1,3]:
 a=next(x for x in before['actors'] if x['id']==id);c=next(x for x in after['actors'] if x['id']==id)
 assert a['world']==c['world'] and c['screen']==[a['screen'][0]+24,a['screen'][1]+12]
assert all([x['x'],x['y']]==[100,120] for x in trace if x['object_id']==4 and x['phase']=='BATTLE')
live_verified=False
if r.get('live'):
 live={role:read(p/f'{role}-observer-live.json') for role in ['a','b','o']}
 assert live['a']['moves'] and live['a']['attacks']>0 and live['b']['attacks']==live['o']['attacks']==0
 for role,d in live.items():
  assert not d['authority_or_clock_injection'] and d['result']['phase']=='battle_result' and d['result']['world']['battle_result']==0
  assert d['lobby']['phase']=='lobby_ready' and d['lobby']['joined_room'] is None and d['lobby']['world']['actors']=={} and d['lobby']['world']['observer_camera'] is None
 observed=live['o']['observed_frames'];assert [x['label'] for x in observed]==['move','hurt','death']
 for row in observed:
  label=row['label'];frame=row['frame'];world=row['world'];assert frame['view_mode']=='observer' and '4' not in world['actors']
  if label=='move':
   a=world['actors']['1'];assert any(t['object_id']==1 and [t['x'],t['y']]==[a['x'],a['y']] for t in trace)
  else:
   target=world['actors']['3'];assert target['hp_ratio']<100
   if label=='death':assert target['dead'] and target['hp_ratio']==0 and next(a for a in frame['actors'] if a['id']==3)['phase']=='death'
  with Image.open(p/f'o-observer-{label}.png') as img:assert hashlib.sha256(img.convert('RGBA').tobytes()).hexdigest().upper()==frame['rgba_sha256'].upper()
 assert any(t['object_id']==3 and t['hp']==0 for t in trace)
 live_verified=True
result=dict(status='verified_three_role_observer_original_view_and_local_camera_subset',receipt_sha256=sha(p/'receipt.json'),test_binary_sha256=r['test_binary_sha256'],jar_sha256=r['jar_sha256'],actual_client_sessions=6,observer_id=4,combatant_ids=[1,3],no_synthetic_observer_actor=True,local_camera_bounded=True,camera_network_packets=0,actor_positions_unchanged=True,old_camera_frame_retired=True,original_pngs_match_render_receipts=True,gui_verified=False,live_combat_or_departure_verified=False,full_product_verified=False)
result['live_move_hurt_death_and_settlement_return_verified']=live_verified
result['detached_review_candidate']=bool(r.get('detached_review_candidate'))
result['formal_deployed']=False
if live_verified:result['status']='verified_three_role_observer_original_view_live_updates_and_result_return_subset'
(p/'observer-comparison.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');print(json.dumps({'status':result['status'],'observer_id':4,'camera_network_packets':0}))
