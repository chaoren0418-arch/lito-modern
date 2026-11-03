// Appended to the previously qualified private queue oracle, never to the client.
static void direct_green(){reinterpret_cast<void(__cdecl*)()>(code.at("green"))();}
static void green_dump(std::ofstream& q,std::ofstream& state,const char* scenario,bool fired=false){
    guards();U pulse=12u>green_counter.value%24?12u-green_counter.value%24:green_counter.value%24-12u;
    state<<scenario<<','<<green_counter.value<<','<<pulse+16<<','<<(pulse+16)/2<<','<<count.value<<','<<clock_now<<','<<clock_reads<<','<<int(fired)<<'\n';
    for(U i=0;i<count.value;++i){auto e=events()[i];q<<scenario<<','<<i<<','<<e.due<<','<<kind(e.callback)<<','<<e.owner<<'\n';}
}
int wmain(int argc,wchar_t** argv){try{
    require(argc==2,"Expected retained input directory");std::filesystem::path dir(argv[1]);load(dir);
    std::ofstream scalar(dir/L"green-vectors.csv"),q(dir/L"green-queue-vectors.csv"),states(dir/L"green-state-vectors.csv");
    scalar<<"initial,counter_out,schedule_clock,deadline,clock_reads,pulse,step\n";
    q<<"scenario,index,deadline,callback,owner\n";
    states<<"scenario,counter,pulse,step,queued,clock_now,clock_reads,fired\n";
    std::vector<U> seeds;for(U i=0;i<128;++i)seeds.push_back(i);
    for(U i:{0x7ffffffeu,0x7fffffffu,0x80000000u,0xfffffffeu,0xffffffffu})seeds.push_back(i);
    U rows=0;U green=address(code.at("green"));
    for(U seed:seeds)for(U now:{0u,49u,1000u,0xfffffff0u}){
        reset();green_counter.value=seed;clock_now=now;direct_green();guards();
        require(green_counter.value==seed+1u&&count.value==1&&clock_reads==1,"Green callback scalar changed");
        auto event=events()[0];require(event.callback==green&&event.owner==0&&event.due==now+50u,"Green callback schedule changed");
        U rem=green_counter.value%24,pulse=(rem>12?rem-12:12-rem)+16;
        scalar<<seed<<','<<green_counter.value<<','<<now<<','<<event.due<<','<<clock_reads<<','<<pulse<<','<<pulse/2<<'\n';++rows;
    }
    reset();clock_now=1000;direct_green();green_dump(q,states,"entry");
    clock_now=1049;bool fired=tick();require(green_counter.value==1&&count.value==1&&!fired,"Early callback dispatched");green_dump(q,states,"early",fired);
    clock_now=1050;fired=tick();require(green_counter.value==2&&count.value==1&&events()[0].due==1100,"Due callback differs");green_dump(q,states,"due",fired);

    reset();direct_green();clock_now=1000;clock_step=7;fired=tick();
    require(green_counter.value==2&&count.value==1&&events()[0].due==1057&&clock_reads==3,"Late green callback caught up or reused cutoff");green_dump(q,states,"late",fired);
    clock_step=0;clock_now=1056;fired=tick();require(green_counter.value==2&&!fired,"Fresh deadline fired early");green_dump(q,states,"late_not_due",fired);
    clock_now=1057;fired=tick();require(green_counter.value==3&&events()[0].due==1107,"Fresh callback differs");green_dump(q,states,"late_due",fired);

    reset();direct_green();direct_green();clock_now=1000;clock_step=7;fired=tick();
    require(green_counter.value==4&&count.value==2&&events()[0].due==1057&&events()[1].due==1064,"Duplicate callbacks were coalesced or caught up");green_dump(q,states,"duplicate",fired);

    reset();green_counter.value=41;direct_green();schedule(50,green,77);cancel(green,0);
    require(green_counter.value==42&&count.value==1&&events()[0].owner==77,"Cancel owner scope differs");green_dump(q,states,"cancel_owned");
    clock_now=50;fired=tick();require(green_counter.value==43&&count.value==1&&events()[0].owner==0,"Green callback did not restore native owner0");green_dump(q,states,"cancel_other_owner_dispatched",fired);

    reset();clock_now=1000;green_counter.value=41;direct_green();
    for(U i=0;i<9;++i)schedule(3000+i,keep[i],i+1);
    schedule(50,green,77);schedule(70,address(reinterpret_cast<void*>(&note_one)),88);green_dump(q,states,"map_before_filter");
    map_clear();require(green_counter.value==42&&count.value==10,"Map filter reset phase or removed kept callbacks");
    for(U i=0;i<count.value;++i)require(events()[i].callback!=green,"Green callback survived original map filter");
    green_dump(q,states,"map_after_filter");
    // Explicit invocation only: this does not prove all production entry paths restart it.
    direct_green();require(green_counter.value==43&&count.value==11,"Explicit restart reset counter");green_dump(q,states,"explicit_restart_after_filter");

    reset();clock_now=1000;green_counter.value=77;direct_green();
    for(U i=0;i<9;++i)schedule(3000+i,keep[i],i+1);
    schedule(50,green,77);schedule(70,address(reinterpret_cast<void*>(&note_one)),88);
    schedule(60,address(code.at("map_aux")),0xffffffffu);green_dump(q,states,"scene_before_cleanup");
    reinterpret_cast<void(__cdecl*)()>(code.at("scene_cleanup"))();
    require(green_counter.value==78&&count.value==7,"Scene cleanup reset counter or kept wrong callbacks");
    for(U i=0;i<7;++i)require(events()[i].callback==keep[i]&&events()[i].owner==i+1,"Scene cleanup retained an unsafe callback");
    green_dump(q,states,"scene_after_cleanup");
    direct_green();require(green_counter.value==79&&count.value==8,"Post-cleanup explicit start reset counter");green_dump(q,states,"explicit_restart_after_cleanup");

    reset();green_counter.value=0xffffffffu;direct_green();cancel(green,0);direct_green();
    require(green_counter.value==1&&count.value==1,"Cancel/restart counter wrap differs");green_dump(q,states,"cancel_then_restart");
    scalar.close();q.close();states.close();require(scalar&&q&&states,"Green vector retention failed");
    guards();free_stub(events());queue_pointer.value=0;require(allocations.empty()&&!allocation_guard_failed,"Private allocation leaked");
    for(auto pair:code)require(VirtualFree(pair.second,0,MEM_RELEASE),"Code release failed");
    std::cout<<"{\"status\":\"original_green_callback_and_queue_fragments_executed\",\"scalar_vectors\":"<<rows<<",\"queue_snapshots\":16,\"original_functions\":10,\"guard_failures\":0,\"game_sessions\":0,\"whole_client_executed\":false}\n";
    return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
