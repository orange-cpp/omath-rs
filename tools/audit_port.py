"""Audit port completeness beyond the baseline comparison. Exit 1 means unresolved gaps.

Does not change library implementations or C++ sources. Writes evidence to target/port-audit.
Use --cpp ../omath and optionally --imgui-include <directory containing imgui.h>.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import compare_cpp as compare

ROOT = Path(__file__).resolve().parents[1]


def header_mapping(relative):
    name = relative.stem
    parts = relative.parts
    if parts[0] == "internal":
        return ["src/scalar.rs"], "delegated to Rust scalar/standard-library operations; const capability differs"
    if relative.as_posix() == "omath.hpp":
        return ["src/lib.rs"], "crate module exports"
    if len(parts) == 1:
        return [f"src/{name}.rs"], "Rust module"
    if parts[0] == "engines":
        engine = parts[1].removesuffix("_engine")
        if name == "primitives":
            return [f"src/engines/{engine}/mod.rs"], "engine aliases"
        if name in {"mesh_trait", "camera_trait"}:
            name = name.removesuffix("_trait")
        return [f"src/engines/{engine}/{name}.rs"], "engine module; shared macros also live in src/engines/mod.rs"
    if parts[0] == "linear_algebra" and name.startswith("vector"):
        return ["src/linear_algebra/vector.rs"], "shared vector implementation"
    if parts[0] == "3d_primitives":
        return [f"src/primitives/{'box_primitive' if name == 'box' else name}.rs"], "Rust primitive"
    if parts[0] in {"hooks", "rev_eng", "containers", "lua"}:
        return [f"src/{parts[0]}/mod.rs"], "Rust module and adjacent implementation files"
    if parts[0] == "hud":
        target = {"canvas_box": "mod", "gradient": "mod", "hud_renderer_interface": "mod",
                  "entity_overlay_widgets": "widget", "entity_overlay": "overlay", "screen_overlay": "overlay",
                  "widget_render": "render"}.get(name, name)
        return [f"src/hud/{target}.rs"], "renderer/widget implementation; API presence is not proof of behavior parity"
    if name == "proj_pred_engine_avx2":
        name = "proj_pred_engine_batched"
    if name == "pred_engine_concept":
        name = "pred_engine"
    return [f"src/{parts[0]}/{name}.rs"], "Rust module"


def rust_conversion_api(output):
    artifacts = compare.run(["cargo", "build", "--locked", "--features", "imgui", "--lib", "--message-format", "json"])
    library = None
    for line in artifacts.splitlines():
        event = json.loads(line)
        if event.get("reason") == "compiler-artifact" and event["target"]["name"] == "omath":
            library = next((path for path in event["filenames"] if path.endswith(".rlib")), library)
    if not library:
        raise RuntimeError("Cargo did not report the omath library artifact")
    source = output / "imgui-api-rust.rs"
    source.write_text("pub fn check() { let _ = omath::Vector2::<f32>::from_im_vec2; let _ = omath::Vector4::<f32>::from_im_vec4; }\n", encoding="utf-8")
    process = subprocess.run(["rustc", "--edition=2024", "--crate-type=lib", "--crate-name=imgui_api_audit",
                              str(source), "--extern", f"omath={library}", "-L", f"dependency={ROOT / 'target/debug/deps'}",
                              "--out-dir", str(output)], cwd=ROOT, capture_output=True, text=True)
    (output / "imgui-api-rust.log").write_text(process.stderr, encoding="utf-8")
    return {"compiled": process.returncode == 0, "diagnostics": "target/port-audit/imgui-api-rust.log"}


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--cpp", type=Path, default=ROOT.parent / "omath")
    parser.add_argument("--imgui-include", type=Path)
    parser.add_argument("--check-x86", action="store_true", help="check all features for i686-pc-windows-msvc (requires installed Rust target)")
    args = parser.parse_args()
    cpp = args.cpp.resolve()
    output = ROOT / "target/port-audit"
    output.mkdir(parents=True, exist_ok=True)
    compiler, env = compare.compiler_environment(None)
    binary = output / ("reference.exe" if sys.platform == "win32" else "reference")
    compiler_flags = compare.compile_reference(cpp, compiler, env, binary, ROOT / "tests/cpp/port_audit.cpp")
    compare.run(["cargo", "build", "--locked", "--features", "imgui", "--example", "port_audit_probe"])
    probe = ROOT / "target/debug/examples" / ("port_audit_probe.exe" if sys.platform == "win32" else "port_audit_probe")
    report = {"rust_revision": compare.run(["git", "rev-parse", "HEAD"]).strip(),
              "cpp_revision": compare.reference_metadata(cpp)[0],
              "behavior": {}, "header_mapping": []}
    report["rustc"] = compare.run(["rustc", "-Vv"]).strip()
    report["rust_snapshot"] = compare.rust_metadata()
    report["compiler_flags"] = [str(flag) for flag in compiler_flags]
    report["cpp_tree_status"] = compare.reference_metadata(cpp)[1]
    report["cpp_source_sha256"] = {path.relative_to(cpp).as_posix():hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted((cpp / "include/omath").rglob("*.hpp"))}
    for case in ["corner_glow", "dashed_right", "dashed_left", "dashed_top", "dashed_bottom", "gradient_channel", "fov_wrap", "fov_roll"]:
        a, b = compare.run([binary, case]), compare.run([probe, case])
        for language, text in [("cpp", a), ("rust", b)]:
            (output / f"{case}-{language}.txt").write_text(text, encoding="utf-8")
        try:
            report["behavior"][case] = {"status": "match", "rows": compare.compare_tokens(a, b)}
        except AssertionError as error:
            report["behavior"][case] = {"status": "mismatch", "details": str(error)}
        if case.startswith("dashed_"):
            report["behavior"][case]["cpp_outline"] = a.splitlines()[-1]
            report["behavior"][case]["rust_outline"] = b.splitlines()[-1]
        if case.startswith("fov_"):
            report["behavior"][case]["classification"] = "intentional correction; C++ template compiles and is callable"
        if case == "dashed_top":
            report["behavior"][case]["classification"] = "coordinate-order normalization; visual impact depends on renderer"
        if case == "gradient_channel":
            report["behavior"][case]["classification"] = "C++ conversion formula versus Rust vertex color; not a whole-renderer pixel comparison"
    report["imgui_conversion_api"] = {"rust": rust_conversion_api(output), "cpp": {"status": "not compiled"}}
    include = args.imgui_include
    if include is None:
        choices = list((cpp / "cmake-build/vcpkg_installed").glob("*/include/imgui.h"))
        include = choices[0].parent if choices else None
    if include:
        source = ROOT / "tests/cpp/imgui_api_audit.cpp"
        if Path(compiler).name.lower() in {"cl", "cl.exe"}:
            command = [compiler, "/nologo", "/std:c++latest", "/EHsc", "/c", f"/I{cpp / 'include'}",
                       f"/I{include}", source, f"/Fo{output / 'imgui-api-cpp.obj'}"]
        else:
            command = [compiler, "-std=c++23", "-c", f"-I{cpp / 'include'}", f"-I{include}", source,
                       "-o", output / "imgui-api-cpp.o"]
        compare.run(command, env=env)
        report["imgui_conversion_api"]["cpp"] = {"status": "compiled", "include": str(include)}
    for path in sorted((cpp / "include/omath").rglob("*.hpp")):
        relative = path.relative_to(cpp / "include/omath")
        targets, note = header_mapping(relative)
        report["header_mapping"].append({"cpp": relative.as_posix(), "rust": targets,
                                         "all_files_present": all((ROOT / p).is_file() for p in targets), "note": note})
    report["mapping_warning"] = "A mapped implementation file does not certify complete API or semantic parity."
    report["windows_x86"] = {"status": "not requested"}
    if args.check_x86:
        result = subprocess.run(["cargo", "check", "--locked", "--all-targets", "--all-features", "--target", "i686-pc-windows-msvc", "--message-format", "short"],
                                cwd=ROOT, capture_output=True, text=True)
        (output / "windows-x86.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        report["windows_x86"] = {"status": "passed" if result.returncode == 0 else "failed", "log": "target/port-audit/windows-x86.log"}
    report["known_gaps_detected"] = any(row["status"] != "match" and not case.startswith("fov_")
        for case, row in report["behavior"].items()) or not report["imgui_conversion_api"]["rust"]["compiled"] or report["windows_x86"]["status"] == "failed" or any(not row["all_files_present"] for row in report["header_mapping"])
    report["fully_ported"] = False if report["known_gaps_detected"] else None
    report["verdict"] = "gaps_confirmed" if report["known_gaps_detected"] else "no_gaps_in_probed_subset; not a completeness certificate"
    path = output / "report.json"
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    summary_keys = ["rust_revision", "cpp_revision", "behavior", "imgui_conversion_api", "windows_x86",
                    "known_gaps_detected", "fully_ported", "verdict"]
    print(json.dumps({key: report[key] for key in summary_keys}, indent=2))
    print(f"Header mappings: {len(report['header_mapping'])}; report: {path}")
    return 1 if report["known_gaps_detected"] else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except subprocess.CalledProcessError as error:
        print(error.stdout or "")
        print(error.stderr or "", file=sys.stderr)
        raise
