#include <chrono>
#include <time.h>
#include <lean/lean.h>
#include <stdio.h>

extern "C" lean_object* mk_io_user_error(lean_object*);

extern "C" lean_obj_res lean_clock_monotonic_raw_gettime(lean_obj_arg r) {
    struct timespec tp;
    if (clock_gettime(CLOCK_MONOTONIC, &tp)) {
        return lean_io_result_mk_error(mk_io_user_error(lean_mk_string("clock_gettime failed.")));
    }
    lean_object* res = lean_alloc_ctor(0, 0, 16);
    lean_ctor_set_uint64(res, 0, tp.tv_sec);
    lean_ctor_set_uint64(res, 8, tp.tv_nsec);
    return lean_io_result_mk_ok(res);
}

/* timeit' {α : Type} (fn : IO α) : IO (α times double) */
extern "C" lean_obj_res leanclock_io_timeit(lean_obj_arg fn, lean_obj_arg w) {
    auto start = std::chrono::steady_clock::now();
    lean_obj_res fr = lean_apply_1(fn, w);
    auto end   = std::chrono::steady_clock::now();

    if (lean_ptr_tag(fr) != 0) {
        return fr;
    }
    uint64_t int_ms = std::chrono::duration_cast<std::chrono::duration<uint64_t, std::micro>>(end - start).count();

    lean_object* r = lean_alloc_ctor(0, 2, 0);
    lean_ctor_set(r, 0, lean_ctor_get(fr, 0));
    lean_ctor_set(r, 1, lean_box_uint64(int_ms));

    lean_ctor_set(fr, 0, r);
    return fr;
}