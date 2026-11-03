    unsafe fn refresh_shape(&mut self) {
        use super::native_shape67::{Control,FORMS};
        let token=self.controller.shape_prompt().map(|p|p.token());
        if token!=self.shape_token {
            self.held.clear();self.held_attack.clear();self.shape_choice=None;
            send(self.control(SHAPE_OPTIONS),CB_SHOWDROPDOWN,WPARAM(0),LPARAM(0));
            send(self.control(SHAPE_OPTIONS),CB_RESETCONTENT,WPARAM(0),LPARAM(0));
            if token.is_some() {
                self.inventory_view=false;self.ground_view=false;self.rental.open=false;
                self.target=None;self.ground=None;self.skill_item_target=None;self.summon_choice=None;
                self.inventory_choice=None;self.item_cursor=None;
                for label in std::iter::once("請選擇變形種類".to_string()).chain(std::iter::once("解除目前變身".to_string()))
                    .chain(FORMS.iter().map(|f|format!("Lv{} {}",f.minimum_level,f.action))) {
                    let text=wide(&label);send(self.control(SHAPE_OPTIONS),CB_ADDSTRING,WPARAM(0),LPARAM(text.as_ptr() as isize));
                }
                send(self.control(SHAPE_OPTIONS),CB_SETCURSEL,WPARAM(0),LPARAM(0));
            }
            self.shape_token=token;self.layout(self.parent,false);
        }
        if let Some(t)=self.shape_token.as_ref() {
            for (id,role) in [(SHAPE_OPTIONS,Control::List),(SHAPE_APPLY,Control::Apply),(SHAPE_CLOSE,Control::Close)] {
                let _=SetWindowLongPtrW(self.control(id),GWLP_ID,t.command_id(role) as isize);
            }
            self.text(SHAPE_LABEL,"收到變形選單，請選擇並套用；關閉只收起選單。資格由伺服器判定。");
            let _=EnableWindow(self.control(SHAPE_OPTIONS),!self.closing);
            let _=EnableWindow(self.control(SHAPE_APPLY),self.shape_choice.is_some()&&!self.closing);
            let _=EnableWindow(self.control(SHAPE_CLOSE),!self.closing);
        }
    }
