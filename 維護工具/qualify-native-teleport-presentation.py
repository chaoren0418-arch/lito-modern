"""Read raw handoff authority, actual published-frame receipts and PNG pixels independently."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,struct,subprocess,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-presentation');r=read(run/'receipt.json')
assert r['status']=='executed_current_Presentation_handoff_diagnostic' and r['owned_process'] is None and r['actual_sessions']==6 and r['asset_metadata_unchanged']
assert all(j['exit_code']==0 for j in r['process_history']) and len(r['process_history'])==2
build=Path(r['compile']);b=read(build/'receipt.json');assert sha(build/'receipt.json')==r['compile_receipt_sha256'] and b['status']=='compiled_current_production_Presentation_test'
assert r['source_manifest_sha256']==b['source_manifest_sha256'] and r['profile']==b['profile']
assert sha(r['product'])==r['product_sha256']==b['product_sha256'] and sha(run/'probe.rs')==sha(build/'probe.rs')==b['probe_sha256']
with Path(r['product']).open('rb') as f:
    assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0' and struct.unpack('<H',f.read(2))[0]==0x8664
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for n,h in r['pins'].items():assert sha(n)==h,n
for v in b['libraries'].values():assert sha(v['path'])==v['sha256']
nq=read(b['normal_qualification']);assert sha(b['normal_qualification'])==b['normal_qualification_sha256'] and nq['source_manifest_sha256']==r['source_manifest_sha256'] and nq['passed']==31
assert sha(nq['normal']['path'])==nq['normal']['sha256']
g06=Path(r['prerequisite_G06']);gq=read(g06/'qualified/qualified.json');assert gq['actual_formal_sessions']==96 and gq['source_manifest_sha256']==r['source_manifest_sha256']
case=Path(r['case']);c=read(case/'receipt.json');runtime=Path(c['runtime']);comparison=read(case/'teleport-handoff-comparison.json')
assert c['inputs_unchanged'] and c['test_exit_code']==c['server_exit_code']==0 and c['test_binary_sha256']==r['product_sha256']==sha(c['immutable_test_binary'])
assert c['server']['actual_client_sessions']==6 and c['server']['memberships_empty'] and c['server']['authority_mutations']==0 and not c['server']['sql_access'] and not c['server']['state_persistent']
assert c['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')==c['jar_sha256']
for n,h in c['source_pins'].items():assert sha(case/'source-snapshot'/n)==h
assert c['harness_source_sha256']==r['pins'][c['harness_source']]
classes={p.relative_to(runtime).as_posix():sha(p) for p in (runtime/'oracle').rglob('*.class')};assert classes
assert read(run/'asset-metadata-before.json')==read(run/'asset-metadata-after.json')
for n,v in read(run/'asset-metadata-after.json').items():assert (Path(n).stat().st_size,Path(n).stat().st_mtime_ns)==(v['bytes'],v['mtime_ns'])
assert comparison['receipt_sha256']==sha(case/'receipt.json') and comparison['test_binary_sha256']==r['product_sha256'] and len(comparison['checks'])==2
assert comparison['duplicate_destination_character_requests']==0 and not comparison['gui_verified']
captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1 and sha(captures[0])==comparison['capture_sha256']
server_error=(case/'server.stderr.txt').read_bytes()
try:text=server_error.decode('utf-8-sig')
except UnicodeDecodeError:text=server_error.decode('cp950');assert text.encode('cp950')==server_error
assert 'Picked up JAVA_TOOL_OPTIONS: '+r['JAVA_TOOL_OPTIONS_for_owned_children_only'] in text
reports={role:read(runtime/(role+'-handoff.json')) for role in ['a','b']};pngs=[];proofs=[];retained_cues=0
for role,v in reports.items():
    assert sha(runtime/(role+'-handoff.json'))==sha(case/(role+'-handoff.json'))
    assert v['actual_Presentation_executed'] and not v['GUI_acceptance'] and not v['physical_audio_verified']
    assert v['new_epoch']==v['old_epoch']+1 and v['old_peer']!=v['new_peer'] and v['old_frame_rejected'] and not v['same_endpoint']
    before=v['before'];destination=v['destination'];battle=v['battle'];assert before['selected_character']==destination['selected_character']==battle['selected_character']
    assert not destination['world']['actors'] and destination['joined_room'] is None and destination['services'] is None and destination['rental_set'] is None
    assert battle['world']['generation']>before['world']['generation'] and battle['world']['map_id']==564
    retired=v['destination_retirement'];assert retired['view_quiescent'] and retired['frame_absent'] and retired['old_surface_rejected']
    assert retired['audio_deliveries_before']==retired['audio_deliveries_after'] and retired['old_cue_device_deliveries']==0
    assert len(retired['cue_outcomes'])==retired['retained_source_cues'];retained_cues+=retired['retained_source_cues']
    shutdown=read(runtime/(role+'-presentation-shutdown.json'));assert shutdown['client_quiescent'] and shutdown['Presentation_quiescent'] and shutdown['shutdown_error'] is None
    journal=read(runtime/(role+'-presentation-journal.json'));assert journal['error'] is None and not journal['GUI_acceptance']
    for row in v['presentation_trace']:
        s=row['state'];frame=s['frame']
        if frame is not None:assert frame['generation']==s['world_generation'] and s['world_ready'] and s['phase'] in ['battle_pending','world_ready']
    for label,snapshot in [('source',before),('battle',battle)]:
        picture=v[label+'_picture'];p=Path(picture['png']);receipt=picture['receipt'];assert not picture['GUI_acceptance']
        assert receipt['world_generation']==snapshot['world']['generation'] and receipt['map_id']==snapshot['world']['map_id'] and receipt['local_object_id']==snapshot['world']['status']['object_id']
        with Image.open(p) as im:assert im.size==(800,600) and im.mode=='RGBA' and hashlib.sha256(im.tobytes()).hexdigest().lower()==receipt['rgba_sha256'].lower()
        pngs.append(dict(path=str(p),sha256=sha(p),rgba_sha256=receipt['rgba_sha256'],role=role,stage=label,map_id=receipt['map_id'],generation=receipt['world_generation']))
    proofs.append(dict(role=role,epochs=[v['old_epoch'],v['new_epoch']],generations=[before['world']['generation'],battle['world']['generation']],source_map=before['world']['map_id'],destination_map=battle['world']['map_id'],published_frames_observed=sum(row['state']['frame'] is not None for row in v['presentation_trace']),old_surface_rejected=True,old_Presentation_quiescent_at_destination=True,retained_source_cues=retired['retained_source_cues']))
jobs=r['process_history']+[read(run/'runner-identity.json'),b['compiler']]+[dict(pid=j['pid'],creation_utc=j['start_utc']) for j in c['owned_processes']]
ids=','.join(str(j['pid']) for j in jobs);command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
output=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(output.decode('utf-8-sig')) if output.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for j in jobs:
        if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'recorded-process-CIM.json').write_bytes(output)
files={}
for family,root in [('driver',run),('case',case),('runtime',runtime),('compile',build)]:
    for p in root.rglob('*'):
        if p.is_file() and not p.is_relative_to(out) and p.suffix.lower() not in {'.exe','.class','.zip'}:files[family+'/'+p.relative_to(root).as_posix()]=p
pins={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_current_C02_Presentation_handoff_diagnostic_scope',run=str(run),profile=r['profile'],source_manifest_sha256=r['source_manifest_sha256'],original_session_id=a.session_id,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_product_sha256=r['product_sha256'],actual_sessions=6,peers=proofs,diagnostic_PNGs=pngs,actual_harness_class_pins=classes,actual_Presentation_executed=True,retained_source_cues=retained_cues,repeat_source_cue_no_delivery_observed=retained_cues>0,unplayed_source_audio_rejection_verified=False,recorded_owned_identities=len({(j['pid'],j['creation_utc']) for j in jobs}),recorded_owned_all_terminal=True,raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps({k:q[k] for k in ['status','profile','actual_sessions','actual_Presentation_executed','retained_source_cues']},ensure_ascii=False));print('QUALIFICATION='+str(out/'qualified.json'))
