// Appended to the qualified private oracle helpers, never a product dependency.
struct ImageObject {U table=0,data=0,id=0,guard=0x9999aaaa;};
struct ResourceTable {U before=0x11112222;std::array<U,924> entries{};U after=0x33334444;};
struct Actor {U before=0x12344321;std::array<U,13> data{};U after=0x87655678;};
using Geometry=void(__fastcall*)(int);
static Image real_image(const std::filesystem::path& out,int id){
    auto raw=read(out/(std::to_string(id)+".img"));need(raw.size()>=8,"real IMG header");
    W width=0,height=0;std::memcpy(&width,raw.data(),2);std::memcpy(&height,raw.data()+2,2);
    need(width>0&&height>0&&raw.size()==8+width*height*2,"real IMG dimensions");
    Image result{width,height,std::vector<unsigned char>(GUARD+raw.size()+GUARD,0xa5)};
    std::memcpy(result.data(),raw.data(),raw.size());return result;
}
int wmain(int argc,wchar_t** argv){
    need(argc==2,"expected preparation directory");std::filesystem::path out(argv[1]);
    auto initial=read(out/"data-initial.bin");need(initial.size()==GE-GB,"data arena length");arena.assign(GUARD+GE-GB+GUARD,0xa5);
    auto constant_input=read(out/"screen-constants.bin");need(constant_input.size()==constants.size(),"constant length");std::memcpy(constants.data(),constant_input.data(),constants.size());
    // CRT provenance setup is inserted verbatim from the qualified oracle.
    /* CRT_SETUP */
    std::ofstream addresses(out/"private-addresses.csv");addresses<<"function,kind,offset,address\n";
    auto init=reinterpret_cast<Init>(load_code(out,"init",addresses));
    auto rgb555=reinterpret_cast<Kernel>(load_code(out,"rgb555",addresses));
    auto geometry=reinterpret_cast<Geometry>(load_code(out,"geometry",addresses));
    addresses.close();need(bool(addresses),"address output");
    std::memcpy(global(GB),initial.data(),initial.size());*global(0x641298)=0;ftol_calls=0;init();
    need(ftol_calls==1024&&get(0x768878)==0x4bf500,"original RGB555 initialization");arena_guards();
    write(out/"initialized-mode0.bin",global(GB),GE-GB);
    vtable[25]=reinterpret_cast<void*>(&lock);vtable[27]=reinterpret_cast<void*>(&forbidden);vtable[32]=reinterpret_cast<void*>(&unlock);
    const int ids[]={800,803,892,922,923};std::array<Image,5> images;
    std::array<std::vector<unsigned char>,5> image_copies;std::array<ImageObject,5> objects;
    ResourceTable resources;Actor actor;U encoded_class=0,loader=addr(reinterpret_cast<void*>(&forbidden));
    for(size_t i=0;i<5;++i){images[i]=real_image(out,ids[i]);image_copies[i]=images[i].bytes;objects[i]={addr(&loader),addr(images[i].data()),static_cast<U>(ids[i]),0x9999aaaa};resources.entries[ids[i]]=addr(&objects[i]);}
    const auto resources_before=resources;const auto objects_before=objects;
    set(0x4f7430,addr(reinterpret_cast<void*>(&forbidden)));set(0x768878,addr(reinterpret_cast<void*>(rgb555)));
    set(0x8316e4,0xb104fa87);set(0x8316e8,addr(&encoded_class));set(0x8316ec,0xa55a8877);
    set(0x834bd4,addr(resources.entries.data()));set(0x834ee4,addr(actor.data.data()));
    std::ifstream cases(out/"cases.csv");need(bool(cases),"case input");std::string line;std::getline(cases,line);
    std::ofstream results(out/"results-native.csv");results<<"case,calls,locks,unlocks,changed_words\n";
    U count=0,total_calls=0,total_changes=0;
    while(std::getline(cases,line)){
        auto c=row(line);need(c.size()==11&&c[0]==static_cast<int>(count),"case shape/order");
        std::vector<W> frame(GUARD+GATE_WIDTH*HEIGHT+GUARD,0xa55a);W* target=frame.data()+GUARD;std::fill(target,target+GATE_WIDTH*HEIGHT,0x1777);
        std::vector<unsigned char> gate(GUARD+GATE_WIDTH*HEIGHT+GUARD,0x5a);auto plane=gate.data()+GUARD;
        for(U y=0;y<HEIGHT;++y)for(U x=0;x<GATE_WIDTH;++x)plane[y*GATE_WIDTH+x]=c[10]==0?0:c[10]==1?1:((x+y)%2?7:0);
        const auto gate_before=gate;Surface surface{vtable.data(),target,GATE_WIDTH,HEIGHT};
        set(0x6476c4,addr(plane));set(0x831560,addr(target+GATE_WIDTH*HEIGHT));set(0x6476d0,addr(&surface));
        set(0x62f324,static_cast<U>(c[1]));encoded_class=static_cast<U>(c[3])^get(0x8316ec);
        actor.data[11]=static_cast<U>(c[4]);actor.data[12]=static_cast<U>(c[5]);
        set(0x75a998,static_cast<U>(c[6]));set(0x75a99c,static_cast<U>(c[7]));set(0x62f4c0,static_cast<U>(c[8]));set(0x62f4c4,static_cast<U>(c[9]));
        const auto arena_before=arena;const auto external_before=external_values;const auto actor_before=actor;const auto class_before=encoded_class;
        geometry(c[2]);
        need(surface.locks>0&&surface.locks<=9&&surface.unlocks==surface.locks*2,"surface lifecycle");
        need(arena==arena_before&&external_values==external_before&&gate==gate_before,"global/gate changed");arena_guards();
        need(std::memcmp(&resources,&resources_before,sizeof(resources))==0&&std::memcmp(&objects,&objects_before,sizeof(objects))==0,"resource object changed");
        need(std::memcmp(&actor,&actor_before,sizeof(actor))==0&&encoded_class==class_before,"actor/class changed");
        for(size_t i=0;i<5;++i)need(images[i].bytes==image_copies[i],"real IMG changed");
        for(U i=0;i<GUARD;++i)need(frame[i]==0xa55a&&frame[GUARD+GATE_WIDTH*HEIGHT+i]==0xa55a,"frame guard");
        U changes=0;for(U i=0;i<GATE_WIDTH*HEIGHT;++i)if(target[i]!=0x1777)++changes;
        write(out/("frame-"+std::to_string(count)+".bin"),target,GATE_WIDTH*HEIGHT*2);
        results<<count<<','<<surface.locks<<','<<surface.locks<<','<<surface.unlocks<<','<<changes<<'\n';
        total_calls+=surface.locks;total_changes+=changes;++count;
    }
    results.close();need(count==24&&bool(results),"case completion");
    std::cout<<"{\"status\":\"original_geometry_and_RGB555_pixels_ran_with_real_warm_IMG\",\"cases\":"<<count<<",\"pixel_calls\":"<<total_calls<<",\"changed_words\":"<<total_changes<<",\"host_ftol_calls\":"<<ftol_calls<<",\"guards_unchanged\":true,\"resource_loader_executed\":false,\"historical_CRT_DLL_identity_verified\":false}\n";
    FreeLibrary(crt);return 0;
}
