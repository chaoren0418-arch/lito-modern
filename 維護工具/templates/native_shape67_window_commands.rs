                WM_COMMAND if lparam.0!=0 && [ui.control(SHAPE_OPTIONS).0 as isize,ui.control(SHAPE_APPLY).0 as isize,ui.control(SHAPE_CLOSE).0 as isize].contains(&lparam.0) => {
                    use super::native_shape67::Control;
                    let role=if lparam.0==ui.control(SHAPE_OPTIONS).0 as isize {Control::List}
                        else if lparam.0==ui.control(SHAPE_APPLY).0 as isize {Control::Apply}else{Control::Close};
                    if let Some(token)=ui.shape_token.clone().filter(|t|t.command_id(role)==(wparam.0&0xffff) as u16) {
                        if !ui.closing && ui.controller.shape_prompt().is_some_and(|p|p.token()==token) {
                            ui.held.clear();ui.held_attack.clear();
                            let result=match role {
                                Control::List if (wparam.0>>16)==CBN_SELCHANGE as usize => {
                                    let index=send(ui.control(SHAPE_OPTIONS),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;
                                    ui.shape_choice=index.checked_sub(1).and_then(|n|usize::try_from(n).ok()).filter(|n|*n<=super::native_shape67::FORMS.len());Ok(())
                                }
                                Control::Apply if (wparam.0>>16)==BN_CLICKED as usize => {
                                    if let Some(index)=ui.shape_choice {ui.controller.answer_shape(&token,index)}else{Ok(())}
                                }
                                Control::Close if (wparam.0>>16)==BN_CLICKED as usize => ui.controller.dismiss_shape(&token),
                                _=>Ok(()),
                            };
                            if let Err(e)=result {ui.text(STATUS,&format!("{e:#}"));}
                        }
                    }
                    ui.refresh();return LRESULT(0);
                }
