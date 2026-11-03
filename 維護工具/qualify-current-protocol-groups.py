"""Qualify the exact 8 current protocol groups for each default-feature main."""
from pathlib import Path
import argparse,hashlib,json,re,zipfile
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('--since',default='20261015-113300');ap.add_argument('--groups',nargs='+',choices=['room','motion','emblems','label-updates','actor-states','navigation','lobby','inventory']);ap.add_argument('--output',type=Path);a=ap.parse_args()
normal=root/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes'
q=read(normal/'qualified.json');pins=read(normal/'source-inputs.json');assert len(pins)==968
for name,expected in pins.items():assert sha(root/name)==expected,name
reference=read(root/'驗證紀錄/current/reference-snapshot.json')
assert reference['sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(reference['snapshot'])==sha(reference['source'])==reference['sha256']
expected_modes={'room':['room-join','room-ready','room-leave','room-rejoin','room-reject','room-retry'],'motion':['room-world-move','room-world-correct'],'emblems':['room-world-emblems'],'label-updates':['room-world-labels'],'actor-states':['room-world-states'],'navigation':['room-world-route'],'lobby':['enter','switch','return','reselect','restricted'],'inventory':['inventory','inventory-reselect']}
selected=a.groups or list(expected_modes);assert len(set(selected))==len(selected)
all_required_groups=selected==list(expected_modes)
expected_modes={k:expected_modes[k] for k in selected}
results=[];scenarios=sessions=0
for profile in ['Debug','Release']:
 assert q['profiles'][profile]['sha256']==sha(q['profiles'][profile]['path'])
 for group,modes in expected_modes.items():
  tool=f'維護工具/verify-native-{group}.ps1';matches=[]
  for p in (root/'驗證紀錄/current'/('native-'+group)).glob('*/receipt.json'):
   if p.parent.name<a.since:continue
   d=read(p)
   if d.get('schema')==2 and d.get('profile')==profile and d.get('exe_sha256')==q['profiles'][profile]['sha256'] and d.get('source_pins',{}).get(tool)==sha(root/tool):matches.append((p,d))
  assert len(matches)==1,(profile,group,'expected one completed case receipt',len(matches))
  p,d=matches[0];out=p.parent
  assert d['jar_sha256']==reference['sha256'] and d['compilation_inputs_checked']==968
  assert not d['sql_access'] and not d['playable_game']
  assert [x['mode'] for x in d['cases']]==modes
  for source,expected in d['source_pins'].items():
   assert sha(root/source)==expected and sha(out/'source'/source)==expected,source
  script=(out/'source'/tool).read_text(encoding='utf-8-sig')
  for mode in modes:assert "'"+mode+"'" in script
  if d['fixture_parity']:
   parity=d['fixture_parity'];assert parity['all_non_jar_fields_equal'] and not parity['historical_fixture_relabelled']
   assert parity['current_jar_sha256']==reference['sha256']
  counts=[];archive_files={}
  for case in d['cases']:
   mode=case['mode'];assert case['verified'];c=case['client'];s=case['server']
   assert c==read(out/(mode+'-client.json'))
   assert c['pointer_bits']==64 and c['worker_quiescent'] and c['shutdown']=='closed'
   assert c['game']['selected_source']=='測試乙' and c['transport']['epoch']=='2'
   assert s['status']=='verified' and s['actual_client_sessions'] and not s['sql_access'] and not s['state_persistent']
   assert s['jar_sha256'].upper()==reference['sha256'] and s['mode']==mode
   runtime=Path(d['runtime'])/mode
   if 'fixed_probe_coordination_root' in case:
    assert s==read(out/'runtime-files/result.json')
    for file,h in case['runtime_file_pins'].items():assert sha(out/'runtime-files'/file)==h
    for file,h in case['previous_runtime_files'].items():assert sha(out/'previous-runtime-files'/file)==h
   else:assert s==read(runtime/'result.json')
   capture=Path(s['capture_file']);assert capture.is_relative_to(root/'LiTo381-Modern/target/current')
   rows=[json.loads(line) for line in capture.read_text(encoding='utf-8-sig').splitlines()]
   count=sum(row['direction']=='server_to_client' and row.get('note')=='S_KEY' for row in rows)
   assert count==2,(profile,group,mode,count)
   assert not (runtime/'state.properties').exists()
   archive_files[mode+'/frames.jsonl']=capture
   for suffix in ['server.log','server.stderr.txt','client.json','client.stderr.txt']:
    f=out/(mode+'-'+suffix);assert f.exists();archive_files[mode+'/'+suffix]=f
   counts.append({'mode':mode,'actual_ClientSessions':count,'LOGIN_sessions':1,'GAME_sessions':1,'capture_sha256':sha(capture)})
   scenarios+=1;sessions+=count
  archive=out/'protocol-raw-evidence.zip';files={key:sha(file) for key,file in archive_files.items()}
  if not archive.exists():
   with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED) as z:
    for key,file in archive_files.items():z.write(file,key)
  with zipfile.ZipFile(archive) as z:
   assert set(z.namelist())==set(files)
   for key,h in files.items():assert hashlib.sha256(z.read(key)).hexdigest().upper()==h
  results.append({'profile':profile,'group':group,'receipt':str(p),'receipt_sha256':sha(p),'cases':counts,'raw_archive_sha256':sha(archive),'raw_files':files})
assert len(results)==2*len(expected_modes) and scenarios==2*sum(map(len,expected_modes.values())) and sessions==2*scenarios
result={'status':'qualified_current_C02_both_mains_8_protocol_groups' if all_required_groups else 'qualified_current_C02_both_mains_selected_protocol_groups','groups':results,'groups_per_profile':len(expected_modes),'selected_groups':selected,'all_8_groups_verified':all_required_groups,'scenarios':scenarios,'actual_ClientSessions':sessions,'LOGIN_sessions':scenarios,'GAME_sessions':scenarios,'SQL_access':False,'GUI_verified':False,'natural_clan_gameplay_verified':False,'complete_gameplay_verified':False,'teleport_recovery_verified':False,'full_current_TCP_regression_verified':False,'whole_goal_complete':False,'source_manifest_sha256':sha(normal/'source-inputs.json'),'qualifier_sha256':sha(__file__)}
output=(a.output or root/'驗證紀錄/current/native-protocol-current-bootstrap/protocol-groups-qualified.json').resolve();assert output.is_relative_to(root/'驗證紀錄/current');assert not output.exists()
output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(f'Qualified current {len(expected_modes)} groups/profile:{scenarios} scenarios,{sessions} actual ClientSessions; all8={all_required_groups}; no full gameplay/TCP/GUI claim.')
