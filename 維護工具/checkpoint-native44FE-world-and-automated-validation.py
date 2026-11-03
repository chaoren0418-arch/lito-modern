"""Record both WORLD branches and retire human handoff; release reservation for broad regression."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
paths={'Debug':V/'native44FE-world-tree/run-20261018-173508-298197/qualified-20261018-173641-037877/qualified.json',
       'Release':V/'native44FE-world-tree/run-20261018-173810-370193/qualified-20261018-173924-489690/qualified.json'}
proofs={}
for profile,p in paths.items():
    q=read(p);assert q['status']=='independently_qualified_same_connection_arena_WORLD_audio_and_authority'
    assert q['actual_sessions']==4 and q['profile']==profile and q['recorded_owned_all_terminal'] and len(q['recorded_identities'])==3
    assert q['source_manifest_sha256']=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
    assert q['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
    assert q['normal_source_bodies_unchanged'] and not q['diagnostic_API_extension'] and not q['source_overlays_used']
    assert q['passive_same_GAME_authority_and_walk_verified'] and sha(p.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(p.parent/'raw.zip')as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    proofs[profile]=dict(path=str(p),sha256=sha(p),original_session_id=q['original_session_id'],original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'],product_sha256=q['product_sha256'],normal_sha256=q['normal_sha256'],actual_sessions=4,recorded_owned_all_terminal=True)
retirement=V/'native44FE-interactive-teleport/20261018-172654-846629/retirement.json';r=read(retirement)
assert r['status']=='interactive_fixture_retired_by_user_direction' and r['recorded_GUI_and_server_processes_terminal'] and not r['authentication_outcome_inferred']
failure=V/'native44FE-world-tree/run-20261018-173346-505185/preflight-failure/failure.json';f=read(failure)
assert f['Rust_exit_code']==101 and f['server_exit_code']==1 and f['recorded_owned_all_terminal'] and not f['gameplay_qualification_granted']
prior=V/'native44FE-hotel/20261018-172052-514017/checkpoint-20261018-172907-930803/readback.json';before=read(prior)['after_files']
assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=V/'native44FE-world-tree'/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='both_current_normal_body_same_GAME_arena_WORLD_branches_qualified_limited',source_manifest_sha256=q['source_manifest_sha256'],reference_jar_sha256=q['reference_jar_sha256'],
    profiles=proofs,actual_sessions=8,actual_world_tree_requests=4,actual_heal1_requests=8,exact_S5A=4,exact_C08=4,old_unplayed_cues_rejected=4,old_cue_deliveries=0,new_own_heal_events_delivered_once=4,
    recorded_owned_identities=6,recorded_owned_all_terminal=True,normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_API_extension=False,
    initial_repository_fixture='Skill131 initially added to both isolated memory books; original class/stats/graphics/HP/MP retained. No runtime authority mutation.',
    passive_same_GAME_authority_and_walk_verified=True,preserved_preflight_failure=str(failure),preserved_preflight_failure_sha256=sha(failure),
    normal_EXE_runtime_inferred=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,full_G06_verified=False,whole_goal_complete=False)
policy=dict(status='finish_development_and_automated_end_to_end_before_human_acceptance',development_stage_human_login_handoffs_allowed=False,independent_work_must_continue=True,source='User explicitly requested this after the 56044 login handoff.',rules=str(ROOT/'AGENTS.md'))
save(out/'evidence.json',dict(current_WORLD=record,interactive_retirement=dict(path=str(retirement),sha256=sha(retirement),record=r),human_acceptance_policy=policy))
work='當版44FEA738／75DA69B0正常模組兩版戰場→普通WORLD已獨立核對8sessions：同GAME進map4、世代加一、角色／物品接續與舊輸入退役，權威WORLD位置與後續移動一致；4個舊待播cue零交付，4個新場景本人heal事件各一次。只含初始測試書加入131的隔離契約，無正常GUI／實體音效或C72同端點資格。首輪舊目錄防護Rust101／強制回收服務1在任何玩家連線前失敗，已保存，不移記通過。人工登入測試依使用者指示回收：原58583exit1、服務1，原視窗已不存在，捕獲1個遮罩登入請求但無GAME，不推定認證結果；開發期間不再請使用者登入或操作，先自行完成開發與自動化端到端後才交人工驗收。兩版23場景、控制器／音效各96sessions及旅館16sessions保持各自範圍。接續當版112整合目標＋主程式廣域及完整library；同端點契約缺口、其他服務、必要正常GUI、有效不同資料集ABA與G11／全G01–G12仍待，原時限與預算不變，相鄰唯讀且產物獨立。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==record['source_manifest_sha256']
        x.update(current_native44FE_WORLD=record,current_visible_login=dict(status=r['status'],retirement=str(retirement),retirement_sha256=sha(retirement),GUI_acceptance=False),human_acceptance_policy=policy,
            active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'updated_date'in x:x['updated_date']=date
        x['current_focus']=dict(item='current-source-broad-and-full-library-automated-validation',work=work,applies_to_current_source=True,goal_complete=False)
        for item in x.get('items',[]):
            if item.get('id','').startswith('G06-'):
                item['current_same_GAME_WORLD_evidence']=record
                for step in item.get('rental_acceptance_steps',[]):
                    if step.get('id')=='G06.3d':step['current_same_GAME_WORLD_evidence']=record
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(out.parent/'current-continuation.json',dict(status='both_current_WORLD_branches_qualified_broad_library_next',latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),profiles=proofs,actual_sessions=8,human_acceptance_policy=policy,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),WORLD_sessions=8,human_login_handoff_retired=True,whole_goal_complete=False),ensure_ascii=False))
