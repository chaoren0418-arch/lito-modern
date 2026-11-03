from r491_common import *
import shutil
root=P/'target/r502-documentation-draft';root.mkdir(exist_ok=False);pins=[]
for p in [P/'tools/map-r502-findings.py',L/'r502-entry-callback-event-mapping.json']:
 d=root/p.name;shutil.copyfile(p,d);pins.append(pin(d))
study=read(L/'r502-source-study.json');q=study['PE_values'][0];assert q['address']=='0x5033c4'and q['section']['writable']and q['section']['flags']=='0xe0000060'
p=P/'tools/map-r502-findings.py';s=p.read_text(encoding='utf-8').replace('5033C4 PEconstant114F8 inread-only.rdata andfiniteASMno directwrites','5033C4 PEinitial114F8 inrecoveredwritableexecutable section (flagsE0000060);finiteASMno directwrites');p.write_text(s,encoding='utf-8')
save(L/'r502-documentation-correction.json',dict(state='R502_DRAFT_SECTION_DESCRIPTION_CORRECTED_BEFORE_VERIFICATION',preserved_draft=pins,actual_PE_section=q['section'],correction='Draft prose assumedread-only.rdata; actualrecoveredPEsectionwritable/executable. Source-study sectionmetadata alwayscorrect. NoReadonly/currentversionauthorityclaim;Nativecurrent fields remainunqualified;production/consumer unchanged,no rerun.'))
print('R502 draft PEsection prose corrected to writable/executable;currentrevision stillunqualified')
