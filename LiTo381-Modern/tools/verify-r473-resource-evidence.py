from pathlib import Path
import json,hashlib,struct,collections,re,sys
project=Path.cwd();logs=project/'logs/continuation-20261003'
inv=json.loads((logs/'r473-complete-campaign-inventory.json').read_text(encoding='utf-8'))
def check(pin):
 b=Path(pin['path']).read_bytes();assert len(b)==pin['bytes'] and hashlib.sha256(b).hexdigest().upper()==pin['sha256'].upper();return b
verified=0
for pair in inv['pairs']:
 idx=check(pair['decoded_index']);count=struct.unpack('<I',idx[:4])[0];rows=[line.split('\t') for line in check(pair['complete_inventory']).decode('utf-8').splitlines()];assert count==len(rows)==pair['index_count'];pak=check(pair['archives'][1]['snapshot']);actual=[]
 for i,row in enumerate(rows):
  n=idx[4+i*28:4+(i+1)*28];off=struct.unpack('<I',n[:4])[0];size=struct.unpack('<I',n[24:])[0];name=n[4:24].split(b'\0')[0].decode('utf-8');assert [str(i),name,str(off),str(size)]==row[:4];assert off+size<=len(pak);actual.append(name.lower())
 groups=collections.Counter(actual);duplicates=sum(v-1 for v in groups.values());assert duplicates==pair['duplicate_rows'];expected={n:c for n,c in groups.items() if c>1};stored={g['name'].lower():len(g['records']) for g in pair['duplicate_groups']};assert expected==stored
 for m in pair['campaign_members']:
  raw=check(m['raw']);decoded=check(m['decoded']);assert raw==pak[m['offset']:m['offset']+m['bytes']];assert rows[m['record_index']][1]==m['name'];assert groups[m['name'].lower()]==1
 verified+=count
asm=json.loads((logs/'r473-original-source-pins.json').read_text(encoding='utf-8'));table=next(p for p in asm['fragments'] if Path(p['path']).name=='language-switch-tables.asm-fragment');text=check(table).decode('utf-8');values=[]
for line in text.splitlines():
 code=line.split(';')[0]
 m=re.search(r'\bdb\s+([0-9 ,]+)',code)
 if m:values.extend(int(v.strip()) for v in m.group(1).split(','))
assert values==[0,4,4,4,1,4,4,4,4,4,4,4,4,4,4,4,4,2,3]
targets=['j','h','k','c','e'];switch=[{'code':932+i,'table_index':v,'language':targets[v]} for i,v in enumerate(values)]
variants=[]
enc={'c':'cp950','e':'ascii','h':'gbk','j':'cp932','k':'cp949'}
for m in inv['pairs'][0]['campaign_members']:
 b=check(m['decoded']);lines=b.replace(b'\r',b'').split(b'\n');count=int(lines[0]);parsed={int(line.split(b' ',1)[0]):line.split(b' ',1)[1] for line in lines[1:] if line};assert len(parsed)==count and sorted(parsed)==list(range(count));letter=m['name'][13];nums={str(i):int(parsed[i]) for i in [0,1,4,10]};assert nums=={'0':3600,'1':600,'4':5,'10':3000};other=next(n for n in inv['pairs'][1]['campaign_members'] if n['name']==m['name']);assert m['raw']['sha256']==other['raw']['sha256'] and m['decoded']['sha256']==other['decoded']['sha256'];variants.append({'member':m['name'],'count':count,'base381_record':m['record_index'],'base381_offset':m['offset'],'existing_client_record':other['record_index'],'existing_client_offset':other['offset'],'numeric_indices':nums,'initializer_divisor':count-nums['4']-7,'conditional_delays_ms':{'47CEC0_to_47CDF0_index1':600000,'47CDF0_to_47CEC0_index0_minus1':3000000,'47CE50_repeat_index4plus5':3000000},'indexed_rows':[{'index':i,'byte_hex':parsed[i].hex(),'diagnostic_text':parsed[i].decode(enc[letter])} for i in parsed],'diagnostic_encoding':enc[letter],'diagnostic_encoding_not_runtime_codepage_grant':True,'raw':m['raw'],'decoded':m['decoded']})
print(json.dumps({'state':'ALL75140_DECODED_INDEX_LEDGER_ROWS_AND_RAW_CAMPAIGN_SLICES_INDEPENDENTLY_VERIFIED','verified_rows':verified,'archive_pairs':3,'campaign_members':10,'unique_variants':5,'language_source_switch':switch,'default_language':'e','gate_codes':[949,1252],'gate_languages':['k','e'],'variants':variants,'raw_and_decoded_base_versions_identical':True,'native_callbacks_executed':False,'runtime_inputs_observed':False,'all_remaining_R472_cells_unexecuted':True},ensure_ascii=True))
