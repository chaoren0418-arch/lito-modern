"""Combine the independently checked skill and cancel candidates for source apply."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
old=w['current_product_identity']['source_manifest_sha256'];assert old=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);pins=read(normal/'source-inputs.json');inputs=read(normal/'compiled-inputs.json');assert sha(normal/'source-inputs.json')==old and len(pins)==1048 and len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
required=['current_native_haste43_components','current_native_haste43_self_peer_formal75','current_native_haste43_low_MP_refusal','current_native_cancel_retirement_components','current_native_cancel_retirement_formal_repair']
proofs={}
for key in required:
    q=w[key];assert q['source_manifest_sha256']==old and q['qualified'] and sha(q['qualification'])==q['qualification_sha256'];proofs[key]=dict(path=q['qualification'],sha256=q['qualification_sha256'])
for key in ['current_native_haste43_components','current_native_haste43_self_peer_formal75','current_native_haste43_low_MP_refusal','current_native_cancel_retirement_components','current_native_cancel_retirement_formal_repair']:
    q=read(proofs[key]['path']);assert q.get('original_identities_terminal',q.get('recorded_original_identities_terminal'))
haste_path=Path(w['current_native_haste43_components']['candidate']);cancel_path=Path(w['current_native_cancel_retirement_components']['candidate']);haste=read(haste_path);cancel=read(cancel_path)
files={**haste['candidate_files'],**cancel['candidate_files']};before={**haste['before'],**cancel['before']};after={**haste['after'],**cancel['after']}
assert len(files)==len(before)==len(after)==6 and set(files)==set(before)==set(after)
assert all(sha(R/n)==before[n] and sha(p)==after[n] for n,p in files.items())
newpins=dict(pins);newpins.update(after);newinputs=dict(inputs);newinputs.update(after);new=hashlib.sha256(data(newpins)).hexdigest().upper();assert new!=old
assert set(newpins)==set(pins) and set(newinputs)==set(inputs)
for name in ['LiTo381-Modern/tests/native_skills.rs','LiTo381-Modern/tests/native_skill_selection.rs']:
    original=(R/name).read_text('utf-8-sig').splitlines();changed=Path(files[name]).read_text('utf-8-sig').splitlines();at=0
    for line in original:
        if 'assert' in line:
            while at<len(changed) and changed[at]!=line:at+=1
            assert at<len(changed),(name,line);at+=1
assert Path(files['LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs']).read_text('utf-8-sig').startswith((R/'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs').read_text('utf-8-sig'))
out=R/'維護工具/candidates'/('native-haste43-cancel-apply-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();diff=[]
for n,p in files.items():diff.extend(difflib.unified_diff((R/n).read_text('utf-8-sig').splitlines(True),Path(p).read_text('utf-8-sig').splitlines(True),fromfile=n+'/before',tofile=n+'/after'))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');(out/'source-inputs.json').write_bytes(data(newpins));(out/'compiled-inputs.json').write_bytes(data(newinputs));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'before-sources.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(R/n,n)
with zipfile.ZipFile(out/'before-sources.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='prepared_six_file_haste43_cancel_application_not_applied',before_source_manifest_sha256=old,candidate_source_manifest_sha256=new,source_files=1048,compiled_input_files=1049,changed_files=list(files),before_files=before,after_files=after,candidate_files=files,skill_candidate=str(haste_path),skill_candidate_sha256=sha(haste_path),cancel_candidate=str(cancel_path),cancel_candidate_sha256=sha(cancel_path),proofs=proofs,original_existing_assertions_preserved=True,added_library_tests=1,expected_default_library_passed=1975,expected_default_library_ignored=488,expected_broad_additional_tests=0,expected_current_recognized_skills=38,expected_remaining_of_original74=36,game_or_protocol_clocks_changed=False,source_overlays_used=False,formal_JAR_modified=False,source_applied=False,normal_products_built=False,GUI_verified=False,whole_goal_complete=False,review_sha256=sha(out/'review.diff'),before_archive_sha256=sha(out/'before-sources.zip'))
(out/'candidate.json').write_bytes(data(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),candidate_source_manifest_sha256=new,files=6,source_applied=False),ensure_ascii=False))
