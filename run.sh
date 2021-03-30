#!/bin/bash
set -ex
LEAN_HOME=$HOME/.elan/toolchains/leanprover-lean4-nightly-2021-03-14

LEAN_INCLUDE=$LEAN_HOME/include/
LEAN_LIB=$LEAN_HOME/lib/lean

c++ -std=c++11 -I $LEAN_INCLUDE -c -o c/clock.o c/clock.cpp
leanpkg build
leanc -c -o build/temp/Bench.o build/temp/Bench.c -O3 -DNDEBUG
leanc -o "build/bin/Bench" -x none build/temp/Bench.o c/clock.o
./build/bin/Bench $LEAN_LIB