"""Read BootstrapMethods after the existing complete classfile/Code validation."""
from native_E149_classfile_scan import Reader,parse,member,require

def bootstraps(data):
 parsed=parse(data);cp=parsed['cp'];r=Reader(data);r.take(8);count=r.u2();i=1
 widths={3:4,4:4,5:8,6:8,7:2,8:2,9:4,10:4,11:4,12:4,15:3,16:2,17:4,18:4,19:2,20:2}
 while i<count:
  tag=r.u1()
  if tag==1:r.take(r.u2())
  else:r.take(widths[tag])
  i+=2 if tag in [5,6] else 1
 r.take(6);r.take(2*r.u2())
 def attrs():
  out=[]
  for _ in range(r.u2()):
   name=cp[r.u2()][1];out.append((name,r.take(r.u4())))
  return out
 for _ in range(r.u2()):r.take(6);attrs()
 for _ in range(r.u2()):r.take(6);attrs()
 outer=attrs();require(r.at==len(data),'bootstrap reader trailing bytes')
 bodies=[body for name,body in outer if name=='BootstrapMethods'];require(len(bodies)==1,'expected one bootstrap attribute');b=Reader(bodies[0]);rows=[]
 for _ in range(b.u2()):rows.append(dict(method_ref=b.u2(),arguments=[b.u2() for _ in range(b.u2())]))
 require(b.at==len(b.data),'bootstrap attribute trailing bytes')
 def handle(index):
  row=cp[index];require(row[0]==15,'expected method handle');return dict(kind=row[1],**member(cp,row[2]))
 invoked={}
 for index,row in enumerate(cp):
  if row and row[0]==18:
   bootstrap=rows[row[1]];nt=cp[row[2]]
   invoked[index]=dict(bootstrap_index=row[1],name=cp[nt[1]][1],descriptor=cp[nt[2]][1],
    bootstrap_method=handle(bootstrap['method_ref']),method_handles=[handle(a) for a in bootstrap['arguments'] if cp[a][0]==15])
 return parsed,invoked
