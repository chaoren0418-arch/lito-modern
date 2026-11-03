"""Bounded raw PE64 imports/exports reader. No DLL or image is loaded."""
from pathlib import Path
import struct,hashlib
from native_E149_recovery_contract import require

def inspect(path):
 data=Path(path).read_bytes()
 def number(at,kind):
  size=struct.calcsize(kind);require(0<=at<=len(data)-size,'PE read outside file');return struct.unpack_from(kind,data,at)[0]
 u16=lambda n:number(n,'<H');u32=lambda n:number(n,'<I');u64=lambda n:number(n,'<Q')
 require(data[:2]==b'MZ','DOS signature absent');pe=u32(0x3c);require(data[pe:pe+4]==b'PE\0\0','PE signature absent');optional=pe+24
 require(u16(pe+4)==0x8664 and u16(optional)==0x20b and u16(pe+20)>=240,'not AMD64 PE32+')
 count=u16(pe+6);require(0<count<=96,'section count invalid');sections=[]
 for i in range(count):
  at=optional+u16(pe+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
 def at(rva,size):
  require(rva>=0 and size>=0,'negative PE range')
  if rva+size<=u32(optional+60):require(rva+size<=len(data),'header outside file');return rva
  found=[raw+rva-base for base,length,raw in sections if base<=rva and rva+size<=base+length]
  require(len(found)==1 and found[0]+size<=len(data),'PE RVA not uniquely file-backed');return found[0]
 def string(rva,limit=4096):
  raw=bytearray()
  for i in range(limit):
   b=data[at(rva+i,1)]
   if b==0:
    require(raw and all(32<=v<127 for v in raw),'empty/non-ASCII PE name');return raw.decode('ascii')
   raw.append(b)
  raise RuntimeError('unterminated PE name')
 def directory(index):
  if u32(optional+108)<=index:return (0,0)
  rva,size=u32(optional+112+8*index),u32(optional+116+8*index)
  require((rva==0)==(size==0),'partial PE directory');return rva,size
 def thunks(rva):
  require(rva!=0,'missing import name table');result=[]
  for i in range(65536):
   value=u64(at(rva+8*i,8))
   if value==0:return result
   if value&(1<<63):
    require(value&0x7fffffffffff0000==0,'reserved ordinal thunk bits');result.append(dict(ordinal=value&0xffff))
   else:
    require(value<=0xffffffff,'unresolved bound import address');result.append(dict(name=string(value+2),hint=u16(at(value,2))))
  raise RuntimeError('import thunk count exceeds bound')
 imports=[];rva,size=directory(1)
 if rva:
  require(20<=size<=4096*20,'import directory size invalid');terminated=False
  for i in range(size//20):
   entry=at(rva+20*i,20);fields=[u32(entry+j*4) for j in range(5)]
   if not any(fields):terminated=True;break
   imports.append(dict(dll=string(fields[3],260),symbols=thunks(fields[0] or fields[4])))
  require(terminated,'import descriptor terminator absent')
 delay=[];rva,size=directory(13);base=u64(optional+24)
 if rva:
  require(32<=size<=4096*32,'delay import size invalid');terminated=False
  for i in range(size//32):
   entry=at(rva+32*i,32);fields=[u32(entry+j*4) for j in range(8)]
   if not any(fields):terminated=True;break
   require(fields[0] in [0,1],'unknown delay import attributes')
   def relative(value):
    if fields[0]==1:return value
    require(value>=base,'delay VA cannot be converted to RVA');return value-base
   delay.append(dict(dll=string(relative(fields[1]),260),symbols=thunks(relative(fields[4] or fields[3]))))
  require(terminated,'delay import descriptor terminator absent')
 exports=[];rva,size=directory(0)
 if rva:
  p=at(rva,40);ordinal_base=u32(p+16);functions=u32(p+20);names=u32(p+24)
  require(functions<=65536 and names<=65536,'export table exceeds bound');function_table=u32(p+28);name_table=u32(p+32);ordinal_table=u32(p+36)
  by_ordinal={}
  for i in range(functions):
   target=u32(at(function_table+4*i,4))
   if target:by_ordinal[i]=dict(ordinal=ordinal_base+i,names=[],forwarder=string(target) if rva<=target<rva+size else None)
  for i in range(names):
   index=u16(at(ordinal_table+2*i,2));require(index in by_ordinal,'named export without target');by_ordinal[index]['names'].append(string(u32(at(name_table+4*i,4))))
  exports=list(by_ordinal.values())
 dynamic=sorted({s['name'] for row in imports+delay for s in row['symbols'] if s.get('name') in ['LoadLibraryA','LoadLibraryW','LoadLibraryExA','LoadLibraryExW','GetProcAddress']})
 version=None;resource_rva,resource_size=directory(2)
 if resource_rva:
  raw=data[at(resource_rva,resource_size):at(resource_rva,resource_size)+resource_size];signature=b'\xbd\x04\xef\xfe';positions=[];offset=0
  while True:
   offset=raw.find(signature,offset)
   if offset<0:break
   if offset+52<=len(raw) and struct.unpack_from('<I',raw,offset+4)[0]==0x10000:positions.append(offset)
   offset+=4
  if len(positions)==1:
   values=struct.unpack_from('<13I',raw,positions[0]);parts=lambda a,b:[a>>16,a&0xffff,b>>16,b&0xffff]
   version=dict(file=parts(values[2],values[3]),product=parts(values[4],values[5]),source='unique VS_FIXEDFILEINFO signature in file-backed resource directory')
 return dict(schema='PE64-raw-import-export-v1',path=str(Path(path)),sha256=hashlib.sha256(data).hexdigest().upper(),machine='AMD64',linker_version=[data[optional+2],data[optional+3]],PE_kind='DLL' if u16(pe+22)&0x2000 else 'EXE',
  imports=imports,delay_imports=delay,exports=exports,fixed_file_version=version,dynamic_loader_imports=dynamic,raw_file_only=True,image_loaded=False,entry_point_invoked=False,dynamic_dependency_closure_proven=False)
