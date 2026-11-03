from r491_common import *
import shutil
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源')
S=Path(r'D:/lineage開發/3-lito開發/lito伺服器')
roots=[(B/'S0C-status-owner-contract.json','EC24D44166CB1E738B6FF22C9B1AFE2E606AB6DD4B3CFECEF20FE42704074C92'),
 (S/'server-channel-return-consumer-contract.json','2FBFC3D5E4ADCF494F75C984B69A76943F84DC059D2A967657247DC304DB906A'),
 (S/'server-channel-return-network-lifecycle.json','A6FEADD3AC90D4DB53887C5299BDCA5F1D2A6F6D0747A7D6C342045D7E66CA0F'),
 (S/'server-nonempty-skill-network.json','0137C9E382E246C5D8B6C75A6E478B863C356A4047D9E43BF423224C82E4FE96'),
 (S/'server-skill-publication-origin-comparison.json','A16B1FA66350D50056B78CF5B62F8ABECBC6D399D048B3409A66779C565B6963')]
pins={};seen=set();code=set();fail=[]
pe=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe').read_bytes()
def verify(v):
 p=Path(v['path']);key=(str(p),v['sha256'].upper())
 if key not in pins:
  e=pin(p);assert e['bytes']==v['bytes'] and e['sha256']==key[1],(v,e);pins[key]=e
def walk(v,scope,recurse=True):
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   p=Path(v['path']);p=p if p.is_absolute() else scope/p;verify(dict(v,path=str(p)));key=(str(p),v['sha256'].upper())
   if recurse and p.suffix=='.json' and scope in p.parents and key not in seen:seen.add(key);walk(read(p),scope)
  if 'va' in v and 'raw_hex' in v and v.get('raw_backed',True):
   raw=bytes.fromhex(v['raw_hex']);at=int(v['va'],16)-0x400000
   assert pe[at:at+len(raw)]==raw;code.add((v['va'],v['raw_hex']))
  for x in v.values():walk(x,scope,recurse)
 elif isinstance(v,list):
  for x in v:walk(x,scope,recurse)
dest=L/'r512-inputs';dest.mkdir(exist_ok=True)
for p,sha in roots:
 e=pin(p);assert e['sha256']==sha;verify(e);d=read(p)
 scope=B/'S0C-status-owner' if p.parent==B else Path(d['evidence_root'])
 walk(d,scope,p.parent==B)
 if 'manifest' in d:
  m=read(d['manifest']['path'])
  for f in m.get('files',[]):
   q=(scope/f['path']).resolve();assert scope.resolve() in q.parents
   verify(dict(f,path=str(q)))
 shutil.copyfile(p,dest/p.name)
save(L/'r512-input-adoption.json',dict(contracts=[pin(p) for p,_ in roots],payloads=list(pins.values()),fixed_PE_unique_raw_claims=len(code),historical_PASS_reruns=0,new_Java_SQL_runs=0,GUI=0,skill_correction='A45 first-lobby S1E came only from TEST_ONLY six-line overlay; baseline B1/A45/AD368 have no ordinary enterChannel hook. No SQL/Main cause or speculative producer change.'))
print('verified',len(pins),'file fingerprints,',len(code),'unique existing raw claims')
