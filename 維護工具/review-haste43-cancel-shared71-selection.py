"""Carry forward a source-identical operation review, never runtime qualification."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('plan',type=Path);a=ap.parse_args();path=a.plan.resolve(strict=True);assert path.is_relative_to(V/'native-installed-resource-profile-shared-resource-plan')
current=read(path);w=read(R/'工作佇列.json');source=current['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
prior_path=V/'native-installed-resource-profile-shared-resource-plan/20261020-014830-385893/plan.json';prior_review=prior_path.with_name('review.json');old=read(prior_path);review=read(prior_review)
assert sha(prior_review)=='2DBA8B7DE7296B8535C34D397BC9F1C2FE3BC35D1FACE14D3DF5614E12A15966'and review['review_completed']
assert review['plan_sha256']==sha(prior_path)and review['resources_read_only']and review['diagnostic_output_preservation_required']and not review['qualification_granted']
for key in ['selected_tests','excluded_legacy_tests','selected_bodies_sha256','shared_helpers_sha256','explicit_operation_review']:assert current[key]==old[key],key
assert len(current['selected_tests'])==current['expected_tests']==71
for field,digest in [('selected_bodies','selected_bodies_sha256'),('shared_helpers','shared_helpers_sha256')]:assert sha(current[field])==sha(old[field])==current[digest]
assert len(read(current['shared_helpers']))==3
for row in current['selected_tests']:assert sha(R/row['source'])==row['source_sha256']
pins=read(Path(current['library'])/'compiled-inputs.json');assert len(pins)==1049 and all(sha(R/n)==h for n,h in pins.items())
record=dict(source_manifest_sha256=source,plan_sha256=sha(path),review_completed=True,review_kind='All 71 selected source pins, exact test/helper bodies and explicit operations identical to the reviewed D308 plan; new runtime still required.',prior_plan=str(prior_path),prior_plan_sha256=sha(prior_path),prior_review=str(prior_review),prior_review_sha256=sha(prior_review),all_71_test_bodies_identical_to_prior_review=True,all_three_helpers_identical_to_prior_review=True,explicit_operations_identical=True,selected_bodies_sha256=current['selected_bodies_sha256'],shared_helpers_sha256=current['shared_helpers_sha256'],diagnostic_output_preservation_required=True,resources_read_only=True,compiler_or_runtime_started=False,qualification_granted=False,observed_utc=datetime.now(timezone.utc).isoformat())
target=path.with_name('review.json');assert not target.exists();target.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');path.with_name('reviewer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(review=str(target),review_sha256=sha(target),exact_bodies=71,shared_helpers=3,execution_qualification_granted=False),ensure_ascii=False))
