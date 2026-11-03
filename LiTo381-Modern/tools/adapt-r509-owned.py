from r491_common import *
import shutil,uuid
v=read(L/'r509-java-inputs.json');old=Path(v['work']);work=P/'target'/('r509b-java-'+uuid.uuid4().hex)
shutil.copytree(old,work)
base=work/'source/NativeLetterAuthenticatedWireMain.java'
s=base.read_text(encoding='utf-8');before=pin(base)
needle=' NativeLetterAuthenticatedWireMain(Path root,Path work){this.root=root;'
assert needle in s
guard=''' static Path checkedOwnedRoot(Path path)throws Exception{
  Path real=path.toRealPath();Path parent=Path.of("D:/lito-dev-runtime/20261003-supervision/modern").toRealPath();
  require(real.equals(path.toAbsolutePath().normalize())&&real.getParent().equals(parent)&&real.toString().matches("[\\\\x20-\\\\x7e]+")&&real.getFileName().toString().matches("nw-[a-f0-9]{32}"),"canonical unique owned Modern root");
  require(Path.of(System.getProperty("java.io.tmpdir")).toRealPath().equals(real),"same owned tmpdir");return real;
 }
'''
s=s.replace(needle,guard+' NativeLetterAuthenticatedWireMain(Path root,Path work)throws Exception{this.root=checkedOwnedRoot(root);')
base.write_text(s,encoding='utf-8')
outer=P/'tools/R509NativeFixtureMain.java';s=outer.read_text(encoding='utf-8');s=s.replace('root=Path.of(args[0]).toRealPath();','root=NativeLetterAuthenticatedWireMain.checkedOwnedRoot(Path.of(args[0]));');outer.write_text(s,encoding='utf-8')
v.update(work=str(work),previous_failed_or_unexecuted_work=str(old));save(L/'r509b-java-inputs.json',v)
save(L/'r509-ms950-failed-vector.json',dict(candidate=read(L/'r509-consumer-candidate.json'),exit=1,observed='測試允',expected='測試乙',incorrect_hex='b4fab8d5a4b9',independent_python_cp950=[dict(text=t,encoded=t.encode('cp950').hex(),decoded=t.encode('cp950').decode('cp950')) for t in ['測試允','測試乙']],production_decoder_change=False,reason='fixture literal used A4B9 (允), A441 is 乙; expected Chinese text preserved'))
body=P/'tools/r509-character-consumer-body.rs';s=body.read_text(encoding='utf-8');s=s.replace('assert_eq!(source_identity::fields::display_ms950(&hex("b4fab8d5a4b9"))?,"測試乙");','assert_eq!(source_identity::fields::display_ms950(&hex("b4fab8d5a4b9"))?,"測試允");assert_eq!(source_identity::fields::display_ms950(&hex("b4fab8d5a441"))?,"測試乙");')
body.write_text(s,encoding='utf-8');prefix=(P/'tools/r508-bootstrap-consumer.rs').read_text(encoding='utf-8').split('use lito_modern::modern_lito::native_city_runtime::login_bootstrap')[0];(P/'tools/r509b-character-consumer.rs').write_text(prefix+s,encoding='utf-8')
s=(P/'tools/build-r509-consumer.ps1').read_text(encoding='utf-8').replace('r509-fixed-consumer','r509b-fixed-consumer').replace('r509-character-consumer.rs','r509b-character-consumer.rs').replace('r509-character-consumer.exe','r509b-character-consumer.exe').replace('r509-consumer-build','r509b-consumer-build').replace('r509-consumer-candidate','r509b-consumer-candidate');(P/'tools/build-r509b-consumer.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/run-r509-integration.py').read_text(encoding='utf-8').replace("L/'r509-java-inputs.json'","L/'r509b-java-inputs.json'").replace("L/'r509-consumer-candidate.json'","L/'r509b-consumer-candidate.json'")
s=s.replace('import subprocess,time,os,sys,shutil,ctypes,socket','import subprocess,time,os,sys,shutil,ctypes,socket,re').replace("assert R.parent==Path(r'D:\\lito-dev-runtime\\20261003-supervision\\modern') and R.name.startswith('nw-')","assert R.resolve()==R and R.parent.resolve()==Path(r'D:\\lito-dev-runtime\\20261003-supervision\\modern').resolve() and re.fullmatch('nw-[a-f0-9]{32}',R.name) and str(R).isascii()")
(P/'tools/run-r509b-integration.py').write_text(s,encoding='utf-8')
c=Path(r'D:/lineage開發/3-lito開發/lito伺服器/server-channel-return-consumer-contract.json');cp=pin(c);assert cp['sha256']=='2FBFC3D5E4ADCF494F75C984B69A76943F84DC059D2A967657247DC304DB906A'
save(L/'r509-owned-fixture-adaptation.json',dict(base_before=before,base_after=pin(base),outer=pin(outer),runtime=v['runtime'],work=str(work),writes='Modern only',canonical_tmpdir_same_exact_root=True,retained_datadir_PID_FILETIME_idle_loopback_deadline_retirement_checks=True,return_contract=cp,return_reconnect='queued after finite R509; do not automatically CA100 after S_C8',failed_ms950=pin(L/'r509-ms950-failed-vector.json')))
print('Owned fixture adapted; frozen failed candidate preserved; correct MS950 vectors independently verified')
