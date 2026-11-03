"""Rebind the separate advertised-service probe to current normal library inputs.

Preserves the failed ordinary-channel contract and does not execute the probe.
"""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
encode=lambda value:(json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
SOURCE='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==SOURCE
normal=w['current_native_rental_service_entry_return_normal_qualification']
assert normal['source_manifest_sha256']==SOURCE and normal['qualification_granted']
assert sha(normal['qualification'])==normal['qualification_sha256']
original=T/'candidates/native33A3-same-endpoint-rental';prep=read(original/'preparation.json')
assert sha(prep['base_probe'])==prep['base_probe_sha256'] and sha(original/'probe.rs')==prep['probe_sha256']
assert not prep['compiled']and not prep['runtime_overrides']and not prep['old_common_channel_expectations_removed_from_original_test']
probe=(original/'probe.rs').read_bytes();base=Path(prep['base_probe']).read_bytes()
assert probe[:prep['original_prefix_preserved_bytes']]==base[:prep['original_prefix_preserved_bytes']]
candidate=T/'candidates/native-rental-service-same-endpoint';assert not candidate.exists();candidate.mkdir()
(candidate/'probe.rs').write_bytes(probe)
(candidate/'original-common-channel-branch.rs.txt').write_bytes((original/'original-common-channel-branch.rs.txt').read_bytes())
prep.update(status='current_normal_separate_advertised_rental_probe_prepared_not_compiled',
    base_source_manifest_sha256=SOURCE,normal_qualification=normal['qualification'],
    normal_qualification_sha256=normal['qualification_sha256'],
    prior_preparation=str(original/'preparation.json'),prior_preparation_sha256=sha(original/'preparation.json'),
    current_probe_bytes_unchanged_from_separate_case=True,
    required_next='After current matching-source full-library and active workloads terminate, compile each profile, run/independently qualify the real same-endpoint advertised rental route. Register exact original jobs before runtime. Ordinary-channel failure remains distinct; no GUI or nondefault-stat promotion.')
(candidate/'preparation.json').write_bytes(encode(prep))
out=V/'native-rental-service-entry-return-tooling'/('same-endpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
records={}
for prefix in ['compile','run','qualify']:
    origin=T/(prefix+'-native33A3-same-endpoint-rental.py')
    body=origin.read_text('utf-8-sig')
    body=body.replace('native33A3-library-default','native-rental-service-entry-return-library-default')
    body=body.replace('native33A3-same-endpoint-rental','native-rental-service-same-endpoint')
    body=body.replace('33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21',SOURCE)
    body=body.replace('1048','__COMPILED_INPUT_COUNT__').replace('1047','1048').replace('__COMPILED_INPUT_COUNT__','1049')
    if prefix=='compile':
        start=body.index("qp=next(normal.glob(")
        end=body.index('libraries={};native=set()',start)
        body=body[:start]+'''qp=normal/'qualified/qualified.json';q=read(qp)
assert q['status']=='independently_qualified_rental_service_entry_return_normal_related_both_profiles'
assert q['source_manifest_sha256']==source and q['passed_per_profile']==19 and q['recorded_owned_processes_all_terminal']and not q['source_overlays_used']
assert q['profiles'][a.profile]['normal']==c['profiles'][a.profile]['normal']
'''+body[end:]
    if prefix=='qualify':
        needle="outgoing=[(i,body)for i,row,body in packets if row['direction']=='client_to_server']"
        assert body.count(needle)==1
        body=body.replace(needle,needle+"\nsource_requests=[body for _,body in outgoing if body[0]==0xc8]\nassert len(source_requests)==4 and all(body[1:].rstrip(b'\\0').decode('cp950')=='測試乙'for body in source_requests), 'initial C8 confirmation or staged destination replay mismatch'\n")
    ast.parse(body)
    target=T/(prefix+'-native-rental-service-same-endpoint.py');assert not target.exists()
    target.write_text(body,encoding='utf-8',newline='\n')
    records[target.name]=dict(path=str(target),sha256=sha(target),origin=str(origin),origin_sha256=sha(origin))
record=dict(status='current_same_endpoint_advertised_rental_tools_prepared_not_executed',
    source_manifest_sha256=SOURCE,candidate=str(candidate/'preparation.json'),candidate_sha256=sha(candidate/'preparation.json'),
    tools=records,probe_bytes_unchanged=True,actual_sessions=0,compiled=False,GUI_verified=False,
    exact_job_checkpoint_preparation_remaining=True,ordinary_channel_failure_preserved=True,
    nondefault_stat_continuity_verified=False,whole_goal_complete=False)
(out/'same-endpoint-tools.json').write_bytes(encode(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tooling=str(out/'same-endpoint-tools.json'),compiled=False,actual_sessions=0),ensure_ascii=False))
