use omath::{
    Vector3,
    algorithm::*,
    containers::EncryptedVariable,
    engines::{source, unreal::*},
    rev_eng::*,
    utility::{LoadedModule, PePatternScanner},
};
use std::sync::Arc;
#[test]
fn crate_imports_replace_cpp_module_import() {
    let v = omath::Vector2::new(1.0_f32, 2.0);
    assert_eq!((v.x, v.y), (1.0, 2.0));
}
#[test]
fn wildcard_scanner_matches_exhaustive_reference_on_generated_inputs() {
    use omath::utility::PatternScanner;
    let mut seed = 12345_u32;
    for length in 0..48 {
        for pattern_len in 0..12 {
            for _ in 0..10 {
                let mut next = || {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    (seed >> 24) as u8 % 4
                };
                let bytes: Vec<_> = (0..length).map(|_| next()).collect();
                let pattern: Vec<_> = (0..pattern_len)
                    .map(|_| match next() {
                        3 => None,
                        b => Some(b),
                    })
                    .collect();
                let expected = if bytes.is_empty() || pattern_len > length {
                    None
                } else if pattern_len == 0 {
                    Some(0)
                } else {
                    bytes.windows(pattern_len).position(|w| {
                        w.iter()
                            .zip(&pattern)
                            .all(|(b, p)| p.is_none_or(|p| p == *b))
                    })
                };
                assert_eq!(
                    PatternScanner::scan_for_parsed_pattern(&bytes, &pattern),
                    expected,
                    "bytes={bytes:?}, pattern={pattern:?}"
                );
            }
        }
    }
}
#[test]
fn encrypted_guard_writes_back_and_encrypts_during_unwind() {
    let mut value = EncryptedVariable::new(1234_u32, [1, 2, 3, 4]);
    let original = value.bytes().to_vec();
    assert!(value.is_encrypted());
    assert_ne!(value.value(), 1234);
    assert_eq!(*value.drop_anchor(), 1234);
    assert_eq!(value.bytes(), original);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut anchor = value.drop_anchor();
        *anchor = 5678;
        panic!("unwind");
    }));
    assert!(value.is_encrypted());
    assert_eq!(*value.drop_anchor(), 5678);
    value.decrypt();
    value.decrypt();
    assert_eq!(value.value(), 5678);
    value.encrypt();
    value.encrypt();
    assert_eq!(*value.drop_anchor(), 5678);
}
#[test]
fn process_memory_roundtrip_and_invalid_reads() {
    let mut value = 21_u64;
    let address = &mut value as *mut _ as usize;
    let memory = ProcessMemory::open(std::process::id(), true).unwrap();
    assert_eq!(memory.read_memory::<u64>(address).unwrap(), 21);
    memory.write_memory(address, &42_u64).unwrap();
    assert_eq!(value, 42);
    assert!(memory.read_memory::<u64>(1).is_err());
    let object = ExternalReverseEngineeredObject::new(address, &memory);
    assert_eq!(object.get_by_offset::<u64>(0).unwrap(), 42);
    object.set_by_offset(0, &99_u64).unwrap();
    assert_eq!(value, 99);
    let image = MemoryView {
        base: 0x100,
        bytes: &0x12345678_u32.to_ne_bytes(),
    };
    assert_eq!(read_pointer(&image, 0x100, 4).unwrap(), 0x12345678);
    assert!(read_pointer(&image, 0x100, 3).is_err());
}
fn put(bytes: &mut [u8], address: usize, value: impl AsRef<[u8]>) {
    let v = value.as_ref();
    bytes[address..address + v.len()].copy_from_slice(v);
}
#[test]
fn unreal_flat_and_chunked_objects_preserve_target_pointer_width() {
    let mut bytes = vec![0; 0x1000];
    put(&mut bytes, 0x100, 0x200_u32.to_ne_bytes());
    put(&mut bytes, 0x104, 2_u32.to_ne_bytes());
    put(&mut bytes, 0x200, 0x456_u32.to_ne_bytes());
    let view = MemoryView {
        base: 0,
        bytes: &bytes,
    };
    let layout = ObjectArrayLayout::ue2_5();
    assert_eq!(get_object_count(&view, 0x100, &layout).unwrap(), 2);
    assert_eq!(
        get_object_by_index(&view, 0x100, 0, &layout).unwrap(),
        0x456
    );
    assert!(matches!(
        get_object_by_index(&view, 0x100, 1, &layout),
        Err(ObjectArrayError::NullObject)
    ));
    assert!(matches!(
        get_object_by_index(&view, 0x100, 2, &layout),
        Err(ObjectArrayError::IndexOutOfRange)
    ));
    put(&mut bytes, 0x110, 0x200_u64.to_ne_bytes());
    put(&mut bytes, 0x124, 3_u32.to_ne_bytes());
    put(&mut bytes, 0x208, 0x300_u64.to_ne_bytes());
    put(&mut bytes, 0x300, 0x789_u64.to_ne_bytes());
    let view = MemoryView {
        base: 0,
        bytes: &bytes,
    };
    let layout = ObjectArrayLayout {
        elements_per_chunk: 2,
        ..ObjectArrayLayout::ue5()
    };
    assert_eq!(
        get_object_by_index(&view, 0x100, 2, &layout).unwrap(),
        0x789
    );
    let layout = ObjectArrayLayout {
        elements_per_chunk: 0,
        ..layout
    };
    assert!(matches!(
        get_object_by_index(&view, 0x100, 0, &layout),
        Err(ObjectArrayError::InvalidLayout)
    ));
}
#[test]
fn unreal_names_decode_legacy_and_block_pools_and_instance_suffixes() {
    let mut bytes = vec![0; 0x1000];
    put(&mut bytes, 0x124, 1_u32.to_ne_bytes());
    put(&mut bytes, 0x200, 0x300_u32.to_ne_bytes());
    put(&mut bytes, 0x304, 0x400_u32.to_ne_bytes());
    let name = "Actor🦀";
    let wide: Vec<_> = name
        .encode_utf16()
        .flat_map(u16::to_ne_bytes)
        .chain([0, 0])
        .collect();
    put(&mut bytes, 0x40c, &wide);
    let view = MemoryView {
        base: 0,
        bytes: &bytes,
    };
    assert_eq!(
        get_actor_name(&view, 0x100, 0x200, &NameLayout::ue2_5()).unwrap(),
        name
    );
    put(&mut bytes, 0x118, 2_u32.to_ne_bytes());
    put(&mut bytes, 0x11c, 3_u32.to_ne_bytes());
    put(&mut bytes, 0x210, 0x400_u64.to_ne_bytes());
    put(&mut bytes, 0x404, (5_u16 << 6).to_ne_bytes());
    put(&mut bytes, 0x406, b"Hello");
    let view = MemoryView {
        base: 0,
        bytes: &bytes,
    };
    assert_eq!(
        get_actor_name(&view, 0x100, 0x200, &NameLayout::ue5()).unwrap(),
        "Hello_2"
    );
    assert!(matches!(
        get_actor_name(&view, 0, 0x200, &NameLayout::ue5()),
        Err(ActorNameError::NullPointer)
    ));
}
#[test]
fn loaded_pe_scans_virtual_sections_including_zero_fill() {
    let mut bytes = vec![0; 0x500];
    put(&mut bytes, 0, b"MZ");
    put(&mut bytes, 0x3c, 0x80_u32.to_le_bytes());
    put(&mut bytes, 0x80, b"PE\0\0");
    put(&mut bytes, 0x86, 1_u16.to_le_bytes());
    put(&mut bytes, 0x94, 0xf0_u16.to_le_bytes());
    let section = 0x188;
    put(&mut bytes, section, b".text\0\0\0");
    put(&mut bytes, section + 8, 0x100_u32.to_le_bytes());
    put(&mut bytes, section + 12, 0x300_u32.to_le_bytes());
    put(&mut bytes, section + 16, 0x10_u32.to_le_bytes());
    put(&mut bytes, 0x380, [0xde, 0xad, 0xbe, 0xef]);
    let view = MemoryView {
        base: 0x10000,
        bytes: &bytes,
    };
    assert_eq!(
        PePatternScanner::scan_for_pattern_in_mapped_image(
            &view,
            view.base,
            "DE AD ?? EF",
            ".text"
        ),
        Some(0x10380)
    );
    assert!(
        PePatternScanner::scan_for_pattern_in_mapped_image(&view, view.base, "DE AD", "absent")
            .is_none()
    );
}
#[test]
fn live_module_discovery_and_scanning() {
    let module = LoadedModule::find("").unwrap().unwrap();
    assert!(module.path.is_file());
    #[cfg(windows)]
    assert!(PePatternScanner::scan_for_pattern_in_loaded_module(&module, "??", ".text").is_some());
    #[cfg(target_os = "linux")]
    assert!(
        omath::utility::ElfPatternScanner::scan_for_pattern_in_loaded_module(
            &module, "??", ".text"
        )
        .is_some()
    );
    #[cfg(target_os = "macos")]
    assert!(
        omath::utility::MachOPatternScanner::scan_for_pattern_in_loaded_module(
            &module, "??", "__text"
        )
        .is_some()
    );
}
#[test]
fn targeting_filters_ties_and_yaw_wrap() {
    let points = [
        Vector3::new(-10.0, 0.2, 0.0),
        Vector3::new(-10.0, -0.2, 0.0),
    ];
    let camera = source::Camera::new(
        Vector3::default(),
        source::ViewAngles::new(
            source::PitchAngle::from_degrees(0.0),
            source::YawAngle::from_degrees(-179.0),
            source::RollAngle::from_degrees(0.0),
        ),
        omath::projection::ViewPort::new(800.0, 600.0),
        omath::projection::FieldOfView::from_degrees(90.0),
        0.1,
        1000.0,
    );
    assert_eq!(
        get_closest_target_by_fov(points, &camera, |p| *p, |_| true),
        Some(points[1])
    );
    assert_eq!(
        get_closest_target_by_distance(points, Vector3::default(), |p| *p, |_| true),
        Some(points[0])
    );
    assert!(
        get_closest_target_by_distance(points, Vector3::default(), |p| *p, |_| false).is_none()
    );
}
#[test]
fn walkbot_and_navigation_are_public_modules() {
    let mesh = Arc::new(omath::pathfinding::NavigationMesh::new());
    let _bot = omath::pathfinding::WalkBot::new(&mesh, 1.0);
}
