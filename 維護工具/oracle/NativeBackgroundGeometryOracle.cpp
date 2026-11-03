// Private x86 background geometry only. The real pixel kernel is a recorder.
// This file is never linked into the Rust product.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>
static_assert(sizeof(void*)==4,"Original geometry requires x86");
using U=uint32_t;
[[noreturn]] static void die(const char* s){std::fprintf(stderr,"oracle guard: %s\n",s);ExitProcess(77);}
static void need(bool b,const char* s){if(!b)die(s);}
static U addr(const void* p){return reinterpret_cast<U>(p);}
struct Slot {U before=0x1a2b3c4d,value=0,after=0xa1b2c3d4;};
static constexpr U keys[]={0x4f7430,0x62f324,0x62f4c0,0x62f4c4,0x6476d0,0x75a998,0x75a99c,0x768878,0x8316e4,0x8316e8,0x8316ec,0x834bd4,0x834ee4};
static std::array<Slot,13> slots;
static U& slot(U key){for(size_t i=0;i<slots.size();++i)if(keys[i]==key)return slots[i].value;die("unknown data relocation");}
struct ResourceTable {U before=0x11112222;std::array<U,924> entries{};U after=0x33334444;};
struct Image {U before=0x55556666;std::array<U,4> data{};U after=0x77778888;};
struct ImageObject {U vtable=0,data=0,id=0,guard=0x9999aaaa;};
struct Actor {U before=0x12344321;std::array<U,13> data{};U after=0x87655678;};
struct Surface {U tag=0xabcd5678;};
static ResourceTable resources;
static std::array<Image,5> images;
static std::array<ImageObject,5> objects;
static Actor actor;
static Surface surface;
static U encoded_class=0;
static constexpr U ids[]={800,803,892,922,923};
static std::ofstream pixels,counters;
static int current_case=0,current_calls=0,total_calls=0;
[[noreturn]] static void __fastcall forbidden(){die("unexpected loader/error path");}
static int image_id(const void* data){for(size_t i=0;i<images.size();++i)if(data==images[i].data.data())return ids[i];die("unowned image pointer");}
struct Rect {int left,top,right,bottom;};
static void __fastcall record(const void* base,Surface* target,Rect* src,int x,int y,
                             const void* overlay,int ox,int oy,int strength,int mode){
    need(target==&surface&&src!=nullptr,"kernel target/rectangle");
    need(current_calls<9&&src->left>=0&&src->top>=0&&src->right<=384&&src->bottom<=384,"source clipping");
    need(src->right>src->left&&src->bottom>src->top&&x>=0&&y>=0,"positive region");
    need(x+src->right-src->left<=744&&y+src->bottom-src->top<=552,"destination clipping");
    pixels<<current_case<<','<<current_calls<<','<<image_id(base)<<','<<image_id(overlay)<<','
          <<src->left<<','<<src->top<<','<<src->right<<','<<src->bottom<<','<<x<<','<<y<<','
          <<ox<<','<<oy<<','<<strength<<','<<mode<<'\n';
    ++current_calls;++total_calls;
}
static std::vector<unsigned char> read(const std::filesystem::path& p){std::ifstream f(p,std::ios::binary);need(bool(f),"missing input");return {(std::istreambuf_iterator<char>(f)),{}};}
static void write(const std::filesystem::path& p,const std::vector<unsigned char>& b){std::ofstream f(p,std::ios::binary);f.write(reinterpret_cast<const char*>(b.data()),b.size());need(bool(f),"binary output");}
static std::vector<int> row(const std::string& line){std::istringstream f(line);std::string part;std::vector<int> v;while(std::getline(f,part,','))v.push_back(std::stoi(part));return v;}
using Geometry=void(__fastcall*)(int);
int wmain(int argc,wchar_t** argv){
    need(argc==2,"expected retained preparation directory");std::filesystem::path out(argv[1]);
    auto original=read(out/"geometry-original.bin");need(original.size()==1294,"original function size");
    auto code=original;auto memory=static_cast<unsigned char*>(VirtualAlloc(nullptr,code.size(),MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE));need(memory!=nullptr,"code allocation");
    std::ofstream addresses(out/"private-addresses.csv");need(bool(addresses),"address output");
    addresses<<"kind,original,address\n";
    for(U key:keys)addresses<<"D,"<<key<<','<<addr(&slot(key))<<'\n';
    addresses<<"C,"<<0x497c60<<','<<addr(reinterpret_cast<void*>(&forbidden))<<'\n';
    addresses<<"B,"<<0x487e10<<','<<addr(memory)<<'\n';addresses.close();need(bool(addresses),"address flush");
    std::ifstream reloc(out/"relocations.csv");need(bool(reloc),"relocation input");std::string line;int data_patches=0,call_patches=0;
    while(std::getline(reloc,line)){
        std::istringstream s(line);std::string kind,offset_text,key_text;std::getline(s,kind,',');std::getline(s,offset_text,',');std::getline(s,key_text,',');
        U offset=static_cast<U>(std::stoul(offset_text)),key=static_cast<U>(std::stoul(key_text));need(offset+4<=code.size(),"relocation bound");
        U prior=0;std::memcpy(&prior,code.data()+offset,4);U value=0;
        if(kind=="D"){need(prior==key,"original data operand");value=addr(&slot(key));++data_patches;}
        else {need(kind=="C"&&key==0x497c60,"unexpected call relocation");need(static_cast<U>(0x487e10+offset+4+prior)==key,"original call target");value=addr(reinterpret_cast<void*>(&forbidden))-addr(memory+offset+4);++call_patches;}
        std::memcpy(code.data()+offset,&value,4);
    }
    need(data_patches==31&&call_patches==15,"relocation totals");
    std::memcpy(memory,code.data(),code.size());DWORD before=0;need(VirtualProtect(memory,code.size(),PAGE_EXECUTE_READ,&before)!=0,"code protection");FlushInstructionCache(GetCurrentProcess(),memory,code.size());write(out/"geometry-relocated.bin",code);
    U loader_entry=addr(reinterpret_cast<void*>(&forbidden));
    for(size_t i=0;i<objects.size();++i){objects[i].vtable=addr(&loader_entry);objects[i].data=addr(images[i].data.data());objects[i].id=ids[i];resources.entries[ids[i]]=addr(&objects[i]);}
    slot(0x4f7430)=addr(reinterpret_cast<void*>(&forbidden));slot(0x6476d0)=addr(&surface);slot(0x768878)=addr(reinterpret_cast<void*>(&record));
    slot(0x8316e4)=0xb104fa87;slot(0x8316e8)=addr(&encoded_class);slot(0x8316ec)=0xa55a8877;
    slot(0x834bd4)=addr(resources.entries.data());slot(0x834ee4)=addr(actor.data.data());
    auto fn=reinterpret_cast<Geometry>(memory);
    pixels.open(out/"calls-native.csv");counters.open(out/"counts-native.csv");need(bool(pixels)&&bool(counters),"CSV output");
    pixels<<"case,ordinal,base_id,overlay_id,left,top,right,bottom,x,y,offset_x,offset_y,strength,mode\n";
    counters<<"case,calls\n";
    std::ifstream cases(out/"cases.csv");need(bool(cases),"case input");std::getline(cases,line);int count=0;
    while(std::getline(cases,line)){
        auto c=row(line);need(c.size()==10&&c[0]==count,"case sequence");current_case=c[0];current_calls=0;
        slot(0x62f324)=static_cast<U>(c[1]);encoded_class=static_cast<U>(c[3])^slot(0x8316ec);
        actor.data[11]=static_cast<U>(c[4]);actor.data[12]=static_cast<U>(c[5]);
        slot(0x75a998)=static_cast<U>(c[6]);slot(0x75a99c)=static_cast<U>(c[7]);slot(0x62f4c0)=static_cast<U>(c[8]);slot(0x62f4c4)=static_cast<U>(c[9]);
        auto slot_before=slots;auto resource_before=resources;auto image_before=images;auto object_before=objects;auto actor_before=actor;auto class_before=encoded_class;
        fn(c[2]);
        need(std::memcmp(&slot_before,&slots,sizeof(slots))==0&&std::memcmp(&resource_before,&resources,sizeof(resources))==0,"global/resource table changed");
        need(std::memcmp(&image_before,&images,sizeof(images))==0&&std::memcmp(&object_before,&objects,sizeof(objects))==0,"image input changed");
        need(std::memcmp(&actor_before,&actor,sizeof(actor))==0&&class_before==encoded_class&&surface.tag==0xabcd5678,"actor/class/surface changed");
        counters<<current_case<<','<<current_calls<<'\n';++count;
    }
    pixels.close();counters.close();need(bool(pixels)&&bool(counters),"CSV flush");
    std::cout<<"{\"status\":\"original_geometry_ran_with_recording_pixel_stub\",\"cases\":"<<count<<",\"recorded_calls\":"<<total_calls<<",\"data_relocations\":"<<data_patches<<",\"error_call_relocations\":"<<call_patches<<",\"unexpected_loader_or_error_calls\":0,\"guards_unchanged\":true}\n";
    VirtualFree(memory,0,MEM_RELEASE);return 0;
}
