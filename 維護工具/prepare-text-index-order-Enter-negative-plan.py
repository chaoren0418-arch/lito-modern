"""Declare additional physical-key negative cases before the owned GUI inputs."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');assert c['source_manifest_sha256']==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert c['scenario']=='rental-hotel-return-reconnect'and c['status']=='normal_windows_running_for_Sky_inputs'and c['runtime_deadline_seconds']==1800
assert w['current_native_catalog_gui_active']['case']==str(case)and not(case/'Sky-events.json').exists()
plan=dict(status='declared_before_normal_GUI_inputs',source_manifest_sha256=c['source_manifest_sha256'],profile=c['profile'],case=str(case),declared_utc=datetime.now(timezone.utc).isoformat(),
    no_request=dict(role='a',press_label='hotel-no-input-enter',after_label='hotel-no-input-after',expected_new_C51=0),
    consumed_request=dict(role='a',press_label='hotel-quantity-repeat',after_label='hotel-repeat-after',expected_additional_C51=0,held_key_autorepeat_claimed=False),
    expired_request=dict(role='b',request_label='hotel-quantity-expiry-requested',value_label='hotel-expiry-quantity-value',wait_start_label='hotel-quantity-expiry-wait-start',wait_finished_label='hotel-quantity-expiry-wait-finished',press_label='hotel-expired-quantity-enter',after_label='hotel-expired-quantity-after',actual_wait_seconds_minimum=30,expected_old_request_C51=0,fresh_original_NPC_request_required_for_positive_case=True),
    positive_required_quantities=dict(a=8,b=1),positive_key='Return',expected_total_C51=2,protocol_or_gameplay_deadlines_changed=False,authority_injections=False,GUI_runtime_verified=False,whole_goal_complete=False)
target=case/'quantity-Enter-negative-plan.json';assert not target.exists();target.write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(case/'quantity-Enter-plan-preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(plan=str(target),sha256=hashlib.sha256(target.read_bytes()).hexdigest().upper(),runtime_verified=False),ensure_ascii=False))
