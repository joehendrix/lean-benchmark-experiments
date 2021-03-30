import Lean.Elab.Frontend

open Lean

-- Time with seconds and nanoseconds.
structure Timespec where
  sec: UInt64
  nsec: UInt64

@[extern 1 "lean_clock_monotonic_raw_gettime"]
constant clockMonotonicRawGettime : IO Timespec

/--
 - @binAddBench n@ builds a balanced binary tree of height @n@ using additions of 1,
 - and reduces them.  It can be used to validate that reduction memoizes shared subterms.
 -/
def binAddBench (n:Nat) : MetaM Unit := do
  let z := mkConst `Nat.zero
  let s := mkConst `Nat.succ
  let p := mkConst `Nat.add
  let mut r := mkApp s z
  for i in [0:n] do
    r := mkApp (mkApp p r) r
  let r ← Lean.Meta.reduce r
  pure ()

open IO

section
open Lean.Elab

-- Create environment from file
def environmentFromFile (fname:String) (moduleName:String) (opts : Options := {})
  : IO (MessageLog × Environment) := do
  let input ← IO.FS.readFile fname
  let inputCtx := Parser.mkInputContext input fname

  let (header, parserState, messages) ← Parser.parseHeader inputCtx
  let (env, messages) ← processHeader header opts messages inputCtx
  let env := env.setMainModule moduleName
  let s ← IO.processCommands inputCtx parserState (Command.mkState env messages opts)
  pure (s.commandState.messages, s.commandState.env)

end

def msecDiff (e s : Timespec) : String := do
  let en : Nat := e.sec.toNat * 10^9 + e.nsec.toNat
  let sn : Nat := s.sec.toNat * 10^9 + s.nsec.toNat
  let musec := (en - sn) / 1000
  let musecFrac := s!"{musec%1000}"
  let musecPadding := "".pushn '0' (3 - musecFrac.length)
  pure s!"{musec/1000}.{musecPadding}{musecFrac}"

-- This outputs how long it takes to run the given meta option in the
-- context created from the filename.
def benchmarkC (fname:String) (testName:String) (action:MetaM Unit) : IO Unit := do
  let (msgs, env) ← environmentFromFile fname "Test"
  if (← msgs.hasErrors) then
    IO.println s!"Errors loading {fname}..."
    for msg in msgs.toList do
      IO.print s!"  {← msg.toString (includeEndPos := Lean.Elab.getPrintMessageEndPos {})}"
  else
    IO.print s!"{testName} "
    let s ← clockMonotonicRawGettime
    let a ← (action.run.run {} {env := env}).toIO'
    let e ← clockMonotonicRawGettime
    IO.println s!"{msecDiff e s}ms"
    match a with
    | Except.error msg => do
      IO.println s!"  Error: {←msg.toMessageData.toString}"
    | Except.ok _ => pure ()

@[extern "leanclock_io_timeit"] constant timeit2 (fn : IO α) : IO (α × UInt64)

-- This outputs how long it takes to run the given meta option in the
-- context created from the filename.
def benchmarkCPP (fname:String) (testName:String) (action:MetaM Unit) : IO Unit := do
  let (msgs, env) ← environmentFromFile fname "Test"
  if (← msgs.hasErrors) then
    IO.println s!"Errors loading {fname}..."
    for msg in msgs.toList do
      IO.print s!"  {← msg.toString (includeEndPos := Lean.Elab.getPrintMessageEndPos {})}"
  else
    IO.print s!"{testName} "
    let (a,t) ← timeit2 (action.run.run {} {env := env}).toIO'
    let musecFrac := s!"{t%1000}"
    let musecPadding := "".pushn '0' (3 - musecFrac.length)
    IO.println s!"{t/1000}.{musecPadding}{musecFrac}"
    match a with
    | Except.error msg => do
      IO.println s!"  Error: {←msg.toMessageData.toString}"
    | Except.ok _ => pure ()

-- This outputs how long it takes to run the given meta option in the
-- context created from the filename.
def benchmarkOrig (fname:String) (testName:String) (action:MetaM Unit) : IO Unit := do
  let (msgs, env) ← environmentFromFile fname "Test"
  if (← msgs.hasErrors) then
    IO.println s!"Errors loading {fname}..."
    for msg in msgs.toList do
      IO.print s!"  {← msg.toString (includeEndPos := Lean.Elab.getPrintMessageEndPos {})}"
  else
    IO.print s!"{testName} "
    match ← timeit s!"{testName}" (action.run.run {} {env := env}).toIO' with
    | Except.error msg => do
      IO.println s!"  Error: {←msg.toMessageData.toString}"
    | Except.ok _ => pure ()

def benchmark := benchmarkCPP

def main (args:List String) : IO Unit := do
  match args with
  | [path] => do
      Lean.initSearchPath (some path)
      benchmark "environments/initial.lean" "Binary add   30" (binAddBench   1)
      benchmark "environments/initial.lean" "Binary add   60" (binAddBench   60)
      benchmark "environments/initial.lean" "Binary add  200" (binAddBench  200)
      benchmark "environments/initial.lean" "Binary add 2000" (binAddBench 2000)
  | _ => do
    IO.println "Please provide path to standard library, For example:"
    IO.println "  ./build/bin/Bench $HOME/.elan/toolchains/leanprover-lean4-nightly-2021-03-14/lib/lean"