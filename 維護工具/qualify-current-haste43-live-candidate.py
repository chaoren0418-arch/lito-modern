"""Independently read exact formal casts; never grant applied product/GUI scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-candidate')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['status']=='executed_off_tree_haste43_live_candidate_both_profiles' and c['recorded_children_terminal'] and not c['owned_processes']
assert not c['normal_product_modified'] and not c['candidate_applied'] and not w['active_workloads']
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and all(sha(p)==h for p,h in c['generated_source_pins'].items())
assert sha(c['jar'])==c['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert sha(c['fixture_origin'])==sha(run/'NativePeerHarness.java')==c['fixture_sha256']
assert sha(c['controller_original'])==c['controller_original_sha256']
assert (run/'native_login.rs').read_text('utf-8-sig')==Path(c['controller_original']).read_text('utf-8-sig').partition(c['controller_cut_marker'])[0]
for name in ['native_room','native_item_actions','native_ground_pickup','native_portal']:assert sha(run/(name+'.rs'))==sha(P/'src/modern_lito'/(name+'.rs'))
candidate=read(c['candidate']);assert sha(c['candidate'])==c['candidate_sha256']
assert all(sha(p)==candidate['after'][n] for n,p in candidate['candidate_files'].items())
assert sha(c['component_qualification'])==c['component_qualification_sha256'];cq=read(c['component_qualification']);assert cq['qualified'] and cq['candidate_tests_each']==20 and cq['recorded_original_identities_terminal']
terms=list(run.glob('checkpoint-terminal-*/evidence.json'));assert len(terms)==1;terminal=read(terms[0]);assert terminal['observed_exit_code']==0 and terminal['recorded_original_identities_terminal']
assert terminal['terminal_tool_chunk']=='68b1cd' and terminal['original_session']==70824
assert len(c['process_history'])==7 and all(x['exit_code']==0 and not x.get('terminated_during_failure_cleanup') and not x.get('exact_owned_timeout_cleanup') for x in c['process_history'])
checks={};actual_casts=actual_sessions=0
for profile in ['Debug','Release']:
    info=c['profiles'][profile];case=Path(info['case']);assert case==run/profile/'case' and sha(info['executable'])==info['executable_sha256']
    assert all(sha(x['path'])==x['sha256'] for x in info['extern'].values())
    result=read(case/'result.json');client=read(case/'client-result.json')
    assert result==info['server_result'] and client==info['client_result']
    assert result['actual_client_sessions']==4 and result['game_sessions']==2 and result['memberships_empty'] and result['authority_mutations']==result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==0
    assert result['initial_fixture']=='default memory characters' and not result['sql_access'] and not result['state_persistent']
    assert client['actual_casts']==2 and client['prediction_absent_twice'] and client['pending_duplicate_refused_twice'] and client['original_Mage_book_retained']
    assert not(case/'error.json').exists()
    for label,target,mp_before,mp_after in [('self',3,110,70),('peer',1,70,30)]:
        request=read(case/(label+'-request.json'));confirmed=read(case/(label+'-confirmed.json'))
        assert request['target']==(None if label=='self' else 1) and request['request_only_no_authority_prediction'] and request['immediate_duplicate_refused_without_new_tx']
        assert request['before']==request['immediate']['world'] and request['before']['mp']==[mp_before,110] and 43 in request['before']['learned_skills']
        assert request['before']['status']['object_id']==3
        assert confirmed['mana_before']==mp_before and confirmed['mana_after']==[mp_after,110] and confirmed['target']==target
        for key,own in [('caster',3),('observer',1)]:
            world=confirmed[key]['world'];actor=world['actors'][str(target)]
            assert actor['speed']==1 and actor['speed_seconds']==(1200 if target==own else 0)
            assert any(e['graphics']==755 and e['target'][0]==target for e in world['effects'] if e['target'] is not None)
            assert confirmed[key]['selected_source']=='測試乙'
    frames_path=next((case/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in frames_path.read_text('utf-8-sig').splitlines()]
    assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
    packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
    outgoing=[b for x,b in packets if x['direction']=='client_to_server'];incoming=[b for x,b in packets if x['direction']=='server_to_client']
    assert [b.hex().upper() for b in outgoing if b[0]==0x14]==['14050203000000','14050201000000']
    assert [b[1:].rstrip(b'\0').decode('cp950') for b in outgoing if b[0]==0xc8]==['測試乙']*4
    assert sorted(b[1:].rstrip(b'\0').decode('cp950') for b in outgoing if b[0]==0xc9)==sorted(['PeerFixture(測試乙)','PeerFixture2(測試乙)'])
    assert all(b==b'\xca\0' for b in outgoing if b[0]==0xca)
    assert not any(b[0]==0x10 for b in outgoing)
    for target in [3,1]:
        assert incoming.count(b'\x37'+target.to_bytes(4,'little')+(755).to_bytes(2,'little'))==2
        for seconds in [0,1200]:assert incoming.count(b'\x29'+target.to_bytes(4,'little')+b'\x01'+seconds.to_bytes(2,'little'))==1
    # Formal catalogue action for Haste is19; exactly two broadcasts to two peers.
    assert incoming.count(bytes.fromhex('200300000013'))==4
    for mp in [70,30]:assert incoming.count(b'\x4d'+mp.to_bytes(2,'little')+(110).to_bytes(2,'little'))==1
    log=(case/'server.stdout.log').read_text('utf-8-sig')
    logs=re.findall(r'C_USE_SPELL Haste caster=TestMage phase=BATTLE target=(\d+) player-target=true companion-target=false world-monster-target=false npc-applied=false resulting-speed=1 duration=1200 mp-cost=40 haste-helm=false mp=(\d+)/110',log)
    assert logs==[('3','70'),('1','30')]
    trace=[json.loads(l)['state'] for l in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
    mage=[x for x in trace if x['test_account']=='PeerFixture2' and x['phase']=='BATTLE'];assert all(any(x['mp']==mp for x in mage) for mp in [110,70,30])
    checks[profile]=dict(actual_formal_sessions=4,actual_casts=2,source='測試乙',learned43_original_book=True,exact_C14_self_and_peer=True,exact_action19_effect755_speed29_broadcasts=True,MP=[110,70,30],per_cast_cost40=True,duplicate_pending_no_extra_wire=True,request_world_not_predicted=True,original_processes_exit0=True,frames=str(frames_path),frames_sha256=sha(frames_path))
    actual_casts+=2;actual_sessions+=4
rows=[c['pipeline'],*c['process_history']];ids=','.join(str(x['pid']) for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
obs=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert obs.returncode==0 and not obs.stderr
live=json.loads(obs.stdout.decode('utf-8-sig')) if obs.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
assert not any(x['ProcessId']==y['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(y['creation_utc'])).total_seconds())<.001 for x in live for y in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/'qualified-self-peer-formal75';out.mkdir();files={}
for path in run.rglob('*'):
    if path.is_file() and out not in path.parents and not any(p.startswith('checkpoint-') for p in path.relative_to(run).parts):files[path.relative_to(run).as_posix()]=path
files.update({'reader.py':Path(__file__),'terminal.json':terms[0],'candidate.json':Path(c['candidate'])})
prior=[]
for failed in ['20261020-112106-101614','20261020-112145-821160','20261020-112231-512077']:
    original=V/'native-haste43-live-candidate'/failed;proof=next(original.glob('checkpoint-terminal-*/evidence.json'));j=read(proof);assert j['observed_exit_code']==1 and j['recorded_original_identities_terminal'] and sha(j['raw_archive'])==j['raw_archive_sha256']
    prior.append(str(proof));files['prior-tooling-failures/'+failed+'/evidence.json']=proof;files['prior-tooling-failures/'+failed+'/raw.zip']=Path(j['raw_archive'])
hashes={n:sha(p) for n,p in files.items()}
reader_rejections=[]
for proof in run.glob('reader-rejection-*/evidence.json'):
    j=read(proof);assert j['source_manifest_sha256']==source and j['reader_exit_code']==1 and sha(proof.with_name('raw.zip'))==j['raw_archive_sha256']
    reader_rejections.append(str(proof))
    for path in [proof,proof.with_name('raw.zip')]:
        name=path.relative_to(run).as_posix();files[name]=path;hashes[name]=sha(path)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_off_tree_haste43_self_peer_formal75_both_profiles',source_manifest_sha256=source,qualified=True,run=str(run),candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],profiles=checks,actual_formal_sessions=actual_sessions,actual_formal_casts=actual_casts,original_identities_terminal=True,original_session=70824,terminal_tool_chunk='68b1cd',observed_exit_code=0,prior_tooling_failures_preserved=prior,normal_product_modified=False,candidate_applied=False,normal_product_qualification_granted=False,full_skill43_qualified=False,MP_refusal_qualified=False,natural1200s_expiry_qualified=False,GUI_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
record['prior_reader_rejections_preserved']=reader_rejections
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'));(out/'evidence.json').write_bytes(data(record));(out/'owned-CIM.json').write_bytes(obs.stdout)
work='#43隔離候選Debug／Release正式75各自用與對人兩次、共8sessions／4施放已核對：原Mage書、C14、動作19／755／S29雙方通知、個別1200／0時數、MP110→70→30、重按不多送及不預測。候選未套用產品；下一步低MP拒絕、目標／狀態／生命週期及正常GUI，再決定套用與重驗。已通過當版完整library／85資源及GUI三正向與EOF兩版保留；完整G01–G12未完成，不交人工。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_native_haste43_self_peer_formal75']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(qualification=str(out/'qualified.json'),actual_formal_sessions=actual_sessions,actual_formal_casts=actual_casts,normal_product_modified=False,GUI_verified=False),ensure_ascii=False))
