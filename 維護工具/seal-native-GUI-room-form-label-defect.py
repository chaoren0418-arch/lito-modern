"""Seal a real normal-window room-form label defect and limited observed flow."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
case=V/'native-installed-resource-profile-sky-gui/case-20261020-011031-282205-debug-ordinary-room-service'
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='D9B0F468FA65E186B246269A6F00B8D6B4EB76742049E3FBCC744357A780022D'
assert c['profile']=='Debug' and c['scenario']=='ordinary-room-service' and c['fixture_profile']=='gui-room-admin'
assert c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'
assert c['recorded_owned_processes_terminal'] and all(x['exit_code']==0 and not x.get('terminated_during_failure_cleanup')for x in c['processes'])
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
assert p['source_manifest_sha256']==source and len(p['compiled_inputs'])==1049
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h,n
for key in ['fixture_sources','fixture_origins']:
 for n,h in p[key].items():assert sha(n)==h,n
for n,h in p['class_files'].items():assert sha(Path(p['classes'])/n)==h,n
for product in p['products']['Debug'].values():assert sha(product['path'])==sha(product['original_path'])==product['sha256']
assert sha(p['jar'])==p['jar_sha256'] and sha(p['catalog'])==p['catalog_sha256']
assert sha(case/'runner.py')==c['launcher_sha256']
terminal=next(case.glob('checkpoint-terminal-*/evidence.json'));t=read(terminal)
assert t['original_session_id']==37239 and t['terminal_tool_chunk']=='105de7' and t['observed_actual_exit_code']==0 and t['original_identities_terminal']
result=read(case/'result.json');assert result['actual_client_sessions']==4 and result['game_sessions']==2
assert result['memberships_empty'] and result['authority_mutations']==result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==0
assert not result['sql_access'] and not result['state_persistent']
events=read(case/'Sky-events.json');by={(x['role'],x['label']):x for x in events}
for role in ['a','b']:
 for label in ['login','commit-source','get-characters','select-character','enter-channel','leave-room']:
  assert by[role,label]['action']==dict(method='press_key',args={'key':'space'})
 assert by[role,'source-choice']['action']['args']['value']=='測試乙'
 assert by[role,'close-normal']['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
 assert '大廳房間' in by[role,'lobby-after-leave']['tree']
defect=[]
for label in ['create-room-form','edit-room-form']:
 event=by['a',label];lines=event['tree'].splitlines()
 assert any('新密碼' in line and 'ID: 213' in line for line in lines)
 assert any('人數上限' in line and 'ID: 315' in line for line in lines)
 assert any('編輯' in line and 'ID: 214' in line and '人數上限' not in line for line in lines)
 defect.append(dict(label=label,observed_utc=event['observed_utc'],wrong_room_name='新密碼',expected_room_name='房間名稱',capacity_accessible_label_missing=True))
for role,label,text in [('b','force-changed','PeerFixture2(測試乙) 隊伍2 未準備'),('a','guest-force-synchronization-readback','PeerFixture2(測試乙) 隊伍2 未準備'),('b','ready-confirmed','PeerFixture2(測試乙) 隊伍2 已準備'),('a','guest-ready-synchronized','PeerFixture2(測試乙) 隊伍2 已準備'),('b','unready-confirmed','PeerFixture2(測試乙) 隊伍2 未準備')]:assert text in by[role,label]['tree']
assert 'GUI改名房' in by['a','room-edited']['tree']
assert 'GUI改名房' not in by['a','lobby-after-leave']['tree']
frames=next((case/'captures').glob('*/frames.jsonl'));f=[json.loads(l)for l in frames.read_text('utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN'for x in f)==4
packets=[(x,bytes.fromhex(x['plain']))for x in f if x['plain']!='MASKED_C_LOGIN']
out=[b for x,b in packets if x['direction']=='client_to_server'];incoming=[b for x,b in packets if x['direction']=='server_to_client']
name='PeerFixture2(測試乙)'.encode('cp950')+b'\0'
assert [b for b in out if b[0]==0xd3]==[b'\xd3\x01']
assert [b for b in out if b[0]==0xd2]==[b'\xd2\x01',b'\xd2\x00']
assert incoming.count(b'\xd4'+name+b'\x01')==2
assert incoming.count(b'\xd1'+name+b'\x01')==incoming.count(b'\xd1'+name+b'\x00')==2
assert [b for b in out if b[0]==0xcc]==[b'\xcc'+'GUI測試房'.encode('cp950')+b'\0\0\0\x04\0']
assert [b for b in out if b[0]==0xdd]==[b'\xdd'+'GUI改名房'.encode('cp950')+b'\0\x20\0\x05\0'+b'\0'*5]
assert [b for b in out if b[0]==0xcf]==[b'\xcf\x02\0\0\0\0']
assert out.count(b'\xd0')==2 and incoming.count(b'\xcf\x02\0\0\0')==1
trace=[json.loads(l)['state']for l in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
assert any(x['room_state'] and x['room_state']['name']=='GUI測試房' and x['room_state']['capacity']==4 and len(x['room_state']['members'])==2 for x in trace)
assert any(x['room_state'] and x['room_state']['name']=='GUI改名房' and x['room_state']['capacity']==5 and len(x['room_state']['members'])==1 for x in trace)
rows=[c['pipeline'],*c['processes']];ids=','.join(str(x['pid'])for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',cmd]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live for row in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
outdir=case/('label-defect-analysis-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));outdir.mkdir()
files={str(x.relative_to(case)).replace('\\','/'):x for x in case.iterdir()if x.is_file()}
files.update({'captures/frames.jsonl':frames,'terminal.json':terminal,'prepared.json':Path(c['prepared'])/'prepared.json','original-native_login_window.rs':R/'LiTo381-Modern/src/modern_lito/native_login_window.rs','native_room.rs':R/'LiTo381-Modern/src/modern_lito/native_room.rs','sealer.py':Path(__file__)})
hashes={n:sha(x)for n,x in files.items()}
with zipfile.ZipFile(outdir/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(outdir/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='confirmed_normal_GUI_room_form_accessible_label_defect_preserved',source_manifest_sha256=source,case=str(case),profile='Debug',original_session_id=37239,terminal_tool_chunk='105de7',observed_original_exit_code=0,original_owned_identities_terminal=True,defects=defect,actual_formal_sessions=4,aggregate_wire_CC_create=1,aggregate_wire_DD_edit=1,aggregate_wire_CF_join=1,aggregate_wire_D3_force=1,aggregate_wire_D2_ready_unready=2,aggregate_wire_D0_leave=2,aggregate_wire_state_notifications_verified=True,individual_socket_notification_delivery_claimed=False,normal_GUI_room_behavior_observed=True,normal_GUI_room_form_labels_correct=False,normal_GUI_room_service_acceptance_granted=False,GUI_full_flow_verified=False,whole_goal_complete=False,raw_archive=str(outdir/'raw.zip'),raw_archive_sha256=sha(outdir/'raw.zip'),raw_files=hashes)
(outdir/'evidence.json').write_bytes(data(record));(outdir/'original-owned-CIM.json').write_bytes(raw)
work='當版Debug普通房間正常GUI已實跑4sessions並正常退出；建立／加入／切隊／準備取消／修改／離房封包與狀態已核對。建立及修改表單ROOM_NAME213誤標新密碼、CAPACITY214缺名稱，原始資料已封存；先修標籤，再同來源完整library Debug/Release及GUI回歸。此批不授普通房間或完整GUI驗收；其餘G01–G12未完成，先自行驗證才交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
 path=R/n
 if path.suffix=='.json':
  v=read(path);v['current_normal_GUI_room_form_label_defect']=record;v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(outdir/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);v['current_focus']=dict(item='native_room_form_accessible_labels',work=work,applies_to_current_source=True,goal_complete=False)
  if n.endswith('next-validation-action.json'):v['next_action']=work
  staged[n]=data(v)
 else:
  first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(outdir/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in before:z.write(R/n,n)
with zipfile.ZipFile(outdir/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(outdir/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(outdir/'before.zip'),evidence_sha256=sha(outdir/'evidence.json'))))
print(json.dumps(dict(evidence=str(outdir/'evidence.json'),actual_formal_sessions=4,label_defect_confirmed=True,GUI_full_flow_verified=False),ensure_ascii=False))
