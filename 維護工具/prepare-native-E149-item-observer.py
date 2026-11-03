"""Prepare isolated shared-init/native-item observer revision; no application run."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,struct,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
prior=Path(w['current_E149_integrated_fixture_development']['candidate']);old=read(prior)
assert sha(prior)==w['current_E149_integrated_fixture_development']['candidate_sha256']
assert len(old['candidate_files'])==4 and all(sha(prior.parent/n)==h for n,h in old['candidate_files'].items())
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==sha(ref['cached_reference'])==old['formal_target_sha256']
closure=V/'native-E149-resource-initialization-closure/20261023-215833-041052';inspection=read(closure/'inspection.json')
assert inspection['formal_target_sha256']==ref['formal_sha256'] and all(sha(n)==h for n,h in inspection['output_files'].items())
shared=(closure/'SharedClientResources.stdout.txt').read_text('utf-8-sig')
assert 'Method initializeProfile:()Z' in shared and 'Method loadDurations:()Ljava/util/Map;' in shared
assert 'int 60303' in shared and 'Common catalog SHA-256 mismatch' in shared
catalog=P/'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(catalog)=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert struct.unpack_from('<IIII',catalog.read_bytes())==(827479116,1,30000,285)
with zipfile.ZipFile(ref['cached_reference']) as z:
 blob=z.read('battle/spr-duration-tw13081901.csv')
 lines=[line.split(',') for line in blob.decode('utf-8-sig').splitlines() if line and not line.startswith('#')]
 rows=[tuple(map(int,line)) for line in lines]
 assert len(rows)==60303 and len({(sprite,action) for sprite,action,_ in rows})==60303
 assert all(0<=sprite<30285 and 0<=action<100 and duration>=0 for sprite,action,duration in rows)
resource=dict(embedded_duration_resource='battle/spr-duration-tw13081901.csv',embedded_duration_sha256=hashlib.sha256(blob).hexdigest().upper(),
 rows=60303,catalog=str(catalog),catalog_sha256=sha(catalog),catalog_header=[827479116,1,30000,285],
 initialize_body='prints selected profile; JVM clinit first validates catalog and freezes durations',
 SQL_or_network_calls_in_SharedClientResources=False,original_initializer_invoked=False)
after={n:(prior.parent/n).read_bytes() for n in old['candidate_files']}
observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
text=after[observer].decode('utf-8-sig');start=text.index('    private static Object field(');end=text.index('    private static LinkedHashMap',start)
fragment=T/'templates/NativeRecoveryItemFields.fragment.java'
text=text[:start]+fragment.read_text('utf-8')+text[end:]
before='if(inventory==null)return object("exists",false,"rows",List.of(),"stable_observed",true);'
assert text.count(before)==1;text=text.replace(before,'if(inventory==null)return object("exists",false,"rows",List.of(),"native_attributes",List.of(),"stable_observed",true);')
before='var rows=new ArrayList<Map<String,Object>>();boolean stable=true;'
assert text.count(before)==1;text=text.replace(before,'var rows=new ArrayList<Map<String,Object>>();var nativeRows=new ArrayList<Map<String,Object>>();boolean stable=true;')
before='var first=item(entry);var second=item(entry);stable&=first.equals(second);rows.add(first);'
assert text.count(before)==1
text=text.replace(before,'''var first=item(entry);var second=item(entry);
                var nativeFirst=nativeAttributes(entry);var nativeSecond=nativeAttributes(entry);boolean nativeStable=nativeFirst.equals(nativeSecond);
                stable&=first.equals(second)&&nativeStable;rows.add(first);
                nativeRows.add(object("id",first.get("id"),"state",nativeFirst,"stable_observed",nativeStable));''')
before='"native_owner_id",id(owner),"mutation_revision",after,"stable_observed",stable,"rows",rows,'
assert text.count(before)==1;text=text.replace(before,'"native_owner_id",id(owner),"mutation_revision",after,"stable_observed",stable,"rows",rows,"native_attributes",nativeRows,')
after[observer]=text.encode('utf-8')
harness='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java';text=after[harness].decode('utf-8-sig')
before=' static String stagedSnapshot(GameHub hub)throws Exception {'
assert text.count(before)==1
text=text.replace(before,''' static void initializeSharedProfile()throws Exception {
  require("tw13081901-v1".equals(System.getProperty("lito.resourceProfile")),"explicit common resource profile required");
  String selected=System.getProperty("lito.commonCatalog");require(selected!=null&&!selected.isBlank(),"common catalog path required");
  Path catalog=Path.of(selected).toRealPath();require(catalog.startsWith(REBUILD.toRealPath()),"catalog outside rebuild root");
  require(hash(catalog).equals("B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01"),"common catalog changed");
  require(hash(Path.of(SharedClientResources.class.getProtectionDomain().getCodeSource().getLocation().toURI())).equals(FORMAL),"shared initializer is not fixed E149");
  SharedClientResources.initialize(); // same original entry call, no static-field replacement
 }
'''+before)
before='require(sha.equals(FORMAL),"formal JAR changed");'
assert text.count(before)==1;text=text.replace(before,before+'\n  initializeSharedProfile();')
before='\\"original_resource_factories_initialized\\":true,'
assert text.count(before)==2
text=text.replace(before,'\\"shared_client_resources_initialized\\":true,\\"native_item_attribute_observer_kind\\":\\"G06_E149_native_item_fields_v1\\",'+before)
after[harness]=text.encode('utf-8')
assert {n for n,b in after.items() if hashlib.sha256(b).hexdigest().upper()!=old['candidate_files'][n]}=={observer,harness}
out=T/'candidates'/('native-E149-item-observer-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
for n,b in after.items():
 path=out/n;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b)
 (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff((prior.parent/n).read_text('utf-8-sig').splitlines(True),b.decode('utf-8').splitlines(True),fromfile='prior/'+n,tofile='extended/'+n)),encoding='utf-8')
(out/'resource-closure.json').write_bytes(data(resource))
record=dict(old,status='isolated_native_item_observer_and_shared_profile_revision_not_applied_not_tested',
 predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),
 candidate_files={n:hashlib.sha256(b).hexdigest().upper() for n,b in after.items()},
 changed_files=[observer,harness],all_Rust_inputs_byte_identical=True,Runtime_files_changed=0,
 fragment=str(fragment),fragment_sha256=sha(fragment),initialization_inspection=str(closure/'inspection.json'),initialization_inspection_sha256=sha(closure/'inspection.json'),
 resource_closure=str(out/'resource-closure.json'),resource_closure_sha256=sha(out/'resource-closure.json'),
 permanent_fields=['elemental_kind','elemental_level','greater','inlay_cards','magic_stone_item_id','magic_stone_level','magic_stone_time','awaken_power_ids'],
 diagnostics_are_not_continuity_invariants=True,cached_ability_materialization_is_not_forced=True,
 original_item_rows_still_have_17_fields=True,metadata_cache_is_observer_owned=True,
 known_uncovered=['dark-item payload','native timer lease internals','active expiry and mutation-denial matrix','nondefault attribute cases'],
 Java_compiled=False,Rust_metadata_compiled=False,tests_executed=0,original_initializer_invoked=False,production_applied=False,whole_goal_complete=False)
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'candidate.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,b in after.items():z.writestr(n,b)
 for name in ['candidate.json','resource-closure.json']:z.write(out/name,name)
assert sha(ref['formal'])==ref['formal_sha256']
print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_Java_files=2,Rust_files_changed=0,resource_rows=60303,tests_executed=0),ensure_ascii=False))
