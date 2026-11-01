// Minimal driver for unchanged upstream benchmark sources. This is not Google
// Benchmark: it provides bounded iterations, timing and counters for port checks.
#pragma once
#include <chrono>
#include <cstdint>
#include <iostream>
#include <map>
#include <string>
#include <vector>
#ifdef _MSC_VER
#include <intrin.h>
#endif
namespace benchmark {
using Clock=std::chrono::steady_clock;
struct State {
    std::int64_t count,argument; Clock::time_point started; std::int64_t nanos=0,executed=0;
    std::map<std::string,double> counters;
    struct Iterator { State* state; std::int64_t index;
        bool operator!=(const Iterator& other) const { if(index==other.index){state->nanos=std::chrono::duration_cast<std::chrono::nanoseconds>(Clock::now()-state->started).count();return false;}return true; }
        void operator++(){++index;++state->executed;} std::int64_t operator*() const{return index;}
    };
    Iterator begin(){started=Clock::now();return {this,0};} Iterator end(){return {this,count};}
    std::int64_t range(int)const{return argument;}std::int64_t iterations()const{return count;}
    void SetBytesProcessed(std::int64_t){} void SetItemsProcessed(std::int64_t){}
};
struct Registration {std::string name;void(*function)(State&);std::vector<std::int64_t> arguments;std::int64_t count=100;
    Registration* Arg(std::int64_t value){arguments.push_back(value);return this;} Registration* Iterations(std::int64_t value){count=value;return this;}
};
inline std::vector<Registration*>& registry(){static std::vector<Registration*> value;return value;}
inline Registration* Register(const char* name,void(*function)(State&)){auto* r=new Registration{name,function};registry().push_back(r);return r;}
#ifdef _MSC_VER
__declspec(noinline) inline void escape(const void* value){static const void* volatile sink;sink=value;_ReadWriteBarrier();}
template<class T> void DoNotOptimize(const T& value){escape(&value);}
#else
template<class T> void DoNotOptimize(const T& value){asm volatile("" : : "g"(&value) : "memory");}
#endif
}
#define OMATH_JOIN_(a,b) a##b
#define OMATH_JOIN(a,b) OMATH_JOIN_(a,b)
#define BENCHMARK(function) static auto* OMATH_JOIN(omath_benchmark_,__COUNTER__)=benchmark::Register(#function,function)
