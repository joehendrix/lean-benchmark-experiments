mod lean;

use lean::estatem::Result;
use lean::estatem::ResultPat;
use lean::IOError;
use lean::LeanObject;

pub fn rusttime<A: LeanObject>(
    act: lean::IO<A>,
    rw: lean::Realworld,
) -> Result<IOError, lean::Realworld, lean::Pair<A, lean::BoxedUInt64>> {
    use std::time::Instant;
    let start = Instant::now();
    let r = act.apply(rw);
    let finish = Instant::now();

    match r.pat() {
        ResultPat::Error(e, s) => Result::error(e, s),
        ResultPat::Ok(a, s) => {
            let elapsed = finish.duration_since(start).as_micros();
            let rr = lean::Pair::mk(a, lean::BoxedUInt64::mk(elapsed as u64));
            Result::ok(rr, s)
        }
    }
}

#[no_mangle]
pub extern "C" fn leanclock_io_rusttime(
    act_ptr: *mut lean::runtime::Object,
    rw_ptr: *mut lean::runtime::Object,
) -> *const lean::runtime::Object {
    let act = lean::IO::<lean::Realworld>::acquire(act_ptr);
    let rw = lean::Realworld::acquire(rw_ptr);
    return rusttime(act, rw).release();
}
