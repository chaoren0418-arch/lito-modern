"""Prepare owned relocated images for app-local CRT loader verification only."""
from pathlib import Path
import hashlib,json,shutil,struct,datetime
root=Path(__file__).resolve().parents[1]
sha=lambda p:hashlib.file_digest(Path(p).open('rb'),'sha256').hexdigest().upper()
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
normal=root/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json';q=read(normal)
stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out=root/'驗證紀錄/current/native-portable-runtime'/stamp;runtime=root/'LiTo381-Modern/target/current/portable-runtime-probe'/stamp
out.mkdir(parents=True);runtime.mkdir(parents=True)
crt=Path('C:/Program Files/Microsoft Visual Studio/18/Insiders/VC/Redist/MSVC/14.51.36231/x64/Microsoft.VC145.CRT/vcruntime140.dll')
def imports(path):
 b=Path(path).read_bytes();pe=struct.unpack_from('<I',b,60)[0];assert b[pe:pe+4]==b'PE\0\0';machine,n=struct.unpack_from('<HH',b,pe+4);opt=pe+24;size=struct.unpack_from('<H',b,pe+20)[0];assert machine==0x8664 and struct.unpack_from('<H',b,opt)[0]==0x20b
 sections=[]
 for i in range(n):
  vsize,va,rawsize,raw=struct.unpack_from('<IIII',b,opt+size+i*40+8);sections.append((va,max(vsize,rawsize),raw))
 def off(va):
  for start,n,raw in sections:
   if start<=va<start+n:return raw+va-start
  raise AssertionError(('invalid import RVA',va))
 rva=struct.unpack_from('<I',b,opt+120)[0];pos=off(rva);names=[]
 while any(struct.unpack_from('<IIIII',b,pos)):
  name=off(struct.unpack_from('<I',b,pos+12)[0]);names.append(b[name:b.index(0,name)].decode('ascii'));pos+=20
 return {'machine':'AMD64','optional_header':'PE32+','DLL_imports':names}
products={}
for profile,item in q['profiles'].items():
 original=Path(item['path']);assert sha(original)==item['sha256'] and item['features']==[]
 folder=runtime/profile;folder.mkdir();main=folder/'LiToModern.exe';dll=folder/'vcruntime140.dll'
 shutil.copy2(original,main);shutil.copy2(crt,dll);assert sha(main)==sha(original) and sha(dll)==sha(crt)
 products[profile]={'main':str(main),'main_sha256':sha(main),'runtime_DLL':str(dll),'runtime_DLL_sha256':sha(dll),'main_PE':imports(main),'CRT_PE':imports(dll)}
 assert 'vcruntime140.dll' in [x.lower() for x in products[profile]['main_PE']['DLL_imports']]
(runner:=runtime/'unrelated-cwd').mkdir()
result={'status':'prepared_relocated_loader_probe_not_executed','output':str(out),'runtime':str(runtime),'unrelated_cwd':str(runner),'normal_qualification':str(normal),'normal_qualification_sha256':sha(normal),'products':products,'CRT_source':str(crt),'CRT_source_sha256':sha(crt),'app_local_deployment_reference':'https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files?view=msvc-170','redistribution_approved':False,'private_validation_only':True,'game_assets_copied':False,'launcher_or_server_artifacts_copied':False,'gameplay_verified':False,'clean_host_verified':False,'portable_delivery_complete':False,'whole_goal_complete':False,'preparer_sha256':sha(__file__)}
(out/'manifest.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(out)
