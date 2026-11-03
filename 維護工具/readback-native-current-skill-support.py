"""Check current source/book matrix as recognition evidence, never cast evidence."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-current-skill-support')
ar=read(run/'archive-readback.json');assert sha(run/'raw.zip')==ar['archive_sha256']
with zipfile.ZipFile(run/'raw.zip') as z:
    assert set(z.namelist())==set(ar['files'])
    for name,value in ar['files'].items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==value
    matrix=json.loads(z.read('support-matrix.json').decode('utf-8'))
    assert sha(run/'support-matrix.json')==ar['files']['support-matrix.json']
    source=z.read('native_skills.rs').decode('utf-8-sig').replace('\r\n','\n')
    source=source[source.index('pub fn request('):source.index('pub fn reuse_millis(')]
    recognized=set()
    for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',source,re.M):recognized.update(map(int,re.findall(r'\d+',match.group(1))))
    assert sorted(recognized)==matrix.get('native_non_ground_request_ids',matrix['native_object_or_selector_request_ids'])
    self_tick=set(matrix.get('native_self_tick_request_ids',[]))
    assert self_tick in [set(),{60}] and recognized-self_tick==set(matrix['native_object_or_selector_request_ids'])
    ground=set(matrix['native_ground_request_ids']);assert ground=={58,63}
    learned=set();role_sets={}
    for profile,obs in matrix['observations'].items():
        frames=z.read(profile+'/frames.jsonl').decode('utf-8-sig').splitlines()
        for role,record in obs['roles'].items():
            report=json.loads(z.read(profile+'/'+role+'-arena-portal.json').decode('utf-8-sig'))
            ids=report['entered']['world']['learned_skills'];assert ids==record['learned_ids']==report['after_walk']['world']['learned_skills']
            if role in role_sets:assert role_sets[role]==set(ids)
            else:role_sets[role]=set(ids)
            learned.update(ids)
            for row in record['matching_wire_books']:
                actual=json.loads(frames[row['capture_LF_line']-1]);body=bytes.fromhex(actual['plain'])
                assert actual['direction']=='server_to_client' and body.hex()==row['payload_hex'] and body[0]==0x1e
                assert len(body)==body[1]+2 and body[1]<=28
                decoded=[i*8+j+1 for i,bits in enumerate(body[2:]) for j in range(8) if bits&(1<<j)]
                assert decoded==ids
            assert record['missing_request']==sorted(set(ids)-(recognized|ground))
            assert record['request_recognized']==sorted(set(ids)&(recognized|ground))
    missing=learned-(recognized|ground)
    assert len(learned)==74 and len(missing)==matrix['missing_unique_learned_requests'] and len(learned-missing)==matrix['request_recognized_learned_ids']
    assert (len(learned-missing),len(missing))==((23,51) if self_tick else (21,53))
    assert sorted(missing)==matrix['missing_ids']
    assert {row['skill_id'] for row in matrix['rows']}==learned and len(matrix['rows'])==74
    for row in matrix['rows']:
        assert row['request_recognized']==(row['skill_id'] in recognized|ground)
        assert not row['current_cast_success_inferred'] and not row['current_GUI_skill_selection_verified']
    pins=json.loads(z.read('compiled-inputs.json').decode('utf-8-sig'));assert len(pins)==1025
for name,value in pins.items():assert sha(ROOT/name)==value
assert sha(matrix['runtime_qualification'])==matrix['runtime_qualification_sha256']
assert sha(matrix['factory_presence_reference'])==matrix['factory_presence_reference_sha256']
if self_tick:
    vpath=Path(matrix['visibility_pair_qualification']);visibility=read(vpath)
    assert sha(vpath)==matrix['visibility_pair_qualification_sha256'] and visibility['source_manifest_sha256']==matrix['source_manifest_sha256']
    assert visibility['actual_client_sessions']==64 and visibility['accepted_casts']==16 and visibility['negative_cases']==12
q=dict(status=f'independently_read_back_current_74_learned_{len(learned-missing)}_recognized_{len(missing)}_missing_skill_requests',
    evidence=str(run/'support-matrix.json'),evidence_sha256=sha(run/'support-matrix.json'),
    source_manifest_sha256=matrix['source_manifest_sha256'],jar_sha256=matrix['jar_sha256'],
    source_files=1024,compiled_input_files=1025,observed_unique_learned_ids=74,request_recognized_learned_ids=len(learned-missing),
    missing_unique_learned_requests=len(missing),missing_ids=sorted(missing),
    missing_by_role={role:sorted(ids-(recognized|ground)) for role,ids in role_sets.items()},
    request_recognition_is_not_gameplay_acceptance=True,new_sessions_executed=0,new_casts_executed=0,
    all_formal_reachable_operations_audited=False,all_learned_skills_native_supported=False,
    GUI_verified=False,product_source_changed=False,whole_goal_complete=False,
    raw_archive_sha256=ar['archive_sha256'],raw_files=ar['files'],observed_utc=datetime.now(timezone.utc).isoformat())
assert not (run/'qualified.json').exists()
(run/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(run/'independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','observed_unique_learned_ids','request_recognized_learned_ids','missing_unique_learned_requests']},ensure_ascii=False))
