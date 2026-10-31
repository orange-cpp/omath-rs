//! `omath.SectionScanResult` and the scanners. Only buffers and files can be scanned: a scan of a loaded module would
//! read a live process, so `scan_in_module` does not exist.

use super::support::subtable;
use crate::utility::{ElfPatternScanner, MachOPatternScanner, PatternScanner, PePatternScanner, SectionScanResult};
use mlua::{
    Lua, MetaMethod, MultiValue, Result, String as LuaString, Table, UserData, UserDataFields, UserDataMethods,
};

impl UserData for SectionScanResult {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        // C++ pushes a `uintptr_t` as is: addresses past `i64::MAX` wrap instead of turning into floats
        fields.add_field_method_get("virtual_base_addr", |_, this| Ok(this.virtual_base_addr as i64));
        fields.add_field_method_get("raw_base_addr", |_, this| Ok(this.raw_base_addr as i64));
        fields.add_field_method_get("target_offset", |_, this| Ok(this.target_offset));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // A type with fields but no methods would raise on an unknown key, where sol2 gives nil
        methods.add_meta_function(MetaMethod::Index, |_, _: MultiValue| Ok(()));
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(format!(
                "SectionScanResult(vbase=0x{:X}, raw_base=0x{:X}, offset={})",
                this.virtual_base_addr, this.raw_base_addr, this.target_offset
            ))
        });
    }
}

fn text(string: &LuaString) -> String {
    String::from_utf8_lossy(&string.as_bytes()).into_owned()
}

/// `omath[name].scan_in_file(path, pattern, section)`, where `section` defaults to the format's own.
macro_rules! file_scanner {
    ($lua:ident, $omath:ident, $name:literal, $scanner:ident) => {
        subtable($lua, $omath, $name)?.set(
            "scan_in_file",
            $lua.create_function(|_, (path, pattern, section): (LuaString, LuaString, Option<LuaString>)| {
                let section = section.as_ref().map_or_else(|| $scanner::DEFAULT_SECTION_NAME.to_owned(), text);
                Ok($scanner::scan_for_pattern_in_file(text(&path), &text(&pattern), &section))
            })?,
        )?
    };
}

pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    omath.set("SectionScanResult", lua.create_proxy::<SectionScanResult>()?)?;

    subtable(lua, omath, "PatternScanner")?.set(
        "scan",
        lua.create_function(|_, (data, pattern): (LuaString, LuaString)| {
            Ok(PatternScanner::scan_for_pattern(&data.as_bytes(), &text(&pattern)))
        })?,
    )?;

    file_scanner!(lua, omath, "PePatternScanner", PePatternScanner);
    file_scanner!(lua, omath, "ElfPatternScanner", ElfPatternScanner);
    file_scanner!(lua, omath, "MachOPatternScanner", MachOPatternScanner);

    Ok(())
}
