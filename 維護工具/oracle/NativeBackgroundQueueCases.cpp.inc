// Private source slices and original queue bodies; clocks and CRT are supplied.
static void direct(const char* name){reinterpret_cast<void(__cdecl*)()>(code.at(name))();}
static void background_reset(){reset();background_x.value=65484;background_y.value=65534;}
static U snapshots=0;
#if BACKGROUND_READY_FILTER
static constexpr U expected_snapshots=18,expected_functions=15;
#else
static constexpr U expected_snapshots=16,expected_functions=14;
#endif
static void background_dump(std::ofstream& q,std::ofstream& state,const char* scenario,bool fired=false){
    guards();state<<scenario<<','<<background_x.value<<','<<background_y.value<<','<<green_counter.value<<','
        <<static_cast<int32_t>(phase.value)<<','<<counter.value<<','<<enabled.value<<','<<count.value<<','
        <<clock_now<<','<<clock_reads<<','<<int(fired)<<'\n';
    for(U i=0;i<count.value;++i){auto e=events()[i];q<<scenario<<','<<i<<','<<e.due<<','<<kind(e.callback)<<','<<e.owner<<'\n';}
    ++snapshots;
}
int wmain(int argc,wchar_t** argv){try{
    require(argc==2,"Expected owned input directory");std::filesystem::path dir(argv[1]);load(dir);
    std::ofstream q(dir/L"queue-vectors.csv"),state(dir/L"state-vectors.csv");
    q<<"scenario,index,deadline,callback,owner\n";
    state<<"scenario,x,y,green,phase,counter,enabled,queued,clock_now,clock_reads,fired\n";
    U bg=address(code.at("background")),water=address(code.at("phase")),note=address(reinterpret_cast<void*>(&note_one));
    background_reset();enabled.value=1;clock_now=1000;direct("initial_slice");background_dump(q,state,"initial");
    direct("initial_slice");background_dump(q,state,"initial_again");

    background_reset();clock_now=1000;
    schedule(1,bg,0);schedule(2,bg,0);schedule(3,bg,9);schedule(4,note,0);schedule(5,water,0);direct("green");
    direct("rearm_slice");background_dump(q,state,"rearm");
    direct("stop_slice");background_dump(q,state,"stop");
    clock_now=1003;bool fired=tick();background_dump(q,state,"other_owner_due",fired);

    background_reset();clock_now=1000;direct("background");direct("green");schedule(70,water,0);schedule(80,note,0);
    for(U i=0;i<9;++i)schedule(3000+i,keep[i],i+1);
    background_dump(q,state,"map_before");map_clear();background_dump(q,state,"map_after");

    background_reset();clock_now=1000;direct("background");direct("green");schedule(70,water,0);schedule(80,note,0);
    for(U i=0;i<9;++i)schedule(3000+i,keep[i],i+1);
    schedule(60,address(code.at("map_aux")),0xffffffffu);background_dump(q,state,"scene_before");
    direct("scene_cleanup");background_dump(q,state,"scene_after");

#if BACKGROUND_READY_FILTER
    background_reset();clock_now=1000;direct("background");direct("green");schedule(70,water,0);schedule(80,note,0);
    for(U i=0;i<9;++i)schedule(3000+i,keep[i],i+1);
    schedule(60,address(code.at("map_aux")),0xffffffffu);background_dump(q,state,"ready_before");
    direct("ready_filter");background_dump(q,state,"ready_after");
#endif

    background_reset();direct("background");clock_now=1000;clock_step=7;fired=tick();background_dump(q,state,"late",fired);
    clock_step=0;clock_now=1070;fired=tick();background_dump(q,state,"late_early",fired);
    clock_now=1071;fired=tick();background_dump(q,state,"late_due",fired);

    background_reset();direct("background");direct("background");clock_now=1000;clock_step=7;
    fired=tick();background_dump(q,state,"duplicate",fired);

    background_reset();phase.value=19;counter.value=98;enabled.value=1;
    direct("background");direct("green");direct_phase();clock_now=1000;clock_step=7;
    fired=tick();background_dump(q,state,"mixed",fired);

    background_reset();background_x.value=0xffffffffu;background_y.value=65535;clock_now=0xfffffff0u;
    direct("background");background_dump(q,state,"wrap");
    background_reset();direct("stop_slice");background_dump(q,state,"stopped_idle");
    require(snapshots==expected_snapshots,"Snapshot inventory");q.close();state.close();require(q&&state,"Output retention");
    guards();free_stub(events());queue_pointer.value=0;require(allocations.empty()&&!allocation_guard_failed,"Allocation leak");
    for(auto pair:code)require(VirtualFree(pair.second,0,MEM_RELEASE),"Code release");
    std::cout<<"{\"status\":\"original_background_callback_queue_and_selected_lifecycle_slices_executed\",\"snapshots\":"<<expected_snapshots
        <<",\"original_functions_and_slices\":"<<expected_functions<<",\"guard_failures\":0,\"clock_source_stubbed\":true,\"whole_client_executed\":false}\n";
    return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
