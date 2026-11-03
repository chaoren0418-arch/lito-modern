"""Record all compiled lib test names and opt-in omissions without executing them."""
from pathlib import Path
import argparse,hashlib,json,os,re,subprocess,collections
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
ap=argparse.ArgumentParser();ap.add_argument('build_run',type=Path);a=ap.parse_args();run=a.build_run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current')
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
product=read(run/'product-retention.json');exe=Path(product['product']);assert sha(exe)==product['product_sha256']
out=run/'inventory-audit';out.mkdir();env=os.environ.copy();env['TEMP']=env['TMP']=str(PROJECT/'target/current/scratch')
sets={}
for name,args in [('all',['--list']),('ignored',['--list','--ignored'])]:
 result=subprocess.run([str(exe),*args],cwd=PROJECT,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
 (out/(name+'.stdout.txt')).write_bytes(result.stdout);(out/(name+'.stderr.txt')).write_bytes(result.stderr);assert result.returncode==0
 sets[name]=re.findall(r'^(.+): test$',result.stdout.decode('utf-8-sig'),re.M)
assert len(sets['all'])==len(set(sets['all'])) and set(sets['ignored']).issubset(sets['all'])
assert sha(exe)==product['product_sha256']
counts=collections.Counter('::'.join(name.split('::')[:2]) for name in sets['ignored'])
proof={'status':'inventoried_compiled_library_tests_and_opt_in_omissions','product_sha256':product['product_sha256'],'total_tests':len(sets['all']),'default_tests_selected':len(sets['all'])-len(sets['ignored']),'ignored_tests':len(sets['ignored']),'ignored_by_module':dict(sorted(counts.items())),'all_test_names':sets['all'],'ignored_test_names':sets['ignored'],'test_bodies_executed_by_this_audit':0,'ignored_tests_verified':False,'whole_goal_complete':False,'inventory_tool_sha256':sha(__file__),'raw_pins':{f.name:sha(f) for f in out.iterdir() if f.is_file()}}
(out/'inventory.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','total_tests','default_tests_selected','ignored_tests']},ensure_ascii=False));print('Ignored module groups:',len(counts))
