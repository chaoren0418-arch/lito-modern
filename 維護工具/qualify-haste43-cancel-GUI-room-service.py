"""Qualify current normal room service keyboard flow and three correct form labels; no full GUI scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);args=ap.parse_args();case=args.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
profile=c['profile'];assert profile in ['Debug','Release'];assert c['scenario']=='ordinary-room-service' and c['fixture_profile']=='gui-room-admin'
assert c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'
assert c['recorded_owned_processes_terminal'] and all(x['exit_code']==0 and not x.get('terminated_during_failure_cleanup')for x in c['processes'])
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
plan_path=Path(r"D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-haste43-cancel-tooling\GUI-20261020-115815-159424\room-positive-plan.json")
assert sha(plan_path)=='6723A0AF7A29C7AD7C5680EF479784F87E64D8933CF420EC77D2CA303AF50AE6';plan=read(plan_path);assert plan['source_manifest_sha256']==source
normal=read(p['normal_qualification']);assert sha(p['normal_qualification'])==p['normal_qualification_sha256'] and normal['source_manifest_sha256']==source and normal['passed_per_profile']==19
library=Path(p['library']);assert sha(library/'completion-observation.json')==p['library_completion_sha256'] and read(library/'completion-observation.json')['source_manifest_sha256']==source and w['current_full_default_library_verified']
for key in ['current_native_library_resource_optin','current_native_library_shared_resource_optin','current_native_rental_return_reconnect_api_qualification']:
 q=w[key];assert q['source_manifest_sha256']==source and q['qualified'] and sha(q['qualification'])==q['qualification_sha256']

assert p['source_manifest_sha256']==source and len(p['compiled_inputs'])==1049
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h,n
for key in ['fixture_sources','fixture_origins']:
 for n,h in p[key].items():assert sha(n)==h,n
for n,h in p['class_files'].items():assert sha(Path(p['classes'])/n)==h,n
for product in p['products'][profile].values():assert sha(product['path'])==sha(product['original_path'])==product['sha256']
assert sha(p['jar'])==p['jar_sha256'] and sha(p['catalog'])==p['catalog_sha256']
assert sha(case/'runner.py')==c['launcher_sha256']
terminals=list(case.glob('checkpoint-terminal-*/evidence.json'));assert len(terminals)==1;terminal=terminals[0];t=read(terminal)
assert isinstance(t['original_session_id'],int) and t['original_session_id']>0 and t['terminal_tool_chunk'] not in ['',None,'pending','unknown'] and t['observed_actual_exit_code']==0 and t['original_identities_terminal']
result=read(case/'result.json');assert result['actual_client_sessions']==4 and result['game_sessions']==2
assert result['memberships_empty'] and result['authority_mutations']==result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==0
assert not result['sql_access'] and not result['state_persistent']
events=read(case/'Sky-events.json');by={}
times=[datetime.fromisoformat(x['observed_utc'].replace('Z','+00:00'))for x in events];assert times==sorted(times)
for event in events:
 key=(event['role'],event['label'])
 if key in by:assert by[key]['action'] is None and event['action'] is None
 by[key]=event
for role in ['a','b']:
 own=[event for event in events if event['role']==role];product=p['products'][profile][role]
 assert len({event['window']['id']for event in own})==1 and all(event['window']['app'].casefold()==('process:'+product['path']).casefold()for event in own)
 actions={'login':102,'commit-source':103,'get-characters':105,'select-character':103,'enter-channel':103,'leave-room':112}
 actions.update({'create-room':121,'edit-room':121}if role=='a'else{'join-room':109,'change-force':118,'ready':110,'unready':111})
 for label,control in actions.items():
  assert by[role,label]['action']==dict(method='press_key',args={'key':'space'})
  at=next(i for i,event in enumerate(own)if event is by[role,label]);assert at>0 and ('ID: '+str(control))in (own[at-1].get('focused_element')or'')
 choice=by[role,'source-choice'];action=choice['action']
 if action['method']=='set_value':
  assert action['args']['value']=='測試乙'
 else:
  assert action==dict(method='press_key',args={'key':'End'})
  at=next(i for i,event in enumerate(own)if event is choice);assert at>0
  assert '來源伺服器 Value: 測試甲 ID: 203' in (own[at-1].get('focused_element')or'')
  assert '來源伺服器 Value: 測試乙 ID: 203' in choice['tree']
  assert '來源伺服器 Value: 測試乙 ID: 203' in (choice.get('focused_element')or'')
 assert by[role,'close-normal']['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
 assert '大廳房間' in by[role,'lobby-after-leave']['tree']
label_checks=[]
for label in plan['form_regressions']:
 event=by['a',label];lines=event['tree'].splitlines()
 for pair in plan['labels']:
  row=[line for line in lines if not line.startswith('The focused UI element') and '編輯' in line and ('ID: '+str(pair['EDIT'])) in line]
  assert len(row)==1 and pair['label'] in row[0],(label,pair,row)
  if label.endswith('secret-enabled') and pair['EDIT']==215:assert '(disabled' not in row[0]
 label_checks.append(dict(label=label,observed_utc=event['observed_utc'],correct_EDIT_names_verified=True))
for label,value in [('create-room-secret','QA42'),('edit-room-secret','QA43')]:
 assert by['a',label]['action']['method']=='set_value' and by['a',label]['action']['args']['value']==value
password=by.get(('b','lobby-password-choice'),by.get(('b','join-room-secret')))
assert password and password['action']['method']=='set_value' and password['action']['args']['value']=='QA42'
assert 'ID: 206' in (password.get('focused_element')or'')
password_index=password['action']['args']['element_index']
assert any(line.lstrip().startswith(str(password_index)+' 編輯 ') and '房間密碼' in line and 'ID: 206' in line for line in password['tree'].splitlines())
for role,label,text in [('b','force-changed','PeerFixture2(測試乙) 隊伍2 未準備'),('a','guest-force-synchronized','PeerFixture2(測試乙) 隊伍2 未準備'),('b','ready-confirmed','PeerFixture2(測試乙) 隊伍2 已準備'),('a','guest-ready-synchronized','PeerFixture2(測試乙) 隊伍2 已準備'),('b','unready-confirmed','PeerFixture2(測試乙) 隊伍2 未準備')]:assert text in by[role,label]['tree']
assert 'GUI改名房' in by['a','room-edited']['tree']
assert 'GUI改名房' not in by['a','lobby-after-leave']['tree']
frames=next((case/'captures').glob('*/frames.jsonl'));f=[json.loads(l)for l in frames.read_text('utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN'for x in f)==4
packets=[(x,bytes.fromhex(x['plain']))for x in f if x['plain']!='MASKED_C_LOGIN']
out=[b for x,b in packets if x['direction']=='client_to_server'];incoming=[b for x,b in packets if x['direction']=='server_to_client']
name='PeerFixture2(測試乙)'.encode('cp950')+b'\0'
assert [b[1:].rstrip(b'\0').decode('cp950')for b in out if b[0]==0xc8]==['測試乙']*4
assert [b[1:].rstrip(b'\0').decode('cp950')for b in out if b[0]==0xc9]==['PeerFixture(測試乙)','PeerFixture2(測試乙)']
assert all(b==b'\xca\0'for b in out if b[0]==0xca)
assert not any(b[0]in [0x08,0x10]for b in out)
assert [b for b in out if b[0]==0xd3]==[b'\xd3\x01']
assert [b for b in out if b[0]==0xd2]==[b'\xd2\x01',b'\xd2\x00']
assert incoming.count(b'\xd4'+name+b'\x01')==2
assert incoming.count(b'\xd1'+name+b'\x01')==incoming.count(b'\xd1'+name+b'\x00')==2
assert [b for b in out if b[0]==0xcc]==[bytes.fromhex(plan['expected_aggregate_CC'])]
assert [b for b in out if b[0]==0xdd]==[bytes.fromhex(plan['expected_aggregate_DD'])]
assert [b for b in out if b[0]==0xcf]==[b'\xcf\x02\0\0\0QA42\0']
assert out.count(b'\xd0')==2 and incoming.count(b'\xcf\x02\0\0\0')==1
trace=[json.loads(l)['state']for l in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
assert any(x['room_state'] and x['room_state']['name']=='GUI測試房' and x['room_state']['capacity']==4 and x['room_state']['password_required'] and len(x['room_state']['members'])==2 for x in trace)
assert any(x['room_state'] and x['room_state']['name']=='GUI改名房' and x['room_state']['capacity']==5 and x['room_state']['password_required'] and len(x['room_state']['members'])==1 for x in trace)
for account in ['PeerFixture','PeerFixture2']:
 own=[x for x in trace if x['test_account']==account and x['source']=='測試乙'];assert own
 assert any(x['phase']=='ROOM'and x['room']==2 and x['channel']==0 for x in own)
 assert any(x['phase']=='CHANNEL'and x['room']==0 and x['channel']==0 for x in own)
rows=[c['pipeline'],*c['processes']];ids=','.join(str(x['pid'])for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',cmd]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live for row in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
outdir=case/'qualified-room-service-keyboard';outdir.mkdir()
files={str(x.relative_to(case)).replace('\\','/'):x for x in case.iterdir()if x.is_file()}
files.update({'captures/frames.jsonl':frames,'terminal.json':terminal,'prepared.json':Path(c['prepared'])/'prepared.json','original-native_login_window.rs':R/'LiTo381-Modern/src/modern_lito/native_login_window.rs','native_room.rs':R/'LiTo381-Modern/src/modern_lito/native_room.rs','qualifier.py':Path(__file__),'positive-plan.json':plan_path,'normal-qualification.json':Path(p['normal_qualification']),'library-completion.json':library/'completion-observation.json'})
hashes={n:sha(x)for n,x in files.items()}
reader_rejections=[]
for failure_path in case.glob('reader-input-binding-rejection-*/evidence.json'):
 failure=read(failure_path);assert failure['source_manifest_sha256']==source and failure['reader_exit_code']==1 and not failure['GUI_qualification_granted']
 archive=failure_path.with_name('raw.zip');assert sha(archive)==failure['raw_archive_sha256']
 reader_rejections.append(str(failure_path))
 for original in [failure_path,archive]:
  name=original.relative_to(case).as_posix();files[name]=original;hashes[name]=sha(original)
with zipfile.ZipFile(outdir/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(outdir/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_normal_room_service_keyboard_and_form_labels_limited',source_manifest_sha256=source,case=str(case),profile=profile,original_session_id=t['original_session_id'],terminal_tool_chunk=t['terminal_tool_chunk'],observed_original_exit_code=0,original_owned_identities_terminal=True,label_checks=label_checks,actual_formal_sessions=4,aggregate_wire_CC_create=1,aggregate_wire_DD_edit=1,aggregate_wire_CF_join=1,aggregate_wire_D3_force=1,aggregate_wire_D2_ready_unready=2,aggregate_wire_D0_leave=2,aggregate_wire_state_notifications_verified=True,individual_socket_notification_delivery_claimed=False,normal_GUI_room_behavior_observed=True,normal_GUI_room_form_labels_correct=True,normal_GUI_room_service_acceptance_granted=True,GUI_full_flow_verified=False,whole_goal_complete=False,raw_archive=str(outdir/'raw.zip'),raw_archive_sha256=sha(outdir/'raw.zip'),raw_files=hashes)
record.update(prior_reader_rejections_preserved=reader_rejections,source_choice_keyboard_or_value_bound_to_exact_C8=True,room_password_bound_to_observed_EDIT206=True)
(outdir/'qualified.json').write_bytes(data(record));record.update(qualification=str(outdir/'qualified.json'),qualification_sha256=sha(outdir/'qualified.json'));(outdir/'evidence.json').write_bytes(data(record));(outdir/'original-owned-CIM.json').write_bytes(raw)
work='當版'+profile+'正常普通房間鍵盤GUI限定核對：自有密碼建立及加入、切隊、準備取消、房主修改、雙方離房及正常退出，4sessions／四程序皆終止。建立與修改三個EDIT可及名稱正確，新密碼欄啟用與真實CC/DD欄位核對；只授此範圍，完整GUI、戰鬥、觀戰踢人／模式地圖／共享鑰匙／自然到期及G01–G12仍待，先自行完整測試才交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
 path=R/n
 if path.suffix=='.json':
  v=read(path);v.setdefault('current_normal_GUI_room_service_keyboard',{})[profile]=record;v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(outdir/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);v['current_focus']=dict(item='normal_GUI_full_flow_automation',work=work,applies_to_current_source=True,goal_complete=False)
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
print(json.dumps(dict(evidence=str(outdir/'evidence.json'),actual_formal_sessions=4,room_form_labels_correct=True,profile=profile,GUI_full_flow_verified=False),ensure_ascii=False))
