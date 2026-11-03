"""Keep actual component evidence separate from pending full World/resource/live integration."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--library',type=Path);ap.add_argument('--session',type=int,default=67934);a=ap.parse_args()
broad=a.library.resolve(strict=True)if a.library else V/'native44FE-default112/20261018-174151-282798';b=read(broad/'cohort.json')
if a.library:
    assert broad.is_relative_to(V/'native44FE-library-default')and a.session>0
    assert b['status']in ['compiling_current_full_library_test_image','running_current_full_library_default_selection'];key='full_library'
else:assert b['status']=='executing_current_default112_profile'and b['active_profile']=='Release';key='broad_default'
continuation=read(broad.parent/'current-continuation.json');before=read(continuation['latest_readback'])['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n,h in read(Path(b['normal_qualification']).parent.parent/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(b['pipeline_pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
r=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert r.returncode==0 and not r.stderr
live=json.loads(r.stdout.decode('utf-8-sig'));assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(b['pipeline_creation_utc'])).total_seconds())<.001
candidate=ROOT/'維護工具/candidates/native44FE-ability-buff-requests';prep=read(candidate/'preparation.json');component=Path(prep['component']);cp=read(component/'receipt.json');proof=read(component/'readback.json')
assert cp['status']=='isolated_ability_request_and_selection_component_passed'and cp['passed']==5 and cp['failed']==cp['ignored']==0 and cp['owned_process']is None
assert all(p['exit_code']==0 and not p.get('cleanup_stopped_original_owned_handle')for p in cp['process_history'])and len(cp['process_history'])==3
assert sha(component/'readback.json')==prep['component_readback_sha256']and sha(component/'raw.zip')==proof['raw_archive_sha256']
with zipfile.ZipFile(component/'raw.zip')as z:
    for n,h in proof['raw_files'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
assert not prep['candidate_applied']and not prep['world_integration_executed']and not prep['live_probe_executed']
if prep.get('world_integration_typecheck_verified'):
    metadata=read(prep['world_integration_typecheck']);assert sha(prep['world_integration_typecheck'])==prep['world_integration_typecheck_sha256']
    assert metadata['status']=='isolated_whole_crate_metadata_typecheck_qualified_only'and not metadata['machine_code_generated']and not metadata['tests_executed']
    for n in ['native_world.rs','native_ability_buffs.rs','native_skills.rs','native_skill_selection.rs']:assert sha(candidate/n)==metadata['raw_files'][n],n
for n,h in {**prep['files'],**prep['world_files']}.items():assert sha(candidate/n)==h,n
assert sha(candidate/'live_probe.rs')==prep['live_probe_sha256']and sha(candidate/'native_ability_buffs.rs')==sha(component/'native_ability_buffs.rs')
codec=Path(prep['current_codec_evidence']);assert sha(codec/'readback.json')==prep['current_codec_readback_sha256'];wire=read(codec/'codec.stdout');assert wire['level_ability_enabled']is False and len(wire['cases'])==6
scripts=[ROOT/'維護工具'/n for n in ['compile-native44FE-extra-finite-effects.py','run-native44FE-extra-finite-effects.py','qualify-native44FE-extra-finite-effects.py','run-native44FE-ability-live.py','qualify-native44FE-ability-live.py']]
for p in scripts:ast.parse(p.read_text('utf-8-sig'))
files={p.relative_to(ROOT).as_posix():p for p in scripts};files.update({p.relative_to(ROOT).as_posix():p for p in candidate.iterdir()if p.is_file()})
files['component/ability_buff_component.rs']=component/'ability_buff_component.rs';pins={n:sha(p)for n,p in files.items()}
out=V/'native44FE-ability-buff-requests'/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'candidate-inputs.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'candidate-inputs.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status='isolated_ability_requests_and_codec_parser_component5_passed_World_integration_pending',source_manifest_sha256=b['source_manifest_sha256'],candidate=str(candidate),
    candidate_preparation_sha256=sha(candidate/'preparation.json'),skills=[26,42],component=str(component),component_readback_sha256=sha(component/'readback.json'),passed=5,failed=0,ignored=0,
    actual_formal_codec_examples=6,current_codec_evidence=str(codec),current_codec_readback_sha256=sha(codec/'readback.json'),default_S50_level_ability_enabled=False,
    missing_normal_opcodes=['0x6B','0x6C'],staged_receiver='Exact server score/weight/duration receipts, stop removal, no local spending/buff arithmetic, retire with new owner/world.',
    full_library_world_integration_compiled=False,whole_crate_runtime_metadata_typecheck_verified=prep.get('world_integration_typecheck_verified',False),world_receiver_lifecycle_executed=False,SPR_PCM_integration_executed=False,live_probe_normal_API_typecheck_verified=True,live_probe_executed=False,
    prepared_live_cases=['non-clan target refusal','self dexterity +5, MP50','self strength +5, MP50','natural low MP278 refusal','refusal recovery movement witnessed by the other peer'],
    prepared_files=pins,candidate_archive_sha256=sha(out/'candidate-inputs.zip'),normal_source_modified=False,candidate_applied=False,actual_sessions=0,actual_casts=0,same_clan_positive_target_verified=False,natural_expiry_verified=False,normal_GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',dict(ability_development=record,original_validation_kind=key,original_session=a.session,original_pipeline_live=live))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='當版44FEA738廣域原67934之Debug531過0敗21既有opt-in忽略已獨立讀回，Release沿原程序建置／執行，產品與對應QA固定；整批廣域及完整library未授予資格。兩種能力強化26／42之隔離請求／選取與原正式codec解析共5案通過；六個當版codec例證確認預設不啟用S50能力更新，須處理S6B／S6C。已備妥接收能力值／weight／duration及停止／換世界清除候選、11特效整合與雙玩家實際施法探針，完整World／SPR／PCM／實際施法尚未編譯執行，未套正常產品。當版技能書74ID辨識35／缺39數量未改；WORLD8sessions、旅館16sessions及控制器／正常模組音效各96sessions維持各自資格，同端點契約失敗保留。先完成自行開發及自動化端到端後才人工驗收，開發期間不請使用者登入；8秒／60秒／Release與G11預算不變，相鄰只讀、產物獨立，完整G01–G12未完成。'
if a.library:work=read(ROOT/'工作佇列.json')['next_task']+'雙玩家能力強化候選已補自然低MP278拒絕與恢復移動／對端觀察，元件5案及新探針公開API編譯檢查通過；實際候選資源與sessions仍未執行。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['active_workload_sessions']=={key:a.session}and x['source_and_corresponding_QA_frozen']
        x.update(current_native44FE_ability_development=record,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'current_focus'in x:x['current_focus']['work']=work
        if 'updated_date'in x:x['updated_date']=date
        for item in x.get('items',[]):
            if item.get('id','').startswith('G05-'):item.update(state='current_book74_request35_missing39_isolated_ability5_component_World_pending',complete=False,current_ability_development=record)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation.update(latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),ability_development=record);save(broad.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),passed_component=5,original_validation_kind=key,original_session=a.session,normal_modified=False,actual_casts=0),ensure_ascii=False))
