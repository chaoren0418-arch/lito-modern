"""Review 70 identical bodies plus the independently verified Text reference."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('plan',type=Path);a=ap.parse_args();path=a.plan.resolve(strict=True);assert path.is_relative_to(V/'native-installed-resource-profile-shared-resource-plan')
current=read(path);w=read(R/'工作佇列.json');source=current['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
prior_path=V/'native-installed-resource-profile-shared-resource-plan/20261020-014830-385893/plan.json';prior_review=prior_path.with_name('review.json');old=read(prior_path);review=read(prior_review)
assert sha(prior_review)=='2DBA8B7DE7296B8535C34D397BC9F1C2FE3BC35D1FACE14D3DF5614E12A15966'and review['review_completed']
assert review['plan_sha256']==sha(prior_path)and review['resources_read_only']and review['diagnostic_output_preservation_required']and not review['qualification_granted']
for key in ['excluded_legacy_tests','shared_helpers_sha256','explicit_operation_review']:assert current[key]==old[key],key
assert len(current['selected_tests'])==current['expected_tests']==71
for field,digest in [('selected_bodies','selected_bodies_sha256'),('shared_helpers','shared_helpers_sha256')]:assert sha(current[field])==current[digest]
assert sha(current['shared_helpers'])==sha(old['shared_helpers'])
old_bodies=read(old['selected_bodies']);bodies=read(current['selected_bodies']);TEXT='modern_lito::resources::tests::real_shared_text_matches_csharp_reference'
assert set(bodies)==set(old_bodies) and len(bodies)==71 and {n for n in bodies if bodies[n]!=old_bodies[n]}=={TEXT}
old_rows={x['test']:x for x in old['selected_tests']}
for row in current['selected_tests']:
 assert all(row[k]==old_rows[row['test']][k] for k in ['test','source','ignore_reason'])
 assert hashlib.sha256(bodies[row['test']].encode()).hexdigest().upper()==row['body_sha256']
application=w['current_native_installed_resource_profile_application'];candidate=read(application['candidate']);assert sha(application['candidate'])==application['candidate_sha256']
assert candidate['only_cfg_test_reference_changed'] and not candidate['Runtime_changed']
reviewed=read(application['candidate_review']);proof=read(reviewed['component_qualification']);assert sha(reviewed['component_qualification'])==reviewed['component_qualification_sha256'] and proof['qualified'] and proof['candidate_sha256']==application['candidate_sha256'] and proof['recorded_original_owned_processes_terminal']
with zipfile.ZipFile(Path(application['candidate_review']).parent/'before.zip') as z:before_source=z.read('LiTo381-Modern/src/modern_lito/resources.rs').decode('utf-8').replace('\r\n','\n')
assert before_source.count(old_bodies[TEXT])==1
assert before_source.replace(old_bodies[TEXT],bodies[TEXT])==(R/'LiTo381-Modern/src/modern_lito/resources.rs').read_text('utf-8-sig')
assert sha(R/'LiTo381-Modern/src/modern_lito/resources.rs')==candidate['after_files']['LiTo381-Modern/src/modern_lito/resources.rs']
assert all(line in bodies[TEXT] for line in old_bodies[TEXT].splitlines() if 'assert' in line)
assert len(read(current['shared_helpers']))==3
for row in current['selected_tests']:assert sha(R/row['source'])==row['source_sha256']
pins=read(Path(current['library'])/'compiled-inputs.json');assert len(pins)==1049 and all(sha(R/n)==h for n,h in pins.items())
record=dict(source_manifest_sha256=source,plan_sha256=sha(path),review_completed=True,review_kind='70 exact prior bodies plus one applied independently qualified Text index-order reference; same helpers, operations and assertions. Runtime still required.',prior_plan=str(prior_path),prior_plan_sha256=sha(prior_path),prior_review=str(prior_review),prior_review_sha256=sha(prior_review),all_71_test_bodies_identical_to_prior_review=False,unchanged_test_bodies=70,exact_reviewed_Text_reference_change=True,Text_component_qualification=reviewed['component_qualification'],Text_component_qualification_sha256=reviewed['component_qualification_sha256'],all_three_helpers_identical_to_prior_review=True,explicit_operations_identical=True,selected_bodies_sha256=current['selected_bodies_sha256'],shared_helpers_sha256=current['shared_helpers_sha256'],diagnostic_output_preservation_required=True,resources_read_only=True,compiler_or_runtime_started=False,qualification_granted=False,observed_utc=datetime.now(timezone.utc).isoformat())
target=path.with_name('review.json');assert not target.exists();target.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');path.with_name('reviewer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(review=str(target),review_sha256=sha(target),exact_bodies=71,shared_helpers=3,execution_qualification_granted=False),ensure_ascii=False))
