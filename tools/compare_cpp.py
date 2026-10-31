"""Build unchanged C++ sources and Rust, compare behavior, and measure matched release kernels.

Usage: python tools/compare_cpp.py --cpp ../omath [--repeats 5] [--compiler clang++]
No sources or build files in the C++ checkout are changed.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def run(args, **kwargs):
    return subprocess.run([str(arg) for arg in args], check=True, capture_output=True,
                          text=True, cwd=ROOT, **kwargs).stdout


def compiler_environment(compiler):
    env = os.environ.copy()
    if compiler:
        return compiler, env
    if os.name != "nt":
        return shutil.which("clang++") or shutil.which("g++") or "c++", env
    vswhere = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")) / "Microsoft Visual Studio/Installer/vswhere.exe"
    installation = run([vswhere, "-latest", "-prerelease", "-products", "*", "-property", "installationPath"]).strip()
    if not installation:
        raise RuntimeError("No Visual Studio C++ installation found; supply --compiler")
    script = Path(installation) / "Common7/Tools/VsDevCmd.bat"
    # Read the developer environment once; compiler arguments below use a list, never a shell.
    helper = ROOT / "target/cpp-comparison/environment.cmd"
    helper.write_text(f'@echo off\ncall "{script}" -arch=x64 -host_arch=x64 >nul\nif errorlevel 1 exit /b 1\nset\n')
    environment = subprocess.run(["cmd.exe", "/d", "/c", helper.name], cwd=helper.parent,
                                 check=True, capture_output=True, text=True).stdout
    for line in environment.splitlines():
        if "=" in line and not line.startswith("="):
            key, value = line.split("=", 1)
            env[key] = value
    compiler = shutil.which("cl.exe", path=env.get("Path", env.get("PATH")))
    if not compiler:
        raise RuntimeError("cl.exe is missing from the Visual Studio developer environment")
    return compiler, env


def compile_reference(cpp, compiler, env, output):
    sources = [ROOT / "tests/cpp/reference.cpp"] + [cpp / "source" / name for name in [
        "pathfinding/navigation_mesh.cpp", "utility/pattern_scan.cpp", "hud/canvas_box.cpp",
        "hud/widget_render.cpp", "hud/entity_overlay.cpp", "hud/screen_overlay.cpp"]]
    msvc = Path(compiler).name.lower() in {"cl", "cl.exe"}
    if msvc:
        args = [compiler, "/nologo", "/std:c++latest", "/EHsc", "/O2", "/fp:precise", "/MD",
                f"/I{cpp / 'include'}", f"/Fo{output.parent}{os.sep}", *sources, f"/Fe{output}"]
    else:
        args = [compiler, "-std=c++23", "-O3", "-ffp-contract=off", f"-I{cpp / 'include'}", *sources, "-o", output]
    run(args, env=env)
    return args


def compare_tokens(cpp, rust):
    left, right = cpp.splitlines(), rust.splitlines()
    if len(left) != len(right):
        raise AssertionError(f"row counts differ: C++ {len(left)}, Rust {len(right)}")
    for row, (a, b) in enumerate(zip(left, right), 1):
        if len(a.split()) != len(b.split()):
            raise AssertionError(f"row {row} shapes differ:\nC++ {a}\nRust {b}")
        for x, y in zip(a.split(), b.split()):
            if x == y:
                continue
            try:
                equal = math.isclose(float(x), float(y), rel_tol=2e-5, abs_tol=2e-5)
            except ValueError:
                equal = False
            if not equal:
                raise AssertionError(f"row {row} differs:\nC++ {a}\nRust {b}")
    return len(left)


def normalized_nav(text):
    # Vertex iteration order is intentionally unspecified in both implementations.
    lines = iter(text.splitlines())
    records = []
    for line in lines:
        fields = line.split()
        neighbors = tuple(next(lines).strip() for _ in range(int(fields[3])))
        records.append((line.strip(), neighbors))
    return sorted(records)


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--cpp", type=Path, default=ROOT.parent / "omath")
    parser.add_argument("--compiler")
    parser.add_argument("--repeats", type=int, default=5)
    args = parser.parse_args()
    if args.repeats < 1:
        parser.error("--repeats must be positive")
    cpp = args.cpp.resolve()
    output = ROOT / "target/cpp-comparison"
    output.mkdir(parents=True, exist_ok=True)
    compiler, env = compiler_environment(args.compiler)
    reference = output / ("reference.exe" if os.name == "nt" else "reference")
    flags = compile_reference(cpp, compiler, env, reference)
    run(["cargo", "build", "--release", "--example", "parity_probe"])
    probe = ROOT / "target/release/examples" / ("parity_probe.exe" if os.name == "nt" else "parity_probe")
    report = {"host": platform.platform(), "cpu": platform.processor(), "compiler": compiler,
              "compiler_flags": [str(flag) for flag in flags], "rustc": run(["rustc", "-Vv"]).strip(),
              "cpp_revision": run(["git", "-C", cpp, "rev-parse", "HEAD"]).strip(), "cases": {}}
    report["cpp_dirty_files"] = run(["git", "-C", cpp, "status", "--short"]).splitlines()
    report["cpp_source_sha256"] = {str(path.relative_to(cpp)):hashlib.sha256(path.read_bytes()).hexdigest()
        for folder in ["include/omath","source/hud","source/pathfinding","source/utility"]
        for path in sorted((cpp / folder).rglob("*")) if path.suffix in {".hpp",".cpp"}}
    for mode in ["cases", "hud"]:
        a, b = run([reference, mode]), run([probe, mode])
        (output / f"{mode}-cpp.txt").write_text(a)
        (output / f"{mode}-rust.txt").write_text(b)
        report["cases"][mode] = compare_tokens(a, b)
    fixture = output / "nav-input.txt"
    fixture.write_text("1 2.5 -3 2 jump\n4 0.25 0\n-0 7 8\n1000000 999999.5 1234567 0 -\n0.0001 0.00001 -0.000012345 0 teleport\n3.1415927 1.5e30 1e-30 0 -\n")
    cpp_nav, rust_nav = run([reference, "nav", fixture]), run([probe, "nav", fixture])
    if normalized_nav(cpp_nav) != normalized_nav(rust_nav):
        raise AssertionError(f"nav serialization differs:\nC++ {cpp_nav}\nRust {rust_nav}")
    # Both programs consume the other one's output and reach the same serialized graph.
    for name, text, consumer in [("cpp", cpp_nav, probe), ("rust", rust_nav, reference)]:
        path = output / f"nav-{name}.txt"
        path.write_text(text)
        if normalized_nav(run([consumer, "nav", path])) != normalized_nav(text):
            raise AssertionError(f"nav {name} -> other language roundtrip differs")
    report["cases"]["nav_cross_language_roundtrips"] = 2
    timings = {}
    # Warm up both executables; process startup is outside the timed regions.
    for program in [reference, probe]:
        run([program, "bench"])
    for label, program in [("cpp", reference), ("rust", probe)]:
        for _ in range(args.repeats):
            for line in run([program, "bench"]).splitlines():
                kernel, count, nanos, checksum = line.split()
                entry = timings.setdefault(kernel, {"iterations": int(count)})
                entry.setdefault(label, []).append(int(nanos))
                if "checksum" in entry and not math.isclose(entry["checksum"], float(checksum), rel_tol=1e-9):
                    raise AssertionError(f"{kernel} benchmark checksums differ")
                entry["checksum"] = float(checksum)
    report["benchmarks"] = {}
    for kernel, measurements in timings.items():
        cpp_ns, rust_ns = statistics.median(measurements["cpp"]), statistics.median(measurements["rust"])
        report["benchmarks"][kernel] = {**measurements, "cpp_median_ns": cpp_ns, "rust_median_ns": rust_ns,
                                          "rust_over_cpp": rust_ns / cpp_ns}
    report["timestamp_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    destination = output / "report.json"
    destination.write_text(json.dumps(report, indent=2))
    print(json.dumps({"cases": report["cases"], "benchmarks": report["benchmarks"]}, indent=2))
    print(f"Report: {destination}")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        print(error.stdout or "")
        print(error.stderr or "")
        raise
