"""Bounded read-only PE instructions/data for the unresolved original gfx-base binding."""
from datetime import datetime,timezone
from pathlib import Path
import hashlib,json,struct
import capstone,pefile
from native_E149_recovery_contract import R,sha,read,encoded,owned,require
V=R/'驗證紀錄/current'
def main():
 previous=V/'native-E149-fireball25-cycle-contract/20261024-053504-604374/contract.json';old=read(previous);require(sha(old['PE'])==old['PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','original PE changed');pe=pefile.PE(old['PE'],fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase;cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ranges={}
 for name,start,end in [('sprite_parser_base_writer',0x43de10,0x43de8d),('embedded_list_loader',0x4d3ff0,0x4d416e),('no_hurt_family',0x4df7c0,0x4df818)]:
  raw=pe.get_data(start-base,end-start);rows=list(cs.disasm(raw,start));require(sum(row.size for row in rows)==len(raw),'incomplete static decode: '+name);ranges[name]=dict(start=start,end=end,sha256=hashlib.sha256(raw).hexdigest().upper(),instructions=[dict(address=row.address,hex=row.bytes.hex(),mnemonic=row.mnemonic,operands=row.op_str) for row in rows])
 data={}
 for name,address,size in [('relative_no_hurt_table',0x636940,18),('embedded_record_count_literal',0x62b818,4),('embedded_payload_bytes',0x5091b0,4),('embedded_checksum',0x5091b4,4),('snapshot_uninitialized_base',0x647e90,4),('list_name',0x505d7c,9),('LTlist_name',0x508804,11)]:
  blob=pe.get_data(address-base,size);require(len(blob)==size,'data range absent');data[name]=dict(address=address,hex=blob.hex(),sha256=hashlib.sha256(blob).hexdigest().upper())
 pe.close();count=struct.unpack('<I',bytes.fromhex(data['embedded_record_count_literal']['hex']))[0];relative=list(struct.unpack('<9H',bytes.fromhex(data['relative_no_hurt_table']['hex'])));require(count==5894 and relative==[8,52,53,54,55,110,111,273,277],'primary literals differ')
 spr=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(spr)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','shared original changed')
 with spr.open('rb') as f:header=f.readline().strip().decode('ascii')
 require(header=='11297 0 41210','shared catalog header differs')
 result=dict(status='static_original_graphics_base_loader_trace_not_yet_modern_bound',PE=old['PE'],PE_sha256=old['PE_sha256'],previous_contract=str(previous),previous_contract_sha256=sha(previous),inspection_source=str(Path(__file__).resolve()),inspection_source_sha256=sha(Path(__file__)),ranges=ranges,data=data,relative_no_hurt_offsets=relative,embedded_declared_record_count=count,embedded_actual_split_record_count_proven=False,loader_final_count_written_from_split_loop=True,shared_catalog=str(spr),shared_catalog_sha256=sha(spr),shared_catalog_header=header,shared_catalog_count_must_not_be_substituted_for_original_base=True,modern_graphics_binding_verified=False,tests_executed=0,PE_executed=False,GUI_verified=False,limitation='4D3FF0 uses embedded chunks and a checked payload; 4D4162 stores actual split count, which still needs decoding/record-boundary proof and mapping to modern catalog aliases.')
 out=owned(V/'native-E149-graphics-base-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'contract.json').write_bytes(encoded(result));print(json.dumps(dict(contract=str(out/'contract.json'),embedded_declared_count=count,shared_header=header,modern_binding_verified=False,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
