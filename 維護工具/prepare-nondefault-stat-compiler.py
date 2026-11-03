"""Prepare a compiler for the passive nondefault probe; execute no compiler here."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
s=(T/'compile-native-installed-resource-profile-same-endpoint.py').read_text('utf-8-sig')
s=s.replace("ap.add_argument('--library-cohort',type=Path,required=True);a=ap.parse_args()", "ap.add_argument('--library-cohort',type=Path,required=True);ap.add_argument('--candidate',type=Path,required=True);a=ap.parse_args()")
anchor="candidate=ROOT/'維護工具/candidates/native-installed-resource-profile-same-endpoint';prep=read(candidate/'preparation.json');prep_sha=sha(candidate/'preparation.json')";assert s.count(anchor)==1
s=s.replace(anchor,"""candidate=a.candidate.resolve(strict=True);assert candidate.is_relative_to(ROOT/'維護工具/candidates')
prep=read(candidate/'preparation.json');prep_sha=sha(candidate/'preparation.json')
assert prep['passive_snapshot_only']and not prep['handler_or_constructor_or_clocks_changed']and not prep['diagnostic_API_extension']
assert prep['passive_stat_fields']==14 and prep['wire_stat_projection_fields']==9 and not prep['original_five_wire_claimed']
resource_proofs={}
for key in ['current_native_library_resource_optin','current_native_library_shared_resource_optin']:
    proof=queue[key];assert proof['source_manifest_sha256']==source and proof['qualified']and sha(proof['qualification'])==proof['qualification_sha256']
    resource_proofs[key]=dict(qualification=proof['qualification'],qualification_sha256=proof['qualification_sha256'])
fixture_pins={n:sha(candidate/n)for n in prep['fixture_sources']}
assert fixture_pins==prep['fixture_sources']
""")
s=s.replace("    assert sha(candidate/'preparation.json')==prep_sha and sha(candidate/'probe.rs')==prep['probe_sha256']", "    assert sha(candidate/'preparation.json')==prep_sha and sha(candidate/'probe.rs')==prep['probe_sha256']\n    for n,h in fixture_pins.items():assert sha(candidate/n)==h\n    for proof in resource_proofs.values():assert sha(proof['qualification'])==proof['qualification_sha256']")
s=s.replace("for name in ['probe.rs','preparation.json']:", "for name in ['probe.rs','preparation.json',*fixture_pins]:")
s=s.replace('native-installed-resource-profile-same-endpoint','native-nondefault-stat-continuity')
s=s.replace('same-endpoint-rental.exe','nondefault-stat-continuity.exe').replace('native_same_endpoint_rental_probe','native_nondefault_stat_probe')
s=s.replace('normal_public_API_same_endpoint_rental_probe','normal_public_API_nondefault_stat_probe')
s=s.replace("candidate_preparation_sha256=prep_sha,libraries=pins", "candidate_preparation_sha256=prep_sha,fixture_source_pins=fixture_pins,main_class=prep['main_class'],resource_qualifications=resource_proofs,passive_server_stat_fields=14,wire_stat_projection_fields=9,stat_continuity_verified=False,libraries=pins")
ast.parse(s);p=T/'compile-native-nondefault-stat-continuity.py';assert not p.exists();p.write_text(s,encoding='utf-8',newline='\n')
out=V/'native-nondefault-stat-tooling'/('compiler-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes());(out/'receipt.json').write_text(json.dumps(dict(tool=str(p),sha256=sha(p),requires_current_full_library_and_both_resource_qualifications=True,compiler_started=False,runtime_started=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),compiler_started=False),ensure_ascii=False))
