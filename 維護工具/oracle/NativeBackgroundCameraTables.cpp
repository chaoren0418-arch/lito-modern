// Private x86 numerical oracle. Never linked into LiToModern.
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
static_assert(sizeof(void*)==4,"Original initializers require an isolated x86 process");
using U=uint32_t;
static void require(bool b,const char* m){if(!b)throw std::runtime_error(m);}
struct Guarded {U before=0xa51c0ffe;std::array<U,80> values;U after=0xfedc315a;};
static Guarded table;
static U address(const void* p){return reinterpret_cast<U>(p);}
static constexpr U first=0x834c68;
int wmain(int argc,wchar_t** argv){try{
    require(argc==2,"Expected owned output directory");
    std::filesystem::path out(argv[1]);
    std::array<std::string,5> names{"table_834D68","table_834D28","table_834CE8","table_834CA8","table_834C68"};
    std::array<U,5> starts{0x834d68,0x834d28,0x834ce8,0x834ca8,0x834c68};
    std::ofstream addresses(out/L"relocation-addresses.tsv");
    addresses<<"table_data "<<address(table.values.data())<<"\n";
    std::cout<<"{\"status\":\"isolated_original_camera_initializers_executed\",\"tables\":[";
    for(size_t group=0;group<names.size();++group){
        const auto& name=names[group];
        std::ifstream input(out/(name+"-original.bin"),std::ios::binary);
        std::vector<unsigned char> bytes((std::istreambuf_iterator<char>(input)),{});
        require(!bytes.empty()&&bytes.size()<4096,"Initializer code size");
        auto code=static_cast<unsigned char*>(VirtualAlloc(nullptr,4096,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));
        require(code!=nullptr,"Private code allocation");
        std::memcpy(code,bytes.data(),bytes.size());
        std::ifstream relocations(out/(name+"-relocations.tsv"));
        size_t offset;U original;size_t patched=0;
        while(relocations>>offset>>original){
            require(offset+4<=bytes.size()&&original>=first&&original<first+320&&original%4==0,"Relocation domain");
            U old;std::memcpy(&old,code+offset,4);require(old==original,"Relocation original bytes");
            U relocated=address(table.values.data())+original-first;
            std::memcpy(code+offset,&relocated,4);++patched;
        }
        require(patched==16,"Every initializer must write sixteen words");
        DWORD previous=0;require(VirtualProtect(code,4096,PAGE_EXECUTE_READ,&previous)!=0,"Private code protection");
        require(FlushInstructionCache(GetCurrentProcess(),code,bytes.size())!=0,"Instruction cache");
        table.values.fill(0x7eadbeef);
        reinterpret_cast<void(__cdecl*)()>(code)();
        require(table.before==0xa51c0ffe&&table.after==0xfedc315a,"Table guards");
        size_t begin=(starts[group]-first)/4;
        for(size_t i=0;i<table.values.size();++i)
            require((i>=begin&&i<begin+16)||table.values[i]==0x7eadbeef,"Initializer wrote another table");
        std::ofstream relocated(out/(name+"-relocated.bin"),std::ios::binary);
        relocated.write(reinterpret_cast<char*>(code),static_cast<std::streamsize>(bytes.size()));
        if(group)std::cout<<",";
        std::cout<<"{\"name\":\""<<name<<"\",\"values\":[";
        for(size_t direction=0;direction<8;++direction){
            if(direction)std::cout<<",";
            std::cout<<"["<<static_cast<int32_t>(table.values[begin+2*direction])<<","<<static_cast<int32_t>(table.values[begin+2*direction+1])<<"]";
        }
        std::cout<<"]}";
        require(VirtualFree(code,0,MEM_RELEASE)!=0,"Private code release");
    }
    std::cout<<"],\"initializer_functions\":5,\"heading_vectors\":40,\"guard_failures\":0,\"whole_callback_executed\":false,\"game_sessions\":0}"<<std::endl;
    return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<std::endl;return 1;}}
