// Executes two extracted x86 arithmetic/state primitives in this helper only.
// Only the gmtime/registry CALL dependencies are relocated to synthetic stubs.
// No original game process, server, clock, or framebuffer is run or modified.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>

static_assert(sizeof(void*)==4,"This isolated source oracle requires x86");
static int tm_fixture[9]{};
static uintptr_t registry_fixture=0;
static uintptr_t own_actor_fixture=0;
static int weather_flags[3]{};

static unsigned char* region(size_t size) {
    auto p=static_cast<unsigned char*>(VirtualAlloc(nullptr,size,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));
    if(!p)throw std::runtime_error("Private isolated region unavailable");
    return p;
}
static void relocate_data(unsigned char* code,size_t size,uint32_t original,uintptr_t private_address) {
    int occurrences=0;
    for(size_t i=0;i+4<=size;++i) {
        uint32_t value=0;memcpy(&value,code+i,4);
        if(value==original){memcpy(code+i,&private_address,4);++occurrences;i+=3;}
    }
    if(occurrences!=1)throw std::runtime_error("Data relocation is not the single reviewed operand");
}
static void load(unsigned char* base,size_t offset,const std::filesystem::path& file,size_t expected) {
    std::ifstream f(file,std::ios::binary);std::vector<char> b((std::istreambuf_iterator<char>(f)),{});
    if(b.size()!=expected)throw std::runtime_error("Extracted routine size mismatch");
    memcpy(base+offset,b.data(),b.size());
}
static void executable(unsigned char* p) {
    DWORD old=0;if(!VirtualProtect(p,65536,PAGE_EXECUTE_READ,&old))throw std::runtime_error("Code protection failed");
    if(!FlushInstructionCache(GetCurrentProcess(),p,65536))throw std::runtime_error("Code cache flush failed");
}
struct Expected {int level,rise,set,minute;};
static Expected scalar(int day,int minute,int flags) {
    // C++ signed division truncates toward zero, like the original IDIV/magic division.
    int rise=day<=172?452-day*192/172:260+(day-173);
    int set=day<=172?986+day*197/172:1183+(173-day)*197/192;
    int v=2;
    if(minute>rise-40 && minute<rise)v=1+(minute-rise+40)*15/40;
    else if(minute>=rise && minute<=set)v=15;
    else if(minute>set && minute<set+40)v=1+(set-minute+40)*15/40;
    int level=std::min(15,5+2*v/3);
    if(level>=10 && flags) --level;
    return {std::max(5,level),rise,set,minute};
}

int wmain(int argc,wchar_t** argv) {
    try {
        if(argc!=4)throw std::runtime_error("Expected curve.bin light.bin boundary-output.jsonl");
        auto curve=region(65536),light=region(65536);
        load(curve,0x2f0,argv[1],380);load(light,0xaf90,argv[2],43);
        // Address-only relocations; no instruction, constant, branch or arithmetic changes.
        relocate_data(curve+0x2f0,380,0x75a7f0,reinterpret_cast<uintptr_t>(&weather_flags[0]));
        relocate_data(curve+0x2f0,380,0x75a7ec,reinterpret_cast<uintptr_t>(&weather_flags[1]));
        relocate_data(curve+0x2f0,380,0x75ab18,reinterpret_cast<uintptr_t>(&weather_flags[2]));
        relocate_data(light+0xaf90,43,0x834ee4,reinterpret_cast<uintptr_t>(&own_actor_fixture));
        // Stub returns an ordinary tm fixture; it does not read or change a clock.
        curve[0x1000]=0xb8;auto tm_address=reinterpret_cast<uintptr_t>(tm_fixture);
        memcpy(curve+0x1001,&tm_address,4);curve[0x1005]=0xc3;
        // Stub returns the selected synthetic actor, or null for a missing ID.
        light[0x1000]=0xa1;auto registry_address=reinterpret_cast<uintptr_t>(&registry_fixture);
        memcpy(light+0x1001,&registry_address,4);light[0x1005]=0xc3;
        executable(curve);executable(light);
        using Curve=int(__fastcall*)(uint32_t,int*,int*,int*);
        using Light=void(__fastcall*)(uint32_t,int);
        auto original_curve=reinterpret_cast<Curve>(curve+0x2f0);
        auto original_light=reinterpret_cast<Light>(light+0xaf90);
        std::ofstream vectors(std::filesystem::path(argv[3]),std::ios::binary);
        if(!vectors)throw std::runtime_error("Boundary output unavailable");
        uint64_t cases=0,boundaries=0;
        for(int day=0;day<366;++day)for(int minute=0;minute<1440;++minute)for(int flags=0;flags<8;++flags) {
            tm_fixture[7]=day;tm_fixture[2]=minute/60;tm_fixture[1]=minute%60;
            weather_flags[0]=flags&1;weather_flags[1]=flags&2;weather_flags[2]=flags&4;
            int rise=-1,set=-1,observed_minute=-1;
            int level=original_curve(0,&rise,&set,&observed_minute);
            auto e=scalar(day,minute,flags);
            if(level!=e.level || rise!=e.rise || set!=e.set || observed_minute!=minute) {
                std::cerr<<"Mismatch day="<<day<<" minute="<<minute<<" flags="<<flags<<" actual="<<level<<','<<rise<<','<<set<<','<<observed_minute<<" expected="<<e.level<<','<<e.rise<<','<<e.set<<'\n';
                return 2;
            }
            ++cases;
            std::array<int,15> edges={0,1439,720,rise-41,rise-40,rise-39,rise-1,rise,rise+1,set-1,set,set+1,set+39,set+40,set+41};
            if(std::find(edges.begin(),edges.end(),minute)!=edges.end()) {
                vectors<<"{\"day_of_year_zero_based\":"<<day<<",\"minute\":"<<minute<<",\"flags\":"<<flags<<",\"sunrise\":"<<rise<<",\"sunset\":"<<set<<",\"level\":"<<level<<"}\n";
                ++boundaries;
            }
        }
        alignas(16) unsigned char actor[64]{};
        uint64_t light_cases=0;
        for(int own=0;own<2;++own)for(int exists=0;exists<2;++exists)for(int raw=0;raw<256;++raw) {
            actor[0x12]=3;registry_fixture=exists?reinterpret_cast<uintptr_t>(actor):0;
            own_actor_fixture=own?reinterpret_cast<uintptr_t>(actor):0;
            const int signed_range=static_cast<int>(static_cast<int8_t>(raw));
            original_light(1,signed_range);
            const auto expected=exists?static_cast<unsigned char>(own?std::max(5,signed_range):signed_range):3;
            if(actor[0x12]!=expected)throw std::runtime_error("Signed actor-light receiver mismatch");
            ++light_cases;
        }
        vectors.close();if(!vectors)throw std::runtime_error("Boundary output failed");
        std::cout<<"{\"status\":\"extracted_x86_primitive_matches_scalar\",\"curve_cases\":"<<cases<<",\"boundary_vectors\":"<<boundaries<<",\"signed_actor_light_cases\":"<<light_cases<<",\"actual_game_sessions\":0,\"original_client_executed\":false,\"framebuffer_verified\":false}\n";
        return 0;
    }catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}
}
