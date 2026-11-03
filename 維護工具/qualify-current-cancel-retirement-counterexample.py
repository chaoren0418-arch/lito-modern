"""Qualify the original controller cancel defect, not a successful product run."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--repaired',action='store_true');a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-haste43-live-candidate')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']
assert c['scenario']==('cancel-repaired' if a.repaired else 'cancel-retirement') and c['cancel_counterexample_expected']==(not a.repaired) and c['status']=='executed_off_tree_haste43_live_candidate_both_profiles'
assert not w['active_workloads'] and not c['normal_product_modified'] and c['recorded_children_terminal']
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and all(sha(p)==h for p,h in c['generated_source_pins'].items())
assert sha(c['controller_original'])==c['controller_original_sha256'] and(run/'native_login.rs').read_text('utf-8-sig')==Path(c['controller_original']).read_text('utf-8-sig').partition(c['controller_cut_marker'])[0]
assert sha(c['fixture_origin'])==sha(run/'NativePeerHarness.java')==c['fixture_sha256']
assert [x['exit_code'] for x in c['process_history']]==([0]*7 if a.repaired else [0,0,0,1,0,0,1]) and not any(x.get('terminated_during_failure_cleanup') or x.get('exact_owned_timeout_cleanup') for x in c['process_history'])
if a.repaired:
    proposal=read(c['controller_cancel_repair_candidate']);assert not proposal['source_applied'] and all(sha(R/n)==h for n,h in proposal['before'].items()) and all(sha(p)==proposal['after'][n] for n,p in proposal['candidate_files'].items())
    base=Path(c['controller_base']);assert sha(base)==c['controller_base_sha256']
    text=base.read_text('utf-8-sig');start=text.index('    pub fn cancel(&mut self) {');end=text.index('    fn accept(&mut self, packet: Packet)',start);body=text[start:end]
    old='        if self.teleport.is_some()||self.recovery_character.is_some(){self.retire_teleport_scene();}\n';anchor='        self.teleport_recovery=self.teleport_recovery_candidate();\n';assert body.count(old)==body.count(anchor)==1
    expected=text[:start]+body.replace(old,'').replace(anchor,anchor+'        self.retire_teleport_scene();\n')+text[end:]
    assert Path(c['controller_original']).read_text('utf-8-sig')==expected
    counter=w['current_native_cancel_retirement_counterexample'];assert counter['qualified_counterexample'] and sha(counter['qualification'])==counter['qualification_sha256']
terms=list(run.glob('checkpoint-terminal-*/evidence.json'));assert len(terms)==1;t=read(terms[0]);assert t['observed_exit_code']==0 and t['recorded_original_identities_terminal'] and t['recorded_original_identities']==8
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
before_files=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before_files)==8 and all(sha(R/n)==h for n,h in before_files.items())
out=run/('qualified-cancel-repair' if a.repaired else 'qualified-cancel-counterexample');out.mkdir()
files={'cohort.json':run/'cohort.json','reader.py':Path(__file__),'terminal.json':terms[0],'terminal-raw.zip':Path(t['raw_archive'])};hashes={n:sha(p) for n,p in files.items()};assert sha(t['raw_archive'])==t['raw_archive_sha256']
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_off_tree_controller_cancel_repair_both_profiles' if a.repaired else 'independently_reproduced_original_controller_cancel_retains_world_both_profiles',source_manifest_sha256=source,qualified_counterexample=not a.repaired,qualified=a.repaired,run=str(run),profiles=checks,actual_formal_sessions=8,actual_formal_casts=2,original_controller_cancel_defect_reproduced=not a.repaired,normal_product_modified=False,candidate_applied=False,product_acceptance_granted=False,GUI_rendering_failure_claimed=False,GUI_verified=False,whole_goal_complete=False,original_identities_terminal=True,original_session=t['original_session'],terminal_tool_chunk=t['terminal_tool_chunk'],raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'counterexample.json').write_bytes(data(record));record.update(qualification=str(out/'counterexample.json'),qualification_sha256=sha(out/'counterexample.json'));(out/'evidence.json').write_bytes(data(record))
work='當版原生控制器取消退役缺口已雙版正式重現：各4sessions、成功加速後755已有限退役，取消兩傳輸均靜止且Closed，但快照仍保留map564／房間／角色／技能／速度。原控制器主體未改，兩probe預期exit1、兩server0，八身分終止；不推定GUI顯示失敗或扣除已核對資格。下一步在隔離候選保存傳送回復關聯後無條件退役舊場景，補原斷言回歸，正式雙版新批次重驗；未套用產品、不交人工。'
if a.repaired:work='取消退役隔離修補已正式雙版核對：各4sessions／一次加速，755有限退役後取消，兩傳輸靜止且Closed，世界／角色／技能／房間／服務清空，generation保留，八原身分終止且全部0。正常產品仍A695未套用；下一步新增取消回歸原版反例與候選兩版、保留全部原返回斷言，再決定套用及完整來源／GUI重驗，不交人工。'
staged={}
for n in before_files:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_native_cancel_retirement_formal_repair' if a.repaired else 'current_native_cancel_retirement_counterexample']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',whole_goal_complete=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before_files:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before_files.items())
assert all(sha(R/n)==h for n,h in before_files.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before_files,after_files={n:sha(R/n) for n in before_files},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(qualification=str(out/'counterexample.json'),original_cancel_defect_reproduced=not a.repaired,isolated_repair_qualified=a.repaired,profiles=list(checks)),ensure_ascii=False))
