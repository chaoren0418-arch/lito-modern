    unsafe fn refresh_mass_destinations(&mut self) {
        let active=self.skills.selected()==Some(69)&&self.world_view&&!self.result_view&&!self.closing&&!self.inventory_view&&!self.rental.open
            &&self.controller.mass_prompt().is_none()&&!self.controller.mass_travel_pending()&&self.controller.shape_prompt().is_none()&&self.controller.resurrection_prompt().is_none();
        match self.controller.sync_mass_menu(&mut self.mass_menu,active) {
            Ok(changed)=>{if changed {
                self.mass_choice=None;send(self.control(MASS_OPTIONS),CB_SHOWDROPDOWN,WPARAM(0),LPARAM(0));send(self.control(MASS_OPTIONS),CB_RESETCONTENT,WPARAM(0),LPARAM(0));
                let placeholder=wide("請選擇集體傳送目的地");send(self.control(MASS_OPTIONS),CB_ADDSTRING,WPARAM(0),LPARAM(placeholder.as_ptr() as isize));
                for row in self.mass_menu.rows(){let label=wide(&row.label);send(self.control(MASS_OPTIONS),CB_ADDSTRING,WPARAM(0),LPARAM(label.as_ptr() as isize));}
                send(self.control(MASS_OPTIONS),CB_SETCURSEL,WPARAM(0),LPARAM(0));
                let _=SetWindowLongPtrW(self.control(MASS_OPTIONS),GWLP_ID,self.mass_menu.command_id().unwrap_or(0) as isize);
            }}
            Err(e)=>{self.mass_menu.retire();self.mass_choice=None;let _=SetWindowLongPtrW(self.control(MASS_OPTIONS),GWLP_ID,0);self.text(STATUS,&format!("{e:#}"));}
        }
        if self.mass_choice.as_ref().is_some_and(|c|!self.controller.mass_choice_current(c)){self.mass_choice=None;send(self.control(MASS_OPTIONS),CB_SETCURSEL,WPARAM(0),LPARAM(0));}
        let _=EnableWindow(self.control(MASS_OPTIONS),active&&self.mass_menu.command_id().is_some());
    }
    unsafe fn refresh_mass(&mut self) {
        let prompt=self.controller.mass_prompt().cloned();let token=prompt.as_ref().map(|p|p.token());let pending=self.controller.mass_travel_pending();
        if token!=self.mass_token||pending!=self.mass_was_pending {
            self.held.clear();self.held_attack.clear();
            if token.is_some()||pending{self.inventory_view=false;self.ground_view=false;self.rental.open=false;self.target=None;self.ground=None;self.skill_item_target=None;self.summon_choice=None;self.mass_choice=None;self.inventory_choice=None;self.item_cursor=None;}
            self.mass_token=token;self.mass_was_pending=pending;self.layout(self.parent,false);
            if GetForegroundWindow()==self.parent&&!self.closing {
                if self.mass_token.is_some(){let _=SetFocus(Some(self.control(MASS_NO)));}
                else if [self.control(MASS_YES),self.control(MASS_NO)].contains(&GetFocus()){let _=SetFocus(Some(self.parent));}
            }
        }
        if pending{self.text(MASS_LABEL,"等待伺服器傳送結果。");}
        else if let Some(p)=prompt {
            let t=p.token();let _=SetWindowLongPtrW(self.control(MASS_YES),GWLP_ID,t.command_id(true) as isize);let _=SetWindowLongPtrW(self.control(MASS_NO),GWLP_ID,t.command_id(false) as isize);
            self.text(MASS_LABEL,if p.waiting(){"已同意集體傳送，等待伺服器結果；可放棄等待。"}else{"收到集體傳送邀請。是否同意？"});
            self.text(MASS_NO,if p.waiting(){"放棄等待"}else{"不同意"});let _=EnableWindow(self.control(MASS_YES),!p.waiting()&&!self.closing);let _=EnableWindow(self.control(MASS_NO),!self.closing);
            if p.waiting()&&GetFocus()==self.control(MASS_YES)&&GetForegroundWindow()==self.parent{let _=SetFocus(Some(self.control(MASS_NO)));}
        }
    }
