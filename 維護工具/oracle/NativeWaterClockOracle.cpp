// Private x86 fragments only. Never linked into the game client.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <stdexcept>
#include <string>
#include <vector>
static_assert(sizeof(void*)==4,"The retained functions are x86");
using U=uint32_t;
static void require(bool ok,const char* text){if(!ok)throw std::runtime_error(text);}
struct Slot {U before=0xa51c0ffe,value=0,after=0xfedc315a;void check()const{require(before==0xa51c0ffe&&after==0xfedc315a,"Global guard changed");}};
static Slot enabled,counter,phase,scroll_x,scroll_y,count,capacity,queue_pointer,pool_begin,pool_end,move_slot;
static std::array<Slot*,11> slots{&enabled,&counter,&phase,&scroll_x,&scroll_y,&count,&capacity,&queue_pointer,&pool_begin,&pool_end,&move_slot};
struct Event {U due,callback,owner;};
static_assert(sizeof(Event)==12);
static std::array<U,9> keep{0x47cec0,0x47ce50,0x47cdf0,0x4159b0,0x4c32a0,0x48d700,0x4878c0,0x4875a0,0x4876e0};
static std::array<U,4> secondary{};
static std::map<void*,U> allocations;
static bool allocation_guard_failed=false;
static U allocated_calls=0,freed_calls=0,clock_now=0,clock_step=0,clock_reads=0,tail_calls=0;
static std::vector<std::array<U,2>> notes;
static U __cdecl clock_stub(){U result=clock_now;clock_now+=clock_step;++clock_reads;return result;}
static void* __cdecl allocate_stub(U bytes){
    auto p=static_cast<unsigned char*>(std::malloc(bytes+8));if(!p)std::abort();
    U magic=0xbadc0ffe;std::memcpy(p,&magic,4);std::memcpy(p+4+bytes,&magic,4);
    allocations[p+4]=bytes;++allocated_calls;return p+4;
}
static void __cdecl free_stub(void* pointer){
    if(!pointer)return;auto it=allocations.find(pointer);if(it==allocations.end())std::abort();
    auto p=static_cast<unsigned char*>(pointer)-4;U first,last;
    std::memcpy(&first,p,4);std::memcpy(&last,p+4+it->second,4);
    if(first!=0xbadc0ffe||last!=0xbadc0ffe)allocation_guard_failed=true;
    allocations.erase(it);std::free(p);++freed_calls;
}
static void* __cdecl move_stub(void* to,const void* from,size_t bytes){return std::memmove(to,from,bytes);}
static void __cdecl tail_stub(){++tail_calls;}
static void __fastcall note_one(U owner){notes.push_back({1,owner});}
static void __fastcall note_two(U owner){notes.push_back({2,owner});}
static std::map<std::string,unsigned char*> code;
static U address(const void* p){return reinterpret_cast<U>(p);}
static Event* events(){return reinterpret_cast<Event*>(queue_pointer.value);}

// Original schedule is ECX=delay, EDX=callback, one callee-popped owner.
__declspec(naked) static void schedule_call(void*,U,U,U){__asm{
    push ebp
    mov ebp,esp
    mov ecx,[ebp+12]
    mov edx,[ebp+16]
    push dword ptr [ebp+20]
    call dword ptr [ebp+8]
    pop ebp
    ret
}}
__declspec(naked) static void pair_call(void*,U,U){__asm{
    push ebp
    mov ebp,esp
    mov ecx,[ebp+12]
    mov edx,[ebp+16]
    call dword ptr [ebp+8]
    pop ebp
    ret
}}
static void schedule(U delay,U callback,U owner){schedule_call(code.at("schedule"),delay,callback,owner);}
static void cancel(U callback,U owner){pair_call(code.at("cancel"),callback,owner);}
static void direct_phase(){reinterpret_cast<void(__cdecl*)()>(code.at("phase"))();}
static void map_clear(){reinterpret_cast<void(__cdecl*)()>(code.at("mapclear"))();}
static void scroll(int dx,int dy){pair_call(code.at("scroll"),static_cast<U>(dx),static_cast<U>(dy));}
static void guards(){
    for(auto s:slots)s->check();require(!allocation_guard_failed,"Allocation guard changed");
    require(count.value<=capacity.value&&capacity.value<=4096,"Queue bounds changed");
    auto it=allocations.find(events());require(it!=allocations.end()&&it->second==capacity.value*12,"Queue allocation changed");
    for(U i=1;i<count.value;++i)require(events()[i-1].due<=events()[i].due,"Queue no longer sorted");
}
static void reset(U initial_capacity=2){
    if(queue_pointer.value)free_stub(events());for(auto s:slots)s->value=0;
    capacity.value=initial_capacity;queue_pointer.value=address(allocate_stub(initial_capacity*12));
    pool_begin.value=pool_end.value=address(secondary.data());move_slot.value=address(reinterpret_cast<void*>(&move_stub));
    clock_now=clock_step=clock_reads=tail_calls=0;notes.clear();notes.reserve(128);guards();
}
static bool tick(){
    // Never call a retained original address masquerading as a callback.
    for(U i=0;i<count.value&&events()[i].due<=clock_now;++i){
        U f=events()[i].callback;require(f==address(code.at("phase"))||f==address(code.at("map_aux"))||f==address(reinterpret_cast<void*>(&note_one))||f==address(reinterpret_cast<void*>(&note_two)),"Unsafe due callback");
    }
    bool fired=reinterpret_cast<unsigned char(__cdecl*)()>(code.at("dispatch"))()!=0;guards();return fired;
}
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);require(bool(f),"Input absent");return {(std::istreambuf_iterator<char>(f)),{}};}
static std::string kind(U callback){
    if(callback==address(code.at("phase")))return "phase";
    if(callback==address(code.at("map_aux")))return "map_aux";
    if(callback==address(reinterpret_cast<void*>(&note_one)))return "note1";
    if(callback==address(reinterpret_cast<void*>(&note_two)))return "note2";
    for(U i=0;i<9;++i)if(callback==keep[i])return "keep"+std::to_string(i);
    throw std::runtime_error("Unknown retained callback");
}
static void dump(std::ofstream& q,std::ofstream& state,const char* scenario,bool fired=false){
    guards();state<<scenario<<','<<static_cast<int32_t>(phase.value)<<','<<counter.value<<','<<scroll_x.value<<','<<scroll_y.value<<','<<count.value<<','<<clock_now<<','<<clock_reads<<','<<int(fired)<<'\n';
    for(U i=0;i<count.value;++i){auto e=events()[i];q<<scenario<<','<<i<<','<<e.due<<','<<kind(e.callback)<<','<<e.owner<<'\n';}
}
static void load(const std::filesystem::path& dir){
    auto original_keep=read(dir/L"keep-original.bin");
    require(original_keep.size()==sizeof(keep)&&std::memcmp(original_keep.data(),keep.data(),sizeof(keep))==0,"Original preserved callback table differs");
    std::map<U,U> targets{{0x64bc2d,address(&enabled.value)},{0x75ac08,address(&counter.value)},{0x75ac1c,address(&phase.value)},
        {0x75ac20,address(&scroll_x.value)},{0x75ac24,address(&scroll_y.value)},{0x834c00,address(&count.value)},
        {0x834c04,address(&capacity.value)},{0x834c08,address(&queue_pointer.value)},{0x64b484,address(&pool_begin.value)},
        {0x64b488,address(&pool_end.value)},{0x4f7228,address(&move_slot.value)},{0x6365e0,address(keep.data())},
        {0x6365fc,address(keep.data()+7)},{0x636604,address(keep.data()+9)},
        {0x4d69f0,address(reinterpret_cast<void*>(&clock_stub))},{0x4f118c,address(reinterpret_cast<void*>(&allocate_stub))},
        {0x4f103c,address(reinterpret_cast<void*>(&free_stub))},{0x77409100,address(reinterpret_cast<void*>(&move_stub))},
        {0x4c89f0,address(reinterpret_cast<void*>(&tail_stub))}};
    std::map<std::string,U> sizes;
    std::ifstream manifest(dir/L"functions.tsv");std::string name;U original,size;
    while(manifest>>name>>std::hex>>original>>std::dec>>size){
        auto bytes=read(dir/(name+"-original.bin"));require(bytes.size()==size,"Fragment size mismatch");
        auto p=static_cast<unsigned char*>(VirtualAlloc(nullptr,65536,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE));require(p!=nullptr,"Code allocation failed");
        std::memcpy(p,bytes.data(),size);code[name]=p;sizes[name]=size;targets[original]=address(p);
    }
    require(code.size()==8,"Fragment inventory mismatch");
    std::ofstream addresses(dir/L"addresses.tsv");
    for(auto pair:targets)addresses<<std::hex<<pair.first<<' '<<pair.second<<'\n';
    addresses.close();require(bool(addresses),"Address retention failed");
    std::ifstream relocations(dir/L"relocations.tsv");std::string type;U offset,target,next;
    while(relocations>>name>>type>>std::dec>>offset>>std::hex>>target>>std::dec>>next){
        require(offset+4<=sizes.at(name),"Relocation outside fragment");U replacement=targets.at(target);
        if(type=="relative")replacement-=address(code.at(name))+next;else require(type=="absolute","Unknown relocation type");
        std::memcpy(code.at(name)+offset,&replacement,4);
    }
    for(auto pair:code){
        std::ofstream f(dir/(pair.first+"-relocated.bin"),std::ios::binary);f.write(reinterpret_cast<char*>(pair.second),sizes.at(pair.first));f.close();require(bool(f),"Relocated retention failed");
        DWORD old;require(VirtualProtect(pair.second,65536,PAGE_EXECUTE_READ,&old)&&FlushInstructionCache(GetCurrentProcess(),pair.second,65536),"Code protection failed");
    }
}
int wmain(int argc,wchar_t** argv){try{
    require(argc==2,"Expected retained input directory");std::filesystem::path dir(argv[1]);load(dir);
    std::ofstream phases(dir/L"phase-vectors.csv"),scrolls(dir/L"scroll-vectors.csv"),q(dir/L"queue-vectors.csv"),states(dir/L"state-vectors.csv");
    phases<<"enabled,counter_in,phase_in,counter_out,phase_out,queued,deadline,clock_reads\n";
    scrolls<<"x,y,dx,dy,out_x,out_y\n";q<<"scenario,index,deadline,callback,owner\n";
    states<<"scenario,phase,counter,x,y,queued,clock_now,clock_reads,fired\n";
    U phase_rows=0,scroll_rows=0;
    auto phase_case=[&](U flag,U before,int before_phase){
        reset();enabled.value=flag;counter.value=before;phase.value=static_cast<U>(before_phase);clock_now=12345;
        direct_phase();guards();phases<<flag<<','<<before<<','<<before_phase<<','<<counter.value<<','<<static_cast<int32_t>(phase.value)<<','<<count.value<<','<<(count.value?events()[0].due:0)<<','<<clock_reads<<'\n';
        if(flag)require(count.value==1&&events()[0].callback==address(code.at("phase"))&&events()[0].owner==0,"Phase callback binding changed");++phase_rows;
    };
    for(U flag:{0u,1u})for(U c=0;c<100;++c)for(int p=-20;p<=21;++p)phase_case(flag,c,p);
    for(U flag=0;flag<256;++flag)phase_case(flag,99,21);
    reset();
    auto scroll_case=[&](int x,int y,int dx,int dy){
        scroll_x.value=x;scroll_y.value=y;enabled.value=0;scroll(dx,dy);guards();
        scrolls<<x<<','<<y<<','<<dx<<','<<dy<<','<<scroll_x.value<<','<<scroll_y.value<<'\n';++scroll_rows;
    };
    for(int x=0;x<640;++x)for(int dx:{-640,-13,-6,-1,0,1,6,13,640})scroll_case(x,419,dx,0);
    for(int y=0;y<420;++y)for(int dy:{-420,-13,-6,-1,0,1,6,13,420})scroll_case(639,y,0,dy);
    for(int x:{0,1,638,639})for(int y:{0,1,418,419})for(int direction=0;direction<8;++direction){
        constexpr int d[8][2]={{4,2},{0,4},{-4,2},{-8,0},{-4,-2},{0,-4},{4,-2},{8,0}};
        scroll_case(x,y,d[direction][0],d[direction][1]);
    }
    U f=address(code.at("phase")),a=address(reinterpret_cast<void*>(&note_one)),b=address(reinterpret_cast<void*>(&note_two));
    reset();clock_now=1000;U allocations_before=allocated_calls;
    schedule(50,a,101);schedule(50,b,102);schedule(50,a,103);schedule(25,a,104);schedule(75,b,105);
    U growths=allocated_calls-allocations_before;dump(q,states,"tie");
    reset();clock_now=1000;schedule(50,a,11);schedule(50,a,11);schedule(50,a,22);schedule(50,b,11);schedule(99,a,11);
    cancel(a,11);dump(q,states,"cancel");
    reset();clock_now=1000;enabled.value=1;phase.value=19;counter.value=98;scroll_x.value=11;scroll_y.value=419;
    for(U i=0;i<9;++i)schedule(3000+i,keep[i],i+1);
    schedule(50,f,0);schedule(90,f,0);schedule(95,f,42);schedule(55,a,88);dump(q,states,"map_before");
    map_clear();dump(q,states,"map_filtered");
    reinterpret_cast<void(__cdecl*)()>(code.at("map_aux"))();dump(q,states,"map_aux_repeat");
    clock_now=2000;schedule(50,f,0);dump(q,states,"map_rescheduled");clock_now=2050;bool fired=tick();dump(q,states,"map_tick",fired);
    reset();clock_now=1000;enabled.value=1;phase.value=7;counter.value=99;schedule(50,f,0);schedule(50,f,0);
    enabled.value=0;cancel(f,0);dump(q,states,"disabled_cancel");direct_phase();dump(q,states,"disabled_callback");
    reset();enabled.value=1;schedule(50,f,0);schedule(50,f,0);clock_now=1000;clock_step=7;
    fired=tick();dump(q,states,"late",fired);clock_step=0;clock_now=1106;fired=tick();dump(q,states,"late_not_due",fired);
    clock_now=1107;fired=tick();dump(q,states,"late_first",fired);clock_now=2000;fired=tick();dump(q,states,"late_again",fired);
    reset();clock_now=1000;schedule(50,a,1);schedule(50,b,2);schedule(25,a,3);clock_now=1050;
    fired=tick();dump(q,states,"dispatch_equal",fired);require(notes==std::vector<std::array<U,2>>{{1,3},{1,1},{2,2}},"Equal deadline dispatch differs");
    reset();clock_now=0xfffffff0;schedule(50,a,9);schedule(0,b,10);dump(q,states,"u32_wrap");
    phases.close();scrolls.close();q.close();states.close();require(phases&&scrolls&&q&&states,"Vector retention failed");
    guards();free_stub(events());queue_pointer.value=0;require(allocations.empty()&&!allocation_guard_failed,"Private allocations leaked");
    for(auto pair:code)require(VirtualFree(pair.second,0,MEM_RELEASE),"Code release failed");
    std::cout<<"{\"status\":\"original_water_clock_fragments_executed\",\"phase_vectors\":"<<phase_rows<<",\"scroll_vectors\":"<<scroll_rows<<",\"queue_scenarios\":15,\"native_allocation_growths_in_tie\":"<<growths<<",\"guard_failures\":0,\"game_sessions\":0,\"whole_client_executed\":false}\n";
    return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
