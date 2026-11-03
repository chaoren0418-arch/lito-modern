"""Interpret a bounded integer instruction slice from the pinned reference PE.
No executable is started and no process memory is accessed. This is static evidence.
"""
from pathlib import Path
import hashlib,json,struct
import capstone,pefile
from capstone.x86_const import X86_OP_REG,X86_OP_IMM,X86_OP_MEM
root=Path(__file__).resolve().parents[1]
stage=root/'驗證紀錄/current/native-draw-flags'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest().upper()
meta=read(stage/'queue-draw-raw.json')['native_image']
raw=Path(meta['path']).read_bytes();assert sha(raw)==meta['sha256']
pe=pefile.PE(data=raw,fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase
begin,end=0x4c19db,0x4c1a17
code=pe.get_data(begin-base,end-begin)
cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);cs.detail=True
program=list(cs.disasm(code,begin));assert program[-1].address+program[-1].size==end
assert all(i.mnemonic in {'mov','lea','cdq','sub','sar','shl'} for i in program)
assert any(i.mnemonic=='sar' and i.op_str=='eax, 1' for i in program)
alias={'ax':('eax',16),'dx':('edx',16),'cl':('ecx',8)}
outputs=[]
for red_shift,green_shift in [(10,5),(11,6)]:
    tables={a:bytearray(2048) for a in [0x80c09c,0x80c89c,0x80d09c]}
    for destination in range(32):
        for source in range(32):
            regs={k:0 for k in ['eax','ebx','ecx','edx','esi','edi','ebp','esp']}
            regs.update(edi=source,ebx=red_shift,esi=2*(destination*32+source),ebp=0x100000)
            memory={}
            def put(addr,size,value):
                for j in range(size):memory[addr+j]=(value>>(8*j))&255
            def get(addr,size):return sum(memory.get(addr+j,0)<<(8*j) for j in range(size))
            put(regs['ebp']-4,4,destination);put(regs['ebp']-8,4,green_shift)
            def reg_read(name):
                name,bits=alias.get(name,(name,32));return regs[name]&((1<<bits)-1)
            def reg_write(name,value):
                name,bits=alias.get(name,(name,32));mask=(1<<bits)-1;regs[name]=(regs[name]&~mask)|(value&mask)
            def address(ins,op):
                m=op.mem;assert m.segment==0
                return (m.disp+(reg_read(ins.reg_name(m.base)) if m.base else 0)+(reg_read(ins.reg_name(m.index))*m.scale if m.index else 0))&0xffffffff
            def value(ins,op):
                if op.type==X86_OP_REG:return reg_read(ins.reg_name(op.reg))
                if op.type==X86_OP_IMM:return op.imm&0xffffffff
                assert op.type==X86_OP_MEM;return get(address(ins,op),op.size)
            def write(ins,op,v):
                if op.type==X86_OP_REG:reg_write(ins.reg_name(op.reg),v)
                else:assert op.type==X86_OP_MEM;put(address(ins,op),op.size,v)
            for ins in program:
                ops=ins.operands;m=ins.mnemonic
                if m=='cdq':regs['edx']=0xffffffff if regs['eax']&0x80000000 else 0;continue
                if m=='mov':write(ins,ops[0],value(ins,ops[1]));continue
                if m=='lea':write(ins,ops[0],address(ins,ops[1]));continue
                a,b=value(ins,ops[0]),value(ins,ops[1]);bits=ops[0].size*8
                if m=='sub':v=a-b
                elif m=='shl':v=a<<(b&31)
                elif m=='sar':v=(a-(1<<bits) if a&(1<<(bits-1)) else a)>>(b&31)
                else:raise AssertionError(m)
                write(ins,ops[0],v)
            index=2*(destination*32+source);half=(destination+source)//2
            for addr,shift in [(0x80c09c,0),(0x80c89c,green_shift),(0x80d09c,red_shift)]:
                actual=get(addr+index,2);assert actual==half<<shift,(destination,source,hex(addr),actual)
                tables[addr][index:index+2]=struct.pack('<H',actual)
    outputs.append({'red_shift':red_shift,'green_shift':green_shift,'channel_pairs':1024,'tables':{hex(a):sha(b) for a,b in tables.items()}})
out={'status':'verified_bounded_static_integer_slice_all_channel_pairs','native_image':meta,'slice_va':hex(begin),'slice_end_exclusive':hex(end),'slice_sha256':sha(code),'instruction_count':len(program),'recognized_instructions':[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in program],'modes':outputs,'formula':'floor((destination5+source5)/2), placed into mode-specific channel bits','scope':'Interprets only the retained integer initializer slice; does not execute a PE, call float helpers, validate full raster integration, or compare original runtime pixels. Entry-loop indexing is observed separately in raw initializer.','original_executable_started':False,'runtime_memory_attached':False,'product_modified':False,'verifier_sha256':sha(Path(__file__).read_bytes())}
(stage/'flag1-channel-static.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:out[k] for k in ['status','instruction_count','formula']},ensure_ascii=False))