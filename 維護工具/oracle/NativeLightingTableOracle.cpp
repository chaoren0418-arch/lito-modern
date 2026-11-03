// Isolated original channel-entry writer and original remap-row selector.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>

static_assert(sizeof(void*)==4,"Original fragments are x86");
static std::array<uint16_t,1024> blue{},green{},red{},unused{};
static std::array<unsigned char,512> normal_bank{},alternate_bank{};
static std::array<unsigned char,32> remap{};
static int cached_level=-1;
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);if(!f)throw std::runtime_error("Input missing");return {(std::istreambuf_iterator<char>(f)),{}};}
static void relocate(unsigned char* c,size_t n,uint32_t old,uintptr_t replacement,int expected){int found=0;for(size_t i=0;i+4<=n;++i){uint32_t v;memcpy(&v,c+i,4);if(v==old){memcpy(c+i,&replacement,4);++found;i+=3;}}if(found!=expected)throw std::runtime_error("Relocation count differs");}
static void* load(const std::filesystem::path& dir,bool entry){
 auto bytes=read(dir/(entry?L"entry-original.bin":L"selector-original.bin"));
 if(bytes.empty()||bytes.size()>512)throw std::runtime_error("Fragment extent differs");
 auto c=static_cast<unsigned char*>(VirtualAlloc(nullptr,65536,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));if(!c)throw std::runtime_error("Private allocation failed");memcpy(c,bytes.data(),bytes.size());
 if(entry){relocate(c,bytes.size(),0x81109c,reinterpret_cast<uintptr_t>(unused.data()),1);relocate(c,bytes.size(),0x80d89c,reinterpret_cast<uintptr_t>(blue.data()),1);relocate(c,bytes.size(),0x80e09c,reinterpret_cast<uintptr_t>(green.data()),1);relocate(c,bytes.size(),0x80e89c,reinterpret_cast<uintptr_t>(red.data()),1);c[bytes.size()]=0xc3;}
 else{relocate(c,bytes.size(),0x6348a4,reinterpret_cast<uintptr_t>(&cached_level),2);relocate(c,bytes.size(),0x6348a8,reinterpret_cast<uintptr_t>(normal_bank.data()),1);relocate(c,bytes.size(),0x634aa8,reinterpret_cast<uintptr_t>(alternate_bank.data()),1);relocate(c,bytes.size(),0x83009c,reinterpret_cast<uintptr_t>(remap.data()),2);}
 std::ofstream out(dir/(entry?L"entry-relocated.bin":L"selector-relocated.bin"),std::ios::binary);out.write(reinterpret_cast<char*>(c),bytes.size());out.close();if(!out)throw std::runtime_error("Retention failed");DWORD old;if(!VirtualProtect(c,65536,PAGE_EXECUTE_READ,&old)||!FlushInstructionCache(GetCurrentProcess(),c,65536))throw std::runtime_error("Code protection failed");return c;
}
__declspec(naked) static void invoke_entry(void*,void*,int,int){
 __asm {
  push ebp
  push ebx
  push esi
  mov eax,[esp+16]
  mov ebp,[esp+20]
  mov ebx,[esp+24]
  mov esi,[esp+28]
  xor edx,edx
  call eax
  pop esi
  pop ebx
  pop ebp
  ret
 }
}
using Selector=void(__fastcall*)(int,int);
int wmain(int argc,wchar_t** argv){try{
 if(argc!=2)throw std::runtime_error("Expected owned evidence directory");const std::filesystem::path dir(argv[1]);
 auto n=read(dir/L"bank-normal.bin"),a=read(dir/L"bank-alternate.bin");if(n.size()!=512||a.size()!=512)throw std::runtime_error("Bank extent differs");memcpy(normal_bank.data(),n.data(),512);memcpy(alternate_bank.data(),a.data(),512);
 auto entry=load(dir,true);auto selector=reinterpret_cast<Selector>(load(dir,false));
 std::ofstream channel_csv(dir/L"channel-vectors.csv",std::ios::binary),table_bytes(dir/L"channel-tables.bin",std::ios::binary),remap_csv(dir/L"remap-vectors.csv",std::ios::binary);
 channel_csv<<"mode,color,level,blue,green,red\n";remap_csv<<"call,requested,alternate,cached_before,cached_after,index,actual\n";
 unsigned entry_cases=0,selector_calls=0,remap_bytes=0;
 for(int mode=0;mode<2;++mode){blue.fill(0xeeee);green.fill(0xeeee);red.fill(0xeeee);
  for(int color=0;color<32;++color)for(int level=0;level<32;++level){alignas(16) int frame[16]{};int* ebp=frame+12;ebp[-7]=color*level;ebp[-2]=mode?6:5;ebp[-1]=color;const int at=color*32+level;invoke_entry(entry,ebp,mode?11:10,at*2);const unsigned value=static_cast<unsigned>(color*level/31);if(blue[at]!=value||green[at]!=(value<<(mode?6:5))||red[at]!=(value<<(mode?11:10)))throw std::runtime_error("Native channel writer mismatch");channel_csv<<mode<<','<<color<<','<<level<<','<<blue[at]<<','<<green[at]<<','<<red[at]<<'\n';++entry_cases;}
  for(const auto* table:{&blue,&green,&red})table_bytes.write(reinterpret_cast<const char*>(table->data()),2048);
 }
 auto check=[&](int requested,int alternate,const unsigned char* expected){const int before=cached_level;selector(requested,alternate);if(cached_level!=(requested>15?15:requested)||memcmp(remap.data(),expected,32))throw std::runtime_error("Original row selector mismatch");for(int i=0;i<32;++i){remap_csv<<selector_calls<<','<<requested<<','<<alternate<<','<<before<<','<<cached_level<<','<<i<<','<<unsigned(remap[i])<<'\n';++remap_bytes;}++selector_calls;};
 for(int level=0;level<16;++level)for(int alternate:{0,1,7}){cached_level=-1;remap.fill(0xee);check(level,alternate,(alternate?alternate_bank:normal_bank).data()+level*32);}
 for(int level:{16,31,255,2147483647})for(int alternate:{0,1}){cached_level=-1;remap.fill(0xee);check(level,alternate,(alternate?alternate_bank:normal_bank).data()+15*32);}
 for(int level=0;level<16;++level){cached_level=-1;remap.fill(0xee);check(level,0,normal_bank.data()+level*32);check(level,1,normal_bank.data()+level*32);const int next=(level+1)%16;check(next,1,alternate_bank.data()+next*32);}
 channel_csv.close();table_bytes.close();remap_csv.close();if(!channel_csv||!table_bytes||!remap_csv)throw std::runtime_error("Output failed");VirtualFree(entry,0,MEM_RELEASE);VirtualFree(reinterpret_cast<void*>(selector),0,MEM_RELEASE);
 std::cout<<"{\"status\":\"original_channel_entries_and_original_remap_selector_verified\",\"channel_entry_cases\":"<<entry_cases<<",\"channel_outputs\":"<<entry_cases*3<<",\"selector_calls\":"<<selector_calls<<",\"remap_bytes_checked\":"<<remap_bytes<<",\"complete_initializer_executed\":false,\"game_sessions\":0}\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
