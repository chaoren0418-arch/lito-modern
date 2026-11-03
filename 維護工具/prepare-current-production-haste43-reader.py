"""Reuse all prior actual-case assertions with new production-rlib provenance."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,textwrap
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
origins=['qualify-current-haste43-live-candidate.py','qualify-current-haste43-mp-refused.py','qualify-current-cancel-retirement-counterexample.py'];sections=[]
for name in origins:
    text=(T/name).read_text('utf-8-sig');start=text.index('checks={}');marker='before_files=' if name.startswith('qualify-current-cancel') else'rows=[c[';end=text.index(marker,start);section=text[start:end];ast.parse(section);sections.append(section)
header='''"""Qualify current production API casts, MP refusal or cancel cleanup; no GUI scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\\n').encode('utf-8')
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
assert (run/'live-body.rs').read_text('utf-8-sig')==expected and (run/'main.rs').read_text('utf-8-sig')=='include!(r#"'+str(run/'live-body.rs')+'"#);\\n'
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
'''
body=header+textwrap.indent(sections[0],'    ')+'\nelif c[\'scenario\']==\'mp-refused\':\n'+textwrap.indent(sections[1],'    ')+'\nelse:\n    assert c[\'scenario\']==\'cancel-repaired\'\n'+textwrap.indent(sections[2],'    ')
footer='''
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
'''
body+=footer;ast.parse(body);target=T/'qualify-current-production-haste43.py';assert not target.exists();target.write_text(body,encoding='utf-8',newline='\n')
out=V/'native-text-index-order-tooling'/('production43-reader-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
for name in origins:(out/name).write_bytes((T/name).read_bytes())
(out/'preparation.json').write_text(json.dumps(dict(reader=str(target),reader_sha256=sha(target),original_case_readers={n:sha(T/n) for n in origins},case_assertion_sections_copied_verbatim=True,provenance_changed_to_public_production_rlib=True,executed=False,whole_goal_complete=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(preparation=str(out/'preparation.json'),reader=str(target),executed=False),ensure_ascii=False))
