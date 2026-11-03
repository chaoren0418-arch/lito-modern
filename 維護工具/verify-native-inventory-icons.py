"""Independent bounded TBT/LBT span reference and native 32px cell comparison."""
import argparse,hashlib,json,struct
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
SOURCE=Path(r'D:\lineage開發\0-客戶端')
FIXTURE=ROOT/'LiTo381-Modern/tests/fixtures/native-inventory-icons-current.json'
SAMPLES=[0,80,235,20000,20001,20147]
def sha(b):return hashlib.sha256(b).hexdigest().upper()
indexes={}
def member(stem,name):
 if stem not in indexes:
  b=(SOURCE/(stem+'.idx')).read_bytes();assert len(b)==4+struct.unpack_from('<I',b)[0]*28
  rows={}
  for at in range(4,len(b),28):
   row=b[at:at+28];key=row[4:24].split(b'\0',1)[0].lower();offset,size=struct.unpack_from('<I',row)[0],struct.unpack_from('<I',row,24)[0]
   assert key not in rows;rows[key]=(offset,size)
  indexes[stem]=rows
 row=indexes[stem].get(name.lower().encode('ascii'))
 if row is None:return None
 offset,size=row;assert 0<size<=256*1024
 with (SOURCE/(stem+'.pak')).open('rb') as f:f.seek(offset);b=f.read(size)
 assert len(b)==size;return b
def load(id):
 assert 0<=id<32768
 name=f'{id}.tbt' if id<20000 else f'{id-20000}.lbt'
 route=([f'Sprite{sum(name.encode())%16:02}'] if id<20000 else [])+['LiTo/LTicon','Sprite']
 for stem in route:
  b=member(stem,name)
  if b is not None:break
 else:raise AssertionError(f'missing {name}')
 ox,oy=struct.unpack_from('bb',b);width,height=b[2:4];assert width>0 and 0<height<=127
 at=4;covered=zero=0;cell=bytearray(32*32*4)
 for y in range(height):
  spans=b[at];at+=1;x=0
  for _ in range(spans):
   skip,count=b[at:at+2];at+=2;assert skip%2==0;x+=skip//2;width=max(width,x+count);assert width<=512
   for xx in range(x,x+count):
    v=struct.unpack_from('<H',b,at)[0];at+=2;assert v<32768;covered+=1;zero+=v==0
    dx,dy=xx+ox,y+oy
    if 0<=dx<32 and 0<=dy<32:
     channels=[(v>>n)&31 for n in [10,5,0]];pixel=bytes([(c<<3)|(c>>2) for c in channels]+[255]);start=(dy*32+dx)*4;cell[start:start+4]=pixel
   x+=count
 assert at==len(b)
 return {'id':id,'source':{'archive':stem,'member':name,'bytes':len(b),'sha256':sha(b)},'offset':[ox,oy],'width':width,'height':height,'covered_pixels':covered,'covered_zero':zero,'cell_sha256':sha(cell)},bytes(cell)
def collect():
 samples=[load(id)[0] for id in SAMPLES]
 count=member('LiTo/LTicon','LTlist.ico');assert count==b'233\r\n'
 ids=sorted(int(name[:-4])+20000 for name in indexes['LiTo/LTicon'] if name.endswith(b'.lbt'))
 rows=[load(id)[0] for id in ids];aggregate=sha(''.join(f"{row['id']}:{row['cell_sha256']}\n" for row in rows).encode())
 return {'schema':1,'samples':samples,'lito_declared':233,'lito_present_ids':ids,'lito_count':len(rows),'lito_cells_sha256':aggregate}
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--prepare',action='store_true');parser.add_argument('--output',type=Path,default=ROOT/'驗證紀錄/current/native-inventory-icons');args=parser.parse_args();expected=collect()
 if args.prepare:
  assert not FIXTURE.exists(),'Do not overwrite a pinned reference';FIXTURE.write_text(json.dumps(expected,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('Prepared selected original TBT/LBT reference');return
 assert json.loads(FIXTURE.read_text(encoding='utf-8-sig'))==expected,'Original icon reference changed'
 rust=json.loads((ROOT/'LiTo381-Modern/target/current/verification/native-inventory-icons.json').read_text(encoding='utf-8-sig'))
 assert rust['samples']==expected['samples'];assert rust['lito_count']==expected['lito_count'] and rust['lito_cells_sha256']==expected['lito_cells_sha256']
 sheet=Image.new('RGB',(3*160,2*180),(48,48,48));draw=ImageDraw.Draw(sheet)
 for n,id in enumerate(SAMPLES):
  image=Image.open(ROOT/f'LiTo381-Modern/target/current/verification/native-inventory-icon-{id}.png').convert('RGBA');assert image.size==(32,32);assert image.tobytes()==load(id)[1]
  image=image.resize((128,128),Image.Resampling.NEAREST);x=(n%3)*160+16;y=(n//3)*180+12;sheet.paste(image,(x,y),image);draw.text((x,y+138),str(id),fill='white')
 args.output.mkdir(parents=True,exist_ok=True);sheet.save(args.output/'icon-contact-sheet.png')
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';ref=json.loads((ROOT/'驗證紀錄/current/reference-snapshot.json').read_text(encoding='utf-8-sig'))
 receipt={'exe_sha256':sha(exe.read_bytes()),'jar_sha256':ref['sha256'],'pixel_comparisons':len(SAMPLES),'lito_cells':expected['lito_count'],'reference':expected,'gui_tested':False}
 (args.output/'icon-visual-receipt.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'icon_pixel_comparisons':len(SAMPLES),'lito_cells':expected['lito_count']}))
if __name__=='__main__':main()
