"""Generate strict original-definition guards for finite effects with existing semantics."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
base=ROOT/'驗證紀錄/current/native44FE-skill-book-audit/20261018-175440-043382'
origin=base/'original-effect-definitions-20261018-181149-098541'
proof=read(origin/'readback.json');assert sha(origin/'inspect.stdout')==proof['data_sha256']
data=read(origin/'inspect.stdout');accepted=[];deferred=[]
for row in data['rows']:
    a=row.get('action0',{});frames=a.get('frames',[]);attrs=[];reasons=[]
    if not row['present']or row.get('header_issue')is not None:reasons.append('missing_or_unverified_definition')
    if row.get('actions')!=['ActionKey { id: 0, variant: None }']:reasons.append('other_actions_need_separate_semantics')
    if a.get('directionality')!='Undirected'or a.get('fps')!=24:reasons.append('directional_or_other_clock')
    if not 2<=len(frames)<=128 or a.get('has_uninterpreted_events'):reasons.append('frame_controls_or_bound')
    if row.get('image_count')!=1 or any(f['image']!=0 or not 1<=f['ticks']<=4096 for f in frames):reasons.append('image_or_tick_controls')
    if frames and(frames[0]['subframe']!=len(frames)-1 or any(f['subframe']!=i-1 for i,f in enumerate(frames)if i)):reasons.append('noncanonical_terminal_dwell_order')
    for attr in row.get('attributes',[]):
        m=re.fullmatch(r'ActionKey \{ id: (\d+), variant: None \}',attr['key']);assert m
        key=int(m[1]);body=bytes(attr['body']).strip()
        if (key,body)not in [(102,b'0'),(104,b'8')]:reasons.append('additional_attribute_semantics')
        attrs.append((key,attr['body']))
    for frame in frames:
        if any(not re.fullmatch(r'Sound\(\d+\)',event)for event in frame['events']):reasons.append('non_simple_audio_events')
    if frames and frames[0]['events']:reasons.append('terminal_dwell_event')
    if reasons:deferred.append(dict(graphics=row['graphics'],reasons=sorted(set(reasons))));continue
    accepted.append((row,attrs))
assert accepted
out=ROOT/'維護工具/candidates/native44FE-extra-skill-finite-effects';out.mkdir()
def bytes_literal(values):return '&['+','.join(map(str,values))+']'
lines=['//! Candidate only: exact finite source definitions; no generic graphics fallback.',
    'use super::{sprite_actions::{ActionKey,Animation,Directionality,FrameEvent,SpriteActionCatalog},tower_assets::HonorAssetError};',
    'struct Spec {sprite:u32,bank:u32,header:&\'static[u8],attrs:&\'static[(u32,&\'static[u8])],frames:&\'static[(u32,u32,u32,&\'static[u32])]}',
    'pub const GRAPHICS:&[u16]=&['+','.join(str(row['graphics'])for row,_ in accepted)+'];',
    'pub const GRAPHICS_BINDINGS:&[(u32,u32)]=&['+','.join('('+str(row['graphics'])+','+str(row['graphics_bank'])+')'for row,_ in accepted if row['graphics']!=row['graphics_bank'])+'];',
    'const SPECS:&[Spec]=&[']
for row,attrs in accepted:
    frames=row['action0']['frames']
    attr_text='&['+','.join('('+str(k)+','+bytes_literal(v)+')'for k,v in attrs)+']'
    frame_text='&['+','.join('('+','.join(map(str,[f['image'],f['subframe'],f['ticks']]))+','+bytes_literal([int(re.fullmatch(r'Sound\((\d+)\)',e)[1])for e in f['events']])+')'for f in frames)+']'
    lines.append('Spec{sprite:'+str(row['graphics'])+',bank:'+str(row['graphics_bank'])+',header:'+bytes_literal(row['header_bytes'])+',attrs:'+attr_text+',frames:'+frame_text+'},')
lines.append('];')
lines.append(r'''
pub fn qualify(c:&SpriteActionCatalog)->Result<Vec<(u32,u8,Animation)>,HonorAssetError> {
    let mut result=Vec::new();
    for spec in SPECS {
        let row=c.sprites.get(&spec.sprite).ok_or(HonorAssetError::Unsupported("additional finite effect absent"))?;
        if row.id!=spec.sprite || row.graphics_id!=spec.bank || row.image_count!=Some(1)
            || row.header_issue.is_some() || row.header_bytes!=spec.header || row.actions.len()!=1
            || row.attributes.len()!=spec.attrs.len()
            || row.attributes.iter().zip(spec.attrs).any(|(actual,(key,body))|actual.key!=ActionKey::new(*key)||actual.body_bytes!=*body) {
            return Err(HonorAssetError::Unsupported("additional finite effect definition changed"));
        }
        let resolved=c.resolve_action(spec.sprite,ActionKey::new(0))?;
        let a=resolved.animation;
        if resolved.definition_sprite_id!=spec.sprite || a.directionality!=Directionality::Undirected
            || a.frames_per_second!=24 || a.frames.len()!=spec.frames.len()
            || a.frames.iter().zip(spec.frames).any(|(actual,(image,subframe,ticks,sounds))|
                (actual.image_id,actual.subframe,actual.duration_ticks)!=(*image,*subframe,*ticks)
                || actual.events!=sounds.iter().map(|sound|FrameEvent::Sound(*sound)).collect::<Vec<_>>()) {
            return Err(HonorAssetError::Unsupported("additional finite effect timing changed"));
        }
        result.push((spec.sprite,0,a.clone()));
    }
    Ok(result)
}
''')
source=out/'native_additional_finite_effects.rs';source.write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
record=dict(status='isolated_candidate_original_finite_effect_guards_not_applied',source_manifest_sha256='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0',
    original_metadata=str(origin/'inspect.stdout'),original_metadata_sha256=sha(origin/'inspect.stdout'),catalogue_sha256=bytes(data['catalogue_sha256']).hex().upper(),
    candidate=str(source),candidate_sha256=sha(source),graphics=[row['graphics']for row,_ in accepted],deferred=deferred,
    existing_semantics='One undirected24fps action0, source0 terminal dwell, only original scalar Sound events, original attr102=0 and attr104=8; aliases stay explicit.',
    normal_source_modified=False,applied_to_normal_product=False,component_typecheck_verified=False,SPR_or_PCM_playback_verified=False,actual_sessions=0,actual_casts=0,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(source),finite_graphics=len(accepted),deferred_graphics=len(deferred),normal_modified=False),ensure_ascii=False))
