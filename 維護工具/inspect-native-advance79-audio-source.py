"""Disassemble fixed PE bytes and inspect WAV metadata; never load/run/play them."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct
import capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require
V=R/'驗證紀錄/current'
def main():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'offline development only')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 binary=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(binary)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','fixed original PE changed')
 blob=binary.read_bytes();pe=struct.unpack_from('<I',blob,0x3c)[0]
 require(blob[pe:pe+4]==b'PE\0\0' and struct.unpack_from('<H',blob,pe+4)[0]==0x14c and struct.unpack_from('<H',blob,pe+24)[0]==0x10b,'original x86 PE expected')
 count=struct.unpack_from('<H',blob,pe+6)[0];optional=struct.unpack_from('<H',blob,pe+20)[0];base=struct.unpack_from('<I',blob,pe+52)[0]
 sections=[]
 for index in range(count):
  at=pe+24+optional+index*40;virtual_size,rva,size,offset=struct.unpack_from('<IIII',blob,at+8)
  sections.append(dict(name=blob[at:at+8].rstrip(b'\0').decode('ascii'),rva=rva,virtual_size=virtual_size,bytes=size,offset=offset))
 def raw(address,length):
  rva=address-base;matches=[row for row in sections if row['rva']<=rva and rva+length<=row['rva']+row['bytes']]
  require(len(matches)==1,'requested original code range not mapped');row=matches[0];offset=row['offset']+rva-row['rva'];return blob[offset:offset+length]
 out=owned(V/'native-advance79-original-audio-source'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 engine=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ranges={}
 for name,start,end in [('catalog_parser_43DE10',0x43DE10,0x43EB80),('frame_dispatch_4E5C00',0x4E5C00,0x4E63A0),('class_dispatch_4A5ED0',0x4A5ED0,0x4A63C4),('audio_voice_4C77B0',0x4C77B0,0x4C7E60)]:
  source=raw(start,end-start);rows=[dict(address=f'{instruction.address:08X}',bytes=instruction.bytes.hex(),mnemonic=instruction.mnemonic,operands=instruction.op_str) for instruction in engine.disasm(source,start)]
  (out/(name+'.bytes')).write_bytes(source);(out/(name+'.json')).write_bytes(encoded(rows))
  (out/(name+'.asm')).write_text('\n'.join(row['address']+'  '+row['bytes'].ljust(20)+'  '+row['mnemonic']+' '+row['operands'] for row in rows)+'\n',encoding='utf-8')
  ranges[name]=dict(start=start,end=end,sha256=sha(out/(name+'.bytes')),decoded_instructions=len(rows))
 wave=Path('D:/lineage開發/0-客戶端/sound/1356.wav');require(wave.is_file() and wave.stat().st_size<=16*1024*1024,'bounded shared1356 WAV absent')
 sound=wave.read_bytes();require(sound[:4]==b'RIFF' and sound[8:12]==b'WAVE','1356 is not a WAV')
 length=struct.unpack_from('<I',sound,4)[0]+8;require(length<=len(sound),'truncated original WAV');at=12;chunks=[]
 while at<length:
  require(at+8<=length,'truncated WAV chunk');tag,size=struct.unpack_from('<4sI',sound,at);start=at+8;require(start+size<=length,'WAV chunk outside original source')
  chunks.append(dict(tag=tag.decode('ascii'),offset=start,bytes=size));at=start+size+(size&1)
 fmt=[row for row in chunks if row['tag']=='fmt '];data=[row for row in chunks if row['tag']=='data'];require(len(fmt)==len(data)==1 and fmt[0]['bytes']>=16,'WAV format/data shape differs')
 code,channels,rate,byte_rate,align,bits=struct.unpack_from('<HHIIHH',sound,fmt[0]['offset']);require(code==1 and channels in [1,2] and bits in [8,16] and align==channels*bits//8 and byte_rate==rate*align,'original sound is not the supported PCM layout')
 record=dict(status='original_PE_disassembly_and_shared1356_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  original_PE=str(binary),original_PE_sha256=sha(binary),PE_image_base=base,sections=sections,code_ranges=ranges,
  capstone_version=capstone.__version__,sound=dict(path=str(wave),sha256=sha(wave),bytes=len(sound),riff_declared_bytes=length,
   format_tag=code,channels=channels,rate=rate,byte_rate=byte_rate,block_align=align,bits_per_sample=bits,data_bytes=data[0]['bytes'],chunks=chunks),
  linked_current_sources={str(P/'src/modern_lito'/name):sha(P/'src/modern_lito'/name) for name in ['combat_animation.rs','native_skill_effects.rs','native_audio_delivery.rs','native_selector_effects.rs']},
  PE_executions=0,WAV_playbacks=0,tests_executed=0,desktop_inputs=0,whole_goal_complete=False)
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());(out/'inspection.json').write_bytes(encoded(record))
 print(json.dumps(dict(inspection=str(out/'inspection.json'),original_PE_executions=0,WAV_playbacks=0,tests_executed=0,sound_format=[channels,rate,bits]),ensure_ascii=False))
if __name__=='__main__':main()
