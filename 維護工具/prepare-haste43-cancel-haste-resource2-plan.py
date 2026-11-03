"""Rebind the exact reviewed Haste effect bodies after current library completion."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';T=R/'維護工具';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
w=read(R/'工作佇列.json');SOURCE='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D';assert w['current_product_identity']['source_manifest_sha256']==SOURCE and w['current_full_default_library_verified'] and not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
library=Path(w['current_native_catalog_library']['run']);completion=library/'completion-observation.json';assert sha(completion)==w['current_native_catalog_library']['completion_sha256'] and read(completion)['recorded_original_owned_processes_all_terminal']
for key in ['current_native_library_resource_optin','current_native_library_shared_resource_optin','current_library_motion_resource2']:
    proof=w[key];assert proof['source_manifest_sha256']==SOURCE and proof['qualified'] and sha(proof['qualification'])==proof['qualification_sha256']
prior=V/'native-current-library-haste-resource2/execution-plan-20261020-083528-461441/plan.json';p=read(prior);assert p['source_manifest_sha256'].startswith('A695')
for path_field,digest_field in [('test_bodies','test_bodies_sha256'),('screen_helpers','screen_helpers_sha256'),('helper_source','helper_source_sha256')]:assert sha(p[path_field])==p[digest_field]
for row in p['selected_tests']:assert sha(R/row['source'])==row['source_sha256'] and not row['requires_strict_legacy_TW']
inputs=library/'compiled-inputs.json';pins=read(inputs);assert len(pins)==1049 and all(sha(R/n)==h for n,h in pins.items())
runner=T/'run-haste43-cancel-haste-resource2.py';reader=T/'checkpoint-haste43-cancel-haste-resource2.py'
out=V/'native-current-library-haste-resource2'/('E0E6409E-plan-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
p.update(source_manifest_sha256=SOURCE,library=str(library),library_completion_sha256=sha(completion),original_source_inputs=str(inputs),original_source_inputs_sha256=sha(inputs),runner_sha256=sha(runner),reader=str(reader),reader_sha256=sha(reader),prior_reviewed_execution_plan=str(prior),prior_reviewed_execution_plan_sha256=sha(prior),native_skill43_enabled=False,native_skill43_request_source_implemented=True,normal_skill43_GUI_verified=False,prior_actual_resource_optins=83,prior_remaining_ignored_unexecuted=405,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
(out/'plan.json').write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(plan=str(out/'plan.json'),plan_sha256=sha(out/'plan.json'),tests_each=2,executed=False),ensure_ascii=False))
