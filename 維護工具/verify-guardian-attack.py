from pathlib import Path
import json,hashlib,sys
from PIL import Image
p=Path(sys.argv[1]).resolve();read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'));sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_guardian_destruction_and_input_retirement' and r['test_exit_code']==r['server_exit_code']==0
s=r['server'];assert s['actual_client_sessions']==6 and s['memberships_empty'] and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
raw=[json.loads(x) for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
trace=[row.get('state',row) for row in raw];checks=[]
for role,own in [('a',1),('b',3)]:
 data=read(p/f'{role}-guardian-result.json');target=data['target'];assert not data['authority_or_clock_injection']
 assert data['initial']['joined_room']['mode']==5 and data['initial']['world']['map_id']==510
 assert data['rental']['selected_character']==data['transferred']['selected_character']==data['initial']['selected_character']
 assert any(i['use_type']==1 and i['name'].endswith('($9)') for i in data['transferred']['inventory']['items'].values())
 poses=data['poses'];assert [x['action'] for x in poses]==[0,32,33,34,35]
 for row in poses:
  action=row['action'];actor=row['world']['actors'][str(target)];assert actor['walk_action']==action
  draw=next(x for x in row['frame']['actors'] if x['id']==target)
  assert draw['action']==action and draw['logical_graphics']==2669 and draw['phase']=='object_action' and draw['rendered']
  assert any(tower['id']==target and tower['action']==action and tower['maximum_hp']==3000 for state in trace for tower in state['guardian_towers'])
  assert (p/f'{role}-guardian-{action}.png').is_file()
  with Image.open(p/f'{role}-guardian-{action}.png') as png:
   assert png.size==(800,600) and hashlib.sha256(png.convert('RGBA').tobytes()).hexdigest().upper()==row['frame']['rgba_sha256'].upper()
 dead=data['destroyed']['world']['actors'][str(target)];assert dead['walk_action']==35 and dead['hp_ratio']==0
 held=next(x for x in data['held_destroyed_frame']['actors'] if x['id']==target)
 assert held['rendered'] and held['phase']=='object_action' and held['action']==35 and held['action_complete'] and held['sprite_subframe']==19
 with Image.open(p/f'{role}-guardian-held-destroyed.png') as png:
  assert png.size==(800,600) and hashlib.sha256(png.convert('RGBA').tobytes()).hexdigest().upper()==data['held_destroyed_frame']['rgba_sha256'].upper()
 sounds=data['guardian_death_sound'];assert len(sounds)==1 and sounds[0]['prepared'] and sounds[0]['event']['sound']==360
 assert data['lobby']['phase']=='lobby_ready' and data['lobby']['joined_room'] is None
 assert data['result']['world']['battle_result'] is not None
 checks.append(dict(role=role,attacks_sent=data['attacks'],states=[x['action'] for x in poses],target=target,report_sha256=sha(p/f'{role}-guardian-result.json')))
a=read(p/'a-guardian-result.json');target=a['target']
for role in ['a','b']:
 data=read(p/f'{role}-guardian-result.json');assert data['automatic_tower_settlement'] and data['post_result_move_rejected'] and not data['post_destruction_cell_move_verified']
# S_D3 carries the common winning force index, not a per-client win boolean.
assert a['result']['world']['battle_result']==0 and read(p/'b-guardian-result.json')['result']['world']['battle_result']==0
log=(p/'server.log').read_text(encoding='utf-8-sig')
assert 'battle result player=TestKnight won=true' in log and 'battle result player=TestMage won=false' in log
hp=[]
for state in trace:
 for tower in state['guardian_towers']:
  if tower['id']==target and (not hp or hp[-1]!=tower['hp']):hp.append(tower['hp'])
assert hp[0]==3000 and hp[-1]==0 and all(a>b for a,b in zip(hp,hp[1:]))
out=dict(status='verified_actual3000hp_guardian_states_destruction_and_normal_return',receipt_sha256=sha(p/'receipt.json'),test_binary_sha256=r['test_binary_sha256'],jar_sha256=r['jar_sha256'],checks=checks,hp_samples=len(hp),actual_target_hp=[hp[0],hp[-1]],automatic_tower_settlement=True,post_result_move_rejected=True,post_destruction_cell_move_verified=False,old_target_and_held_attack_retired=True,authority_or_clock_injection=False,gui_verified=False,full_product_verified=False)
(p/'guardian-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'status':out['status'],'hp_samples':len(hp),'attacks':checks[0]['attacks_sent']}))
