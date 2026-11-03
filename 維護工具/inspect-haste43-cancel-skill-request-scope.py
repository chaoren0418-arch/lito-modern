"""Current source recognition only, using retained74-ID books; no new sessions."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
base_path=V/'native-current-skill-request-scope/20261020-044016-468694/scope.json';base=read(base_path);w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
assert sha(base['prior_observed_book'])==base['prior_observed_book_sha256']
src=R/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=src.read_text('utf-8-sig');request=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];ids=set()
for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',request,re.M):ids.update(map(int,re.findall(r'\d+',m.group(1))))
assert ids==set(base['current_object_or_self_request_ids'])|{43}
ground=text[text.index('pub fn ground_request('):text.index('pub fn in_native_screen(')];m=re.search(r'ensure!\(matches!\(skill,\s*([\d |]+)\)',ground);assert m;ground_ids=set(map(int,re.findall(r'\d+',m.group(1))));assert ground_ids=={58,63}
observed=set(base['current_recognized_from_prior_books'])|set(base['missing_request_ids']);assert len(observed)==74
recognized=observed&(ids|ground_ids);missing=observed-recognized;assert len(recognized)==38 and len(missing)==36 and missing==set(base['missing_request_ids'])-{43}
paths=V/'native-current-skill-request-scope/handler-paths-20261020-071630-321475/paths.json';p=read(paths);assert sha(p['original_disassembly'])==p['original_disassembly_sha256']
jar=R/'LiTo381-Modern/target/current/server-reference/formal-server-75DA69B0.jar';assert sha(jar)==p['formal_jar_sha256']
selected=[x for x in p['rows'] if x['skill_id'] in missing];assert {x['skill_id'] for x in selected}==missing
out=V/'native-current-skill-request-scope'/('E0E6409E-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
record=dict(status='current_source_request_recognition_only_against_retained_books',source_manifest_sha256=source,current_request_source=str(src),current_request_source_sha256=sha(src),current_request_body_sha256=hashlib.sha256(request.encode('utf-8')).hexdigest().upper(),prior_scope=str(base_path),prior_scope_sha256=sha(base_path),historical_book=base['prior_observed_book'],historical_book_sha256=base['prior_observed_book_sha256'],prior_observed_skill_IDs=74,current_object_or_self_request_ids=sorted(ids),current_ground_request_ids=sorted(ground_ids),recognized_from_prior_books=sorted(recognized),recognized_count=38,missing_request_ids=sorted(missing),missing_count=36,formal_handler_paths=str(paths),formal_handler_paths_sha256=sha(paths),remaining_formal_static_handler_rows=selected,current_book_observation_performed=False,new_sessions=0,new_casts=0,static_only=True,normal_runtime_or_GUI_skill_qualification_granted=False,whole_goal_complete=False)
(out/'scope.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
files={'scope.json':out/'scope.json','native_skills.rs':src,'inspector.py':Path(__file__),'prior-scope.json':base_path,'retained-formal-handler-paths.json':paths};hashes={n:sha(path) for n,path in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,path in files.items():z.write(path,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
(out/'readback.json').write_text(json.dumps(dict(source_manifest_sha256=source,raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'),static_only=True),ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(scope=str(out/'scope.json'),recognized=38,missing=36,static_only=True,new_sessions=0),ensure_ascii=False))
