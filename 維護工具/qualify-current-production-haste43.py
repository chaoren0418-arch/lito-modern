"""Qualify current production API casts, MP refusal or cancel cleanup; no GUI scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args();a.repaired=True
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-current') and a.session>0 and a.terminal_exit_code==0 and a.terminal_chunk not in ['','unknown','pending']
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert c['status']=='executed_current_production_haste43_both_profiles' and c['recorded_children_terminal'] and not c['owned_processes']
assert c['production_rlib_used'] and not c['production_source_overlays_used'] and not c['normal_product_modified']
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and all(sha(p)==h for p,h in c['generated_source_pins'].items())
normal=read(c['normal_qualification']);assert sha(c['normal_qualification'])==c['normal_qualification_sha256'] and normal['source_manifest_sha256']==source and normal['recorded_owned_processes_all_terminal'] and normal['passed_per_profile']==19
assert sha(c['fixture_origin'])==sha(run/'NativePeerHarness.java')==c['fixture_sha256']
assert sha(c['jar'])==c['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(c['catalog'])==c['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
original=(R/'維護工具/native-haste43-live-body.rs').read_text('utf-8-sig');expected=original.replace('candidate::','lito_modern::modern_lito::').replace('candidate_haste43_','current_haste43_').replace('"candidate label"','"current production label"')
assert (run/'live-body.rs').read_text('utf-8-sig')==expected and (run/'main.rs').read_text('utf-8-sig')=='include!(r#"'+str(run/'live-body.rs')+'"#);\n'
assert len(c['process_history'])==7 and all(p['exit_code']==0 and not p.get('terminated_during_failure_cleanup') and not p.get('exact_owned_timeout_cleanup') for p in c['process_history'])
for profile in ['Debug','Release']:
    info=c['profiles'][profile];assert all(sha(p['path'])==p['sha256'] for p in info['extern'].values())
    normal_run=Path(normal['cohort']);messages=[json.loads(l) for l in (normal_run/profile/'test-build/stdout.log').read_text('utf-8-sig').splitlines() if l.startswith('{')]
    produced={str(Path(f).resolve()) for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']=='lito_modern' for f in m['filenames'] if f.endswith('.rlib')}
    assert str(Path(info['extern']['lito_modern']['path']).resolve()) in produced
    result=read(Path(info['case'])/'result.json');assert result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==result['authority_mutations']==0 and not result['sql_access'] and not result['state_persistent']
positive=None
if c['scenario']!='self-peer':
    positive=w['current_native_haste43_production_self_peer_formal75'];assert positive['source_manifest_sha256']==source and positive['qualified'] and sha(positive['qualification'])==positive['qualification_sha256']
if c['scenario']=='self-peer':
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

elif c['scenario']=='mp-refused':
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

else:
    assert c['scenario']=='cancel-repaired'
    checks={}
    for profile in ['Debug','Release']:
        case=run/profile/'case';info=c['profiles'][profile];assert Path(info['case'])==case and sha(info['executable'])==info['executable_sha256']
        result=read(case/'result.json');assert result['actual_client_sessions']==4 and result['game_sessions']==2 and result['memberships_empty'] and result['authority_mutations']==0 and result['initial_fixture']=='default memory characters' and not result['sql_access'] and not result['state_persistent']
        if a.repaired:assert not(case/'error.json').exists()
        else:assert read(case/'error.json')['error']=='cancel retained previous world/room/skills after owned transport quiescence'
        client=read(case/'client-result.json');assert client['actual_casts']==client['actual_requests']==1 and client['finite755_retired_while_speed1']
        request=read(case/'self-request.json');confirmed=read(case/'self-confirmed.json');before=read(case/'finite-effect-before-cancel.json');after=read(case/'after-cancel.json')
        assert request['before']==request['immediate']['world'] and confirmed['mana_after']==[70,110]
        assert after['both_quiescent']
        for role in ['caster','observer']:
            s=after[role];assert s['phase']=='closed'
            if a.repaired:
                assert s['world']['map_id'] is None and not s['world']['actors'] and not s['world']['learned_skills'] and not s['world']['effects'] and s['world']['mp'] is None
                assert s['joined_room'] is None and s['selected_character'] is None and not s['game_connected'] and s['services'] is None and s['inventory']['owner'] is None
                assert s['world']['generation']==before[role]['world']['generation']
            else:
                assert s['world']['map_id']==564 and s['world']['actors'] and s['world']['learned_skills'] and s['joined_room']['id']==1
                assert s['world']['actors']['3']['speed']==1 and s['selected_character'] and s['game_connected']
            assert before[role]['world']['actors']['3']['speed']==1
        assert all(f['skill_effects']['active']==0 and f['skill_effects']['started']==1 for f in before['render_receipts'])
        frames=next((case/'captures').glob('*/frames.jsonl'));packets=[json.loads(l) for l in frames.read_text('utf-8-sig').splitlines()]
        assert sum(p['plain']=='MASKED_C_LOGIN' for p in packets)==4
        outgoing=[bytes.fromhex(p['plain']) for p in packets if p['plain']!='MASKED_C_LOGIN' and p['direction']=='client_to_server']
        assert [p.hex().upper() for p in outgoing if p[0]==0x14]==['14050203000000']
        assert not any(p[0]==0x10 for p in outgoing)
        checks[profile]=dict(actual_sessions=4,successful_casts=1,expected_probe_exit1=not a.repaired,server_exit0=True,transport_quiescent=True,closed_snapshot_retains_world_room_skills_speed=not a.repaired,closed_snapshot_retirement_qualified=a.repaired,finite755_retired_before_cancel=True,skill_count=len(after['caster']['world']['learned_skills']),actor_count=len(after['caster']['world']['actors']))

rows=[c['pipeline'],*c['process_history']];filters=' OR '.join('ProcessId='+str(p['pid']) for p in rows)
cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert probe.returncode==0 and not probe.stderr
live=json.loads(probe.stdout.decode('utf-8-sig')) if probe.stdout.strip() else[];live=[live] if isinstance(live,dict) else live
assert not any(x['ProcessId']==p['pid'] and abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(p['creation_utc'])).total_seconds())<.001 for x in live for p in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
active=w['active_workloads'];assert not active or w['active_workload_sessions']=={'production_haste43':a.session}
out=run/('qualified-'+c['scenario']);out.mkdir();(out/'owned-CIM.json').write_bytes(probe.stdout)
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not p.is_relative_to(out) and not any(s.startswith('checkpoint-') for s in p.relative_to(run).parts)};files.update({'reader.py':Path(__file__),'normal-qualification.json':Path(c['normal_qualification']),'owned-CIM.json':out/'owned-CIM.json'})
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert set(z.namelist())==set(hashes) and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
key={'self-peer':'current_native_haste43_production_self_peer_formal75','mp-refused':'current_native_haste43_production_low_MP_refusal','cancel-repaired':'current_native_cancel_retirement_production_formal75'}[c['scenario']]
record=dict(status='independently_qualified_current_production_haste43_'+c['scenario'],source_manifest_sha256=source,qualified=True,run=str(run),profiles=checks,actual_formal_sessions=8,actual_formal_casts={'self-peer':4,'mp-refused':0,'cancel-repaired':2}[c['scenario']],actual_formal_requests={'self-peer':4,'mp-refused':2,'cancel-repaired':2}[c['scenario']],production_rlib_linked_directly=True,production_source_overlays_used=False,original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=0,recorded_original_identities_terminal=True,recorded_original_identities=len(rows),GUI_verified=False,natural1200s_expiry_qualified=False,full_skill43_qualified=False,whole_goal_complete=False,raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'))
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'));(out/'evidence.json').write_bytes(data(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j[key]=record;j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',whole_goal_complete=False,goal_complete=False);staged[n]=data(j)
    else:staged[n]=p.read_bytes()
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(qualification=record['qualification'],actual_sessions=8,casts=record['actual_formal_casts'],GUI_verified=False),ensure_ascii=False))
