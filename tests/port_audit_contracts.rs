//! Additional native method-call contracts missing from the initial port validation.
use omath::rev_eng::InternalReverseEngineeredObject;
use std::ffi::c_void;

#[cfg(feature = "imgui")]
#[test]
fn reverse_imgui_conversions_restore_all_components() {
    use omath::{Vector2, Vector4};
    let v = Vector2::new(-1.25f32, 4.5);
    assert_eq!(Vector2::<f32>::from_im_vec2(&v.to_im_vec2()), v);
    let v = Vector4::new(-1.25f32, 4.5, 12.75, 0.5);
    assert_eq!(Vector4::<f32>::from_im_vec4(&v.to_im_vec4()), v);
    assert_eq!(
        Vector4::<f64>::from_im_vec4(&v.to_im_vec4()),
        Vector4::new(-1.25, 4.5, 12.75, 0.5)
    );
}

#[repr(C)]
struct SubObject {
    vtable: *const usize,
    value: i32,
}
#[repr(C)]
struct Object {
    first: SubObject,
    second: SubObject,
}
type Method = unsafe extern "C" fn(*mut c_void, i32) -> i32;
unsafe extern "C" fn add(this: *mut c_void, delta: i32) -> i32 {
    unsafe { (*(this as *const SubObject)).value + delta }
}

#[test]
fn native_direct_calls_preserve_object_and_arguments() {
    let table = [add as *const () as usize];
    let object = SubObject {
        vtable: table.as_ptr(),
        value: 17,
    };
    let wrapper = InternalReverseEngineeredObject::new(&object as *const _ as usize);
    let result =
        unsafe { wrapper.call_method::<Method, _>(table[0], |function, this| function(this, 5)) };
    assert_eq!(result, 22);
}

#[test]
fn secondary_vtable_calls_adjust_this_by_table_offset() {
    let table = [add as *const () as usize];
    let object = Object {
        first: SubObject {
            vtable: table.as_ptr(),
            value: 17,
        },
        second: SubObject {
            vtable: table.as_ptr(),
            value: 100,
        },
    };
    let wrapper = InternalReverseEngineeredObject::new(&object as *const _ as usize);
    for (offset, expected) in [
        (0, 22),
        (std::mem::offset_of!(Object, second) as isize, 105),
    ] {
        assert_eq!(wrapper.virtual_method_address(offset, 0).unwrap(), table[0]);
        let result = unsafe {
            wrapper.call_virtual_method::<Method, _>(offset, 0, |function, this| function(this, 5))
        }
        .unwrap();
        assert_eq!(result, expected);
    }
}
