"""Combine the two immutable-139B hotel cases without promoting their version."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
cases=[ROOT/'驗證紀錄/current/native-real-peers'/name for name in ['20261016-170537-478-rental-hotel-expiry','20261016-190144-533-rental-hotel-middle-expiry']]
rows=[];references={}
for case,session in zip(cases,[15175,26024]):
    p=case/'independent-natural-qualified/qualified.json';q=read(p)
    assert q['status']=='independently_qualified_immutable_Release_real_time_hotel_expiry'
    assert q['original_session_id']==session and q['observed_terminal_exit_code']==0
    assert q['exe_sha256']=='139B5247429B719B5F188CE16EC65FD5036A290E2FAFCDBCAEA3CD165E43C51F'
    assert q['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
    assert q['resource_profile']=='tw13081901-v1' and q['extended_item_details'] and q['exact_original_owned_processes_all_terminal']
    assert not q['clock_or_authority_injection'] and not q['GUI_verified'] and not q['whole_goal_complete']
    archive=p.parent/'raw.zip';assert sha(archive)==q['raw_archive_sha256']
    with zipfile.ZipFile(archive) as z:
        assert len(z.namelist())==len(q['raw_files']) and set(z.namelist())==set(q['raw_files'])
        for name,h in q['raw_files'].items():
            with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    r=read(case/'receipt.json');images=[]
    for role in ['a','b']:
        image=Path(r['clients'][role]['hotel']['natural_expiry']['render']['png']);images.append(dict(role=role,path=str(image),sha256=sha(image)))
    rows.append(dict(case=str(case),qualification=str(p),qualification_sha256=sha(p),session_id=session,maps=q['maps'],checks=q['checks'],actual_formal_sessions=q['actual_formal_sessions'],original_world_PNGs=q['original_world_PNGs'],expired_key_removal_instances=q['exact_expired_key_removals'],exact_S5A=q['exact_S5A'],exact_C08=q['exact_C08'],raw_archive_sha256=q['raw_archive_sha256'],visually_reviewed_return_images=images))
    references[str(session)+'/qualified.json']=p
assert sorted({m for row in rows for m in row['maps']})==[501,502,503]
assert sorted(check['keys'] for row in rows for check in row['checks'])==[1,2,4,8]
assert sorted(check['expected_seconds'] for row in rows for check in row['checks'])==[3600,3600,3600,7200]
out=ROOT/'驗證紀錄/current/native-hotel-current'/('natural-all-rooms-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
references['independent-combiner.py']=Path(__file__);hashes={name:sha(p) for name,p in references.items()}
with zipfile.ZipFile(out/'references.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,p in references.items():z.write(p,name)
with zipfile.ZipFile(out/'references.zip') as z:
    for name,h in hashes.items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
result=dict(status='independently_qualified_retained_Release139B_three_hotel_maps_real_time_expiry',cases=rows,maps=[501,502,503],actual_formal_sessions=8,LOGIN_sessions=4,GAME_sessions=4,real_client_lease_instances=4,original_world_PNGs=16,expired_key_removal_instances=15,exact_S5A=8,exact_C08=8,exe_sha256='139B5247429B719B5F188CE16EC65FD5036A290E2FAFCDBCAEA3CD165E43C51F',jar_sha256='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539',resource_profile='tw13081901-v1',extended_item_details=True,clock_or_authority_injection=False,GUI_verified=False,physical_input_verified=False,new_startup_EXE2904_result_claimed=False,manual_guest_exit_verified=False,complete_framebuffer_oracle=False,G11_gameplay_stability_accepted=False,whole_G06_complete=False,whole_goal_complete=False,references_archive_sha256=sha(out/'references.zip'),reference_files=hashes,qualified_utc=datetime.now(timezone.utc).isoformat())
assert sum(row['actual_formal_sessions'] for row in rows)==8 and sum(row['expired_key_removal_instances'] for row in rows)==15
(out/'qualified.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(qualification=str(out/'qualified.json'),maps=[501,502,503],sessions=8,PNGs=16,expired_key_removal_instances=15),ensure_ascii=False))
