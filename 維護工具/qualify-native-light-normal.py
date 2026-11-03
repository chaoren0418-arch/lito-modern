"""Retain the unique-entry mains after scoped tests; no gameplay/GUI claim."""
from pathlib import Path
import hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
S=ROOT/'驗證紀錄/current/native-light-spell'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def archive(path,files):
    pins={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist())==set(pins)
        for n,h in pins.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
    return {'path':str(path),'sha256':sha(path),'file_pins':pins,'readback_verified':True}
assert not (S/'qualified.json').exists()
pipeline=read(S/'release-and-mains.json')
assert pipeline['status']=='completed_pending_normal_archive_qualification'
manifest=S/'repaired-source-inputs.json';pins=read(manifest)
assert len(pins)==pipeline['source_files']==970
assert sha(manifest)==pipeline['source_manifest_sha256']
for n,h in pins.items():assert sha(ROOT/n)==h,n
tests={p:read(S/(p+'-scoped-qualified.json')) for p in ['Debug','Release']}
for profile,expected in [('Debug',42),('Release',17)]:
    q=tests[profile]
    assert q['passed']==expected and q['failed']==q['ignored']==0
    assert q['source_manifest_sha256']==sha(manifest)
    assert q['archive_readback_verified'] and sha(S/(profile+'-scoped-evidence.zip'))==q['archive_sha256']
normal=S/'normal';normal.mkdir()
(normal/'source-inputs.json').write_bytes(manifest.read_bytes())
profiles={};files={}
for profile,record in pipeline['normal_mains'].items():
    p=Path(record['path']);assert sha(p)==record['sha256']
    raw=p.read_bytes();at=struct.unpack_from('<I',raw,60)[0]
    assert raw[at:at+4]==b'PE\0\0' and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b
    log=S/(profile+'-entry-build.log')
    assert sha(log)==record['entry_build_log_sha256'] and 'Finished `' in log.read_text(encoding='utf-8-sig')
    profiles[profile]={**record,'features':[],'unique_entry':True,'machine':'AMD64','optional_header':'PE32+','GUI_revalidated':False}
    files[profile+'/LiToModern.exe']=p
assert set(profiles)=={'Debug','Release'}
normal_archive=archive(normal/'normal-mains.zip',files)
n={'status':'qualified_unique_entry_normal_mains_with_light_request_authority_and_direct_audio_scoped_tests_only','source_files':970,'source_manifest_sha256':sha(manifest),'profiles':profiles,'features':[],'archive':normal_archive,'whole_goal_complete':False}
save(normal/'qualified.json',n)
files={name:S/name for name in ['release-and-mains.json','Debug-scoped-qualified.json','Release-scoped-qualified.json','repaired-source-inputs.json','Debug-entry-build.log','Release-entry-build.log','Debug-entry-console.txt','Release-entry-console.txt','source-qualified.json','149-selection-supplement-qualified.json']}
files.update({'tools/'+name:ROOT/'維護工具'/name for name in ['qualify-native-light-scoped.py','qualify-native-light-normal.py','run-native-light-release-and-mains.ps1','inspect-original-light-SPR.py']})
files['normal-qualification.json']=normal/'qualified.json'
raw_archive=archive(S/'validation-raw-evidence.zip',files)
q={'status':'qualified_light_request_authority_direct_audio_and_original_single_frame_scoped_tests_only','source_files':970,'source_manifest_sha256':sha(manifest),'Debug_passed':42,'Release_passed':17,'failed':0,'ignored':0,'normal_mains':{k:v['sha256'] for k,v in profiles.items()},'normal_qualification':'驗證紀錄/current/native-light-spell/normal/qualified.json','raw_archive':raw_archive,'Debug_test_archive_sha256':tests['Debug']['archive_sha256'],'Release_test_archive_sha256':tests['Release']['archive_sha256'],'actual_game_sessions':0,'light_actual_TCP_verified':False,'GUI_verified':False,'illumination_raster_verified':False,'natural_3600s_expiry_verified':False,'physical_audio_verified':False,'default90_verified':False,'full_library_verified':False,'whole_goal_complete':False}
save(S/'qualified.json',q)
print(json.dumps({k:q[k] for k in ['status','source_files','Debug_passed','Release_passed','normal_mains','actual_game_sessions','whole_goal_complete']}))
