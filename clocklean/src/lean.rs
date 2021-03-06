pub mod runtime;
use crate::lean::runtime::*;
use std::marker::PhantomData;

pub trait LeanObject {
    // Get the object without incrementing reference count.
    // Note. This code requires that the object is a valid non-null object.
    unsafe fn acquire(o: *mut Object) -> Self
    where
        Self: Sized;
    // Release the object without decrementing reference count.
    fn release(self) -> *mut Object;
}

// Provides a wrapper around a bare Lean object pointer that
// implements reference counting.
pub struct LeanRepr<T> {
    ptr: *mut Object,
    phantom: PhantomData<T>,
}

impl<T> Drop for LeanRepr<T> {
    fn drop(&mut self) {
        unsafe {
            if (self.ptr as *const Object) != std::ptr::null() {
                lean_dec(self.ptr);
            }
        }
    }
}

impl<T> Clone for LeanRepr<T> {
    fn clone(&self) -> Self {
        unsafe {
            lean_inc(self.ptr);
        }
        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T> LeanObject for LeanRepr<T> {
    unsafe fn acquire(o: *mut runtime::Object) -> Self {
        assert!((o as *const Object) != std::ptr::null());
        Self {
            ptr: o,
            phantom: PhantomData,
        }
    }
    fn release(mut self) -> *mut runtime::Object {
        let p = self.ptr;
        // Null pointer so call to LeanRepr::drop will not free it.
        self.ptr = std::ptr::null::<runtime::Object>() as *mut runtime::Object;
        p
    }
}

// A marker type for a Lean type whose internals are hidden
// from Rust.
pub struct Opague(LeanRepr<()>);
impl LeanObject for Opague {
    unsafe fn acquire(o: *mut Object) -> Self {
        Self(LeanRepr::acquire(o))
    }
    fn release(self) -> *mut Object {
        self.0.release()
    }
}

pub mod estatem {
    use super::LeanRepr;
    use crate::lean::runtime::*;
    use crate::LeanObject;

    //inductive Result (ε σ α : Type u) where
    //  | ok    : α → σ → Result ε σ α
    //  | error : ε → σ → Result ε σ α
    pub enum ResultPat<E, S, A> {
        Ok(A, S),
        Error(E, S),
    }

    pub struct Result<E, S, A>(LeanRepr<ResultPat<E, S, A>>);

    impl<E, S, A> LeanObject for Result<E, S, A> {
        unsafe fn acquire(o: *mut Object) -> Self {
            Self(LeanRepr::acquire(o))
        }
        fn release(self) -> *mut Object {
            self.0.release()
        }
    }

    impl<E: LeanObject, S: LeanObject, A: LeanObject> Result<E, S, A> {
        pub fn pat(self) -> ResultPat<E, S, A> {
            unsafe {
                let p = self.0.release();
                match lean_ptr_tag(p) {
                    0 => {
                        let a = lean_ctor_get(p, 0);
                        let s = lean_ctor_get(p, 1);
                        lean_inc(a);
                        lean_inc(s);
                        lean_dec_ref(p);
                        ResultPat::Ok(A::acquire(a), S::acquire(s))
                    }
                    1 => {
                        let e = lean_ctor_get(p, 0);
                        let s = lean_ctor_get(p, 1);
                        lean_inc(e);
                        lean_inc(s);
                        lean_dec_ref(p);
                        ResultPat::Error(E::acquire(e), S::acquire(e))
                    }
                    _ => panic!("Invalid IO result"),
                }
            }
        }
        pub fn ok(a: A, s: S) -> Self {
            unsafe {
                let r = lean_alloc_ctor(0, 2, 0);
                lean_ctor_set(r, 0, a.release());
                lean_ctor_set(r, 1, s.release());
                Result::acquire(r)
            }
        }
        pub fn error(e: E, s: S) -> Self {
            unsafe {
                let r = lean_alloc_ctor(0, 2, 0);
                lean_ctor_set(r, 0, e.release());
                lean_ctor_set(r, 1, s.release());
                Result::acquire(r)
            }
        }
    }
}

use estatem::Result;

pub struct IOError(LeanRepr<()>);

impl LeanObject for IOError {
    unsafe fn acquire(o: *mut Object) -> Self {
        Self(LeanRepr::acquire(o))
    }
    fn release(self) -> *mut Object {
        self.0.release()
    }
}

pub struct IO<A>(LeanRepr<A>);

impl<A> LeanObject for IO<A> {
    unsafe fn acquire(o: *mut Object) -> Self {
        Self(LeanRepr::acquire(o))
    }
    fn release(self) -> *mut Object {
        self.0.release()
    }
}

pub struct IORealworld(LeanRepr<()>);
impl LeanObject for IORealworld {
    unsafe fn acquire(o: *mut Object) -> Self {
        Self(LeanRepr::acquire(o))
    }
    fn release(self) -> *mut Object {
        self.0.release()
    }
}

impl<A> IO<A> {
    pub fn apply(self, rw: IORealworld) -> Result<IOError, IORealworld, A> {
        unsafe {
            let actp = self.release();
            let ptr = runtime::lean_apply_1(actp, rw.release());
            lean_dec_ref(actp);
            Result::acquire(ptr)
        }
    }
}

pub struct Pair<A, B>(LeanRepr<(A, B)>);

impl<A, B> LeanObject for Pair<A, B> {
    unsafe fn acquire(o: *mut Object) -> Self {
        Self(LeanRepr::acquire(o))
    }
    fn release(self) -> *mut Object {
        self.0.release()
    }
}

impl<A: LeanObject, B: LeanObject> Pair<A, B> {
    pub fn pat(self) -> (A, B) {
        unsafe {
            let p = self.0.release();
            let a = runtime::lean_ctor_get(p, 0);
            let b = runtime::lean_ctor_get(p, 1);
            lean_inc(a);
            lean_inc(b);
            lean_dec_ref(p);
            (A::acquire(a), B::acquire(b))
        }
    }
    pub fn mk(a: A, b: B) -> Self {
        unsafe {
            let r = runtime::lean_alloc_ctor(0, 2, 0);
            runtime::lean_ctor_set(r, 0, a.release());
            runtime::lean_ctor_set(r, 1, b.release());
            Self::acquire(r)
        }
    }
}

pub struct BoxedUInt64(LeanRepr<u64>);

impl LeanObject for BoxedUInt64 {
    unsafe fn acquire(o: *mut Object) -> Self {
        Self(LeanRepr::acquire(o))
    }
    fn release(self) -> *mut Object {
        self.0.release()
    }
}

impl BoxedUInt64 {
    pub fn mk(x: u64) -> Self {
        unsafe { BoxedUInt64::acquire(runtime::lean_box_uint64(x)) }
    }
}
