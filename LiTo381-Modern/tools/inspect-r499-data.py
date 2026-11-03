from r491_common import *
import struct,shutil
v=read(L/'r499-source-study.json');b=Path(v['analysis_PE']['path']).read_bytes();o=P/'target/r499-back-login-study/verification-attempt1';o.mkdir(exist_ok=False);shutil.copyfile(P/'tools/verify-r499-segment.py',o/'verify-r499-segment.py')
save(o/'failure.json',dict(state='VERIFIER_SOURCE_METADATA_ASSERT_FAILED_AFTER_CONSUMER_PASS_NO_RERUN',line=39,expression='PEraw636584==3B6 and5033C4==114F8',source=pin(o/'verify-r499-segment.py'),consumer=pin(L/'r499-consumer-exit.json')))
offset=struct.unpack_from('<I',b,0x3c)[0];count=struct.unpack_from('<H',b,offset+6)[0];size=struct.unpack_from('<H',b,offset+20)[0];sect=offset+24+size
sections=[]
for i in range(count):
 p=sect+40*i;name=b[p:p+8].split(b'\0')[0].decode();virtualsize,rva,rawsize,raw=struct.unpack_from('<4I',b,p+8);sections.append(dict(name=name,rva=rva,virtualsize=virtualsize,rawsize=rawsize,raw=raw))
values=[]
for va in [0x636584,0x5033C4]:
 rva=va-0x400000;s=next(s for s in sections if s['rva']<=rva<s['rva']+max(s['virtualsize'],s['rawsize']));off=s['raw']+rva-s['rva'];values.append(dict(va=hex(va),direct=hex(struct.unpack_from('<I',b,rva)[0]),mapped=hex(struct.unpack_from('<I',b,off)[0]),offset=hex(off),section=s))
save(L/'r499-initial-field-inspection.json',dict(state='PINNED_PE_SECTION_AND_INITIAL_FIELD_INSPECTION',source=v['analysis_PE'],values=values,sections=sections));print(json.dumps(values))
