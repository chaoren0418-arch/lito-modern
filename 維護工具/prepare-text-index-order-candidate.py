"""Stage one cfg(test) reference update after an independent whole-index comparison."""
from pathlib import Path
from datetime import datetime,timezone
from collections import Counter
import hashlib,json,difflib,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';T=R/'維護工具'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,x:Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
w=read(R/'工作佇列.json');SOURCE='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
assert w['current_product_identity']['source_manifest_sha256']==SOURCE and not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
old=V/'native-installed-Text-reference/20261019-200910';new=V/'native-installed-Text-reference/20261020-122700'
a=read(old/'readback.json');b=read(new/'readback.json');assert a['oracle_assembly_hashes']==b['oracle_assembly_hashes']
x=a['results'][0];y=b['results'][0];assert x['file_hashes']['Text.idx']=='CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E' and y['file_hashes']['Text.idx']=='B85FAF2685CAB65357800368228ED3A5A96108E512CF883FA63AD2F74A44A91B'
assert {k:v for k,v in x.items() if k!='file_hashes'}=={k:v for k,v in y.items() if k!='file_hashes'}
assert x['file_hashes']['Text.pak']==y['file_hashes']['Text.pak']=='19BFE9D44C55F697D55CA23292509AEA56DBBC1F534427AFC740522081B7448C'
ar=read(old/'current-index.json');br=read(new/'current-index.json');tuples=lambda rows:Counter((r['name'],r['offset'],r['size']) for r in rows)
assert len(ar)==len(br)==37497 and tuples(ar)==tuples(br)
changes=[dict(record=i,before=x,after=y) for i,(x,y) in enumerate(zip(ar,br)) if x!=y]
assert len(changes)==192 and all(c['before']['name']==c['after']['name'] and c['before']['size']==c['after']['size'] for c in changes)
for name,digest in y['file_hashes'].items():assert sha(Path(y['root'])/name)==digest
run=Path(w['current_native_library_shared_resource_optin']['run']);c=read(run/'cohort.json');assert c['status']=='failed_preserved' and set(c['profiles'])=={'Debug'} and c['profiles']['Debug']['passed']==70 and c['profiles']['Debug']['failed']==1 and c['profiles']['Debug']['exit_code']==101
out=T/'candidates'/('text-index-order-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
name='LiTo381-Modern/src/modern_lito/resources.rs';path=R/name;base=path.read_text('utf-8-sig')
needle='''        let current_profile = format!("{:X}", Sha256::digest(&installed_index))
            == "CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E"'''
replacement='''        // The 2026-10-20 index only reorders duplicate-name records. The
        // independent C# reader verified the complete 37497-record multiset.
        // Accept only either exact observed index, with the same pinned pak.
        let current_profile = matches!(format!("{:X}", Sha256::digest(&installed_index)).as_str(),
            "CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E"
            | "B85FAF2685CAB65357800368228ED3A5A96108E512CF883FA63AD2F74A44A91B")'''
assert base.count(needle)==1;patched=base.replace(needle,replacement)
marker='    fn real_shared_text_matches_csharp_reference() {';assert base.index(needle)>base.index(marker)>base.index('#[cfg(test)]')
assert patched[:patched.index(marker)]==base[:base.index(marker)] and patched.replace(replacement,needle)==base
(out/'resources.rs').write_text(patched,encoding='utf-8',newline='\n');(out/'change.diff').write_text(''.join(difflib.unified_diff(base.splitlines(True),patched.splitlines(True),fromfile=name,tofile=name)),encoding='utf-8')
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);sources=read(normal/'source-inputs.json');inputs=read(normal/'compiled-inputs.json');assert len(sources)==1048 and len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
after=dict(sources);after[name]=sha(out/'resources.rs');save(out/'candidate-source-inputs.json',after)
proof=dict(prior_oracle=str(old/'readback.json'),prior_oracle_sha256=sha(old/'readback.json'),new_oracle=str(new/'readback.json'),new_oracle_sha256=sha(new/'readback.json'),complete_record_multiset_equal=True,records=37497,changed_ordered_rows=changes,all_changes_inside_same_name_groups=True,pak_unchanged=True,all_payload_and_ambiguity_assertions_preserved=True,Runtime_changed=False)
save(out/'comparison.json',proof)
files={'cohort.json':run/'cohort.json','tests.log':run/'Debug/tests.log','stderr.log':run/'Debug/stderr.log','runner.py':run/'runner.py','before-resources.rs':path,'resource-metadata-before.json':run/'resource-metadata-before.json','old-oracle.json':old/'readback.json','new-oracle.json':new/'readback.json','old-index.json':old/'current-index.json','new-index.json':new/'current-index.json'}
with zipfile.ZipFile(out/'failure-and-oracles.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'failure-and-oracles.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
record=dict(before_source_manifest_sha256=SOURCE,candidate_source_manifest_sha256=sha(out/'candidate-source-inputs.json'),before_files={name:sha(path)},after_files={name:sha(out/'resources.rs')},candidate_files={name:str(out/'resources.rs')},changed_files=[name],created_files=[],Runtime_changed=False,only_cfg_test_reference_changed=True,original_assertions_preserved=True,game_or_protocol_clocks_changed=False,source_applied=False,comparison=str(out/'comparison.json'),comparison_sha256=sha(out/'comparison.json'),failure_run=str(run),failure_archive_sha256=sha(out/'failure-and-oracles.zip'),proofs={k:dict(path=str(p),sha256=sha(p)) for k,p in [('independent_CSharp',new/'readback.json')]})
save(out/'candidate.json',record);(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(candidate=str(out/'candidate.json'),source=record['candidate_source_manifest_sha256'],Runtime_changed=False),ensure_ascii=False))
