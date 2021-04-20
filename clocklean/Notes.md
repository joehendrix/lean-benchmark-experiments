# Notes on language differences between Lean and Rust.

## Naming conventions

TODO: Capture different naming conventions expected by the languages on types,
definitions, and modules/namespaces.

## Type system

TODO: Capture type system differences between the languages must relevant to
compiled code

## Compound names (Lean namespaces, Rust modules, Rust implementations).

Both Lean and Rust provide mechanisms for compound names.  Lean uses
namespaces which are extensible (one can add new definitions to a namespace
in any module).  Rust provides two mechanisms: modules and implementations
that both have more constraints than Lean.  We describ the Rust constraints
below:

### Rust module constraints

- Rust modules are complete -- one cannot add new definitions.
- Rust modules introduced in a file must be distinct from other types.

### Rust implementation constraints

- Rust implementations are extensible, but always tied to a specific type.
- Rust implementations either define new methods on a type or implement a trait on a type.
- Rust implementations must refer to type or trait defined in current crate.
  ```
  use std::rc::Rc;
  // Not allowed
  impl<A> Rc<A> {}
  // Allowed
  pub trait NewTrait {}
  impl<A> NewTrait for Rc<A> {}
  ```
- Rust implementations have significant limitations on what can be defined within.
  - Do not support type aliases (see https://github.com/rust-lang/rust/issues/8995)
  - Do not support struct
  - Do support definitions
    - Definitions do not have to mention type, but parameters must be defined, e.g.,
    ```
    impl<A> IO<A> {
      pub fn add(x:u32) -> u32 { x }
    }
    let x = IO::<u64>::add(x)
    ```