"""Rebind the unchanged separate probe to actual current normal qualification."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);a=ap.parse_args();normal=a.normal.resolve(strict=True)
assert normal.is_relative_to(V/'native-installed-resource-profile-normal')
qpath=normal/'qualified/qualified.json';q=read(qpath);source=read(R/'工作佇列.json')['current_product_identity']['source_manifest_sha256']
assert source==q['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert q['recorded_owned_processes_all_terminal']and q['observed_terminal_exit_code']==0 and q['passed_per_profile']==19
for n,h in read(normal/'compiled-inputs.json').items():assert sha(R/n)==h
origin=T/'candidates/native-rental-service-same-endpoint';prep=read(origin/'preparation.json')
assert not prep['compiled']and not prep['runtime_overrides']and sha(origin/'probe.rs')==prep['probe_sha256']
base=Path(prep['base_probe']);assert sha(base)==prep['base_probe_sha256']
count=prep['original_prefix_preserved_bytes'];assert count==6218
assert base.read_bytes()[:count]==(origin/'probe.rs').read_bytes()[:count]
assert hashlib.sha256(base.read_bytes()[:count]).hexdigest().upper()==prep['original_prefix_sha256']
out=T/'candidates/native-installed-resource-profile-same-endpoint-A695C127';assert not out.exists();out.mkdir()
(out/'probe.rs').write_bytes((origin/'probe.rs').read_bytes());(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
prep.update(base_source_manifest_sha256=source,normal_qualification=str(qpath),normal_qualification_sha256=sha(qpath),
    prior_preparation=str(origin/'preparation.json'),prior_preparation_sha256=sha(origin/'preparation.json'),
    source_rebind_only=True,current_probe_bytes_unchanged_from_separate_case=True,compiled=False,tests_executed=False,actual_sessions=0,gameplay_qualified=False)
(out/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
e=V/'native-render-test-fixture-tooling'/('same-endpoint-candidate-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));e.mkdir(parents=True)
(e/'readback.json').write_text(json.dumps(dict(candidate=str(out),preparation_sha256=sha(out/'preparation.json'),probe_sha256=sha(out/'probe.rs'),source_manifest_sha256=source,compiled=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(candidate=str(out),readback=str(e/'readback.json'),compiled=False),ensure_ascii=False))
