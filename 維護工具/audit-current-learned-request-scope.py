"""Compare current request arms to retained actual books; no new gameplay proof."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
prior=V/'native44FE-skill-book-audit/20261018-175440-043382/matrix.json';assert sha(prior)=='8ECD4C4A0A9B20B332E0D1704A71AD86B0FCE12202D33CE2EDD7A9805754392D'
book=read(prior);assert book['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
code=R/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=code.read_text('utf-8-sig');body=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];supported=set()
for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):supported.update(map(int,re.findall(r'\d+',m.group(1))))
ground=text[text.index('pub fn ground_request('):text.index('pub fn request(')];assert 'matches!(skill, 58 | 63)'in ground
ground_ids={58,63};observed={row['skill_id']for row in book['matrix']};recognized=observed&(supported|ground_ids);missing=observed-(supported|ground_ids)
assert len(observed)==book['observed_unique_learned_ids']==74 and len(recognized)==37 and len(missing)==37 and len(supported)==35
normal=w['current_native_installed_resource_profile_normal_qualification'];assert normal['source_manifest_sha256']==source and normal['qualification_granted']and sha(normal['qualification'])==normal['qualification_sha256']
inputs=read(Path(normal['run'])/'compiled-inputs.json');assert inputs[code.relative_to(R).as_posix()]==sha(code)
out=V/'native-current-skill-request-scope'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='current_request_recognition_against_retained_formal75_books_static_only',source_manifest_sha256=source,current_request_source=str(code),current_request_source_sha256=sha(code),current_request_body_sha256=hashlib.sha256(body.encode()).hexdigest().upper(),prior_observed_book=str(prior),prior_observed_book_sha256=sha(prior),prior_book_source_manifest_sha256=book['source_manifest_sha256'],prior_observed_skill_IDs=74,current_object_or_self_request_ids=sorted(supported),current_ground_request_ids=sorted(ground_ids),current_recognized_from_prior_books=sorted(recognized),missing_request_ids=sorted(missing),missing_count=37,missing_definitions=[row for row in book['matrix']if row['skill_id']in missing],current_book_observation_performed=False,new_sessions=0,new_casts=0,recognition_is_not_runtime_activation=True,generic_codec_examples_do_not_establish_selected_handler_request_shape=True,next_development='Read actual formal75 per-skill handler/target/range/reuse/cost/status contracts, then implement bounded requests, actual owner/rejection/lifecycle/render/audio behavior and GUI; never infer authority activation from a generic decoder accepting bytes.',GUI_verified=False,all74_gameplay_verified=False,whole_goal_complete=False)
(out/'scope.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'request-body.rs').write_text(body,encoding='utf-8',newline='\n');(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
(out/'readback.json').write_text(json.dumps(dict(scope_sha256=sha(out/'scope.json'),source_manifest_sha256=source,current_source_sha256=sha(code),product_or_active_QA_modified=False,new_sessions=0,new_casts=0),ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(scope=str(out/'scope.json'),historical_books=74,current_recognized=37,missing=37,new_sessions=0,new_casts=0),ensure_ascii=False))
