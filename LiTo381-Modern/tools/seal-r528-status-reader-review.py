from r491_common import *
from html.parser import HTMLParser
import shutil,re
own=L/'r528-status-reader-review';own.mkdir(exist_ok=True)
class Source(HTMLParser):
 def __init__(self):super().__init__();self.on=False;self.out=[]
 def handle_starttag(self,t,a):
  if t=='pre':self.on=True
 def handle_endtag(self,t):
  if t=='pre':self.on=False
 def handle_data(self,d):
  if self.on:self.out.append(d)
sysroot=Path('C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc')
base=sysroot/'share/doc/rust/html/src/std';doc_refs=[]
for name,dest,ranges in [('sys/fs/windows.rs.html','std-windows-fs-source.txt',[(194,214),(337,361),(523,548),(1300,1340)]),('fs.rs.html','std-fs-source.txt',[]),('os/windows/fs.rs.html','std-windows-fs-ext-source.txt',[(168,176)])]:
 original=base/name;parser=Source();parser.feed(original.read_text('utf-8'));lines=''.join(parser.out).splitlines()
 if name=='fs.rs.html':
  for i,line in enumerate(lines):
   if re.search(r'pub fn (open<P: AsRef<Path>>\(path: P\)|metadata\(&self\))',line) and i<1500:ranges.append((i+1,i+7))
 text='\n\n'.join('\n'.join(lines[start-1:end])for start,end in ranges)+'\n'
 (own/dest).write_text(text,'utf-8');doc_refs.append(dict(original=pin(original),excerpt=pin(own/dest),ranges=ranges))
stdlib=pin(sysroot/'lib/rustlib/i686-pc-windows-msvc/lib/libstd-39f74d20e22ab4ef.rlib')
assert 'share_mode: c::FILE_SHARE_READ | c::FILE_SHARE_WRITE | c::FILE_SHARE_DELETE' in (own/'std-windows-fs-source.txt').read_text('utf-8')
save(own/'rust-standard-library-evidence.json',dict(rustc_version='1.95.0',commit='59807616e1fa2540724bfbac14d7976d7e4a3860',commit_date='2026-04-14',target='i686-pc-windows-msvc',sysroot=str(sysroot),local_shipped_source_documentation=doc_refs,target_std_rlib=stdlib,chain='File::open -> OpenOptions::new().read(true).open -> windows OpenOptions default share READ|WRITE|DELETE -> CreateFileW opts.share_mode; File.metadata -> same inner file_attr/GetFileInformationByHandle.',online_primary_crosscheck='https://doc.rust-lang.org/stable/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode',raw_rust_src='Directory exists but rawfs source absent; local shipped rustdoc source HTML used, not invented installedsource.',evidence_limit='Local standard docs+targetstdlib/toolchain pin and actual fresh fileIO behavior. No global processhandle inspection or proof of third-party sharing modes.'))
original_handoff=pin(L/'r528-countdown-cancel-consumer-ready-handoff.json');assert original_handoff['sha256']=='28BC666D4512DA1BFF5D9584BD1C4C7A9F85FC7DDF02F13C5648B2C6F84B9E9F'
candidate=read(L/'r528-final-candidate.json');old_checked=[]
for v in candidate.values():
 if isinstance(v,dict)and {'path','bytes','sha256'}<=v.keys():assert pin(v['path'])==v;old_checked.append(v)
assert candidate['executable']['sha256']=='AB3B9ABEE0A3212D5BECEEB25E3A65B5CAEB4BA274E3AEDF0791D89775A9AF24'
probe=read(L/'r528-status-reader-check-accounting.json');build=read(L/'r528-status-reader-check-build-accounting.json')
assert probe['attempts']==build['attempts']==1 and probe['exit_code']==build['exit_code']==0
assert probe['network']==probe['credentials_reads']==probe['PIDqueries']==probe['JavaSQL']==probe['server_writes']==probe['old_tests_replayed']==0 and not probe['old_consumer_recompiled']
assert not build['consumer_recompiled'] and not build['production_library_rebuilt']
assert '3 passed; 0 failed' in (L/'r528-status-reader-check.stdout.log').read_text('utf-8-sig')
assert not (L/'r528-status-reader-check-build.stderr.log').read_text('utf-8-sig').strip()
assert not (L/'r528-status-reader-check.stderr.log').read_text('utf-8-sig').strip()
assert pin(probe['binary']['path'])==probe['binary'] and pin(probe['source']['path'])==probe['source']
failure_path=Path('D:/lito-dev-runtime/20261003-supervision/server/nw-975e97ed0f74480c9c92edd7d6b7d51e/evidence/result.json')
failure=read(failure_path);assert failure['status']=='FAILED_OR_INCOMPLETE' and 'AccessDeniedException:' in failure['failure'] and 'status.json.tmp' in failure['failure']
assert failure['host_sync']['stage']=='WAITING_GUEST_JOIN_AND_BASELINE_D1_0' and not failure['host_sync']['Host_participant_state1_sent']
assert all(not failure['host_sync'][key]for key in ['baseline_D1_0_write_epochs','Host_D1_1_write_epochs','E103_write_epochs','Guest_notready_D1_write_epochs','E100_write_epochs'])
assert failure['all_run_threads_retired'] and failure['mysql_retired'] and failure['unreleased_ports']==[] and failure['datadir_retained']
assert failure['final_connected_sessions']==failure['final_authenticated_accounts']==failure['final_selected_characters']==0
shutil.copyfile(failure_path,own/'first-provider-failure-result-observed.json')
save(own/'status-read-findings.json',dict(
 original_candidate=candidate['executable'],original_ready_handoff=original_handoff,
 original_read_bound='fs::File::open(path)?.take(bound+1).read_to_end; default standard share-mode unchanged. Applies READY/ownership/launch and status byte reads; handles drop before JSON parsing.',
 sharing_result='Already FILE_SHARE_READ|WRITE|DELETE; no FileShare.Delete repair needed. Actual own-file test permits replacement while this default reader handle remains open.',
 original_status_consistency_gap='fresh_status uses fs::metadata(path).modified().elapsed before a different File::open for bytes. Atomic replacement between calls can combine generationA mtime and generationB completeJSON.',
 deterministic_own_file_reproduction='FreshA pathmtime then replace with staleB JSON: old two-call pattern obtains different generations; new same-handle reader refuses staleB.',
 minimal_reader_proposal=pin(P/'tools/r528-status-reader.rs'),minimal_provider_callsite_proposal=pin(P/'tools/r528-provider-status-consistent-proposal.rs'),
 proposal='Single default sharedFile handle; handlemetadata before/after boundedread, check same length/mtime and bytesize, age<=5sec from that handle, drophandle then completeJSONparse. No sharing override/lock/retry/provideroperation.',
 probe_tests_passed=3,probe_compile_warnings=0,probe_attempts=1,
 causal_status='Provider failure AccessDenied observed, cause not established. Modern preflight/connect never authorized/run; Rust delete-compatible behavior and temporal proximity of supervisor Python reads do not identify causal holder/API.',
 limitations=['Own std::fs::rename replacement check is not JavaATOMIC_MOVE/provider runtime acceptance.','Same-handle metadata aligns opened generation, not necessarily newest pathname generation after concurrent replace.','Concurrent in-place writes with unchangedlen/mtime or timestamp rollback not fully excluded; publisher still needs accepted atomic-file contract.','No global atomicGET/packetfreshness/race/WORLD/GUI claim.'],
 original_AB3_candidate_and_28BC_handoff_unchanged=True,consumer_recompiled=False,
 proposal_not_integrated_into_old_candidate=True,next='Wait supervisor accepts repaired cold provider contract, then integrate status reader proposal and adopt exactnewpins; no fresh runtime/preflight/connect now.',
 retired_failed_READY_read_or_consumed=False,server_scripts_run=0,Modern_network_cases=0,Modern_auth=0,GUI=0))
human=P/'Native-R528_status讀者分享與一致性核對.txt'
human.write_text('''R528 status／ownership／READY讀者冷態核對

結論：現有Rust std::fs::File::open預設已包含FILE_SHARE_READ／WRITE／DELETE，現有read_bound沒有share_mode override，不需另加FileShare.Delete。證據取自本機Rust1.95.0（59807616e1fa2540724bfbac14d7976d7e4a3860）的標準庫rustdoc source：Windows OpenOptions::new預設三個share flags，CreateFileW使用opts.share_mode；File::open走OpenOptions::new().read(true).open。local rawrust-src檔案未實裝，沒有冒稱讀到它；使用同sysroot附帶HTMLsource及targetstd rlib pins，另做真實Windows小型檔案I/O。

一次新reader-only test build與一次新3項IO檢查均PASS、warning0：
1. std File::open持有舊status handle時，std fs::rename替換path成功；舊handle仍讀完整generation1与自己mtime，newpath讀generation22。
2. 重現fresh_status現有兩步漏洞：先取freshA的pathmtime，再替換staleB，稍後File::open得到B JSON；mtime与JSON跨版本。新samehandle讀者按B本身mtime拒絕staleB。
3. incompleteJSON／超出讀取上限立即Err，未重試。

必要最小修正已作為獨立source proposal：r528-status-reader.rs單一File handle讀metadata→bounded bytes→同handle metadata，核對len／mtime／bytesize，按該handle的mtime核5秒，再drophandle並完整JSONparse。r528-provider-status-consistent-proposal.rs只替换fresh_status原pathstat＋另開檔為該reader，READY／ownership既有immutable pinned讀者無分享模式改動。不加lock／share override／retry，也不改Runtime／IO／receipt／Host門檻。尚未編譯新完整consumer／未整合舊候選；待監製接受server修復後新契約再採用與封存。

原AB3B9ABEE0A3212D5BECEEB25E3A65B5CAEB4BA274E3AEDF0791D89775A9AF24 executable与28BC666D4512DA1BFF5D9584BD1C4C7A9F85FC7DDF02F13C5648B2C6F84B9E9F coldready handoff、library/source pins均保留相符。這次只編譯新小型reader probe，不重編consumer／production library，不重跑PASS。

首次供應nw-975e97ed0f74480c9c92edd7d6b7d51e已退休，safe result副本保留FAILED_OR_INCOMPLETE／AccessDeniedException status.json.tmp→status.json；Hostsync仍WAITING_GUEST_JOIN_AND_BASELINE_D1_0，baseline／Host1／E103／取消D1／E100寫入集合皆空。結果記allthreads/mysql retired／portsfree／datadir retained／finalsessions/auth/selected0。Modern未獲許可且沒有preflight/connect；未讀／消費該READY、privatecredentials或rawauthcapture。

不能把AccessDenied判定為Modern所致，也不能以監製Python讀檔與時間相鄰當成已證明因果；目前未識別阻擋handle／分享模式／發生API，server發布修復由server組處理。自己的fs::rename檢查不是JavaATOMIC_MOVE或runtime供應驗收。samehandle保證所開版本mtime／bytes一致，不保证是pathname最新版本；inplace同len同mtime寫入／clockrollback仍不完全排除。這項一致性修正不是status發布AccessDenied的已證明修復。

本批0socket／0JavaSQL/provider／0PIDquery／0credentials／0serverwrites／0GUI。新IO檔案留在自有modern唯一ASCII目錄，無刪除。等待修復後新冷契約接受與後續新runtime通知；GUI／WORLD／sameepochfreshness等仍未驗收。

官方補充： https://doc.rust-lang.org/stable/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode
機器交接：logs/continuation-20261003/r528-status-reader-compatibility-handoff.json
''',encoding='utf-8')
roots=[original_handoff,pin(L/'r528-final-candidate.json'),pin(human)]
roots += [pin(p)for p in sorted(own.iterdir())if p.is_file()]
roots += [pin(P/'tools'/name)for name in ['r528-status-reader.rs','r528-provider-status-consistent-proposal.rs','check-r528-status-reader.ps1','seal-r528-status-reader-review.py']]
for prefix in ['r528-status-reader-check-build','r528-status-reader-check']:
 roots += [pin(L/(prefix+suffix))for suffix in ['.stdout.log','.stderr.log','-accounting.json']]
roots += [probe['binary'],probe['source']]
verification=L/'r528-status-reader-final-verification.json'
save(verification,dict(status='PASS_LIMITED_NEW_OWN_FILE_IO_REVIEW_NO_PROVIDER_RUNTIME',root_references_checked=roots,old_candidates_checked=old_checked,target_std=stdlib,sharing_mode_already_compatible=True,same_handle_status_proposal_verified=True,new_tests_passed=3,consumer_recompiled=False,original_AB3_and_28BC_preserved=True,causality_proven=False))
roots.append(pin(verification))
handoff=L/'r528-status-reader-compatibility-handoff.json'
save(handoff,dict(schema='modern.r528.status-reader-compatibility-handoff.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
 state='DELETE_SHARING_ALREADY_COMPATIBLE_SAMEHANDLE_STATUS_PROPOSAL_VERIFIED_WAITING_ACCEPTED_PUBLISHER_REPAIR_CONTRACT',root_references=roots,human=pin(human),
 original_ready_handoff=original_handoff,original_candidate=candidate['executable'],original_candidate_recompiled=False,
 standard_library_evidence=pin(own/'rust-standard-library-evidence.json'),findings=pin(own/'status-read-findings.json'),
 provider_first_failure=pin(own/'first-provider-failure-result-observed.json'),failed_READY_consumed=False,
 FileShare_Delete_already_compatible=True,sharing_override_added=False,
 proposed_status_reader=pin(P/'tools/r528-status-reader.rs'),proposed_provider_callsite=pin(P/'tools/r528-provider-status-consistent-proposal.rs'),proposal_integrated_into_old_candidate=False,
 new_IO_checks=dict(attempts=1,passed=3,failed=0,warnings=0,own_files_only=True,own_root=probe['own_io_root'],retained=True,old_PASS_replays=0),
 scope=dict(preflight=0,connect=0,provider_start_stop=0,JavaSQL=0,credentials_reads=0,PIDqueries=0,server_writes=0,GUI=0),
 cause_of_AccessDenied='Unknown; no causal inference from adjacent Python reads or this ownFileIO probe.',
 next='Await accepted repaired cold provider contract; then integrate same-handle reader and strictly adopt newpins before separate runtime coordination.',verification=pin(verification),overall_completed=False))
for e in read(handoff)['root_references']:assert pin(e['path'])==e
assert pin(original_handoff['path'])==original_handoff and pin(candidate['executable']['path'])==candidate['executable']
v=read(P.parent/'監製進度.json');v['updated_at']=read(handoff)['updated_at'];v['phase']='R528_status_reader_review_sealed_waiting_accepted_publisher_repair';v['overall_status']='in_progress';v['current_task']='R528有限冷讀者核對：Rust1.95預設READ/WRITE/DELETE已相容；新自有fileIO3checks PASS/warning0，mtime与JSON跨版本漏洞已重現，samehandle reader/source proposal備妥。AB3/28BC保留未重編；首次供應nw975已退休、未消費READY、0preflight/connect。AccessDenied因果未證明，待server修復新冷契約接受再採用，runtime/GUI未验收。'
r=v.setdefault('r528',{});r.update(status_reader_review=pin(handoff),status_reader_human=pin(human),status_reader_verification=pin(verification),status_reader_proposal_pending_integration=True,FileShare_Delete_already_compatible=True,provider_first_failure=pin(own/'first-provider-failure-result-observed.json'),failed_READY_consumed=False,fresh_provider_execution_authorized=False,actual_backend_executions=0,preflight_runs=0,connect_runs=0,publisher_repair_contract_pending=True,overall_completed=False)
save(P.parent/'監製進度.json',v)
print(json.dumps(dict(status='REVIEW_SEALED_NO_PROVIDER_RUNTIME',handoff=pin(handoff),human=pin(human),roots=len(roots),new_IO_tests=3,old_consumer_preserved=True,FileShare_Delete_already_compatible=True,AccessDenied_cause_proven=False),ensure_ascii=False))
