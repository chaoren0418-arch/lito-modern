"""Bounded symbolic evaluator of the pinned4C6700 instruction slice.
Not Native process/PE execution; no calls/provider/OS/machine emulator.
Only mov/xor/and/cmp/inc/jumps/push/pop/ret over synthetic byte memory.
"""
from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from capstone.x86 import X86_OP_REG,X86_OP_IMM,X86_OP_MEM
def source_cipher(body,counter,pe):
 assert 1<=len(body)<=155
 cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True;code={i.address:i for i in cs.disasm(pe[0xC6700:0xC6737],0x4C6700)}
 regs={'eax':0,'ebx':0,'ecx':0x1000,'edx':len(body),'esi':0};mem={0x1000+i:v for i,v in enumerate(body)};mem.update({0x8317C8+i:v for i,v in enumerate(counter.to_bytes(8,'little'))});stack=[];pc=0x4C6700;cmp=0;steps=0
 aliases={'al':'eax','bl':'ebx','cl':'ecx','dl':'edx'}
 def reg(n):return regs[aliases[n]]&255 if n in aliases else regs[n]
 def setreg(n,v):
  if n in aliases:r=aliases[n];regs[r]=(regs[r]&0xffffff00)|(v&255)
  else:regs[n]=v&0xffffffff
 def address(i,o):return o.mem.disp+(reg(i.reg_name(o.mem.base))if o.mem.base else 0)+(reg(i.reg_name(o.mem.index))*o.mem.scale if o.mem.index else 0)
 def readop(i,o):
  if o.type==X86_OP_REG:return reg(i.reg_name(o.reg))
  if o.type==X86_OP_IMM:return o.imm
  assert o.type==X86_OP_MEM and o.size==1;return mem[address(i,o)]
 def writeop(i,o,v):
  if o.type==X86_OP_REG:setreg(i.reg_name(o.reg),v)
  else:assert o.type==X86_OP_MEM and o.size==1;mem[address(i,o)]=v&255
 while True:
  steps+=1;assert steps<=3000;i=code[pc];op=i.operands;nextpc=pc+i.size;m=i.mnemonic
  if m=='ret':break
  if m=='push':stack.append(readop(i,op[0]))
  elif m=='pop':writeop(i,op[0],stack.pop())
  elif m=='mov':writeop(i,op[0],readop(i,op[1]))
  elif m=='xor':writeop(i,op[0],readop(i,op[0])^readop(i,op[1]))
  elif m=='and':writeop(i,op[0],readop(i,op[0])&readop(i,op[1]))
  elif m=='cmp':cmp=readop(i,op[0])-readop(i,op[1])
  elif m=='inc':writeop(i,op[0],readop(i,op[0])+1)
  elif m in ['jl','jle']:
   if cmp<0 or(m=='jle'and cmp==0):nextpc=op[0].imm
  else:raise AssertionError('unqualified symbolic opcode '+m)
  pc=nextpc
 assert not stack;return bytes(mem[0x1000+i]for i in range(len(body))),steps
def counter_add_adc(counter,n):
 low=(counter&0xffffffff)+n;high=((counter>>32)+(low>>32))&0xffffffff
 return(high<<32)|(low&0xffffffff)
