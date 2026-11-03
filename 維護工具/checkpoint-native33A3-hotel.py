"""Reserve/release the exact normal hotel runner; qualification stays scope limited."""
import argparse,zipfile
from datetime import datetime,timezone,timedelta
from native33A3_hotel_gate import *

ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);g,normal,n,inputs=bind(run,terminal=False);terminal=a.terminal_chunk is not None
assert a.session>0
identities=[g['runner_identity'],*g['process_history']]
for binding in g['cases'].values():
    case=Path(binding['directory'])
    for role in ['server','a','b']:identities.append(read(case/(role+'-launch-identity.json')))
live,raw=original_live(identities)
if terminal:
    assert a.exit_code in [0,1] and a.terminal_chunk not in ['','pending','unknown'] and g['owned_process'] is None and not live
    assert g['status'] in ['executed_current33A3_normal_hotel_eight_cases_readback_pending','failed_preserved']
else:
    assert a.exit_code is None and g['status']=='running_current33A3_normal_hotel_eight_cases'
    assert any(x==g['runner_identity'] for x in live)
qualification=run/'qualified.json';qualified=terminal and a.exit_code==0 and qualification.exists()
if qualified:
    q=read(qualification);assert q['status']=='independently_qualified_current33A3_normal_hotel_rooms_keys_and_shared_owner_return' and q['cohort_sha256']==sha(run/'cohort.json')
    assert q['source_manifest_sha256']==SOURCE and q['actual_formal_sessions']==32 and q['original_session_id']==a.session and q['terminal_tool_chunk']==a.terminal_chunk
queue=read(ROOT/'工作佇列.json');assert queue['active_workload_sessions'] in [{},{'G06_hotel':a.session}]
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
out=run/('checkpoint-'+('terminal-' if terminal else 'active-')+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
record=dict(status='current_normal_hotel_qualified_limited' if qualified else 'current_normal_hotel_terminal_readback_pending' if terminal else 'original_current_normal_hotel_live',run=str(run),source_manifest_sha256=SOURCE,original_session_id=a.session,runner_identity=g['runner_identity'],terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.exit_code,recorded_original_owned_processes_all_terminal=terminal,qualification=str(qualification) if qualified else None,qualification_sha256=sha(qualification) if qualified else None,actual_sessions=32 if qualified else None,qualification_granted=qualified,normal_source_modified=False,launcher_artifact_modified=False,GUI_verified=False,natural_expiry_verified=False,manual_guest_exit_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record);(out/'process-CIM.json').write_bytes(raw);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
text='當版33A3CCF7正常EXE旅館自動測試原'+str(a.session)+'：'+('兩版八案32 LOGIN/GAME sessions、三種房間／鑰匙租還／共享拾取與屋主退租返回已獨立核對；44張原地形診斷PNG與33記錄執行身分終止。' if qualified else '原程序已終止，結果仍待獨立讀回或保留失敗。' if terminal else '沿原程序執行，來源及對應QA固定，不重啟。')+'自然到期及房客手動離開仍須另驗，GUI／實體音效與完整G06、G01–G12未完成。開發期間不請使用者登入，登入器與重建版為獨立產物，相鄰只讀，8秒／60秒／Release與G11預算不變。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for name in before:
    path=ROOT/name
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==SOURCE
        value.update(current_native33A3_hotel=record,active_workloads={} if terminal else {'G06_hotel':record},active_workload_sessions={} if terminal else {'G06_hotel':a.session},source_and_corresponding_QA_frozen=not terminal,source_freeze_while_Cargo=False,next_task=text,next_focus=text,last_goal_turn_classification='progress' if terminal else 'verified_wait',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current-normal-hotel',work=text,applies_to_current_source=True,goal_complete=False)
        if 'updated_date' in value:value['updated_date']=date
        if name.endswith('next-validation-action.json'):value['next_action']=text
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+text+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name) for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=not terminal))
print(json.dumps(dict(checkpoint=str(out),original_session=a.session,terminal=terminal,qualified=qualified),ensure_ascii=False))
