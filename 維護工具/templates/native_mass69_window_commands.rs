                WM_COMMAND if lparam.0==ui.control(MASS_OPTIONS).0 as isize&&lparam.0!=0&&(wparam.0>>16)==CBN_SELCHANGE as usize=>{
                    ui.held.clear();ui.held_attack.clear();ui.mass_choice=None;
                    if ui.mass_menu.command_id()==Some((wparam.0&0xffff)as u16)&&ui.skills.selected()==Some(69)&&!ui.closing&&!ui.inventory_view&&!ui.rental.open {
                        let index=send(ui.control(MASS_OPTIONS),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;
                        if let Some(i)=index.checked_sub(1).and_then(|i|usize::try_from(i).ok()){match ui.controller.mass_choice(&ui.mass_menu,(wparam.0&0xffff)as u16,i){Ok(choice)=>ui.mass_choice=Some(choice),Err(e)=>ui.text(STATUS,&format!("{e:#}"))}}
                    }
                    ui.refresh();return LRESULT(0);
                }
                WM_COMMAND if (wparam.0>>16)==BN_CLICKED as usize&&lparam.0!=0&&[ui.control(MASS_YES).0 as isize,ui.control(MASS_NO).0 as isize].contains(&lparam.0)=>{
                    let yes=lparam.0==ui.control(MASS_YES).0 as isize;
                    if !ui.closing {if let Some(token)=ui.mass_token.clone().filter(|t|t.command_id(yes)==(wparam.0&0xffff)as u16){ui.held.clear();ui.held_attack.clear();if let Err(e)=ui.controller.answer_mass(&token,yes){ui.text(STATUS,&format!("{e:#}"));}}}
                    ui.refresh();return LRESULT(0);
                }
