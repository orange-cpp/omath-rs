"""Run all seven unchanged C++ benchmark suites against their Rust counterparts.
The small C++ driver is a bounded workload runner, not Google Benchmark.
Timings include different language/driver optimization barriers; treat ratios as
local measurements, not a general performance claim. Defaults to 100 iterations.
"""
import argparse,json,os,statistics,subprocess,sys
from pathlib import Path
import compare_cpp as common
ROOT=common.ROOT
def main():
    parser=argparse.ArgumentParser(__doc__)
    parser.add_argument('--cpp',type=Path,default=ROOT.parent/'omath')
    parser.add_argument('--iterations',type=int,default=100)
    parser.add_argument('--repeats',type=int,default=3)
    args=parser.parse_args()
    if args.iterations<1 or args.repeats<1:parser.error('counts must be positive')
    cpp=args.cpp.resolve();output=ROOT/'target/upstream-benchmarks';output.mkdir(parents=True,exist_ok=True)
    compiler,env=common.compiler_environment(None)
    binary=output/('cpp.exe' if os.name=='nt' else 'cpp')
    sources=[ROOT/'tests/cpp/benchmark_driver.cpp',*sorted((cpp/'benchmark').glob('benchmark_*.cpp')),cpp/'source/utility/pattern_scan.cpp']
    if Path(compiler).name.lower()=='cl.exe':
        command=[compiler,'/nologo','/std:c++latest','/EHsc','/O2','/fp:precise','/MD',f'/I{ROOT / "tests/cpp"}',f'/I{cpp / "include"}',f'/Fo{output}{os.sep}',*sources,f'/Fe{binary}']
    else:command=[compiler,'-std=c++23','-O3','-ffp-contract=off',f'-I{ROOT / "tests/cpp"}',f'-I{cpp / "include"}',*sources,'-o',binary]
    common.run(command,env=env)
    artifacts=common.run(['cargo','build','--locked','--release','--bench','upstream','--message-format=json'])
    rust=next(Path(event['executable']) for line in artifacts.splitlines() if (event:=json.loads(line)).get('reason')=='compiler-artifact' and event['target']['name']=='upstream' and event.get('executable'))
    cpp_names=sorted(common.run([binary,'--list']).splitlines());rust_names=sorted(common.run([rust,'--list']).splitlines())
    if cpp_names!=rust_names:raise AssertionError({'missing_rust':sorted(set(cpp_names)-set(rust_names)),'extra_rust':sorted(set(rust_names)-set(cpp_names))})
    timings={name:{} for name in cpp_names};counters={};failures={}
    for repeat in range(args.repeats):
        for label,program in ([('cpp',binary),('rust',rust)] if repeat%2==0 else [('rust',rust),('cpp',binary)]):
            text=common.run([program,f'--iterations={args.iterations}'])
            (output/f'{label}-{repeat}.txt').write_text(text)
            lines=text.splitlines()
            for line in lines:
                if line.startswith('error '):
                    _,name,error=line.split(' ',2)
                    failures.setdefault(name,{'language':label,'requested_iterations':args.iterations,'error':error,'bounded_iterations':min(10,args.iterations)})
            # MSVC's monotonic_buffer_resource::release does not rewind the
            # caller-provided initial block. Preserve upstream code and record
            # its exhaustion; compare both languages with a fresh, bounded run.
            for name,failure in failures.items():
                bounded=common.run([program,f'--iterations={failure["bounded_iterations"]}',f'--filter={name}']).splitlines()
                lines=[line for line in lines if not line.startswith(name+' ') and not line.startswith('error '+name+' ')]+bounded
            for line in lines:
                fields=line.split()
                if fields[0]=='error':raise AssertionError('bounded benchmark also failed: '+line)
                if fields[0]=='counter':
                    _,name,key,value=fields;counters.setdefault(name,{}).setdefault(key,{})[label]=float(value)
                else:
                    name,count,nanos=fields
                    expected=failures.get(name,{}).get('bounded_iterations',args.iterations)
                    if int(count)!=expected:raise AssertionError(line)
                    timings[name]['iterations']=int(count)
                    timings[name].setdefault(label,[]).append(int(nanos))
    for name,row in counters.items():
        for key,values in row.items():
            if key in {'hit_pct','on_screen_pct','faces'}:
                if set(values)!={'cpp','rust'} or abs(values['cpp']-values['rust'])>0.001:raise AssertionError((name,key,values))
    for row in timings.values():
        if not {'cpp','rust'}<=set(row):raise AssertionError(row)
        row['cpp_median_ns']=statistics.median(row['cpp']);row['rust_median_ns']=statistics.median(row['rust']);row['rust_over_cpp']=row['rust_median_ns']/max(1,row['cpp_median_ns'])
    report={'cpp_revision':common.reference_metadata(cpp)[0],'cpp_dirty_files':common.reference_metadata(cpp)[1],'rustc':common.run(['rustc','-Vv']),'requested_iterations':args.iterations,'repeats':args.repeats,'cases':len(timings),'benchmarks':timings,'counters':counters,'upstream_failures':failures,'limitations':['C++ uses bounded driver, not Google Benchmark.','Rust reusable EPA workspace adapts PMR; allocator implementations differ.','Optimization barriers differ; timings are local workload observations.','Bounded PMR reruns are recorded per case, not hidden as successful long runs.']}
    report['rust_snapshot']=common.rust_metadata()
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'{len(timings)} benchmark cases registered and executed in both languages; projection/hit-rate/face counters match. Report: {output / "report.json"}')
if __name__=='__main__':
    try:main()
    except subprocess.CalledProcessError as error:
        print(error.stdout or '');print(error.stderr or '',file=sys.stderr);raise
