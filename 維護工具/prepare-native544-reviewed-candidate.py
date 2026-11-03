"""Prepare one exact SPR marker with unchanged original parser checks."""
from pathlib import Path
import ast,hashlib,json
ROOT=Path(__file__).resolve().parents[1]
source=ROOT/'LiTo381-Modern/src/modern_lito/sprite.rs'
reference=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection/csharp544-20261017-162516-832/qualified-20261017-162916-914654/qualified.json'
q=json.loads(reference.read_text(encoding='utf-8-sig'))
assert q['reviewed_null_slots']==32 and q['all_63_frame_offsets_and_bounds_independently_rederived']
out=ROOT/'維護工具/candidates/native544-reviewed';out.mkdir()
before=source.read_text(encoding='utf-8-sig')
old='                    | (352, 120, -288, -360, u16::MAX, 0)'
assert before.count(old)==1
after=before.replace(old,old+'\n                    | (544, 384, -480, -384, u16::MAX, 0)')
test=r'''
    #[test]
    fn reviewed544_marker_preserves_slots_and_rejects_corruption() {
        let marker = || [544i16,384,-480,-384,-1,0,0].into_iter().flat_map(i16::to_le_bytes).collect::<Vec<_>>();
        let mut empty=vec![1];empty.extend(marker());empty.extend([0;8]);
        let decoded=decode_classic_spr(&empty).unwrap();
        let frame=&decoded.frames[0];
        assert_eq!((frame.x_offset,frame.y_offset,frame.width,frame.height,frame.unknown1,frame.unknown2),(544,384,0,0,u16::MAX,0));
        assert!(frame.rgba.is_empty());assert_eq!(frame.frame_type,0);
        let layered=decode_classic_spr_layered(&empty,SpriteLimits::default()).unwrap();
        assert_eq!(layered.flattened,decoded);assert!(layered.fragments.frames[0].is_empty());
        let one=fixture(None,&[(0,0,0)],&[raw_block(&[0x7c00])]);
        let mut mixed=vec![3];mixed.extend_from_slice(&one[1..20]);mixed.extend(marker());mixed.extend_from_slice(&one[1..]);
        let frames=decode_classic_spr(&mixed).unwrap().frames;
        assert_eq!(frames.len(),3);assert_eq!(frames[0],frames[2]);assert!(frames[1].rgba.is_empty());
        let mut canvas=super::super::compositor::Canvas::new(8,8).unwrap();canvas.clear([30,60,90]);let pixels=canvas.rgba().to_vec();
        canvas.blit_screen_rgb555(&frame.rgba,frame.width,frame.height,frame.x_offset,frame.y_offset).unwrap();assert_eq!(canvas.rgba(),pixels);
        for offset in [0,2,4,6,8,10,12] {let mut bad=vec![1];let mut m=marker();m[offset]^=1;bad.extend(m);bad.extend([0;8]);assert!(decode_classic_spr(&bad).is_err(),"marker field {offset}");}
        for end in 0..empty.len(){assert!(decode_classic_spr(&empty[..end]).is_err());}
        let mut bad=empty.clone();bad.push(9);assert!(matches!(decode_classic_spr(&bad),Err(SpriteError::UnsupportedFormat("data without blocks"))));
        let mut bad=vec![1];bad.extend(marker());bad.extend(1u32.to_le_bytes());bad.extend([0;8]);bad.extend([0,24,0,1]);
        assert!(matches!(decode_classic_spr(&bad),Err(SpriteError::InvalidData("RLE block exceeds 24x24"))));
    }
'''
end=after.rfind('\n}');assert end>0
after=after[:end]+test+after[end:]
(out/'sprite.rs').write_text(after,encoding='utf-8',newline='\n')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
record=dict(status='prepared_exact_marker_isolated_candidate_not_applied',source=str(source),source_sha256=sha(source),candidate=str(out/'sprite.rs'),candidate_sha256=sha(out/'sprite.rs'),
    reference=str(reference),reference_sha256=sha(reference),new_marker=[544,384,-480,-384,65535,0],unchanged_zero_reference_guard=True,
    new_nonignored_test='reviewed544_marker_preserves_slots_and_rejects_corruption',original_checks_preserved=True,original_source_changed=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(record,ensure_ascii=False))
