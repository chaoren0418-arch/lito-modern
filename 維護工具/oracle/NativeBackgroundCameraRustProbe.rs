//! Isolated proposed Rust arithmetic, bound to actual original-fragment CSV.
#[path="D:/lineage開發/3-lito開發/lito重建版/維護工具/candidates/native-background-camera/native_background_camera.rs"]
mod candidate;
use std::{collections::BTreeSet,path::PathBuf};
fn main()->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),3);
    let input=PathBuf::from(&args[1]);let output=PathBuf::from(&args[2]);
    assert!(input.is_absolute()&&output.is_absolute());
    let data=std::fs::read_to_string(input)?;let mut cases=0;let mut domains=BTreeSet::new();
    for line in data.lines().skip(1){
        let v:Vec<i32>=line.split(',').map(|s|s.parse().unwrap()).collect();assert_eq!(v.len(),11);
        let result=candidate::advance([v[3],v[4]],v[0].try_into()?,v[1].try_into()?,v[2].try_into()?).unwrap();
        assert_eq!(result.camera,[v[5],v[6]]);assert_eq!(result.scroll_delta,[v[7],v[8]]);
        assert_eq!((v[9],v[10]),(1,0));assert!(domains.insert((v[0],v[1],v[2],v[3],v[4])));cases+=1;
    }
    assert_eq!(cases,192);
    let expected:BTreeSet<_>=[4,6,8,12].into_iter().flat_map(|n|(0..8).flat_map(move|h|[1,2].into_iter().flat_map(move|p|[[48,48],[0,0],[-4096,4096]].into_iter().map(move|[x,y]|(n,h,p,x,y))))).collect();
    assert_eq!(domains,expected);
    let mut rejected=0;
    for count in [0,3,5,16]{assert_eq!(candidate::advance([48,48],count,0,1),Err(candidate::Error::Count));rejected+=1;}
    assert_eq!(candidate::advance([48,48],8,8,1),Err(candidate::Error::Heading));rejected+=1;
    assert_eq!(candidate::advance([48,48],8,0,0),Err(candidate::Error::InitialEvent));rejected+=1;
    for (camera,heading) in [([i32::MIN,0],0),([i32::MAX,0],4),([0,i32::MIN],0),([0,i32::MAX],4)]{
        assert_eq!(candidate::advance(camera,4,heading,1),Err(candidate::Error::Overflow));rejected+=1;
    }
    let json=format!("{{\"status\":\"isolated_Rust_camera_increment_candidate_matches_original_vectors\",\"cases\":{cases},\"checked_host_domain_rejections\":{rejected},\"product_source_changed\":false,\"modern_Pose_binding_verified\":false,\"whole_callback_executed\":false,\"actual_formal_sessions\":0,\"whole_goal_complete\":false}}\n");
    std::fs::write(output,json)?;println!("Rust camera candidate: {cases} native vector comparisons, {rejected} host domain rejections");Ok(())
}
