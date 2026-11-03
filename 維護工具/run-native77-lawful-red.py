"""Reproduce the observed C02 lawful352 feedback bug against unchanged BA production library."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,os
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
out=ROOT/'驗證紀錄/current/native77-lawful'/('red-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
test=ROOT/'維護工具/candidates/native77-lawful/test.rs';s=(PROJECT/'tests/native_attack_skills.rs').read_text(encoding='utf-8')
for f in ['native-attack-skills-current.json','native-world-current.json']:s=s.replace('"fixtures/'+f+'"','"'+(PROJECT/'tests/fixtures'/f).as_posix()+'"')
(out/'probe.rs').write_text(s+'\n'+test.read_text(encoding='utf-8'),encoding='utf-8')
build=ROOT/'驗證紀錄/current/native77-live-compile/20261017-195023-818697';b=read(build/'receipt.json')
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
args=list(b['compiler']['command']);args[1]=str(out/'probe.rs');i=args.index('--crate-name');args[i+1]='native77_lawful_red';i=args.index('--crate-type');args[i:i+2]=['--test'];i=args.index('-o');exe=out/'red.exe';args[i+1]=str(exe)
env=dict(os.environ,CARGO_MANIFEST_DIR=str(PROJECT),TEMP=str(PROJECT/'target/current/scratch'),TMP=str(PROJECT/'target/current/scratch'))
with (out/'compile.stdout').open('wb') as stdout,(out/'compile.stderr').open('wb') as stderr:compiled=subprocess.run(args,env=env,cwd=PROJECT,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW).returncode
assert compiled==0
with (out/'test.stdout').open('wb') as stdout,(out/'test.stderr').open('wb') as stderr:code=subprocess.run([str(exe),'--exact','captured_c02_disintegrate_lawful352_is_a_rejection_without_authority_or_cast','--nocapture'],env=env,cwd=PROJECT,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW).returncode
assert code==101 and 'NoResponse' in (out/'test.stderr').read_text(encoding='utf-8') and 'Rejected(352)' in (out/'test.stderr').read_text(encoding='utf-8')
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
(out/'red.json').write_text(json.dumps(dict(status='captured_current_C02_lawful_rejection_regression_reproduced',source_manifest_sha256=b['source_manifest_sha256'],production_library=b['libraries']['lito_modern'],compile_exit_code=compiled,test_exit_code=code,expected='Rejected(352)',actual='NoResponse',test_sha256=sha(test),product_modified=False),ensure_ascii=False,indent=2),encoding='utf-8')
print('RED='+str(out));print((out/'test.stderr').read_text(encoding='utf-8'))
