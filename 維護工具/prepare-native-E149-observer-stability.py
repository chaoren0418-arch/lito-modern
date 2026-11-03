"""Bracket direct observer fields without invoking game guards or getters."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,json,zipfile,hashlib
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'active workload/freeze')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/formal target changed')
 ref=w['current_E149_fixture_binding'];require(sha(ref['qualification'])==ref['qualification_sha256'],'binding changed');binding=read(ref['qualification']);cp=Path(binding['fixture_candidate']);require(sha(cp)==binding['fixture_candidate_sha256'],'fixture candidate changed');old=read(cp)
 check_pins(binding['Java_source_pins']);owner=V/'native-E149-item-gameplay-reachability/20261023-224206-735514/NativeDeItemApplication$Owner.stdout.txt';require(sha(owner)=='57606E05ECE71C91233AAA392A2033F9A3542D34EE4ABC2BA79343A4FE20D0FD','original owner bytecode changed')
 text=owner.read_text('utf-8-sig');require(all(s in text for s in ['final java.lang.String account;','final java.lang.String source;','volatile boolean active;','Field local/lito381/server/NativeDeItemApplication.closed:Z']),'original owner/lifetime field declarations differ')
 observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java';original=(cp.parent/observer).read_text('utf-8-sig');current=original
 def replace(before,after):
  nonlocal current
  require(current.count(before)==1,'observer source anchor differs');current=current.replace(before,after)
 replace('            Object owner=field(c,"sourceItemOwner");',
 '''            Object owner=field(c,"sourceItemOwner");
            Object account=field(c,"account"),disconnected=field(c,"sourceItemDisconnected");
            Object ownerActive=owner==null?null:field(owner,"active");
            Object ownerSession=owner==null?null:((WeakReference<?>)field(owner,"session")).get();
            Object application=field(hub,"sourceItemApplication");
            Object applicationClosed=application==null?null:field(application,"closed");''')
 replace('"active",field(owner,"active"),','"active",ownerActive,')
 replace('"session_matches",((WeakReference<?>)field(owner,"session")).get()==c);','"session_matches",ownerSession==c);')
 replace('&&Boolean.TRUE.equals(nativeInventory.get("stable_observed"))&&Boolean.TRUE.equals(rentalInventory.get("stable_observed"));',
 '''&&Boolean.TRUE.equals(nativeInventory.get("stable_observed"))&&Boolean.TRUE.equals(rentalInventory.get("stable_observed"))
                &&Objects.equals(account,field(c,"account"))&&Objects.equals(disconnected,field(c,"sourceItemDisconnected"))
                &&(owner==null||(Objects.equals(ownerActive,field(owner,"active"))
                    &&ownerSession==((WeakReference<?>)field(owner,"session")).get()))
                &&application==field(hub,"sourceItemApplication")
                &&(application==null||Objects.equals(applicationClosed,field(application,"closed")));''')
 replace('"session_identity",id(c),"test_account",field(c,"account"),','"session_identity",id(c),"test_account",account,')
 replace('"source_item_application_active",field(hub,"sourceItemApplication")!=null,','"source_item_application_active",application!=null&&Boolean.FALSE.equals(applicationClosed),')
 replace('"source_item_disconnected",field(c,"sourceItemDisconnected"),','"source_item_disconnected",disconnected,')
 out=owned(T/'candidates'/('native-E149-observer-stability-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True)
 after={n:(current.encode('utf-8') if n==observer else (cp.parent/n).read_bytes()) for n in old['candidate_files']}
 for n,blob in after.items():p=owned(out/n,out);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(blob)
 (out/'observer.diff').write_text(''.join(difflib.unified_diff(original.splitlines(True),current.splitlines(True),fromfile='canonical/'+observer,tofile='stability-candidate/'+observer)),encoding='utf-8')
 contract=dict(schema='E149-observer-stability-source-contract-v1',formal_target_sha256=FORMAL,original_owner_bytecode=str(owner),original_owner_bytecode_sha256=sha(owner),
  added_stability_groups=['account','sourceItemDisconnected','owner.active','owner.session.referent','hub.sourceItemApplication','application.closed'],
  captured_output_fields=['test_account','source_item_disconnected','source_item_owner.active','source_item_owner.session_matches','source_item_application_active'],
  application_active_semantics='non-null sourceItemApplication and closed == false; not a life.admits or requireActive authorization',
  snapshot_is_atomic=False,ABA_excluded=False,game_fields_written=False,game_methods_invoked=False,tests_executed=0)
 (out/'stability-contract.json').write_bytes(encoded(contract))
 (out/'contract.json').write_bytes(encoded(dict(read(old['contract']),observer_stability_contract=str(out/'stability-contract.json'),observer_stability_contract_sha256=sha(out/'stability-contract.json'),observer_stability_selected=False,tests_executed=0)))
 candidate=dict(old,status='observer_stability_source_not_selected_not_tested',predecessor_candidate=str(cp),predecessor_candidate_sha256=sha(cp),predecessor_fixture_binding=ref,
  contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),candidate_files={n:hashlib.sha256(b).hexdigest().upper() for n,b in after.items()},changed_files=[observer],
  stability_contract=str(out/'stability-contract.json'),stability_contract_sha256=sha(out/'stability-contract.json'),Rust_files_changed=0,Runtime_files_changed=0,Java_compiled=False,tests_executed=0,integrated_into_active_fixture=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(candidate));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in after:z.write(cp.parent/n,n)
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==old['candidate_files'][n] for n in after),'before archive differs')
 print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_Java_sources=1,added_stability_groups=6,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':prepare()
