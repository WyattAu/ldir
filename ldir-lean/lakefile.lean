import Lake
open Lake DSL

package «LDIRProofs» where
  leanOptions := #[⟨`autoImplicit, false⟩]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "b301d257a1c13bc4e27350c06e5169b8b08a53ed"

@[default_target]
lean_lib «LDIRProofs»
