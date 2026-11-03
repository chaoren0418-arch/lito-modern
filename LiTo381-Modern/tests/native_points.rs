use lito_modern::modern_lito::native_points;
#[test]
fn captured_rental_points_are_not_a_battle_result(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-points-current.json")).unwrap();
    for row in f.as_array().unwrap(){let s=row["hex"].as_str().unwrap();let b:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        let(name,points)=native_points::parse(&b).unwrap();assert_eq!(name,row["name"].as_str().unwrap());assert_eq!(points,5000);
        for n in 0..b.len(){assert!(native_points::parse(&b[..n]).is_err());}
        let mut tail=b.clone();tail.push(0);assert!(native_points::parse(&tail).is_err());
        for i in b.len()-14..b.len(){let mut combat=b.clone();combat[i]=1;assert!(native_points::parse(&combat).is_err());}
        let mut max=b.clone();let at=max.len()-18;max[at..at+4].fill(255);assert_eq!(native_points::parse(&max).unwrap().1,u32::MAX);
    }
}
