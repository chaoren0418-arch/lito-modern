"""Bounded classfile/Code reader. Parses bytes only; never loads Java classes."""
import struct

def require(ok,message):
 if not ok:raise RuntimeError(message)

class Reader:
 def __init__(self,data):self.data=data;self.at=0
 def take(self,n):
  require(n>=0 and self.at+n<=len(self.data),'truncated classfile');value=self.data[self.at:self.at+n];self.at+=n;return value
 def u1(self):return self.take(1)[0]
 def u2(self):return struct.unpack('>H',self.take(2))[0]
 def u4(self):return struct.unpack('>I',self.take(4))[0]

def parse(data):
 require(len(data)<=16*1024*1024,'classfile exceeds bound');r=Reader(data);require(r.take(4)==b'\xca\xfe\xba\xbe','classfile magic differs');minor,major=r.u2(),r.u2();count=r.u2();cp=[None]*count;i=1
 while i<count:
  tag=r.u1()
  if tag==1:cp[i]=(tag,r.take(r.u2()).decode('utf-8','replace'))
  elif tag==3:cp[i]=(tag,struct.unpack('>i',r.take(4))[0])
  elif tag==4:cp[i]=(tag,r.take(4).hex())
  elif tag==5:cp[i]=(tag,struct.unpack('>q',r.take(8))[0]);i+=1
  elif tag==6:cp[i]=(tag,r.take(8).hex());i+=1
  elif tag in [7,8,16,19,20]:cp[i]=(tag,r.u2())
  elif tag in [9,10,11,12,17,18]:cp[i]=(tag,r.u2(),r.u2())
  elif tag==15:cp[i]=(tag,r.u1(),r.u2())
  else:raise RuntimeError('unknown constant-pool tag '+str(tag))
  i+=1
 def utf(index):
  require(0<index<len(cp) and cp[index] and cp[index][0]==1,'expected UTF8 identity');return cp[index][1]
 def class_name(index):
  require(cp[index] and cp[index][0]==7,'expected class identity');return utf(cp[index][1])
 def attributes(reader):
  rows=[]
  for _ in range(reader.u2()):
   name=utf(reader.u2());body=reader.take(reader.u4());rows.append((name,body))
  return rows
 access,this,superclass=r.u2(),r.u2(),r.u2();interfaces=[class_name(r.u2()) for _ in range(r.u2())];fields=[];methods=[]
 for _ in range(r.u2()):
  flags,name,desc=r.u2(),utf(r.u2()),utf(r.u2());fields.append(dict(access=flags,name=name,descriptor=desc,attributes=attributes(r)))
 for _ in range(r.u2()):
  flags,name,desc=r.u2(),utf(r.u2()),utf(r.u2());attrs=attributes(r);code=[v for k,v in attrs if k=='Code'];require(len(code)<=1,'duplicate Code attribute');body=None
  if code:
   cr=Reader(code[0]);max_stack,max_locals=cr.u2(),cr.u2();raw=cr.take(cr.u4());require(len(raw)<=65535,'Code length exceeds JVM bound');exceptions=[tuple(cr.u2() for _ in range(4)) for _ in range(cr.u2())];attributes(cr);require(cr.at==len(cr.data),'Code attribute trailing bytes');body=dict(bytes=raw,max_stack=max_stack,max_locals=max_locals,exceptions=exceptions)
  methods.append(dict(access=flags,name=name,descriptor=desc,code=body))
 attributes(r);require(r.at==len(data),'classfile trailing bytes')
 return dict(class_name=class_name(this),superclass=class_name(superclass) if superclass else None,interfaces=interfaces,major=major,minor=minor,access=access,cp=cp,fields=fields,methods=methods)

def member(cp,index):
 require(0<index<len(cp) and cp[index] and cp[index][0] in [9,10,11],'expected field/method member reference');row=cp[index];owner=cp[cp[row[1]][1]][1];nt=cp[row[2]];return dict(owner=owner,name=cp[nt[1]][1],descriptor=cp[nt[2]][1])

def instructions(code,cp):
 # Every JVM opcode through jsr_w has an explicit operand width; variable
 # switch padding/keys and wide modifiers are decoded rather than byte-searched.
 widths={op:1 for op in range(0xca)}
 for op in [0x10,0x12,*range(0x15,0x1a),*range(0x36,0x3b),0xa9,0xbc]:widths[op]=2
 for op in [0x11,0x13,0x14,0x84,*range(0x99,0xa9),*range(0xb2,0xb9),0xbb,0xbd,0xc0,0xc1,0xc6,0xc7]:widths[op]=3
 widths.update({0xb9:5,0xba:5,0xc5:4,0xc8:5,0xc9:5})
 at=0;rows=[];targets=[]
 while at<len(code):
  start=at;op=code[start];require(op in widths,'reserved/unknown JVM opcode '+hex(op));length=widths[op];row=dict(offset=start,opcode=op)
  if op in [0xaa,0xab]:
   pad=(4-((start+1)%4))%4;head=start+1+pad;require(head+8<=len(code),'truncated switch');default=struct.unpack_from('>i',code,head)[0];targets.append(start+default)
   if op==0xaa:
    require(head+12<=len(code),'truncated tableswitch');lo,hi=struct.unpack_from('>ii',code,head+4);require(0<=hi-lo<16384,'invalid tableswitch bounds');n=hi-lo+1;length=head-start+12+4*n;require(start+length<=len(code),'truncated tableswitch table');targets.extend(start+struct.unpack_from('>i',code,head+12+4*i)[0] for i in range(n))
   else:
    n=struct.unpack_from('>i',code,head+4)[0];require(0<=n<8192,'invalid lookupswitch pairs');length=head-start+8+8*n;require(start+length<=len(code),'truncated lookupswitch pairs');pairs=[struct.unpack_from('>ii',code,head+8+8*i) for i in range(n)];require(all(pairs[i][0]<pairs[i+1][0] for i in range(n-1)),'unordered lookupswitch keys');targets.extend(start+v for _,v in pairs)
  elif op==0xc4:
   require(start+2<=len(code),'truncated wide');modified=code[start+1];require(modified in [*range(0x15,0x1a),*range(0x36,0x3b),0x84,0xa9],'invalid wide opcode');length=6 if modified==0x84 else 4
  require(start+length<=len(code),'truncated JVM instruction');row['hex']=code[start:start+length].hex()
  if op in [*range(0xb2,0xba),0xbb,0xbd,0xc0,0xc1,0xc5,0x13,0x14]:
   index=struct.unpack_from('>H',code,start+1)[0];row['cp_index']=index
   if op in range(0xb2,0xba):row['member']=member(cp,index)
   elif op in [0x13,0x14] and cp[index] and cp[index][0] in [3,5]:row['constant']=cp[index][1]
   elif op in [0x13,0x14] and cp[index] and cp[index][0]==8:row['constant_string']=cp[cp[index][1]][1]
  elif op==0x12:
   index=code[start+1];row['cp_index']=index
   if cp[index] and cp[index][0] in [3,5]:row['constant']=cp[index][1]
   elif cp[index] and cp[index][0]==8:row['constant_string']=cp[cp[index][1]][1]
  if op==0x01:row['constant']=None
  elif 0x02<=op<=0x08:row['constant']=op-3
  elif op in [0x09,0x0a]:row['constant']=op-9
  elif op==0x10:row['constant']=struct.unpack_from('>b',code,start+1)[0]
  elif op==0x11:row['constant']=struct.unpack_from('>h',code,start+1)[0]
  if 0x99<=op<=0xa8 or op in [0xc6,0xc7]:targets.append(start+struct.unpack_from('>h',code,start+1)[0])
  elif op in [0xc8,0xc9]:targets.append(start+struct.unpack_from('>i',code,start+1)[0])
  rows.append(row);at+=length
 starts={row['offset'] for row in rows};require(all(n in starts for n in targets),'branch target is not an instruction boundary')
 return rows
