//! This exports portions of the Lean C Runtime.
use libc::{c_uint, size_t};

#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
const LEAN_RC_NBITS: u8 = 32;

#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
const LEAN_ST_MEM_KIND: u8 = 0;
#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
const LEAN_MT_MEM_KIND: u8 = 1;

#[repr(C)]
#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
pub struct Object {
    header: u64,
}

#[repr(C)]
#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC")
))]
pub struct Object {
    rc: size_t,
    tag: u8,
    mem_kind: u8,
    other: u16,
}

pub fn lean_is_scalar(o: *const Object) -> bool {
    ((o as usize) & 1) == 1
}

#[repr(C)]
struct CtorObject {
    header: Object,
}

#[link(name = "leancpp")]
extern "C" {
    pub fn lean_alloc_small(sz: c_uint, slot_idx: c_uint) -> *mut u8;
    pub fn lean_apply_1(f: *const Object, a1: *const Object) -> *mut Object;
    pub fn lean_del(o: *mut Object);
    #[cfg(not(feature = "LEAN_SMALL_ALLOCATOR"))]
    pub fn lean_inc_heartbeat();
    #[cfg(not(feature = "LEAN_SMALL_ALLOCATOR"))]
    pub fn lean_internal_panic_out_of_memory();
    #[cfg(not(feature = "LEAN_SMALL_ALLOCATOR"))]
    pub fn malloc(size: size_t) -> *mut u8;
}

fn lean_byte(v: u64, idx: u8) -> u8 {
    (v >> (8 * idx)) as u8
}

#[cfg(any(
    feature = "LEAN_COMPRESSED_OBJECT_HEADER",
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
pub unsafe fn lean_ptr_tag(o: *const Object) -> u8 {
    lean_byte((*o).header, 7)
}

#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC")
))]
pub unsafe fn lean_ptr_tag(o: *const Object) -> u8 {
    (*o).tag
}

#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
pub unsafe fn lean_is_st(o: *const Object) -> bool {
    lean_byte((*o).header, 5) == LEAN_ST_MEM_KIND
}

#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
pub unsafe fn lean_is_mt(o: *const Object) -> bool {
    lean_byte((*o).header, 5) == LEAN_MT_MEM_KIND
}

#[cfg(all(
    not(feature = "LEAN_COMPRESSED_OBJECT_HEADER"),
    not(feature = "LEAN_CHECK_RC_OVERFLOW"),
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
pub unsafe fn lean_inc_ref(o: *mut Object) {
    if lean_is_st(o) {
        (*o).header += 1;
    } else if lean_is_mt(o) {
        panic!("Multithreaded lean objects unsupported.")
    }
}

#[cfg(any(
    feature = "LEAN_COMPRESSED_OBJECT_HEADER",
    feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"
))]
pub unsafe fn lean_dec_ref_core(o: *mut Object) -> bool {
    if lean_is_st(o) {
        (*o).header -= 1;
        (((*o).header) & ((1 << LEAN_RC_NBITS) - 1)) == 0
    } else if lean_is_mt(o) {
        panic!("Multithreaded lean objects unsupported.")
    } else {
        false
    }
}

pub unsafe fn lean_dec_ref(o: *mut Object) {
    if lean_dec_ref_core(o) {
        lean_del(o);
    }
}

pub unsafe fn lean_inc(o: *mut Object) {
    if !lean_is_scalar(o) {
        lean_inc_ref(o);
    }
}

pub unsafe fn lean_dec(o: *mut Object) {
    if !lean_is_scalar(o) {
        lean_dec_ref(o);
    }
}

const LEAN_MAX_SMALL_OBJECT_SIZE: size_t = 4096;
const LEAN_OBJECT_SIZE_DELTA: size_t = 8;

fn lean_align(v: size_t, a: size_t) -> size_t {
    ((v + a - 1) / a) * a
}

fn lean_get_slot_idx(sz: c_uint) -> c_uint {
    assert!(sz > 0);
    assert!(lean_align(sz as size_t, LEAN_OBJECT_SIZE_DELTA) == sz as size_t);
    (sz as size_t / LEAN_OBJECT_SIZE_DELTA - 1) as c_uint
}

#[cfg(feature = "LEAN_SMALL_ALLOCATOR")]
fn lean_alloc_small_object(sz: c_uint) -> *mut Object {
    let sz = lean_align(sz as usize, LEAN_OBJECT_SIZE_DELTA);
    assert!(sz <= LEAN_MAX_SMALL_OBJECT_SIZE);
    let slot_idx = lean_get_slot_idx(sz as c_uint);
    unsafe { lean_alloc_small(sz as c_uint, slot_idx) as *mut Object }
}

#[cfg(not(feature = "LEAN_SMALL_ALLOCATOR"))]
fn lean_alloc_small_object(sz: c_uint) -> *mut Object {
    unsafe {
        lean_inc_heartbeat();
        let mem = malloc(std::mem::size_of::<size_t>() + sz as usize) as *mut size_t;
        if (mem as *const usize) == std::ptr::null() {
            lean_internal_panic_out_of_memory();
        }
        *mem = sz as usize;
        mem.offset(1)
    }
}

pub fn lean_alloc_ctor_memory(sz: c_uint) -> *mut Object {
    lean_alloc_small_object(sz)
}

#[cfg(not(feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"))]
pub unsafe fn lean_set_st_header(o: *mut Object, tag: u8, other: u16) {
    (*o).rc = 1;
    (*o).tag = tag;
    (*o).mem_kind = LEAN_ST_MEM_KIND;
    (*o).other = other;
}

#[cfg(feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC")]
pub unsafe fn lean_set_st_header(o: *mut Object, tag: u8, other: u16) {
    (*o).header =
        (tag as u64) << 56 | (other as u64) << 48 | (LEAN_ST_MEM_KIND as u64) << 40 | 1;
}

#[cfg(not(feature = "LEAN_COMPRESSED_OBJECT_HEADER_SMALL_RC"))]
pub unsafe fn lean_set_st_header(o: *mut Object, tag: u8, other: u16) {
    (*o).rc = 1;
    (*o).tag = tag;
    (*o).mem_kind = LEAN_ST_MEM_KIND;
    (*o).other = other;
}

pub fn lean_alloc_ctor(
    tag: libc::c_uint,
    num_objs: c_uint,
    scalar_sz: c_uint,
) -> *mut Object {
    unsafe {
        let o = lean_alloc_ctor_memory(
            std::mem::size_of::<CtorObject>() as u32
                + num_objs * std::mem::size_of::<*const Object>() as u32
                + scalar_sz,
        );
        lean_set_st_header(o, tag as u8, num_objs as u16);
        o
    }
}

pub unsafe fn lean_ctor_obj_cptr(o: *mut Object) -> *mut *mut Object {
    o.offset(1) as *mut *mut Object
}

pub unsafe fn lean_ctor_get(o: *mut Object, i: isize) -> *mut Object {
    *(lean_ctor_obj_cptr(o).offset(i))
}

pub unsafe fn lean_ctor_set(o: *mut Object, i: isize, v: *mut Object) {
    let a = lean_ctor_obj_cptr(o);
    (*a.offset(i)) = v;
}

pub unsafe fn lean_ctor_set_uint64(o: *mut Object, offset: c_uint, v: u64) {
    let p: *mut u8 = lean_ctor_obj_cptr(o) as *mut u8;
    let po = p.offset(offset as isize);
    *(po as *mut u64) = v;
}

pub fn lean_box_uint64(v: u64) -> *mut Object {
    unsafe {
        let r = lean_alloc_ctor(0, 0, std::mem::size_of::<u64>() as c_uint);
        lean_ctor_set_uint64(r, 0, v);
        r
    }
}