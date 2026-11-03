
#[cfg(test)]mod late_bookmark_owner_tests {
    use super::*;
    #[test]fn well_formed_late_bookmark_without_a_current_character_is_consumed_without_authority(){let s=super::super::native_login::LoginController::default().snapshot().clone();let mut book=Bookmarks::default();let mut body=vec![48];body.extend(b"old-role\0");body.extend(123_u16.to_le_bytes());body.extend(234_u16.to_le_bytes());body.extend(4_u16.to_le_bytes());assert!(book.receive(&s,7,&body).unwrap());assert!(book.owner.is_none());assert!(book.rows(&s,7).is_empty());assert_eq!(book.revision,0);assert!(book.receive(&s,7,&body[..body.len()-1]).is_err());assert!(book.owner.is_none());}
}
