
--def simp (e : Expr) (ctx : Simp.Context) : MetaM Simp.Result := do profileitM Exception "simp" (← getOptions) do
--  Simp.main e ctx (methods := Simp.DefaultMethods.methods)

import Lean.Meta

import Lean.Elab.Frontend

open Lean

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
--  monadLift $ IO.println s!"Result {r}"
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


-- This outputs how long it takes to run the given meta option in the
-- context created from the filename.
def benchmark (fname:String) (testName:String) (action:MetaM Unit) : IO Unit := do
  let (msgs, env) ← environmentFromFile fname "Test"
  if (← msgs.hasErrors) then
    IO.println s!"Errors loading {fname}..."
    for msg in msgs.toList do
      IO.print s!"  {← msg.toString (includeEndPos := Lean.Elab.getPrintMessageEndPos {})}"
  else
    match ← timeit s!"{testName}" (action.run.run {} {env := env}).toIO' with
    | Except.error msg => do
      IO.println s!"  Error: {←msg.toMessageData.toString}"
    | Except.ok _ => pure ()

def main (args:List String) : IO Unit := do
  Lean.initSearchPath (some "/Users/jhendrix/.elan/toolchains/leanprover-lean4-nightly-2021-03-14/lib/lean")
  benchmark "environments/initial.lean" "Binary add   30" (binAddBench   30)
  benchmark "environments/initial.lean" "Binary add   60" (binAddBench   60)
  benchmark "environments/initial.lean" "Binary add  200" (binAddBench  200)
  benchmark "environments/initial.lean" "Binary add 2000" (binAddBench 2000)
