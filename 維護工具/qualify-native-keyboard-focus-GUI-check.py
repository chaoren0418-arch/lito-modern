"""Independently bind normal EXEs, Sky focus observations and actual formal sessions."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';F=V/'native-keyboard-focus-sky-gui'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(v):return(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
cases={'Debug':(F/'case-20261019-132118-403441-debug-focus-only',56706,'da7b82'),
       'Release':(F/'case-20261019-132407-587226-release-focus-only',99231,'a9c5e3')}
queue=read(R/'工作佇列.json');source=queue['current_product_identity']['source_manifest_sha256'];assert source=='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
profiles={};identities={};files={}
for profile,(case,session,chunk)in cases.items():
    c=read(case/'cohort.json');assert c['profile']==profile and c['scenario']=='focus-regression-only'
    assert c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'and c['recorded_owned_processes_terminal']
    p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
    assert p['scope']=='normal_focus_regression_only'and p['source_manifest_sha256']==c['source_manifest_sha256']==source
    normal=read(p['normal_qualification']);assert sha(p['normal_qualification'])==p['normal_qualification_sha256']
    assert normal['status']=='independently_qualified_keyboard_focus_normal_related_both_profiles'and normal['passed_per_profile']==16
    assert normal['source_manifest_sha256']==source and normal['observed_terminal_exit_code']==0 and normal['recorded_owned_processes_all_terminal']
    assert p['compiled_inputs']==read(Path(p['normal_qualification']).parent.parent/'compiled-inputs.json')and len(p['compiled_inputs'])==1048
    for name,digest in p['compiled_inputs'].items():assert sha(R/name)==digest,name
    product=p['products'][profile]['a'];assert sha(product['path'])==sha(product['original_path'])==product['sha256']==normal['normal_mains'][profile]
    assert sha(p['jar'])==p['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
    assert sha(p['catalog'])==p['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
    for name,digest in p['class_files'].items():assert sha(Path(p['classes'])/name)==digest
    for name,digest in p['fixture_sources'].items():assert sha(name)==digest
    terminal=sorted(case.glob('checkpoint-terminal-*'));assert len(terminal)==1
    receipt=read(terminal[0]/'evidence.json');assert receipt['original_session_id']==session and receipt['observed_actual_exit_code']==0 and receipt['terminal_tool_chunk']==chunk and receipt['original_identities_terminal']
    assert len(c['processes'])==2 and all(row['exit_code']==0 and not row.get('terminated_during_failure_cleanup')for row in c['processes'])
    for row in [c['pipeline'],*c['processes']]:identities[(row['pid'],row['creation_utc'])]=row
    events=read(case/'Sky-events.json');by_label={row['label']:row for row in events}
    required_labels=['initial','connect-space','login-ready','account','password','focus-login','login','source-ready','source-choice',
                     'focus-source','commit-source','game-ready','focus-after-source','focus-get-characters','get-characters','character-response']
    assert len(by_label)==len(events)and set(required_labels)<=set(by_label)
    positions=[next(i for i,row in enumerate(events)if row['label']==label)for label in required_labels]
    assert positions==sorted(positions)
    expected={'login-ready':'0 視窗','password':'5 編輯','focus-login':'7 按鈕 登入','source-choice':'9 下拉式方塊',
              'focus-source':'12 按鈕 選擇來源','commit-source':'0 視窗','game-ready':'0 視窗',
              'focus-after-source':'13 按鈕 中斷連線','focus-get-characters':'14 按鈕 取得角色','get-characters':'0 視窗','character-response':'0 視窗'}
    for label,prefix in expected.items():assert by_label[label]['focused_element'].startswith(prefix),(profile,label)
    assert '14 按鈕 取得角色 ID: 105'in by_label['game-ready']['tree']
    assert '8 文字 角色 ID: 304'in by_label['character-response']['tree']and'請選擇角色。'in by_label['character-response']['tree']
    assert 'PeerFixture(測試乙)'in by_label['character-response']['tree']
    app='process:'+product['path'];assert all(row['window']['app'].casefold()==app.casefold()for row in events)
    assert len({row['window']['id']for row in events})==1 and all(not row['actual_screenshot_returned']for row in events)
    times=[datetime.fromisoformat(row['observed_utc'].replace('Z','+00:00'))for row in events];assert times==sorted(times)
    result=read(case/'result.json');assert result['actual_client_sessions']==2 and result['game_sessions']==1
    assert result['memberships_empty']and result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==result['authority_mutations']==0
    assert not result['sql_access']and not result['state_persistent']and result['jar_sha256'].upper()==p['jar_sha256']
    trace=[json.loads(line)for line in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
    assert any(row['state']['phase']=='AUTHENTICATED'and row['state']['test_account']=='PeerFixture'and row['state']['source']=='測試乙'and row['state']['available_characters']==['TestKnight']for row in trace)
    assert any(row['state']['phase']=='LOBBY'and row['state']['test_account']=='PeerFixture'for row in trace)
    profiles[profile]=dict(case=str(case),original_session=session,terminal_tool_chunk=chunk,observed_exit_code=0,normal_exe_sha256=product['sha256'],actual_formal_sessions=2,actual_GAME_sessions=1,Sky_observations=len(events),
        source_to_GAME_keyboard_navigation_verified=True,backend_character='TestKnight',wire_character_alias='PeerFixture(測試乙)',character_selected=False,world_entered=False,full_GUI_verified=False)
    for name in ['cohort.json','Sky-events.json','ready.json','result.json','server-trace.jsonl','server.properties','server.stdout.log','server.stderr.log','a.stdout.log','a.stderr.log']:
        files[profile+'/'+name]=case/name
    files[profile+'/prepared.json']=Path(c['prepared'])/'prepared.json';files[profile+'/normal-qualification.json']=Path(p['normal_qualification'])
    files[profile+'/terminal-observation.json']=terminal[0]/'evidence.json'
numeric=','.join(str(pid)for pid,_ in identities)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+numeric+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
assert not any(row['ProcessId']==pid and abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created)).total_seconds())<.001 for row in live for pid,created in identities)
out=F/('completed-focus-check-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'original-owned-CIM.json').write_bytes(raw)
files['qualifier.py']=out/'qualifier.py';files['original-owned-CIM.json']=out/'original-owned-CIM.json';hashes={name:sha(p)for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name,p in files.items():z.write(p,name)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in hashes.items())
record=dict(status='independently_qualified_both_normal_profiles_keyboard_source_to_GAME_focus_regression_limited',source_manifest_sha256=source,
    profiles=profiles,actual_formal_sessions=4,recorded_original_owned_identities=len(identities),recorded_original_owned_processes_terminal=True,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,normal_keyboard_focus_regression_qualified=True,
    character_identity_continuity_qualified=False,world_entry_verified=False,screenshots_verified=False,mouse_verified=False,IME_verified=False,multiple_DPI_verified=False,
    physical_audio_verified=False,full_current_library_verified=False,GUI_full_flow_verified=False,whole_G06_complete=False,whole_goal_complete=False)
(out/'qualified.json').write_bytes(data(record))
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
work='來源AB3CF26C正常兩版焦點定向GUI已獨立核對：Sky鍵盤／可存取性實際登入、選來源、Tab前進取得角色，兩版合4 LOGIN/GAME sessions，原始兩案皆exit0且6組程序皆終止。只通過此焦點範圍，未選角色或入場，圖像／滑鼠／IME／多DPI／裝置與完整GUI未驗；Sky擷取仍逾時。當版正常兩主與16冒煙各版合格；接續嚴格同版控制器、廣域與完整library，2DAF全部舊結果保留原資格。先自行測試再交人工，不請使用者登入，相鄰只讀，G01–G12未完成。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for name in before:
    path=R/name
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value['current_keyboard_focus_GUI_regression']=dict(**record,qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
        value['current_source_goal_validation']['G08-player-gui'].update(complete=False,state='current_normal_keyboard_focus_GUI_regression_qualified_full_flow_pending',focus_regression=value['current_keyboard_focus_GUI_regression'])
        for item in value.get('items',[]):
            if item.get('id')=='G08-player-gui':item.update(complete=False,state='current_normal_keyboard_focus_GUI_regression_qualified_full_flow_pending',current_evidence=value['current_keyboard_focus_GUI_regression'],current_evidence_applies_to_current_source=True)
        value.update(next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='post_keyboard_focus_same_source_regression',work=work,applies_to_current_source=True,goal_complete=False)
        if name.endswith('next-validation-action.json'):value['next_action']=work
        staged[name]=data(value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[name]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items())
(out/'evidence.json').write_bytes(data(record))
assert all(sha(R/name)==h for name,h in before.items())
for name,payload in staged.items():(R/name).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={name:sha(R/name)for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_sessions=4,terminal_identities=len(identities),full_GUI_verified=False),ensure_ascii=False))
