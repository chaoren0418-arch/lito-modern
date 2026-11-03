    pub(crate) fn mass_prompt(&self)->Option<&super::native_mass69::Prompt>{self.mass_invitation.current(&self.state,self.transport.snapshot().epoch,Instant::now())}
    pub(crate) fn mass_travel_pending(&self)->bool{self.mass_portal.pending()}
    pub(crate) fn sync_mass_menu(&self,menu:&mut super::native_mass69::Menu,active:bool)->Result<bool>{menu.synchronize(&self.state,self.transport.snapshot().epoch,&self.mass_books,active)}
    pub(crate) fn mass_choice(&self,menu:&super::native_mass69::Menu,command:u16,index:usize)->Result<super::native_mass69::Choice>{menu.choose(&self.state,self.transport.snapshot().epoch,&self.mass_books,command,index)}
    pub(crate) fn mass_choice_current(&self,choice:&super::native_mass69::Choice)->bool{choice.current(&self.state,self.transport.snapshot().epoch,&self.mass_books)}
    pub(crate) fn mass_choice_ready(&self,choice:&super::native_mass69::Choice)->bool{super::native_skills::mass_request(&self.state,self.transport.snapshot().epoch,&self.mass_books,choice).is_ok()}
    pub(crate) fn answer_mass(&mut self,token:&super::native_mass69::Token,yes:bool)->Result<()> {
        let body=self.mass_invitation.answer(&self.state,self.transport.snapshot().epoch,Instant::now(),token,yes)?;
        if let Some(body)=body {if let Err(error)=self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(body)){self.fail(format!("mass teleport answer send: {error:#}"));return Err(error);}}
        Ok(())
    }
    fn ensure_native_input(&self)->Result<()> {
        ensure!(!self.mass_travel_pending(),"等待伺服器傳送結果");
        ensure!(self.mass_prompt().is_none(),"請先回覆集體傳送邀請");Ok(())
    }
    fn accept_native_summon(&mut self,body:&[u8])->Result<()> {
        let reply=self.mass_portal.receive(&self.state,self.transport.snapshot().epoch,body)?.context("missing summon continuation")?;
        self.mass_invitation.retire();self.resurrection.retire();self.shape.retire();
        self.state.merchant.clear();self.attack_pending=None;self.skill_pending=None;self.skill_equipment_pending=false;self.skill_summon_pending=false;
        self.item_pending=None;self.pickup_pending=None;self.door_pending=None;self.portal_entry=None;self.world_tree_request=None;self.skill_ready=None;
        self.state.attack_feedback=super::native_combat::Feedback::Interrupted;self.state.skill_feedback=super::native_skills::Feedback::Interrupted;
        self.state.item_feedback=super::native_item_actions::Feedback::Interrupted;self.state.pickup_feedback=super::native_ground_pickup::Feedback::Interrupted;
        self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(reply.to_vec()))?;
        self.mass_deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    fn admit_received_portal(&mut self,destination:super::native_portal::Destination)->Result<()> {
        // Even an invalidated in-flight grant must be consumed/rejected here;
        // it cannot fall through to ordinary same-map or rental portal admission.
        if self.mass_portal.pending() {
            let phase=self.mass_portal.take(&self.state,self.transport.snapshot().epoch,destination)?;self.mass_deadline=None;
            if phase==LoginPhase::WorldReady{self.state.phase=LoginPhase::WorldEntering;}
            return Ok(());
        }
        let tree=self.world_tree_request.take().filter(|(stamp,since)|
            *stamp==super::native_navigation::InputStamp::capture(self.transport.snapshot().epoch,&self.state.world)
            &&since.elapsed()<Duration::from_secs(2));
        if tree.is_some(){super::native_portal::admit_world_tree(&self.state,destination)?;self.state.phase=LoginPhase::WorldEntering;}
        else{super::native_portal::admit(&self.state,destination)?;}
        Ok(())
    }
