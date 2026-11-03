"""Independent S32 draw-reference census for a captured native terrain window.

Reads the original first four S32 layers directly. This checks residency and
resource budgets, not the complete framebuffer or later S32 layer semantics.
"""
from pathlib import Path
import hashlib, struct
from PIL import Image

RESOURCE_ROOT=Path(r'D:\lineage開發\0-客戶端')
def sha(path):
    with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()

def census(map_id,position,bounded=True):
    folder=RESOURCE_ROOT/'LiTo/LTmap'/str(map_id)
    files=sorted(folder.glob('????????.s32'))
    assert 0<len(files)<=64
    keys=[(int(p.stem[:4],16),int(p.stem[4:],16)) for p in files]
    assert len(set(keys))==len(keys)
    first=keys[0];origin=tuple(32704+(v-0x7fff)*64 for v in first)
    block=[int(v)&~7 for v in position]
    dx,dy=[v-o for v,o in zip(block,origin)]
    center=(24+24*(dx+dy),768+12*(-dx+dy))
    bounds=[center[0]-496,center[1]-432,center[0]+832,center[1]+432]
    slots=set();whole_slots=set();selected=0;whole=0;resident=set();pins={};layers={}
    for file,(sx,sy) in zip(files,keys):
        assert not file.is_symlink() and file.stat().st_size<=8*1024**2
        raw=file.read_bytes();pins[str(file)]=hashlib.sha256(raw).hexdigest().upper()
        cursor=0
        def take(fmt):
            nonlocal cursor
            size=struct.calcsize(fmt);assert cursor+size<=len(raw),(file,cursor,size)
            values=struct.unpack_from(fmt,raw,cursor);cursor+=size;return values
        tiles=[]
        for i in range(128*64):
            slot,tile,unused=take('<BHB');tiles.append((i%128,i//128,tile,slot,'floor'))
        count,=take('<H');assert count<=262144
        for _ in range(count):
            x,y,slot,tile,unused=take('<BBBHB')
            tiles.append((x,y,tile,slot,'overlay'))
        assert cursor+64*64*4<=len(raw);cursor+=64*64*4
        groups,=take('<i');assert 0<=groups<=65536
        object_count=0
        for _ in range(groups):
            group,count=take('<hH');object_count+=count;assert object_count<=262144
            for _ in range(count):
                x,y,layer,slot,tile,unused=take('<BBBBhB')
                # Fragment coordinates are u8, including sector overhang.
                assert tile>=0
                tiles.append((x,y,tile,slot,'object'))
        assert len(tiles)<=262144
        for x,y,tile,slot,kind in tiles:
            whole+=1;whole_slots.add((tile,slot))
            px=24*((x+1)//2+y)+1536*(sx-first[0]+sy-first[1])
            py=12*(63-x//2+y)+768*(-sx+first[0]+sy-first[1])
            if bounded and not(px<bounds[2] and py<bounds[3] and px+24>bounds[0] and py+24>bounds[1]):continue
            selected+=1;slots.add((tile,slot));resident.add((sx,sy));layers[kind]=layers.get(kind,0)+1
    return dict(map_id=map_id,complete_sector_count=len(files),whole_source_draw_references=whole,whole_source_decoded_slots=len(whole_slots),
        draw_references=selected,decoded_slots=len(slots),decoded_bytes=len(slots)*24*24*4,
        terrain_resident_sectors=len(resident),block=block,window_pixels=[1328,864],projected_bounds=bounds,
        selected_by_layer=layers,original_sector_pins=pins,bounded=bounded)

def qualify(capture,world):
    f=capture['frame'];map_id=world['map_id'];assert f['map_id']==map_id
    actor=world['actors'][str(world['local_object_id'])] if 'local_object_id' in world else world['actors'][str(f['local_object_id'])]
    position=[actor['x'],actor['y']];assert f['view_position']==position and f['view_mode']=='participant'
    assert f['terrain_namespace']=='LiTo/LTmap + LTtile' and (f['width'],f['height'])==(800,600)
    bounded=f.get('native_terrain_block') is not None
    ref=census(map_id,position,bounded)
    budget=f['static_resource_budget']
    for key in ['complete_sector_count','draw_references','decoded_slots','decoded_bytes']:assert budget[key]==ref[key],(key,budget[key],ref[key])
    assert budget['decoded_limit_bytes']==64<<20 and 0<budget['decoded_bytes']<=64<<20
    assert budget['draw_limit']==1048576 and 0<budget['draw_references']<=budget['draw_limit']
    assert f['map_sectors']==ref['complete_sector_count']
    if bounded:
        assert f['native_terrain_block']==ref['block'] and f['native_terrain_block_span']==8
        assert f['native_terrain_window_pixels']==ref['window_pixels']
        assert f['terrain_resident_sectors']==ref['terrain_resident_sectors']
        assert f['native_terrain_center']==[0x7fff+(int(v)-32704)//64 for v in position]
    image=Path(capture['png']);assert image.is_file()
    with Image.open(image) as im:
        rgba=im.convert('RGBA');assert rgba.size==(800,600)
        assert hashlib.sha256(rgba.tobytes()).hexdigest().upper()==f['rgba_sha256'].upper()
        colors=rgba.convert('RGB').getcolors(800*600);assert colors and len(colors)>1024
        terrain_pixels=sum(n for n,c in colors if c not in [(0,0,0),(18,25,35)])
        assert terrain_pixels>48000
    return ref|dict(position=position,png=str(image),png_sha256=sha(image),terrain_colored_pixels=terrain_pixels,
        complete_framebuffer_oracle=False,scope='Original S32 first-four-layer census, exact RGBA digest, bounded residency and image diversity')
