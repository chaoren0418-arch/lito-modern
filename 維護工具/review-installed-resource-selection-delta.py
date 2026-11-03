"""Bind reviewed single-fixture delta to a fresh current resource selection."""
from pathlib import Path
import argparse,json,hashlib,difflib
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('plan',type=Path);a=ap.parse_args();p=a.plan.resolve(strict=True)
assert p.is_relative_to(V/'native-installed-resource-profile-shared-resource-plan')
v=read(p);w=read(R/'工作佇列.json');assert v['source_manifest_sha256']==w['current_product_identity']['source_manifest_sha256']=='603C835908762CAF643F3ECA995039745DD6BE428FE9F533261B226BA2AD744A'
old=V/'native-installed-resource-profile-shared-resource-plan/20261019-210901-226357';prior=read(old/'review.json');assert prior['review_completed'] and prior['plan_sha256']==sha(old/'plan.json')
now=read(p.parent/'test-bodies.json');before=read(old/'test-bodies.json');helpers=read(p.parent/'shared-helper-bodies.json')
assert sha(p.parent/'test-bodies.json')==v['selected_bodies_sha256'] and sha(p.parent/'shared-helper-bodies.json')==v['shared_helpers_sha256']
assert set(now)==set(before) and len(now)==71
changed=[n for n in now if now[n]!=before[n]];assert changed==['modern_lito::world_scene::tests::real_shared_world_all_base_appearances_decode_and_authoritative_camera_moves']
assert helpers==read(old/'shared-helper-bodies.json')
# Reviewed precise change: two original oracle owners now explicitly resolved by ID.
c=read(R/'維護工具/candidates/native-shared-world-identity-fixture-20261019-212326-812135/candidate.json');assert c['test_only_fixture_repair'] and c['production_prefix_unchanged']
assert '[1002, 1001]'in now[changed[0]] and '[first_a.subframe, first_b.subframe], [12, 7]'in now[changed[0]]
for golden in ['9c5b493f6f09d276128f0d38b7f85c1c34d42f82f7eb0287bebb7e538e69ce6c','c1a75f153c4040db08cb3ea32d3732838e95798d90dda44a334e641153f92de1']:
 assert golden in now[changed[0]] and golden in before[changed[0]]
delta=''.join(difflib.unified_diff(before[changed[0]].splitlines(True),now[changed[0]].splitlines(True),fromfile='previous-reviewed-body',tofile='current-body'))
(p.parent/'reviewed-body-change.diff').write_text(delta,encoding='utf-8')
review=dict(prior,source_manifest_sha256=v['source_manifest_sha256'],plan_sha256=sha(p),selected_bodies_sha256=v['selected_bodies_sha256'],shared_helpers_sha256=v['shared_helpers_sha256'],status='reviewed_one_fixture_delta_and_verified_other_70_bodies_and_three_helpers_unchanged',prior_review=str(old/'review.json'),prior_review_sha256=sha(old/'review.json'),changed_bodies=changed,reviewed_delta_sha256=sha(p.parent/'reviewed-body-change.diff'),review_completed=True,actual_test_bodies_executed=False)
(p.parent/'review.json').write_text(json.dumps(review,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(plan=str(p),review_completed=True,unchanged_bodies=70,unchanged_helpers=3,changed_bodies=changed),ensure_ascii=False))
