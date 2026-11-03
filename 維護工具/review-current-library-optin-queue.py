"""Retain exact opt-in prerequisites; labels are search hints, never permission to run."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,collections,hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('library',type=Path);a=ap.parse_args();base=a.library.resolve(strict=True);assert base.is_relative_to(V/'native-installed-resource-profile-library-default')
c=read(base/'cohort.json');w=read(R/'工作佇列.json');assert c['source_manifest_sha256']==w['current_product_identity']['source_manifest_sha256']
pins=read(base/'compiled-inputs.json');assert len(pins)==1049 and all(sha(R/n)==h for n,h in pins.items())
audit_path=base/'Debug/inventory-audit/ignored-source-reasons.json';audit=read(audit_path);assert audit['mapped_unique']==488 and audit['mapped_ambiguous']==audit['unmapped']==0
prior_plan=V/'native-installed-resource-profile-shared-resource-plan/20261020-014830-385893/plan.json';old=read(prior_plan)
family71={x['test']for x in old['selected_tests']};assert len(family71)==71
family10={x['test']for x in audit['tests']if x['test'].startswith('modern_lito::native_disintegrate_tests::native77_probe::')};assert len(family10)==10 and not family10&family71
strict_legacy=set(old['excluded_legacy_tests']);assert len(strict_legacy)==5
rows=[];hints=collections.Counter()
for row in audit['tests']:
    item=row['candidates'][0];assert sha(R/item['source'])==pins[item['source']]==item['source_sha256']
    if row['test']in family10|family71:continue
    reason=item['reason']or'';lower=reason.lower();tags=[]
    for tag,tokens in [('operator_or_HWND',['operator only','interactive','native ui layout','native category ui']),('SQL_fixture',['mysql','sql']),('Java_peer_or_snapshot',['java','compiled peer','production main','modernmain']),('TLS_or_explicit_runtime',['tls','runtime ready','fixture ready','loopback']),('symlink_privilege',['symlink','developer mode']),('audio_device',['winmm','lito_native_audio_opt_in']),('explicit_resource_or_snapshot',['lito_shared_resource_root','pinned','frozen','snapshot','base sprite stream'])]:
        if any(token in lower for token in tokens):tags.append(tag);hints[tag]+=1
    rows.append(dict(test=row['test'],source=item['source'],source_sha256=item['source_sha256'],ignore_line=item['ignore_line'],exact_ignore_reason=reason,search_hints=tags,requires_strict_legacy_TW=row['test']in strict_legacy,execution_state='not_executed_by_this_review',prerequisites_satisfied=False,source_body_review_still_required=True))
assert len(rows)==407 and strict_legacy<=set(x['test']for x in rows)
out=V/'native-library-remaining-optin-review'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='exact_current_optin_remaining_queue_static_only',source_manifest_sha256=c['source_manifest_sha256'],library=str(base),audit=str(audit_path),audit_sha256=sha(audit_path),inventory=488,separately_reviewed_families=81,remaining_exact_tests=407,strict_legacy_TW_tests=5,overlapping_lexical_search_hint_counts=dict(hints),search_hints_are_not_precondition_validation_or_run_authorization=True,prior_71_plan_used_for_name_partition_only=str(prior_plan),prior_71_plan_sha256=sha(prior_plan),tests=rows,tests_executed=0,current_library_qualification_granted=False,whole_goal_complete=False)
(out/'review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'reviewer.py').write_bytes(Path(__file__).read_bytes())
(out/'readback.json').write_text(json.dumps(dict(review_sha256=sha(out/'review.json'),audit_sha256=sha(audit_path),source_manifest_sha256=c['source_manifest_sha256'],product_or_active_QA_modified=False,tests_executed=0),ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(review=str(out/'review.json'),remaining=407,strict_legacy=5,overlapping_search_hints=dict(hints),tests_executed=0),ensure_ascii=False))
