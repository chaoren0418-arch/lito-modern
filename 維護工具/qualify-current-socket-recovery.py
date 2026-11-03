"""Independently qualify both current mains' room/world socket recovery."""
from pathlib import Path
import hashlib,json,zipfile
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
stage=root/'驗證紀錄/current/native-protocol-current-bootstrap'
normal=root/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes'
q=read(normal/'qualified.json');pins=read(normal/'source-inputs.json')
assert len(pins)==968
for name,expected in pins.items():assert sha(root/name)==expected,name
reference=read(root/'驗證紀錄/current/reference-snapshot.json')
assert reference['sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(reference['source'])==sha(reference['snapshot'])==reference['sha256']
names=['20261015-113201-392-Debug-room','20261015-113226-177-Debug-world','20261015-113316-864-Release-room','20261015-113332-319-Release-world']
results=[];seen=set()
for name in names:
 out=root/'驗證紀錄/current/native-recovery'/name;p=out/'receipt.json';d=read(p)
 assert d['schema']==2 and d['status']=='verified_controller_recovery'
 assert d['compilation_inputs_checked']==968 and not d['gui_verified']
 profile=d['profile'];mode=d['mode'];assert (profile,mode) not in seen;seen.add((profile,mode))
 assert d['exe_sha256']==q['profiles'][profile]['sha256']==sha(q['profiles'][profile]['path'])
 assert d['jar_sha256']==reference['sha256']
 for source,expected in d['source_pins'].items():
  assert sha(root/source)==expected and sha(out/'source'/source)==expected,source
 runtime=Path(d['runtime']);c=read(out/'client.json');s=read(out/'result.json')
 assert c==d['client'] and s==d['server']==read(runtime/'result.json')
 assert c['status']=='recovery_completed' and c['worker_quiescent']
 assert c['first_epoch']==2 and c['last_epoch']==4 and c['failure']['phase']=='failed'
 assert s['status']=='reconnected' and s['mode']==mode and s['actual_client_sessions']==4
 assert s['socket_faults']==1 and s['authority_mutations']==0 and not s['sql_access'] and s['memberships_empty']
 assert s['jar_sha256'].upper()==reference['sha256']
 for i,state in enumerate([c['before'],c['after']]):
  obs=s['observations'][i];assert obs['cycle']==i and state['selected_source']=='測試乙'
  assert obs['phase']==('BATTLE' if mode=='world' else 'ROOM')
  assert state['joined_room']['id']==obs['room']
  if mode=='world':
   w=state['world'];own=w['actors'][str(obs['id'])]
   assert w['status']['object_id']==obs['id'] and w['hp'][0]==obs['hp'] and w['mp'][0]==obs['mp']
   assert (own['x'],own['y'])==(obs['x'],obs['y'])
 if mode=='world':assert c['stale_frame_rejected'] and c['after']['world']['generation']>c['before']['world']['generation']
 harness=(out/'source/LiTo381-Modern/tests/oracle/NativeRecoveryHarness.java').read_text(encoding='utf-8-sig')
 assert 'cycle<2' in harness and harness.count('new ClientSession(')==2
 assert 'Endpoint.LOGIN' in harness and 'Endpoint.GAME' in harness and 'gameSocket.close();gt.join(5000)' in harness
 captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1
 rows=[json.loads(line) for line in captures[0].read_text(encoding='utf-8-sig').splitlines()]
 assert sum(x['direction']=='server_to_client' and x.get('note')=='S_KEY' for x in rows)==4
 assert not (runtime/'state.properties').exists()
 files=[f for f in runtime.rglob('*') if f.is_file() and ('captures' in f.parts or f.name in ['server.properties','ready.json','result.json','drop-ready','drop-closed','recovered','recovery-observed'])]
 archive=out/'runtime-evidence.zip';expected={str(f.relative_to(runtime)).replace('\\','/'):sha(f) for f in files}
 if not archive.exists():
  with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED) as z:
   for f in files:z.write(f,str(f.relative_to(runtime)))
 with zipfile.ZipFile(archive) as z:
  assert set(z.namelist())==set(expected)
  for entry,h in expected.items():assert hashlib.sha256(z.read(entry)).hexdigest().upper()==h
 results.append({'profile':profile,'mode':mode,'receipt':str(p),'receipt_sha256':sha(p),'actual_ClientSessions':4,'LOGIN_sessions':2,'GAME_sessions':2,'socket_faults':1,'epoch_before':2,'epoch_after':4,'stale_frame_rejected':c['stale_frame_rejected'] if mode=='world' else None,'runtime_archive_sha256':sha(archive),'runtime_files':expected,'capture_sha256':sha(captures[0])})
assert seen=={(p,m) for p in ['Debug','Release'] for m in ['room','world']}
result={'status':'qualified_current_C02_both_mains_room_world_socket_disconnect_relogin','cases':results,'scenarios':4,'actual_ClientSessions':16,'LOGIN_sessions':8,'GAME_sessions':8,'forced_owned_socket_disconnects':4,'SQL_access':False,'authority_mutations':0,'GUI_verified':False,'teleport_recovery_verified':False,'full_current_TCP_regression_verified':False,'whole_goal_complete':False,'source_manifest_sha256':sha(normal/'source-inputs.json'),'qualifier_sha256':sha(__file__)}
(stage/'recovery-qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Qualified room/world recovery: 4 scenarios,16 actual ClientSessions,2 profiles; no GUI/teleport/full TCP claim.')
