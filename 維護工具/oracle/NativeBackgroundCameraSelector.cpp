// Exact bounded camera-update fragment, in a private x86 numerical process.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstring>
#include <cstdio>
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
struct Slot{U before=0xa51c0ffe,value=0,after=0xfedc315a;void check(){require(before==0xa51c0ffe&&after==0xfedc315a,"Slot guard");}};
struct Tables{U before=0xabcdef01;std::array<U,80> data{};U after=0x10fedcba;};
struct Actor{U before=0xabcdef01;std::array<unsigned char,64> data{};U after=0x10fedcba;};
static Tables tables;
static Actor actor;
static Slot camera_x,camera_y,actor_pointer;
static int scroll_x,scroll_y,scroll_calls,fallback_calls;
static volatile int diagnostic_stage=-1;
static LONG CALLBACK exception_record(EXCEPTION_POINTERS* p){
    std::fprintf(stderr,"private fragment exception stage=%d code=%08lX address=%p fault=%08lX EIP=%08lX EAX=%08lX EBX=%08lX ECX=%08lX EDX=%08lX ESI=%08lX EDI=%08lX ESP=%08lX\n",diagnostic_stage,
        p->ExceptionRecord->ExceptionCode,p->ExceptionRecord->ExceptionAddress,
        p->ExceptionRecord->NumberParameters>1?p->ExceptionRecord->ExceptionInformation[1]:0,
        p->ContextRecord->Eip,p->ContextRecord->Eax,p->ContextRecord->Ebx,p->ContextRecord->Ecx,p->ContextRecord->Edx,p->ContextRecord->Esi,p->ContextRecord->Edi,p->ContextRecord->Esp);
    std::fflush(stderr);return EXCEPTION_CONTINUE_SEARCH;
}
static void __fastcall scroll_stub(int x,int y){scroll_x=x;scroll_y=y;++scroll_calls;}
static unsigned char __fastcall fallback_stub(void*){++fallback_calls;return 0;}
static U addr(const void* p){return reinterpret_cast<U>(p);}
static U memory(U original){
    if(original>=0x834c68&&original<0x834da8)return addr(tables.data.data())+original-0x834c68;
    if(original==0x75a998)return addr(&camera_x.value);
    if(original==0x75a99c)return addr(&camera_y.value);
    if(original==0x834ee4)return addr(&actor_pointer.value);
    throw std::runtime_error("Unknown original memory relocation");
}
struct Code{unsigned char* bytes;size_t original_size;};
static Code load(const std::filesystem::path& out,const std::string& name,std::ostream& addresses){
    std::ifstream f(out/(name+"-original.bin"),std::ios::binary);
    std::vector<unsigned char> bytes((std::istreambuf_iterator<char>(f)),{});
    require(!bytes.empty()&&bytes.size()<4095,"Fragment size");
    auto p=static_cast<unsigned char*>(VirtualAlloc(nullptr,4096,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));
    require(p!=nullptr,"Private code allocation");std::memcpy(p,bytes.data(),bytes.size());
    p[bytes.size()]=0xc3; // Only a test boundary return is appended, outside original bytes.
    addresses<<name<<" "<<addr(p)<<"\n";
    addresses.flush();
    std::ifstream rows(out/(name+"-relocations.tsv"));std::string kind;U offset,target,next;
    while(rows>>kind>>offset>>target>>next){
        require(offset+4<=bytes.size(),"Relocation bounds");U value;
        if(kind=="memory")value=memory(target);
        else{
            require(kind=="relative","Relocation kind");U destination;
            if(target==0x4874d0)destination=addr(reinterpret_cast<const void*>(&scroll_stub));
            else if(target==0x4e8e30)destination=addr(reinterpret_cast<const void*>(&fallback_stub));
            else{require(target==0x4e2d07,"Unknown external control flow");destination=addr(p)+static_cast<U>(bytes.size());}
            value=destination-addr(p)-next;
        }
        std::memcpy(p+offset,&value,4);
    }
    DWORD old;require(VirtualProtect(p,4096,PAGE_EXECUTE_READ,&old)!=0,"Code protect");
    require(FlushInstructionCache(GetCurrentProcess(),p,bytes.size()+1)!=0,"Instruction cache");
    std::ofstream relocated(out/(name+"-relocated.bin"),std::ios::binary);
    relocated.write(reinterpret_cast<char*>(p),static_cast<std::streamsize>(bytes.size()+1));
    return {p,bytes.size()};
}
__declspec(naked) static void invoke(void*,U){__asm{
    push ebp
    mov ebp,esp
    push ebx
    push esi
    push edi
    mov ebx,[ebp+12]
    call dword ptr [ebp+8]
    pop edi
    pop esi
    pop ebx
    pop ebp
    ret
}}
static void guards(){
    camera_x.check();camera_y.check();actor_pointer.check();
    require(tables.before==0xabcdef01&&tables.after==0x10fedcba,"Tables guard");
    require(actor.before==0xabcdef01&&actor.after==0x10fedcba,"Actor guard");
    require(actor_pointer.value==addr(actor.data.data()),"Actor pointer changed");
}
int wmain(int argc,wchar_t** argv){try{
    require(AddVectoredExceptionHandler(1,&exception_record)!=nullptr,"Private exception observer");
    require(argc==2,"Owned output directory required");std::filesystem::path out(argv[1]);
    std::ofstream addresses(out/L"addresses.tsv");
    addresses<<"tables "<<addr(tables.data.data())<<"\nactor_pointer "<<addr(&actor_pointer.value)
        <<"\ncamera_x "<<addr(&camera_x.value)<<"\ncamera_y "<<addr(&camera_y.value)
        <<"\nscroll_stub "<<addr(reinterpret_cast<const void*>(&scroll_stub))
        <<"\nfallback_stub "<<addr(reinterpret_cast<const void*>(&fallback_stub))<<"\n";
    actor_pointer.value=addr(actor.data.data());
    std::vector<Code> code;
    for(const std::string& name:{"table_834D68","table_834D28","table_834CE8","table_834CA8","table_834C68"}){
        --diagnostic_stage;
        code.push_back(load(out,name,addresses));reinterpret_cast<void(__cdecl*)()>(code.back().bytes)();guards();
    }
    auto initial_tables=tables.data;
    Code selector=load(out,"selector",addresses);code.push_back(selector);
    std::ofstream rows(out/L"vectors.csv");
    rows<<"count,heading,actor13,before_x,before_y,after_x,after_y,scroll_x,scroll_y,scroll_calls,fallback_calls\n";
    int count=0;
    for(U frames:{4u,6u,8u,12u})for(U heading=0;heading<8;++heading)for(U phase:{1u,2u})
        for(auto origin:std::array<std::array<int,2>,3>{{{{48,48}},{{0,0}},{{-4096,4096}}}}){
            actor.data.fill(0);actor.data[0x11]=static_cast<unsigned char>(heading);actor.data[0x13]=static_cast<unsigned char>(phase);
            auto original_actor=actor.data;
            camera_x.value=static_cast<U>(origin[0]);camera_y.value=static_cast<U>(origin[1]);
            scroll_x=scroll_y=scroll_calls=fallback_calls=0;
            diagnostic_stage=count;
            invoke(selector.bytes,frames);guards();
            require(actor.data==original_actor&&tables.data==initial_tables,"Read-only actor/table mutated");
            rows<<frames<<","<<heading<<","<<phase<<","<<origin[0]<<","<<origin[1]<<","<<static_cast<int32_t>(camera_x.value)
                <<","<<static_cast<int32_t>(camera_y.value)<<","<<scroll_x<<","<<scroll_y<<","<<scroll_calls<<","<<fallback_calls<<"\n";
            ++count;
        }
    for(auto c:code)require(VirtualFree(c.bytes,0,MEM_RELEASE)!=0,"Code release");
    std::cout<<"{\"status\":\"original_camera_selector_fragment_executed\",\"cases\":"<<count
        <<",\"guard_failures\":0,\"whole_callback_executed\":false,\"scroll_tail_stubbed\":true,\"game_sessions\":0}"<<std::endl;
    return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<std::endl;return 1;}}
