"""Qualify one resource-refused request per profile; no applied skill/GUI claim."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-candidate')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['scenario']=='mp-refused' and c['fixture_profile']=='heal19-refused' and not c['normal_product_modified'] and not c['candidate_applied']
assert c['status']=='executed_off_tree_haste43_live_candidate_both_profiles' and c['recorded_children_terminal'] and not c['owned_processes'] and not w['active_workloads']
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and all(sha(p)==h for p,h in c['generated_source_pins'].items())
assert sha(c['fixture_origin'])==sha(run/'NativePeerHarness.java')==c['fixture_sha256']
assert sha(c['controller_original'])==c['controller_original_sha256'] and(run/'native_login.rs').read_text('utf-8-sig')==Path(c['controller_original']).read_text('utf-8-sig').partition(c['controller_cut_marker'])[0]
for n in ['native_room','native_item_actions','native_ground_pickup','native_portal']:assert sha(run/(n+'.rs'))==sha(P/'src/modern_lito'/(n+'.rs'))
assert sha(c['jar'])==c['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371' and sha(c['catalog'])==c['catalog_sha256']
assert sha(c['candidate'])==c['candidate_sha256'];candidate=read(c['candidate']);assert all(sha(p)==candidate['after'][n] for n,p in candidate['candidate_files'].items())
positive=w['current_native_haste43_self_peer_formal75'];assert positive['qualified'] and sha(positive['qualification'])==positive['qualification_sha256'] and positive['source_manifest_sha256']==source and positive['actual_formal_casts']==4
terms=list(run.glob('checkpoint-terminal-*/evidence.json'));assert len(terms)==1;t=read(terms[0]);assert t['observed_exit_code']==0 and t['recorded_original_identities_terminal'] and t['scenario']=='mp-refused'
assert len(c['process_history'])==7 and all(x['exit_code']==0 and not x.get('terminated_during_failure_cleanup') and not x.get('exact_owned_timeout_cleanup') for x in c['process_history'])
checks={}
for profile in ['Debug','Release']:
    info=c['profiles'][profile];case=Path(info['case']);assert case==run/profile/'case' and sha(info['executable'])==info['executable_sha256'] and all(sha(p['path'])==p['sha256'] for p in info['extern'].values())
    result=read(case/'result.json');client=read(case/'client-result.json');assert result==info['server_result'] and client==info['client_result']
    assert result['actual_client_sessions']==4 and result['game_sessions']==2 and result['memberships_empty'] and result['authority_mutations']==result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==0
    assert not result['sql_access'] and not result['state_persistent'] and 'original learned book' in result['initial_fixture'] and 'max MP2' in result['initial_fixture']
    assert client['actual_casts']==0 and client['actual_requests']==1 and client['prediction_absent'] and client['pending_duplicate_refused']
    request=read(case/'mp-request.json');after=read(case/'mp-confirmed.json')
    assert request['before']==request['immediate']['world'] and request['before']['mp']==[2,2] and request['before']['status']['object_id']==3
    assert request['target'] is None and request['request_only_no_authority_prediction'] and request['immediate_duplicate_refused_without_new_tx']
    original_book=read(Path(positive['run'])/profile/'case/self-request.json')['before']['learned_skills']
    assert request['before']['learned_skills']==original_book and 43 in original_book
    assert after['elapsed_ms_since_request']>=2000 and after['feedback']=='no_response'
    assert after['caster']['skill_feedback']=='NoResponse' and after['caster']['world']['mp']==[2,2]
    assert after['caster']['world']['own_cast_packets']==request['before']['own_cast_packets']==0
    for role in ['caster','observer']:
        world=after[role]['world'];assert world['actors']['3']['speed']==0 and world['actors']['3']['speed_seconds']==0
        assert not any(e['graphics']==755 for e in world['effects'])
    frames_path=next((case/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in frames_path.read_text('utf-8-sig').splitlines()]
    assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
    packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN'];outgoing=[b for x,b in packets if x['direction']=='client_to_server'];incoming=[b for x,b in packets if x['direction']=='server_to_client']
    assert [b.hex().upper() for b in outgoing if b[0]==0x14]==['14050203000000']
    assert [b[1:].rstrip(b'\0').decode('cp950') for b in outgoing if b[0]==0xc8]==['測試乙']*4
    assert sorted(b[1:].rstrip(b'\0').decode('cp950') for b in outgoing if b[0]==0xc9)==sorted(['PeerFixture(測試乙)','PeerFixture2(測試乙)'])
    assert not any(b==bytes.fromhex('200300000013') or(len(b)==7 and b[0]==0x37 and int.from_bytes(b[5:7],'little')==755) or(len(b)==8 and b[0]==0x29 and b[1:5]==(3).to_bytes(4,'little') and b[5]!=0) for b in incoming)
    assert not any(b[0]==0x4d and int.from_bytes(b[3:5],'little')==2 and int.from_bytes(b[1:3],'little')!=2 for b in incoming)
    log=(case/'server.stdout.log').read_text('utf-8-sig');assert log.count('C_USE_SPELL Haste rejected state skill=43 mp=2 cost=40 cooldownNanos=0')==1
    assert 'C_USE_SPELL Haste caster=' not in log
    trace=[json.loads(l)['state'] for l in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
    mage=[x for x in trace if x['test_account']=='PeerFixture2' and x['phase']=='BATTLE'];assert mage and all(x['mp']==2 for x in mage)
    checks[profile]=dict(actual_formal_sessions=4,actual_requests=1,successful_casts=0,original_Mage_book_unchanged=True,initial_MP2_cost40=True,unchanged_MP=True,no_action19_effect755_speed1=True,original_2s_no_response_observed_ms=after['elapsed_ms_since_request'],no_prediction=True,pending_duplicate_no_extra_wire=True,frames=str(frames_path),frames_sha256=sha(frames_path))
rows=[c['pipeline'],*c['process_history']];ids=','.join(str(x['pid']) for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
obs=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert obs.returncode==0 and not obs.stderr
live=json.loads(obs.stdout.decode('utf-8-sig')) if obs.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
assert not any(x['ProcessId']==y['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(y['creation_utc'])).total_seconds())<.001 for x in live for y in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/'qualified-mp-refused';out.mkdir();files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and out not in p.parents and not any(part.startswith('checkpoint-') for part in p.relative_to(run).parts)}
files.update({'reader.py':Path(__file__),'terminal.json':terms[0],'candidate.json':Path(c['candidate']),'positive-qualification.json':Path(positive['qualification'])});hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_off_tree_haste43_low_MP_refusal_both_profiles',source_manifest_sha256=source,qualified=True,run=str(run),profiles=checks,actual_formal_sessions=8,actual_formal_requests=2,successful_casts=0,original_identities_terminal=True,original_session=t['original_session'],terminal_tool_chunk=t['terminal_tool_chunk'],observed_exit_code=0,normal_product_modified=False,candidate_applied=False,normal_product_qualification_granted=False,full_skill43_qualified=False,GUI_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'));(out/'evidence.json').write_bytes(data(record))
work='#43候選兩版低MP正式拒絕已核對：原fixture建立前Mage maxMP2、原書保留，每版僅1個C14，server cost40拒絕，MP2不變／無動作19／755／加速，原2秒NoResponse實等且重按不多送；8sessions／8身分終止，產品未套用。自用／對人正向另保留8sessions／4施放；下一步目標／狀態與取消／換圖／新owner生命週期及正常GUI，再決定套用與來源重驗，不交人工。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_native_haste43_low_MP_refusal']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_formal_sessions=8,requests=2,successful_casts=0,normal_product_modified=False),ensure_ascii=False))
