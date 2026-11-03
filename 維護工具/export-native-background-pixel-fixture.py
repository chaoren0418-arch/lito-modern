"""Export numeric LUT words/frame hashes only; never original instructions."""
from pathlib import Path
import csv,hashlib,json

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
q=read(APP/'background-blend-qualified-current.json');source=Path(q['path'])
assert q['channel_words']==135168 and q['pixel_cases']==1584 and q['all_archives_readback_verified']
blob=bytearray();blocks=[]
for mode in [0,1]:
    state=(source/f'initialized-mode{mode}.bin').read_bytes()
    for kind,bases in enumerate([[0x81089c,0x81109c,0x81189c],[0x80d89c,0x80e09c,0x80e89c],[0x82109c,0x82609c,0x82b09c],[0x81209c,0x81709c,0x81c09c]]):
        for channel,(address,shift) in enumerate(zip(bases,[0,6 if mode else 5,11 if mode else 10])):
            words=1024*(10 if kind>=2 else 1);part=state[address-0x630000:address-0x630000+words*2]
            assert len(part)==words*2
            blocks.append({'format':mode,'blend_mode':kind%2,'strengths':list(range(10)) if kind>=2 else [32],
                           'channel':channel,'shift':shift,'offset':len(blob),'words':words,'sha256':digest(part)})
            blob+=part
assert len(blob)==270336
with (source/'cases.csv').open(encoding='ascii',newline='') as f:r=csv.reader(f);columns=next(r);params=[[int(v) for v in row] for row in r]
vectors=read(source/'full-frame-vectors.json')['vectors'];assert len(params)==len(vectors)==1584
fixture={'format':'LITO_BACKGROUND_PIXELS_ORIGINAL1','native_qualification_sha256':digest((source/'qualified.json').read_bytes()),
 'original_PE_sha256':read(source/'preparation.json')['original_PE_sha256'],'host_CRT_sha256':q['host_CRT_sha256'],
 'historical_CRT_DLL_identity_verified':False,'original_raw_archive_sha256':q['raw_archive_sha256'],
 'numeric_LUT_sha256':digest(blob),'native_channel_words':135168,'table_blocks':blocks,'case_columns':columns,
 'height':552,'gate_pitch':744,'initial_word':0x1777,'base_dimensions':[32,32],'overlay_dimensions':[64,64],
 'source_model':'base RGB=(x,y,x+3y) mod32; overlay RGB=(5x+3y,7x+y,x+11y) mod32; RGB565 also sets low green bit on odd x+y.',
 'cases':[{'params':p,**v} for p,v in zip(params,vectors)],'real_world_GUI_verified':False,'whole_goal_complete':False}
folder=ROOT/'LiTo381-Modern/tests/fixtures';jp=folder/'native-background-pixels-original.json';bp=folder/'native-background-channels-original.bin'
assert not jp.exists() and not bp.exists()
bp.write_bytes(blob);jp.write_text(json.dumps(fixture,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8')
assert bp.read_bytes()==blob and read(jp)==fixture
receipt={'status':'numeric_background_fixtures_exported_not_yet_product_tested','original_qualification':str(source/'qualified.json'),
 'JSON_bytes':jp.stat().st_size,'JSON_sha256':digest(jp.read_bytes()),'LUT_bytes':bp.stat().st_size,'LUT_sha256':digest(bp.read_bytes()),
 'no_original_instructions_or_native_addresses_in_product_fixture':True,'whole_goal_complete':False}
(source/'product-fixture-export.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(receipt,ensure_ascii=False))
