from r491_common import *
from r495_source_oracle import source_cipher
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from capstone.x86 import X86_OP_REG,X86_OP_IMM,X86_OP_MEM
import struct
source=read(L/'r498-source-study.json');pe=Path(source['analysis_PE']['path']).read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True;code={i.address:i for i in cs.disasm(pe[0x94E00:0x94EC0],0x494E00)}
def branch(cls,var):
 regs={'ecx':cls if cls<128 else cls-256,'edx':var,'eax':0};pc=0x494E00;comparison=0;nonzero=False
 for _ in range(32):
  i=code[pc];ops=i.operands;nextpc=pc+i.size
  if i.mnemonic=='lea':assert i.reg_name(ops[0].reg)=='eax';regs['eax']=(regs['ecx']+1)&0xFFFFFFFF
  elif i.mnemonic=='cmp':comparison=regs['eax']-ops[1].imm
  elif i.mnemonic=='ja':
   if comparison>0:nextpc=ops[0].imm
  elif i.mnemonic=='jmp':
   if ops[0].type==X86_OP_MEM:va=ops[0].mem.disp+regs['eax']*ops[0].mem.scale;nextpc=struct.unpack('<I',pe[va-0x400000:va-0x400000+4])[0]
   else:nextpc=ops[0].imm
  elif i.mnemonic=='test':nonzero=regs['edx']!=0
  elif i.mnemonic=='jne':
   if nonzero:nextpc=ops[0].imm
  elif i.mnemonic=='mov'and ops[0].type==X86_OP_REG:
   reg=i.reg_name(ops[0].reg)
   if reg in regs:regs[reg]=ops[1].imm if ops[1].type==X86_OP_IMM else 0x1000
  elif i.mnemonic.startswith('ret'):return regs['eax']
  elif i.mnemonic not in ['push','pop','mov']:raise AssertionError((i.mnemonic,i.op_str))
  pc=nextpc
 raise AssertionError('source494E00 branch bound')
class_tables=[[branch(c,0),branch(c,1)]for c in range(256)];mapping=read(L/'r498-character-resource-mapping.json')
for m in mapping['animation_tables']:assert class_tables[m['class_byte']&255][m['variant_nonzero']]==int(m['table_va'],16)
rects={}
for row in [5,7]:
 va=0x630DF8+row*60;vals=struct.unpack('<15I',pe[va-0x400000:va-0x400000+60]);assert vals[0]==2;rects[str(row)]=[vals[2],vals[3],51,23]
v=read(L/'r498-source-oracle.json');body=bytes([201])+b'Role0\0';cipher,_=source_cipher(body,v['seed'],pe)
save(L/'r498-independent-source-oracle.json',dict(state='R498_STATIC_PE494E00_BRANCH_GRAPH_AND_SOURCE_CONTROL_TABLE_ORACLE',analysis_PE=source['analysis_PE'],source=pin(L/'r498-source-study.json'),class_tables=class_tables,control_rects=rects,expected_submit_wire=list((len(body)+2).to_bytes(2,'little')+cipher),source_fixture=pin(L/'r498-source-oracle.json'),method='bounded symbolic branch interpretation only; noNativePE/process/emulator/provider execution',GUI=0))
print('R498 independentPEoracle512class branches/submitback table/cs201 bytes')
