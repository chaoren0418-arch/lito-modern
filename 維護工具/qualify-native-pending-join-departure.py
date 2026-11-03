"""Independently read red reproduction and both candidate shadow-contract runs."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current/native-pending-join-departure'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
proposal=V/'candidate-20261018-054304-344367';p=read(proposal/'proposal.json')
assert p['status']=='prepared_unapplied_pending_join_peer_departure_ordering_fix'
assert sha(proposal/'proposal.zip')==p['proposal_archive_sha256']
with zipfile.ZipFile(proposal/'proposal.zip')as z:
    for n,h in {**p['sources'],'probe-addon.rs':p['probe_addon_sha256']}.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h==sha(proposal/n)
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
jobs=[];files={};profiles={};cases=None
for name,profile,session,chunk in [('20261018-054029-810877','red',4295,'68f1e0'),('20261018-054316-603936','Debug',46714,'5dca8c'),('20261018-054316-609055','Release',66937,'ef2800')]:
    d=V/name;r=read(d/'receipt.json');observation=read(d/'replay.json')
    assert r['normal_product_modified']is False and r['new_network_sessions']==0
    assert r['base_source_manifest_sha256']==p['base_source_manifest_sha256']
    assert all(j['exit_code']==0 for j in r['process_history'])and len(r['process_history'])==2
    jobs+=r['process_history']+[r['original_runner']]
    for n,h in read(d/'compiled-inputs.json').items():assert sha(ROOT/n)==h
    for pin in r['libraries'].values():assert sha(pin['path'])==pin['sha256']
    assert sha(r['product'])==r['product_sha256']
    pe=Path(r['product']).read_bytes();offset=int.from_bytes(pe[0x3c:0x40],'little');assert pe[:2]==b'MZ'and pe[offset:offset+4]==b'PE\0\0'and int.from_bytes(pe[offset+4:offset+6],'little')==0x8664
    assert sha(d/'native_login.rs')==r['diagnostic_controller_sha256']
    if profile=='red':
        assert r['status']=='executed_unchanged_controller_shadow_replay_red_reproduced'
        assert observation['status']=='unchanged_current_controller_early_D0_rejection_reproduced'
        assert observation['original_rejection_atomic']and observation['original_deadline_unchanged']
    else:
        assert r['status']=='executed_candidate_controller_shadow_contracts'and r['profile']==profile
        assert r['candidate']==str(proposal)and r['candidate_proposal_sha256']==sha(proposal/'proposal.json')
        assert r['candidate_sources']==p['sources']
        expected=(proposal/'native_login.rs').read_text('utf-8-sig')+'\n'+(proposal/'probe-addon.rs').read_text('utf-8-sig')
        assert (d/'native_login.rs').read_text('utf-8-sig')==expected
        assert (proposal/'native_room.rs').as_posix()in(d/'resolved_mod.rs').read_text('utf-8-sig')
        assert observation['status']=='candidate_pending_join_departure_contracts_passed'
        assert len(observation['cases'])==len(set(observation['cases']))==14
        if cases is None:cases=observation['cases']
        else:assert observation['cases']==cases
    assert observation['shadow_state_only']and observation['new_network_sessions']==0 and not observation['product_fix_applied']
    profiles[profile]=dict(run=str(d),receipt_sha256=sha(d/'receipt.json'),product_sha256=r['product_sha256'],original_session=session,observed_actual_exit_code=0,terminal_tool_chunk=chunk,case_count=1 if profile=='red'else 14)
    for f in d.rglob('*'):
        if f.is_file():files[profile+'/'+f.relative_to(d).as_posix()]=f
ids=','.join(str(j['pid'])for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]).decode('utf-8-sig');live=json.loads(raw)if raw.strip()else[];live=[live]if isinstance(live,dict)else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
for f in proposal.iterdir():
    if f.is_file():files['proposal/'+f.name]=f
files['qualifier.py']=Path(__file__)
out=V/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
pins={n:sha(f)for n,f in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,f in files.items():z.write(f,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
q=dict(status='independently_qualified_unchanged_red_and_both_candidate_pending_join_contracts',
    base_source_manifest_sha256=p['base_source_manifest_sha256'],proposal=str(proposal),proposal_sha256=sha(proposal/'proposal.json'),
    profiles=profiles,cases=cases,cases_each=14,recorded_original_identities=len(jobs),recorded_original_identities_all_terminal=True,
    actual_C02_revalidation_executed=False,normal_product_modified=False,normal_product_registered=False,
    shadow_state_only=True,new_network_sessions=0,GUI_verified=False,physical_audio_verified=False,
    full_G06_verified=False,whole_goal_complete=False,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',q);assert read(out/'qualified.json')==q
print(json.dumps(dict(qualification=str(out/'qualified.json'),cases_each=14,recorded_original_identities=len(jobs),normal_product_modified=False),ensure_ascii=False))
