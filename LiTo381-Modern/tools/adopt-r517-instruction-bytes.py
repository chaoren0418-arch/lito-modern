from r491_common import *
p=L/'r517-inputs/native/instruction-witnesses.json';d=read(p)
root=read(L/'r517-inputs/D5-CF-receive-contract.json');pe=Path(root['sources'][0]['path']).read_bytes();w=[]
for v in d['assertions']:
 raw=bytes.fromhex(v['raw_hex']);at=int(v['file_offset'],16);assert pe[at:at+len(raw)]==raw
 w.append(dict(va=v['va'],file_offset=v['file_offset'],raw_hex=v['raw_hex'],matches=True))
save(L/'r517-instruction-byte-adoption.json',dict(input=pin(p),fixed_PE=root['sources'][0],witnesses=w,count=len(w),scope='Raw byte equality only. Mnemonic/operand assertions remain producer report, not independently executed disassembly tests or Native runtime.'))
print('Matched',len(w),'instruction raw witnesses to fixed PE; no instruction test/runtime calls')
