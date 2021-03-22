# Lean Benchmarking Tester

This is a small experimental repo to show how to benchmark operations in
Lean.

To build it, you should install `elan` and make sure you have the
prover version `leanprover-lean4-nightly-2021-03-14`.

You can then build and run the benchmark with the commands:

```
leanpkg build bin
./build/bin/Bench $HOME/.elan/toolchains/leanprover-lean4-nightly-2021-03-14/lib/lean
```