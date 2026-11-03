"""Read owned formal classfile bytes only; starts no JVM or game process."""
from pathlib import Path
from datetime import datetime,timezone
import zipfile,json,hashlib
R=Path(__file__).resolve().parents[1];jar=R/'LiTo381-Modern/target/current/server-reference/formal-server-75DA69B0.jar'
sha=lambda b:hashlib.sha256(b).hexdigest().upper()
assert sha(jar.read_bytes())=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
def parse(b):
 at=8
 def take(n):
  nonlocal at
  value=b[at:at+n];assert len(value)==n;at+=n;return value
 def u2():return int.from_bytes(take(2),'big')
 def u4():return int.from_bytes(take(4),'big')
 count=u2();cp=[None]*count;i=1
 while i<count:
  tag=take(1)[0]
  if tag==1:cp[i]=(tag,take(u2()).decode('utf-8',errors='replace'))
  elif tag in[3,4]:cp[i]=(tag,take(4).hex())
  elif tag in[5,6]:cp[i]=(tag,take(8).hex());i+=1
  elif tag in[7,8,16,19,20]:cp[i]=(tag,u2())
  elif tag in[9,10,11,12,17,18]:cp[i]=(tag,u2(),u2())
  elif tag==15:cp[i]=(tag,take(1)[0],u2())
  else:raise ValueError(tag)
  i+=1
 def resolve(i):
  x=cp[i]
  if x[0]==1:return x[1]
  if x[0]in[7,8,16,19,20]:return resolve(x[1])
  if x[0]==12:return resolve(x[1])+':'+resolve(x[2])
  if x[0]in[9,10,11]:return resolve(x[1])+'.'+resolve(x[2])
  return str(x)
 flags=u2();this=u2();parent=u2();interfaces=[u2()for _ in range(u2())]
 def members():
  result=[]
  for _ in range(u2()):
   flags,name,desc=u2(),u2(),u2();attrs={}
   for _ in range(u2()):key=resolve(u2());attrs[key]=take(u4())
   row=dict(flags=flags,name=resolve(name),descriptor=resolve(desc))
   if'Code'in attrs:
    code=attrs['Code'];size=int.from_bytes(code[4:8],'big');row.update(code_hex=code[8:8+size].hex(),code_length=size)
   result.append(row)
  return result
 fields=members();methods=members();return dict(name=resolve(this),class_sha256=sha(b),fields=fields,methods=methods,references={str(i):resolve(i)for i,x in enumerate(cp)if x and x[0]in[9,10,11]})
record=dict(jar_sha256=sha(jar.read_bytes()),classes={},JVM_or_runtime_started=False,network_sessions=0,GUI_verified=False,whole_goal_complete=False)
out=R/'驗證紀錄/current/native-hotel-lease-readonly-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
with zipfile.ZipFile(jar)as z:
 for name in ['RentalTownHotelRegistry','RentalTownHotelRegistry$Lease']:
  b=z.read('local/lito381/server/'+name+'.class');record['classes'][name]=parse(b);(out/(name+'.class')).write_bytes(b)
(out/'parser.py').write_bytes(Path(__file__).read_bytes());(out/'class-readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(out)
for name,c in record['classes'].items():
 for row in c['methods']:
  if row['name']in ['lease','activeLeaseCount','lambda$activeLeaseCount$2','active']:print(name,json.dumps(row,ensure_ascii=False))
 if name.endswith('$Lease'):print('Lease refs',json.dumps(c['references'],ensure_ascii=False))
