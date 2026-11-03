"""Static original758 start-group evidence; never run the recovered PE."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,pefile,capstone
from native_E149_recovery_contract import R,sha,encoded,require,read,owned
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only scope required');p=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(p)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','original PE differs');pe=pefile.PE(str(p),fast_load=True);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);base=pe.OPTIONAL_HEADER.ImageBase
 def block(start,end):
  raw=pe.get_data(start-base,end-start);rows=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,hex=i.bytes.hex()) for i in md.disasm(raw,start)];require(rows and rows[0]['address']==start and rows[-1]['address']+len(bytes.fromhex(rows[-1]['hex']))==end,'native range not fully decoded');return dict(start=start,end=end,sha256=hashlib.sha256(raw).hexdigest().upper(),instructions=rows)
 ranges=dict(start758=block(0x44f350,0x44f4e7),direction_table=block(0x4b8f40,0x4b9000),direction_step=block(0x4b9390,0x4b93d0));rows=ranges['start758']['instructions'];pairs={(row['address'],row['mnemonic'],row['operands']) for row in rows};require((0x44f388,'cmp','ax, 0x2f6') in pairs and (0x44f38c,'jne','0x44f4e7') in pairs,'758 branch differs')
 require([row['address'] for row in rows if row['mnemonic']=='call' and row['operands']=='0x44ede0']==[0x44f3d4,0x44f42c,0x44f484],'three child constructors differ')
 calls=[row['address'] for row in rows if row['mnemonic']=='call' and row['operands']=='0x4b9390'];require(calls==[0x44f3ba,0x44f412,0x44f46a,0x44f4a3],'four original position calculations differ')
 directions=[]
 for address in calls:
  i=next(i for i,row in enumerate(rows) if row['address']==address);pushes=[row['operands'] for row in rows[max(0,i-9):i] if row['mnemonic']=='push'];require(len(pushes)>=3 and pushes[-3]=='2','original distance differs');directions.append(int(pushes[-2],0))
 require(directions==[3,7,5,1],'original direction order differs')
 regs={};writes={}
 for row in ranges['direction_table']['instructions']:
  if row['mnemonic']=='ret':break
  parts=row['operands'].split(', ')
  if row['mnemonic']=='xor' and parts[0]==parts[1]:regs[parts[0]]=0
  if row['mnemonic']=='mov':
   dest,source=parts;value=regs[source] if source in regs else int(source,0)
   if dest.startswith('dword ptr [0x'):writes[int(dest[11:-1],0)]=value&0xffffffff
   else:regs[dest]=value
 signed=lambda value:value if value<0x80000000 else value-0x100000000
 native=[(signed(writes[0x768828+8*n]),signed(writes[0x76882c+8*n])) for n in range(8)];require(native==[(0,-1),(2,-1),(2,0),(2,1),(0,1),(-2,1),(-2,0),(-2,-1)],'native direction table differs');world_offsets=[[native[n][0],native[n][1]*2] for n in directions];require(world_offsets==[[2,2],[-2,-2],[-2,2],[2,-2]],'native doubled-X conversion differs')
 out=owned(V/'native-E149-tornado-group-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'inspector.py').write_bytes(Path(__file__).read_bytes());record=dict(status='original758_four_copy_start_group_static_only',original_PE=str(p),original_PE_sha256=sha(p),source=str(Path(__file__).resolve()),source_sha256=sha(Path(__file__).resolve()),ranges=ranges,directions=directions,native_distance=2,native_direction_table=native,world_offsets=world_offsets,projected_pixel_offsets=[[96,0],[-96,0],[0,48],[0,-48]],native_doubled_X_conversion='distance2 * nativeX / 2, distance2 * nativeY',three_child_copies_plus_original=4,one_cast_target_list=True,PE_executed=False,tests_executed=0,GUI_verified=False);(out/'contract.json').write_bytes(encoded(record));print(json.dumps(dict(contract=str(out/'contract.json'),world_offsets=world_offsets,PE_executed=False),ensure_ascii=False));pe.close()
if __name__=='__main__':main()
