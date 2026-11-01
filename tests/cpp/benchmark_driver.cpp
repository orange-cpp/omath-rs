#include <benchmark/benchmark.h>
#include <algorithm>
int main(int argc,char**argv){
    bool list=false;std::int64_t iterations=0;std::string filter;
    for(int i=1;i<argc;++i){std::string arg=argv[i];if(arg=="--list")list=true;else if(arg=="--smoke")iterations=1;else if(arg.starts_with("--iterations="))iterations=std::stoll(arg.substr(13));else if(arg.starts_with("--filter="))filter=arg.substr(9);}
    for(auto* r:benchmark::registry()){
        auto arguments=r->arguments;if(arguments.empty())arguments.push_back(0);
        for(auto argument:arguments){
            auto name=r->name+(r->arguments.empty()?"":"/"+std::to_string(argument));
            if(name.find(filter)==std::string::npos)continue;
            if(list){std::cout<<name<<'\n';continue;}
            benchmark::State state{iterations?iterations:r->count,argument};
            try {r->function(state);}
            catch(const std::exception&error){std::cout<<"error "<<name<<' '<<error.what()<<'\n';continue;}
            std::cout<<name<<' '<<state.executed<<' '<<state.nanos<<'\n';
            for(auto&[key,value]:state.counters)std::cout<<"counter "<<name<<' '<<key<<' '<<value<<'\n';
        }
    }
    for(auto*r:benchmark::registry())delete r;
}
