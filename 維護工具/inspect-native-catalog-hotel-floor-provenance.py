"""Independently locate original S32/TIL writers for sampled gray hotel pixels."""
from pathlib import Path
from datetime import datetime,timezone
from PIL import Image
import argparse,hashlib,json,struct,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';P=R/'LiTo381-Modern';A=Path(r'D:\lineage開發\0-客戶端')
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('hotel_run',type=Path);a=ap.parse_args();run=a.hotel_run.resolve(strict=True);assert run.is_relative_to(V/'native-catalog-hotel')
g=read(run/'cohort.json');q=read(run/'qualified.json');assert q['cohort_sha256']==sha(run/'cohort.json')and q['actual_formal_sessions']==32
baseline=Path(g['normal_cohort']);inputs=read(baseline/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(R/n)==h,n
for profile,h in g['normal_mains'].items():assert sha(P/('target/current/x86_64-pc-windows-msvc/'+profile.lower()+'/LiToModern.exe'))==h
case=Path(g['cases']['Debug-rental-hotel-entry']['directory']);receipt=read(case/'receipt.json');h=receipt['clients']['b']['hotel'];world=h['arrival']['world'];frame=h['room_render']['frame'];picture=Path(h['room_render']['png']);own=world['actors'][str(frame['local_object_id'])]
assert world['map_id']==frame['map_id']==501 and frame['view_position']==[own['x'],own['y']]
with Image.open(picture)as im:
    assert im.mode=='RGBA'and im.size==(800,600);im.load();assert hashlib.sha256(im.tobytes()).hexdigest().upper()==frame['rgba_sha256'].upper();rgba=im.tobytes()
files=sorted((A/'LiTo/LTmap/501').glob('????????.s32'));assert len(files)==1
first=(int(files[0].stem[:4],16),int(files[0].stem[4:],16));origin=tuple(32704+(n-0x7fff)*64 for n in first)
px=24+24*(own['x']-origin[0]+own['y']-origin[1]);py=768+12*(-own['x']+origin[0]+own['y']-origin[1]);camera=[px-400,py-300]
draws=[];source_pins={};source_bytes={}
for f in files:
    b=f.read_bytes();source_pins[str(f)]=sha(f);source_bytes['original/'+f.name]=b;sx,sy=int(f.stem[:4],16),int(f.stem[4:],16);at=0
    def take(fmt):
        global at
        size=struct.calcsize(fmt);assert at+size<=len(b);value=struct.unpack_from(fmt,b,at);at+=size;return value
    def append(x,y,tile,slot,kind,offset):
        draws.append(dict(x=x,y=y,tile=tile,slot=slot,kind=kind,source=str(f),source_offset=offset,projected=[24*((x+1)//2+y)+1536*(sx-first[0]+sy-first[1])-camera[0],12*(63-x//2+y)+768*(-sx+first[0]+sy-first[1])-camera[1]]))
    for i in range(128*64):
        offset=at;slot,tile,unused=take('<BHB');append(i%128,i//128,tile,slot,'floor',offset)
    count,=take('<H')
    for _ in range(count):
        offset=at;x,y,slot,tile,unused=take('<BBBHB');append(x,y,tile,slot,'overlay',offset)
    assert at+64*64*4<=len(b);at+=64*64*4;groups,=take('<i');assert 0<=groups<=65536
    for _ in range(groups):
        group,count=take('<hH')
        for _ in range(count):
            offset=at;x,y,layer,slot,tile,unused=take('<BBBBhB');assert tile>=0;append(x,y,tile,slot,'object',offset)
index=A/'LiTo/LTtile.idx';pak=A/'LiTo/LTtile.pak';raw_index=index.read_bytes();count=int.from_bytes(raw_index[:4],'little');assert len(raw_index)==4+count*28
rows=[raw_index[i:i+28]for i in range(4,len(raw_index),28)];entry=next(x for x in rows if x[4:24].split(b'\0',1)[0].lower()==b'0.til');offset=int.from_bytes(entry[:4],'little');size=int.from_bytes(entry[24:],'little');assert 0<size<1024*1024
with pak.open('rb')as f:f.seek(offset);member=f.read(size)
assert len(member)==size;blocks=int.from_bytes(member[:4],'little');start=4+4*(blocks+1);slots={}
for slot in [0,1]:
    left,right=struct.unpack_from('<II',member,4+slot*4);block=member[start+left:start+right];assert len(block)==577 and block[0]==slot and block[1:]==bytes.fromhex('4A29')*288
    slots[slot]=dict(block_sha256=hashlib.sha256(block).hexdigest().upper(),kind=slot,word='294A',channel5=[10,10,10],RGBA=[82,82,82,255])
object_blocks={}
def object_coverage(d,u,v):
    key=(d['tile'],d['slot'])
    if key not in object_blocks:
        row=next(x for x in rows if x[4:24].split(b'\0',1)[0].lower()==(str(key[0])+'.til').encode())
        entry_offset=int.from_bytes(row[:4],'little');entry_size=int.from_bytes(row[24:],'little');assert 0<entry_size<=16*1024*1024
        with pak.open('rb')as f:f.seek(entry_offset);entry_bytes=f.read(entry_size)
        assert len(entry_bytes)==entry_size;count=int.from_bytes(entry_bytes[:4],'little');assert 0<=key[1]<count<=65536
        base=4+4*(count+1);left,right=struct.unpack_from('<II',entry_bytes,4+key[1]*4);assert 0<=left<right<=len(entry_bytes)-base
        block=entry_bytes[base+left:base+right];assert block[0]in [2,3]and len(block)>=5
        xo,yo,w,h=block[1:5];assert xo+w<=24 and yo+h<=24;at=5;coverage={}
        for yy in range(yo,yo+h):
            segments=block[at];at+=1;xx=xo
            for _ in range(segments):
                skip,n=block[at:at+2];at+=2;assert skip%2==0;xx+=skip//2;assert xx+n<=xo+w and at+2*n<=len(block)
                for i in range(n):coverage[xx+i,yy]=int.from_bytes(block[at+2*i:at+2*i+2],'little')
                at+=2*n;xx+=n
        assert at==len(block)
        name=f'original/{key[0]}.til-slot{key[1]}.bin';source_bytes[name]=block
        object_blocks[key]=dict(coverage=coverage,record=dict(tile=key[0],slot=key[1],kind=block[0],rectangle=[xo,yo,w,h],member_offset=entry_offset,member_bytes=entry_size,member_sha256=hashlib.sha256(entry_bytes).hexdigest().upper(),block_offset_in_member=base+left,block_bytes=len(block),block_sha256=hashlib.sha256(block).hexdigest().upper()))
    item=object_blocks[key];word=item['coverage'].get((u,v));assert word is None,'A later object writes this sample; do not attribute it solely to its floor'
    return dict(**item['record'],local_pixel=[u,v],encoded_coverage=False)
samples=[]
for x,y in [(0,0),(399,0),(0,300),(0,599),(15,35),(599,35)]:
    actual=list(rgba[(y*800+x)*4:(y*800+x)*4+4]);assert actual==[82,82,82,255]
    candidates=[];possible_later=[]
    for d in draws:
        u,v=x-d['projected'][0],y-d['projected'][1]
        if not(0<=u<24 and 0<=v<24):continue
        if d['kind']!='floor':possible_later.append(d);continue
        length=2*(v+1)if v<12 else 2*(23-v);left=24-length if d['x']%2==0 else 0
        if not(left<=u<left+length):continue
        assert d['tile']==0 and d['slot']in slots
        candidates.append(dict(**d,local_pixel=[u,v],source_reference=list(source_bytes['original/'+Path(d['source']).name][d['source_offset']:d['source_offset']+4]),TIL=slots[d['slot']]))
    assert len(candidates)==1,('floor writer not isolated',x,y,candidates)
    transparent=[object_coverage(d,x-d['projected'][0],y-d['projected'][1])for d in possible_later]
    samples.append(dict(screen=[x,y],RGBA=actual,original_floor_writer=candidates[0],other_terrain_rectangles_covering_sample=len(possible_later),transparent_object_candidates=transparent,other_terrain_writes_at_sample=0))
out=V/'native-catalog-hotel-floor-provenance'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
source_pins[str(index)]=sha(index);source_pins[str(pak)]=sha(pak)
for n,hsh in source_pins.items():assert sha(n)==hsh
source_bytes['original/0.til']=member;source_bytes['original/0.til-index-row.bin']=entry;source_bytes['actual/map501.png']=picture.read_bytes();source_bytes['actual/frame.json']=(json.dumps(frame,ensure_ascii=False,indent=2)+'\n').encode('utf-8');source_bytes['inspector.py']=Path(__file__).read_bytes()
pins={n:hashlib.sha256(b).hexdigest().upper()for n,b in source_bytes.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,b in source_bytes.items():z.writestr(n,b)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==v for n,v in pins.items())
result=dict(status='current_map501_sampled_gray_pixels_have_original_floor_writers',source_manifest_sha256=g['source_manifest_sha256'],normal_mains=g['normal_mains'],hotel_qualification=str(run/'qualified.json'),hotel_qualification_sha256=sha(run/'qualified.json'),PNG=str(picture),PNG_sha256=sha(picture),map_id=501,view_position=frame['view_position'],camera=camera,original_sector_origin=origin,original_resource_pins=source_pins,original_member=dict(name='0.til',offset=offset,bytes=size,sha256=hashlib.sha256(member).hexdigest().upper()),samples=samples,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),scope='Six sampled pixels match isolated original floor references and original 0.til slots0/1 RGB555 294A. They are not untouched canvas pixels; this does not prove all camera bounds, all pixels or normal GUI.',normal_source_modified=False,normal_GUI_verified=False,all_maps_framebuffer_verified=False,actual_new_sessions=0,whole_goal_complete=False)
save(out/'readback.json',result);print(json.dumps(dict(readback=str(out/'readback.json'),sampled_original_floor_writers=len(samples),actual_new_sessions=0,normal_GUI_verified=False),ensure_ascii=False))
