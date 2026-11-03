"""Record current missing requests and tested isolated guards while the original broad job stays reserved."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,subprocess,time,zipfile
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
audit=V/'native44FE-skill-book-audit/20261018-175440-043382';matrix=read(audit/'matrix.json');mproof=read(audit/'readback.json')
assert sha(audit/'matrix.json')==mproof['matrix_sha256']and(matrix['observed_unique_learned_ids'],matrix['recognized_count'],matrix['missing_count'])==(74,35,39)
candidate=ROOT/'維護工具/candidates/native44FE-extra-skill-finite-effects';prep=read(candidate/'preparation.json');component=Path(prep['component_test']);receipt=read(component/'receipt.json')
assert prep['component_typecheck_verified']and prep['component_metadata_guards_verified']and not prep['applied_to_normal_product']and not prep['SPR_or_PCM_playback_verified']
assert len(prep['graphics'])==11 and sha(candidate/'native_additional_finite_effects.rs')==prep['candidate_sha256']==sha(component/'native_additional_finite_effects.rs')
assert sha(component/'readback.json')==prep['component_test_readback_sha256']
checked=read(component/'inspect.stdout');assert checked['candidate_definitions']==checked['candidate_header_rejections']==checked['candidate_bank_rejections']==11
assert len(receipt['process_history'])==2 and all(j['exit_code']==0 for j in receipt['process_history'])
cp=read(component/'readback.json');assert sha(component/'raw.zip')==cp['raw_archive_sha256']
with zipfile.ZipFile(component/'raw.zip')as z:
    for n,h in cp['raw_files'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
broad=V/'native44FE-default112/20261018-174151-282798';b=read(broad/'cohort.json');assert b['active_profile']=='Release'and b['status']=='executing_current_default112_profile'
assert b['profiles']['Debug']['qualified']and b['profiles']['Debug']['counts']==dict(passed=531,failed=0,ignored=21)
pid=b['pipeline_pid'];command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(pid)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
result=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert result.returncode==0 and not result.stderr
live=json.loads(result.stdout.decode('utf-8-sig'));assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(b['pipeline_creation_utc'])).total_seconds())<.001
for n,h in read(Path(b['normal_qualification']).parent.parent/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
before=read(broad/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=audit/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_books_request_gaps_and_isolated_finite_definition_candidate_audited',source_manifest_sha256=matrix['source_manifest_sha256'],reference_jar_sha256=matrix['reference_jar_sha256'],
    matrix=str(audit/'matrix.json'),matrix_sha256=sha(audit/'matrix.json'),matrix_readback=str(audit/'readback.json'),matrix_readback_sha256=sha(audit/'readback.json'),
    actual_observed_learned_ids=74,request_recognized_count=35,missing_request_count=39,missing_request_ids=matrix['missing_request_ids'],recognition_is_not_successful_cast_count=True,
    candidate=str(candidate/'native_additional_finite_effects.rs'),candidate_sha256=prep['candidate_sha256'],candidate_preparation=str(candidate/'preparation.json'),candidate_preparation_sha256=sha(candidate/'preparation.json'),
    component=str(component),component_readback_sha256=sha(component/'readback.json'),component_original_definitions_accepted=11,component_changed_headers_rejected=11,component_changed_graphics_banks_rejected=11,
    normal_source_modified=False,candidate_applied=False,SPR_bitmap_files_verified=False,PCM_playback_verified=False,actual_new_sessions=0,actual_new_casts=0,GUI_verified=False,full_gameplay_matrix_verified=False,whole_goal_complete=False)
save(out/'evidence.json',dict(skill_development=record,original_broad_live=live,original_broad_session=67934))
work='當版44FEA738廣域原67934之Debug已實際執行並獨立讀回531過0敗21既有opt-in忽略，Release仍沿原程序建置／執行中；整批廣域及完整library未授予資格，產品與對應QA固定，不重啟。當版75DA69B0實際技能書另已核對74個ID，正常請求辨識35／缺39，辨識不是施法通過數。35個所需特效原始定義可讀；已開發11個有限特效原始定義守衛的隔離候選，型別檢查與11個原定義接受、各11個header／圖庫變造拒絕通過，未套正常產品，尚未驗SPR位圖／PCM播放或實際技能流程，其他24個方向／投射物／附層／其他語意待續。當版WORLD8sessions、旅館16sessions與控制器／正常模組音效各96sessions保留各自資格，同端點頻道契約失敗保留。先完成自行開發與自動化端到端後才人工驗收，開發期間不再請使用者登入；原預算／時限與相鄰唯讀、產物獨立不變，完整G01–G12未完成。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==record['source_manifest_sha256']
        assert x['active_workload_sessions']=={'broad_default':67934}
        x.update(current_native44FE_skill_development=record,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'updated_date'in x:x['updated_date']=date
        for item in x.get('items',[]):
            if item.get('id','').startswith('G05-'):item.update(state='current_book74_request35_missing39_isolated_finite_effect_candidate11',complete=False,current_skill_development_audit=record)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation=read(broad.parent/'current-continuation.json');continuation.update(latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),skill_development_audit=record)
save(broad.parent/'current-continuation.json',continuation)
print(json.dumps(dict(checkpoint=str(out),missing_requests=39,candidate_finite_definitions=11,original_broad_session=67934,original_broad_live=True,normal_modified=False),ensure_ascii=False))
