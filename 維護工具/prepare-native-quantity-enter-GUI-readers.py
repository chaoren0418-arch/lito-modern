"""Prepare static GUI readers; do not replace QA used by an active workload."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
SOURCE='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
assert read(R/'工作佇列.json')['current_product_identity']['source_manifest_sha256']==SOURCE
out=V/'native-quantity-enter-failure-tooling'/('GUI-readers-prepared-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
prior=V/'native-room-form-label-tooling/GUI-positive-plan-20261020-013403-619696/plan.json'
assert sha(prior)=='371A24D1DDA0D92632CDD54D28FBF0939C91CA177A47E26F840AD08545E4BE22'
plan=read(prior);assert plan['source_manifest_sha256'].startswith('D308C3E2')and plan['prepared_only']and not plan['GUI_executed']
plan.update(source_manifest_sha256=SOURCE,prior_plan=str(prior),prior_plan_sha256=sha(prior),prior_runtime_qualification_promoted=False,room_protocol_labels_and_actions_unchanged=True)
(out/'room-positive-plan.json').write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
names=['qualify-native-room-form-labels-GUI-room-service.py','qualify-native-rental-return-reconnect-GUI-hotel-return.py'];before={}
for name in names:
    path=T/name;before[name]=sha(path);s=path.read_text('utf-8-sig');assert SOURCE in s
    if name==names[0]:
        old="plan_path=V/'native-room-form-label-tooling/GUI-positive-plan-20261020-013403-619696/plan.json'";assert s.count(old)==1
        s=s.replace(old,'plan_path=Path('+repr(str(out/'room-positive-plan.json'))+')').replace(sha(prior),sha(out/'room-positive-plan.json'))
    else:
        old="'hotel-talk','hotel-rent-request','hotel-quantity-submit','hotel-enter-request'";assert s.count(old)==1
        s=s.replace(old,"'hotel-talk','hotel-rent-request','hotel-enter-request'")
        anchor="    assert by_label['hotel-rent-choice']['action']['args']['value']=='租借房間'";assert s.count(anchor)==1
        extra="""    assert by_label['hotel-quantity-submit']['action']==dict(method='press_key',args={'key':'Return'})
    assert by_label['hotel-quantity-requested']['action']is None
    value=by_label['hotel-quantity-value'];assert value['action']['method']=='set_value'and value['action']['args']['value']==str(quantity)
    assert 'ID: 222'in value['focused_element']and '(disabled'not in value['focused_element']
    submit_at=next(i for i,row in enumerate(own)if row is by_label['hotel-quantity-submit'])
    assert submit_at>0 and own[submit_at-1]is value, 'Enter must follow the observed focused quantity field'
"""
        s=s.replace(anchor,extra+anchor)
        anchor="'hotel-quantity-requested','hotel-quantity-submit'";assert s.count(anchor)==1;s=s.replace(anchor,"'hotel-quantity-requested','hotel-quantity-value','hotel-quantity-submit'")
        anchor='earlier_quantity_requests=len(requests)-1,all_raw_observations_preserved=True,';assert s.count(anchor)==1
        s=s.replace(anchor,'quantity_submit_key="Return",quantity_focus_ID=222,quantity_wire_within_original_30_seconds=reply_seconds,earlier_quantity_requests=len(requests)-1,all_raw_observations_preserved=True,')
    ast.parse(s);(out/name).write_text(s,encoding='utf-8',newline='\n')
record=dict(status='static_GUI_readers_prepared_not_installed',source_manifest_sha256=SOURCE,prepared=str(out),before_reader_sha256=before,after_reader_sha256={n:sha(out/n)for n in names},room_plan=str(out/'room-positive-plan.json'),room_plan_sha256=sha(out/'room-positive-plan.json'),normal_clock_wire_inventory_authority_and_return_assertions_preserved=True,active_QA_modified=False,GUI_executed=False,qualification_granted=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'generator.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(record,ensure_ascii=False))
