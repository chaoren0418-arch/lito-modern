"""Rebind the original-duration matrix without promoting any earlier runtime."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
normal=w['current_native_installed_resource_profile_normal_qualification'];assert normal['source_manifest_sha256']==source and normal['qualification_granted']and sha(normal['qualification'])==normal['qualification_sha256']
prior=V/'native-room-form-label-tooling/natural-expiry-GUI-plan-20261020-014501-282698/plan.json';assert sha(prior)=='153F249337DF06C9E0BDB92100C46FC78056D1CAB4ACC7C719569B16E8311AB8'
plan=read(prior);assert len(plan['cases'])==4 and plan['prepared_only']and not plan['GUI_or_server_runtime_executed']
assert sha(plan['rooms_fixture'])==plan['rooms_fixture_sha256']
plan.update(source_manifest_sha256=source,prior_plan=str(prior),prior_plan_sha256=sha(prior),normal_qualification=normal['qualification'],normal_qualification_sha256=normal['qualification_sha256'],prior_runtime_qualifications_promoted=False,numeric_submit_key='Return',numeric_focus_ID=222,concurrent_owned_roles_required_for_8000_second_outer_budget=True,aggregate_capture_individual_socket_delivery_claimed=False,original_catalogue_durations_preserved=True)
out=V/'native-quantity-enter-failure-tooling'/('natural-expiry-GUI-plan-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'plan.json').write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(plan=str(out/'plan.json'),sha256=sha(out/'plan.json'),cases=4,runtime_executed=False),ensure_ascii=False))
