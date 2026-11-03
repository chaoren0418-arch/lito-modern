"""Reconstruct embedded bytes and audit raw loader semantics; no native code or tests run."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,struct
import capstone,pefile
from native_E149_recovery_contract import R,read,sha,encoded,owned,require,check_pins
V=R/'驗證紀錄/current'
def digest(data):return hashlib.sha256(data).hexdigest().upper()
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_fireball25_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'current source changed');q=read(ref['qualification']);check_pins(q['source_pins']);primary=Path(q['next_graphics_base_contract']);require(sha(primary)==q['next_graphics_base_contract_sha256'],'primary trace changed');old=read(primary);require(sha(old['PE'])==old['PE_sha256'],'original PE changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'],'development-only scope required')
 pe=pefile.PE(old['PE'],fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase
 def raw(address,size):
  require(0<size<=2*1024*1024,'static read size');data=pe.get_data(address-base,size);require(len(data)==size,'short original read');return data
 total,checksum=struct.unpack('<II',raw(0x5091b0,8));count=struct.unpack('<I',raw(0x62b818,4))[0];ptrs=struct.unpack('<10I',raw(0x62b7f0,40));chunk=(total+9)//10;parts=[];chunks=[]
 for n,address in enumerate(ptrs):
  size=chunk if n<9 else total-9*chunk;data=raw(address,size);parts.append(data);chunks.append(dict(address=address,bytes=size,sha256=digest(data)))
 payload=b''.join(parts);require(len(payload)==total==1189423 and count==5894,'embedded source scalars differ');table=raw(0x636380,512);words=struct.unpack('<256H',table);crc=0
 for byte in payload:crc=(((crc&255)<<8)^words[(crc>>8)&255]^byte)&0xffff
 require(crc==checksum==18638,'original embedded checksum differs')
 cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ranges={}
 for name,a,b in [('CRC16',0x4d3e00,0x4d3e3a),('embedded_loader',0x4d3ff0,0x4d416e),('class4_move',0x424540,0x424760),('class4_start',0x424780,0x424837),('class5_move',0x42f300,0x42f543)]:
  data=raw(a,b-a);rows=list(cs.disasm(data,a));require(sum(row.size for row in rows)==len(data),'incomplete original instructions');ranges[name]=dict(start=a,end=b,sha256=digest(data),instructions=[dict(address=row.address,hex=row.bytes.hex(),mnemonic=row.mnemonic,operands=row.op_str) for row in rows])
 instructions={row['address']:row for block in ranges.values() for row in block['instructions']}
 for address,mnemonic,operands in [(0x4d414c,'cmp','cl, 0xb3'),(0x4d414f,'jne','0x4d4143'),(0x4d4154,'cmp','cl, 0xb3'),(0x4d4162,'mov','dword ptr [ebx], edx'),(0x4246cd,'call','0x4e5b50'),(0x42473a,'call','dword ptr [edx + 4]'),(0x42f4cb,'call','dword ptr [edx + 4]'),(0x42f500,'call','0x4e5b50')]:
  require(instructions[address]['mnemonic']==mnemonic and instructions[address]['operands']==operands,'reviewed primary anchor differs')
 # Interpret only the bounded delimiter/data loop in Python. No instruction
 # emulator, executable mapping, native call or modification is performed.
 cursor=0;parsed=0;spans=[]
 while parsed<count:
  cursor+=1;start=cursor;require(cursor<len(payload),'original split cursor outside payload');marker=payload[cursor]
  if marker not in [0xb3,0xf9]:
   cursor=payload.find(b'\xb3',cursor+1);require(cursor>=0,'raw splitter has no terminal marker');marker=payload[cursor]
  spans.append(dict(start=start,end=cursor,bytes=cursor-start,sha256=digest(payload[start:cursor])));parsed+=1
  if marker==0xb3:break
  cursor+=0 # Native's next iteration increments once past this marker.
 positions=[n for n,b in enumerate(payload) if b==0xf9];terminal=[n for n,b in enumerate(payload) if b==0xb3]
 require(len(positions)==5894 and positions[0]==0 and terminal==[len(payload)-1] and parsed==1,'raw data/split discrepancy changed')
 # Literal code/data references are only a bounded locator, never proof that
 # indirect/self-modifying paths are absent.
 file_bytes=Path(old['PE']).read_bytes();patch_refs={hex(a):[m.start() for m in re.finditer(re.escape(struct.pack('<I',a)),file_bytes)] for a in [0x4d414c,0x4d414f,0x4d4154]};pe.close()
 lt=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/LTtext/row00240-LTlist.spr');lt_bytes=lt.read_bytes();require(len(lt_bytes)<2*1024*1024 and lt_bytes.splitlines()[0]==b'285','LT header differs');wanted={8,26,52,53,54,55,110,111,273,277};rows={}
 for line in lt_bytes.splitlines():
  m=re.match(rb'#(\d+)\b',line)
  if m and int(m[1]) in wanted:require(len(line)<=2048,'LT reference row bound');rows[str(int(m[1]))]=dict(raw_hex=line.hex(),sha256=digest(line))
 require(set(map(int,rows))==wanted,'LT family row absent')
 result=dict(status='embedded_checksum_matches_but_raw_splitter_conflicts_with_record_markers',source_qualification=ref,primary_trace=str(primary),primary_trace_sha256=sha(primary),PE=old['PE'],PE_sha256=old['PE_sha256'],analysis_source=str(Path(__file__).resolve()),analysis_source_sha256=sha(Path(__file__)),chunks=chunks,payload_bytes=total,payload_sha256=digest(payload),CRC_table=dict(address=0x636380,hex=table.hex(),sha256=digest(table)),original_checksum=checksum,reconstructed_checksum=crc,embedded_declared_records=count,marker_F9_count=len(positions),marker_positions_sha256=digest(b''.join(struct.pack('<I',n) for n in positions)),terminal_B3_offsets=terminal,raw_instruction_split_count=parsed,raw_instruction_spans=spans,ranges=ranges,literal_patch_address_references=patch_refs,literal_reference_search_excludes_indirect_patching=False,LT_reference=str(lt),LT_reference_sha256=sha(lt),LT_header_count=285,LT_rows=rows,shared_catalog=old['shared_catalog'],shared_catalog_sha256=old['shared_catalog_sha256'],shared_header=old['shared_catalog_header'],base_1_5894_or_11297_not_runtime_qualified=True,modern_graphics_binding_verified=False,class4_advances_flight_before_secondary=True,class5_starts_secondary_before_flight=True,tests_executed=0,native_instructions_executed=False,PE_executed=False,GUI_verified=False,limitation='The unchanged PE byte loop yields one span; this is a static contradiction, not evidence of actual runtime base=1. Initialization patching/provenance and LT-to-modern bindings remain unproved. Keep all earlier component scopes separate.')
 out=owned(V/'native-E149-loader-base-reconstruction'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'contract.json').write_bytes(encoded(result));print(json.dumps(dict(contract=str(out/'contract.json'),payload_sha256=digest(payload),checksum=crc,declared=count,F9_markers=len(positions),raw_split_count=parsed,modern_base_verified=False,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
