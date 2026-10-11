//@ add-core-stubs
//@ compile-flags: --target tricore-unknown-none-elf -Copt-level=0 -Cno-prepopulate-passes -Cpanic=abort -Cdebug-assertions=no
//@ needs-llvm-components: tricore

#![feature(no_core, lang_items, intrinsics)]
#![no_core]
#![crate_type = "lib"]
#![allow(internal_features, unused_imports)]

extern crate minicore;
use minicore::*;

// Scalar pointers must reach LLVM as ptr so the EABI can use A4-A7 and A2.
// CHECK-DAG: define{{.*}} i32 @pointer_address(ptr
#[no_mangle]
pub extern "C" fn pointer_address(p: *const u8) -> usize {
    p as usize
}

// CHECK-DAG: define{{.*}} ptr @pointer_identity(ptr
#[no_mangle]
pub extern "C" fn pointer_identity(p: *mut u8) -> *mut u8 {
    p
}

// CHECK-DAG: define{{.*}} ptr @pointer_from_integer(i32
#[no_mangle]
pub extern "C" fn pointer_from_integer(address: usize) -> *mut u8 {
    address as *mut u8
}

extern "C" {
    #[link_name = "strlen"]
    fn c_strlen(p: *const u8) -> usize;
    fn consume_mixed(p: *const u8, x: u32, q: *const u8, y: u32);
    fn get_pointer() -> *mut u8;
}

// CHECK-DAG: define{{.*}} i32 @call_strlen(ptr
// CHECK-DAG: call{{.*}} i32 @strlen(ptr
#[no_mangle]
pub unsafe extern "C" fn call_strlen(p: *const u8) -> usize {
    c_strlen(p)
}

// CHECK-DAG: define{{.*}} void @call_mixed(ptr{{[^,]*}}, i32{{[^,]*}}, ptr{{[^,]*}}, i32
// CHECK-DAG: call void @consume_mixed(ptr{{[^,]*}}, i32{{[^,]*}}, ptr{{[^,]*}}, i32
#[no_mangle]
pub unsafe extern "C" fn call_mixed(p: *const u8, x: u32, q: *const u8, y: u32) {
    consume_mixed(p, x, q, y);
}

// CHECK-DAG: define{{.*}} ptr @call_pointer_return()
// CHECK-DAG: call{{.*}} ptr @get_pointer()
#[no_mangle]
pub unsafe extern "C" fn call_pointer_return() -> *mut u8 {
    get_pointer()
}

#[repr(C)]
pub struct PointerStruct {
    pub p: *mut u8,
}

// Pointer fields in C structures use data registers, not address registers.
// CHECK-DAG: define{{.*}} i32 @aggregate_pointer(i32
#[no_mangle]
pub extern "C" fn aggregate_pointer(s: PointerStruct) -> PointerStruct {
    s
}

#[repr(transparent)]
pub struct PointerWrapper(pub *mut u8);

// Transparent wrappers have the same ABI as their underlying pointer.
// CHECK-DAG: define{{.*}} ptr @transparent_pointer(ptr
#[no_mangle]
pub extern "C" fn transparent_pointer(p: PointerWrapper) -> PointerWrapper {
    p
}

#[rustc_intrinsic]
fn wrapping_add<T: Copy>(a: T, b: T) -> T;
#[rustc_intrinsic]
unsafe fn arith_offset<T>(p: *const T, offset: isize) -> *const T;

// Exercise the definition as well as the call of a compiler-builtins signature.
// CHECK-DAG: define{{.*}} i32 @strlen(ptr
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn strlen(mut s: *const u8) -> usize {
    let mut n = 0;
    loop {
        match *s {
            0 => return n,
            _ => {
                n = wrapping_add(n, 1);
                s = arith_offset(s, 1);
            }
        }
    }
}
