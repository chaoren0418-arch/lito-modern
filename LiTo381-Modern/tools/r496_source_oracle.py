from r495_source_oracle import source_cipher,counter_add_adc
from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from capstone.x86 import X86_OP_IMM,X86_OP_REG
def schema205(value,pe):
 cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True;code={i.address:i for i in cs.disasm(pe[0xa7f60:0xa82ab],0x4a7f60)}
 pc=0x4a7f60;compare=0;fmt=None;path=[]
 # Evaluate only the source revision branch graph until its first parser call.
 # No Native process, CPU emulator, external call, stack or resource execution.
 for _ in range(160):
  i=code[pc];op=i.operands;nextpc=pc+i.size
  if i.mnemonic=='cmp'and op[0].type==X86_OP_REG and i.reg_name(op[0].reg)=='eax'and op[1].type==X86_OP_IMM:
   compare=value-op[1].imm;path.append(dict(address=i.address,threshold=op[1].imm))
  elif i.mnemonic=='jl':
   if compare<0:nextpc=op[0].imm
  elif i.mnemonic=='push'and op[0].type==X86_OP_IMM and 0x400000<=op[0].imm<0x900000:
   offset=op[0].imm-0x400000;b=pe[offset:offset+40].split(b'\0')[0]
   if b.startswith(b'ds'):fmt=b.decode('ascii')
  elif i.mnemonic=='call':
   assert op[0].imm==0x4a2a50 and fmt;return fmt,path
  pc=nextpc
 raise AssertionError('source205 branch graph bound')
def actor_body(fmt,value,name=b'NewAlias'):
 body=bytearray([205]);strings=0
 for code in fmt:
  if code=='s':body.extend((name if strings==0 else b'T')+b'\0');strings+=1
  elif code=='d':body.extend((777 if len(body)==1 else 0).to_bytes(4,'little'))
  elif code=='c':body.append(4)
  else:raise AssertionError(code)
 if value>=0x9d7c:body.extend(bytes(12));body.extend(bytes(9)if value>=0x9f0a else b'')
 return list(body)
def version_body(value,discriminator=0):
 # Source4A3423 c;4A34A1 cdddddcc;4A34BA c;9E38 extra d.
 body=bytearray([58,discriminator,0]);body.extend(value.to_bytes(4,'little',signed=True));body.extend(bytes(16+2+1));body.extend(bytes(4)if value>=0x9e38 else b'');return list(body)
