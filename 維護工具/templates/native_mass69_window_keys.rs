                let mass_active={let ui=state.0.borrow();ui.controller.mass_prompt().is_some()||ui.controller.mass_travel_pending()};
                if mass_active&&matches!(message.message,WM_KEYDOWN|WM_KEYUP){
                    if message.message==WM_KEYDOWN&&message.wParam.0==0x1b {
                        if message.lParam.0&(1<<30)==0{let mut ui=state.0.borrow_mut();if let Some(t)=ui.mass_token.clone(){if let Err(e)=ui.controller.answer_mass(&t,false){ui.text(STATUS,&format!("{e:#}"));}}ui.refresh();}continue;
                    }
                    if message.wParam.0==0x0d {
                        if message.message==WM_KEYDOWN&&message.lParam.0&(1<<30)==0 {
                            let command={let ui=state.0.borrow();let focus=GetFocus();ui.mass_token.as_ref().and_then(|t|[(MASS_YES,true),(MASS_NO,false)].into_iter().find(|(id,_)|focus==ui.control(*id)&&IsWindowEnabled(focus).as_bool()).map(|(_,yes)|(t.command_id(yes),focus)))};
                            if let Some((id,focus))=command{send(hwnd,WM_COMMAND,WPARAM(id as usize),LPARAM(focus.0 as isize));}
                        }continue;
                    }
                    if (world_focus&&message.wParam.0!=0x09)||matches!(message.wParam.0,0x70|0x71|0x72){continue;}
                }
                if message.message==WM_KEYDOWN&&message.wParam.0==0x0d&&GetFocus()==state.0.borrow().control(MASS_OPTIONS){send(GetFocus(),CB_SHOWDROPDOWN,WPARAM(0),LPARAM(0));continue;}
