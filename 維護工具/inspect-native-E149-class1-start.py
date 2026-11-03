"""Resolve known graphic IDs through the original finite-start compare branches; bytes only."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,pefile,capstone
from native_E149_recovery_contract import R,sha,read,require,owned,encoded
V=R/'驗證紀錄/current'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only scope required');p=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(p)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','original PE changed');pe=pefile.PE(str(p),fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase;md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 def decode(start,end):
  raw=pe.get_data(start-base,end-start);rows=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,hex=i.bytes.hex()) for i in md.disasm(raw,start)];require(rows and rows[0]['address']==start and rows[-1]['address']+len(bytes.fromhex(rows[-1]['hex']))==end,'native range incomplete');return dict(start=start,end=end,sha256=hashlib.sha256(raw).hexdigest().upper(),instructions=rows)
 ranges=dict(start=decode(0x44f350,0x450028),S23_class1=decode(0x4a6234,0x4a6288),common_constructor=decode(0x44ede0,0x44eeb3));byaddr={row['address']:row for row in ranges['start']['instructions']};traces={}
 for graphic in [252,236,1801,1804,1805,2483,2484]:
  at=0x44f370;equal=None;trace=[]
  for _ in range(32):
   if at in [0x45001a,0x44fffb]:break
   row=byaddr[at];name,operand=row['mnemonic'],row['operands'];next_at=at+len(bytes.fromhex(row['hex']));entry=dict(row)
   if name=='mov' and operand=='ax, word ptr [esi + 0x14]':entry['known_graphic']=graphic
   elif name=='cmp' and operand.startswith('ax, '):equal=graphic==int(operand[4:],0);entry.update(known_graphic=graphic,equal=equal)
   elif name in ['je','jne']:
    require(equal is not None,'conditional branch before known compare');taken=equal if name=='je' else not equal;entry['taken']=taken
    if taken:next_at=int(operand,0)
   else:raise RuntimeError('unreviewed symbolic start instruction '+str(row))
   trace.append(entry);at=next_at
  require(at==(0x44fffb if graphic in [2483,2484] else 0x45001a),'original branch destination differs');traces[graphic]=dict(trace=trace,reached=at,two_step_offset_branch=at==0x44fffb,PE_executed=False)
 require(any(row['mnemonic']=='call' and row['operands']=='0x44ede0' for row in ranges['S23_class1']['instructions']),'S23 constructor changed');require(any(row['mnemonic']=='mov' and row['operands']=='word ptr [esi + 0x14], ax' for row in ranges['common_constructor']['instructions']),'graphic field binding changed')
 out=owned(V/'native-E149-class1-start-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'inspector.py').write_bytes(Path(__file__).read_bytes());record=dict(status='original_known_graphic_start_branch_trace_static_only',PE=str(p),PE_sha256=sha(p),source=str(Path(__file__).resolve()),source_sha256=sha(Path(__file__).resolve()),ranges=ranges,graphic_traces=traces,unshifted_current_profiles=[252,236,1801],unshifted_future_area_profiles=[1804,1805],two_step_special_profiles=[2483,2484],explanation='Prior class1 proof showed the 44FFFB two-step code exists but did not establish these IDs reach it. Their compare-chain path reaches 45001A directly.',tests_executed=0,PE_executed=False,GUI_verified=False);(out/'contract.json').write_bytes(encoded(record));print(json.dumps(dict(contract=str(out/'contract.json'),unshifted=[252,236,1801,1804,1805],special_two_step=[2483,2484],PE_executed=False),ensure_ascii=False));pe.close()
if __name__=='__main__':main()
