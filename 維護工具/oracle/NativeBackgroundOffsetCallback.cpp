// Full original offset callback only; scheduler is an argument/order recorder.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <string>
#include <vector>
static_assert(sizeof(void*)==4,"Private original-code oracle is x86");
using U=uint32_t;
static void require(bool b,const char* m){if(!b)throw std::runtime_error(m);}
struct Slot{U before=0xa51c0ffe,value=0,after=0xfedc315a;void check(){require(before==0xa51c0ffe&&after==0xfedc315a,"Global guard");}};
static Slot offset_x,offset_y;
static U delay_arg,callback_arg,owner_arg,observed_x,observed_y,schedule_calls;
static void __fastcall schedule_record(U delay,U callback,U owner){
    delay_arg=delay;callback_arg=callback;owner_arg=owner;
    observed_x=offset_x.value;observed_y=offset_y.value;++schedule_calls;
}
static U address(const void* p){return reinterpret_cast<U>(p);}
int wmain(int argc,wchar_t** argv){try{
    require(argc==2,"Owned output directory required");std::filesystem::path out(argv[1]);
    std::ifstream input(out/L"callback-original.bin",std::ios::binary);
    std::vector<unsigned char> bytes((std::istreambuf_iterator<char>(input)),{});
    require(!bytes.empty()&&bytes.size()<4096,"Callback size");
    auto code=static_cast<unsigned char*>(VirtualAlloc(nullptr,4096,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));
    require(code!=nullptr,"Private code allocation");std::memcpy(code,bytes.data(),bytes.size());
    std::ofstream addresses(out/L"addresses.tsv");
    addresses<<"callback "<<address(code)<<"\noffset_x "<<address(&offset_x.value)<<"\noffset_y "<<address(&offset_y.value)
        <<"\nscheduler "<<address(reinterpret_cast<const void*>(&schedule_record))<<"\n";addresses.flush();
    std::ifstream relocations(out/L"relocations.tsv");std::string kind;U offset,target,next;U patched=0;
    while(relocations>>kind>>offset>>target>>next){
        require(offset+4<=bytes.size(),"Relocation bound");U value;
        if(kind=="memory"){
            require(target==0x62f4c0||target==0x62f4c4,"Memory relocation domain");
            value=target==0x62f4c0?address(&offset_x.value):address(&offset_y.value);
        }else if(kind=="self"){
            require(target==0x4878c0,"Callback pointer relocation");value=address(code);
        }else{
            require(kind=="relative"&&target==0x4d65f0,"Scheduler call relocation");
            value=address(reinterpret_cast<const void*>(&schedule_record))-address(code)-next;
        }
        std::memcpy(code+offset,&value,4);++patched;
    }
    require(patched==6,"Expected only four globals, self pointer and scheduler call");
    DWORD old;require(VirtualProtect(code,4096,PAGE_EXECUTE_READ,&old)!=0,"Private RX protection");
    require(FlushInstructionCache(GetCurrentProcess(),code,bytes.size())!=0,"Instruction cache");
    std::ofstream relocated(out/L"callback-relocated.bin",std::ios::binary);
    relocated.write(reinterpret_cast<char*>(code),static_cast<std::streamsize>(bytes.size()));relocated.close();
    std::ofstream rows(out/L"vectors.csv");
    rows<<"sequence,index,before_x,before_y,after_x,after_y,observed_x,observed_y,delay,owner,callback,calls\n";
    U cases=0;
    auto run=[&](U sequence,U index){
        U x=offset_x.value,y=offset_y.value;schedule_calls=0;
        reinterpret_cast<void(__cdecl*)()>(code)();offset_x.check();offset_y.check();
        require(schedule_calls==1&&callback_arg==address(code),"Schedule call count/self pointer");
        rows<<sequence<<","<<index<<","<<x<<","<<y<<","<<offset_x.value<<","<<offset_y.value<<","<<observed_x<<","<<observed_y
            <<","<<delay_arg<<","<<owner_arg<<","<<callback_arg<<","<<schedule_calls<<"\n";++cases;
    };
    std::array<U,7> values{0,1,65484,65534,65535,0x7fffffff,0xffffffff};
    U index=0;for(U x:values)for(U y:values){offset_x.value=x;offset_y.value=y;run(0,index++);}
    offset_x.value=65484;offset_y.value=65534;
    for(U i=0;i<8;++i)run(1,i);
    require(VirtualFree(code,0,MEM_RELEASE)!=0,"Code release");
    std::cout<<"{\"status\":\"original_background_offset_callback_executed\",\"cases\":"<<cases
        <<",\"guard_failures\":0,\"scheduler_body_stubbed\":true,\"actual_formal_sessions\":0}"<<std::endl;
    return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<std::endl;return 1;}}
