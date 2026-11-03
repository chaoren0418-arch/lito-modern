"""Bind current learned books and request recognition without claiming casts."""
from pathlib import Path
from datetime import datetime,timezone
from collections import defaultdict
import argparse,csv,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--siege-cohort',type=Path,default=ROOT/'驗證紀錄/current/native-startup-siege563/20261016-221755-210184');ap.add_argument('--related-build',type=Path,default=ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254');ap.add_argument('--visibility-pair-qualification',type=Path);args=ap.parse_args()
family=args.siege_cohort.resolve(strict=True)
assert family.is_relative_to(ROOT/'驗證紀錄/current/native-startup-siege563')
qp=family/'qualified/qualified.json';q=read(qp)
assert q['status']=='independently_qualified_current_selected_map563_thirty_player_TW_extended'
assert q['observed_terminal_exit_code']==0 and q['source_files']==1024
assert set(q['profiles'])=={'Debug','Release'} and q['actual_formal_sessions']==120
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(formal)==q['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
build=args.related_build.resolve(strict=True)
assert build.is_relative_to(ROOT/'驗證紀錄/current')
assert sha(build/'source-inputs.json')==q['source_manifest_sha256']
pins=read(build/'compiled-inputs.json');assert len(pins)==1025
for name,value in pins.items():assert sha(ROOT/name)==value
src=ROOT/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=src.read_text(encoding='utf-8-sig')
request=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')]
recognized=set()
for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',request,re.M):recognized.update(map(int,re.findall(r'\d+',m.group(1))))
ground=text[text.index('pub fn ground_request('):text.index('pub fn in_native_screen(')]
m=re.search(r'ensure!\(matches!\(skill,\s*([\d |]+)\)',ground);assert m
ground_ids=set(map(int,re.findall(r'\d+',m.group(1))))
base_recognized={1,2,3,4,6,7,8,9,10,11,15,16,17,18,19,28,35,57,131}
visibility_path=args.visibility_pair_qualification.resolve(strict=True) if args.visibility_pair_qualification else None
if visibility_path:
    assert visibility_path.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
    visibility=read(visibility_path)
    if visibility['status']=='independently_qualified_current_normal_visibility_both_profiles':
        terminal=read(visibility_path.parent/'tool-terminal-observation.json')
        assert visibility['recorded_runtime_owned_identities']==64 and visibility['recorded_runtime_owned_all_terminal']
        assert visibility['upstream_normal_recorded_all_terminal'] and visibility['upstream_broad_recorded_all_terminal']
        assert visibility['actual_cast_requests']==28 and visibility['accepted_casts']==16 and visibility['negative_cases']==12
        normal_path=Path(visibility['normal_qualification']);normal=read(normal_path)
        assert sha(normal_path)==visibility['normal_qualification_sha256'] and normal['source_manifest_sha256']==q['source_manifest_sha256']
        broad_completion=Path(visibility['default_cohort'])/'completion-observation.json'
        assert sha(broad_completion)==visibility['default_completion_sha256']
        assert read(broad_completion)['source_manifest_sha256']==q['source_manifest_sha256']
    else:
        assert visibility['status']=='independently_qualified_native_visibility_settings_and_normal_entry_scope'
        terminal=read(visibility_path.parent.parent/'tool-terminal-observation.json')
    assert visibility['source_manifest_sha256']==q['source_manifest_sha256'] and visibility['actual_client_sessions']==64
    assert terminal['observed_terminal_exit_code']==0 and terminal['qualification_sha256']==sha(visibility_path)
    expected_recognized=base_recognized|{13,60};expected_missing=51
else:expected_recognized=base_recognized;expected_missing=53
assert recognized==expected_recognized and ground_ids=={58,63}
self_tick_ids={60} if visibility_path else set()
contract=ROOT/'驗證紀錄/current/native-current-skill-contract-audit'
factories=read(contract/'current-learned-factories.json');factory_by_id={x['skill_id']:x for x in factories['rows']}
fq=read(contract/'qualified.json');assert fq['archive_readback_verified']
assert fq['jar_sha256']==q['jar_sha256'] and sha(contract/'current-learned-factories.json')==fq['file_pins']['current-learned-factories.json']
assert sha(contract/'current-skill-contract-evidence.zip')==fq['archive_sha256']
with zipfile.ZipFile(contract/'current-skill-contract-evidence.zip') as z:
    assert set(z.namelist())==set(fq['file_pins'])
    for name,value in fq['file_pins'].items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==value
observations={};inputs={};roles=None
for profile,p in q['profiles'].items():
    case=Path(p['case']);receipt=read(case/'receipt.json')
    assert sha(case/'receipt.json')==q['raw_files'][profile+'/case/receipt.json']
    assert receipt['test_exit_code']==receipt['server_exit_code']==0 and receipt['jar_sha256']==q['jar_sha256']
    runtime=Path(receipt['runtime']);captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1
    capture=captures[0];capture_key=profile+'/runtime/'+capture.relative_to(runtime).as_posix()
    assert sha(capture)==q['raw_files'][capture_key]
    books=[]
    for number,line in enumerate(capture.read_text(encoding='utf-8-sig').splitlines(),1):
        row=json.loads(line)
        if row['direction']!='server_to_client' or not re.fullmatch('[0-9A-Fa-f ]*',row.get('plain','')):continue
        b=bytes.fromhex(row['plain'])
        if not b or b[0]!=0x1e:continue
        assert len(b)>=2 and b[1]<=28 and len(b)==b[1]+2 and (b[1]<28 or b[-1]&0xf0==0)
        ids=sorted(j*8+k+1 for j,bits in enumerate(b[2:]) for k in range(8) if bits&(1<<k))
        books.append(dict(capture_LF_line=number,note=row['note'],payload_hex=b.hex(),learned_ids=ids))
    this_roles={}
    for role in ['a','b']:
        report=case/(role+'-arena-portal.json');assert sha(report)==q['raw_files'][profile+'/case/'+report.name]
        d=read(report);learned=d['entered']['world']['learned_skills']
        assert learned==d['after_walk']['world']['learned_skills']
        matching=[v for v in books if v['learned_ids']==learned]
        assert any(v['note']=='S_ADD_SPELL selected-character' for v in matching)
        assert any(v['note']==f'S_ADD_SPELL learned-count={len(learned)}' for v in matching)
        this_roles[role]=dict(learned_ids=learned,request_recognized=sorted(set(learned)&(recognized|ground_ids)),
            missing_request=sorted(set(learned)-(recognized|ground_ids)),matching_wire_books=matching)
        inputs[profile+'/'+report.name]=report
    if roles is None:roles={k:v['learned_ids'] for k,v in this_roles.items()}
    else:assert roles=={k:v['learned_ids'] for k,v in this_roles.items()}
    observations[profile]=dict(case=str(case),receipt_sha256=sha(case/'receipt.json'),actual_test_sha256=p['actual_test_sha256'],
        capture=str(capture),capture_sha256=sha(capture),roles=this_roles)
    inputs[profile+'/receipt.json']=case/'receipt.json';inputs[profile+'/frames.jsonl']=capture
learned=set().union(*(set(v) for v in roles.values()));missing=learned-(recognized|ground_ids)
assert len(learned)==74 and len(missing)==expected_missing
assert learned==set(factory_by_id)
rows=[]
for skill in sorted(learned):
    f=factory_by_id[skill]
    rows.append(dict(skill_id=skill,observed_roles=[role for role,ids in roles.items() if skill in ids],
        request_recognized=skill in recognized|ground_ids,request_kind='ground' if skill in ground_ids else 'self_tick' if skill in self_tick_ids else 'selector_or_object' if skill in recognized else 'missing',
        formal_catalog_definitions=f['matches'],factory_errors=f['factory_errors'],
        catalog_presence_does_not_prove_runtime_cast=True,current_cast_success_inferred=False,current_GUI_skill_selection_verified=False))
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-current-skill-support'/stamp;out.mkdir(parents=True)
result=dict(status=f'current_source_and_actual_both_profile_books_confirm_{expected_missing}_missing_learned_requests',
    source_manifest_sha256=q['source_manifest_sha256'],source_files=1024,compiled_input_files=1025,jar_sha256=q['jar_sha256'],
    runtime_qualification=str(qp),runtime_qualification_sha256=sha(qp),runtime_scope='retained actual map563 integration test PEs; no new casts or sessions',
    native_request_source_sha256=sha(src),native_non_ground_request_ids=sorted(recognized),native_object_or_selector_request_ids=sorted(recognized-self_tick_ids),native_self_tick_request_ids=sorted(self_tick_ids),native_ground_request_ids=sorted(ground_ids),
    observed_unique_learned_ids=74,request_recognized_learned_ids=len(learned-missing),missing_unique_learned_requests=expected_missing,
    missing_ids=sorted(missing),observations=observations,rows=rows,
    factory_presence_reference=str(contract/'qualified.json'),factory_presence_reference_sha256=sha(contract/'qualified.json'),
    visibility_pair_qualification=str(visibility_path) if visibility_path else None,visibility_pair_qualification_sha256=sha(visibility_path) if visibility_path else None,
    old_catalog_BBC59308_or_old_request_table_9719_not_promoted=True,
    new_sessions_executed=0,new_casts_executed=0,request_recognition_is_not_gameplay_acceptance=True,
    all_formal_reachable_operations_audited=False,all_learned_skills_native_supported=False,GUI_verified=False,
    product_source_changed=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
save(out/'support-matrix.json',result)
with (out/'support-matrix.csv').open('w',encoding='utf-8-sig',newline='') as stream:
    writer=csv.writer(stream);writer.writerow(['skill_id','observed_roles','request_recognized','request_kind','formal_factories','gameplay_verified_by_this_audit'])
    for row in rows:writer.writerow([row['skill_id'],'/'.join(row['observed_roles']),row['request_recognized'],row['request_kind'],
        '/'.join(x['factory'] for x in row['formal_catalog_definitions']),False])
inputs.update({'auditor.py':Path(__file__),'compiled-inputs.json':build/'compiled-inputs.json','runtime-qualification.json':qp,
    'native_skills.rs':src,'native_skill_selection.rs':ROOT/'LiTo381-Modern/src/modern_lito/native_skill_selection.rs',
    'current-learned-factories.json':contract/'current-learned-factories.json','factory-qualification.json':contract/'qualified.json',
    'support-matrix.json':out/'support-matrix.json','support-matrix.csv':out/'support-matrix.csv'})
if visibility_path:inputs['visibility-pair-qualification.json']=visibility_path
hashes={name:sha(path) for name,path in inputs.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,path in inputs.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for name,value in hashes.items():
        with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==value
save(out/'archive-readback.json',dict(archive_sha256=sha(out/'raw.zip'),files=hashes))
for name,value in pins.items():assert sha(ROOT/name)==value
print(json.dumps(dict(output=str(out),observed_learned_ids=74,recognized_requests=len(learned-missing),missing_requests=expected_missing,new_sessions=0),ensure_ascii=False))
