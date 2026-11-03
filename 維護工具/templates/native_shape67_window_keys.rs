                // The received menu owns world shortcuts until it closes. No default option/Enter cast.
                let shape_active=state.0.borrow().controller.shape_prompt().is_some();
                if shape_active && matches!(message.message,WM_KEYDOWN|WM_KEYUP) {
                    if message.message==WM_KEYDOWN && message.wParam.0==0x1b {
                        let mut ui=state.0.borrow_mut();
                        if let Some(t)=ui.shape_token.clone() {if let Err(e)=ui.controller.dismiss_shape(&t){ui.text(STATUS,&format!("{e:#}"));}}
                        ui.refresh();continue;
                    }
                    if message.wParam.0==0x0d {
                        if message.message==WM_KEYDOWN && message.lParam.0&(1<<30)==0 {
                            let command={let ui=state.0.borrow();let focus=GetFocus();
                                if focus==ui.control(SHAPE_OPTIONS) {send(focus,CB_SHOWDROPDOWN,WPARAM(0),LPARAM(0));None}
                                else {ui.shape_token.as_ref().and_then(|t|[(SHAPE_APPLY,super::native_shape67::Control::Apply),(SHAPE_CLOSE,super::native_shape67::Control::Close)].into_iter().find(|(id,_)|focus==ui.control(*id)&&IsWindowEnabled(focus).as_bool()).map(|(_,role)|(t.command_id(role),focus)))}};
                            if let Some((id,focus))=command {send(hwnd,WM_COMMAND,WPARAM(id as usize),LPARAM(focus.0 as isize));}
                        }
                        continue;
                    }
                    if world_focus || matches!(message.wParam.0,0x70|0x71|0x72) {continue;}
                    // Arrow/Space/Tab continue to the focused native control.
                }
