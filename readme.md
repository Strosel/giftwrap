# Giftwrap
Wrap and unwrap your types the stylish way with derive macros for From/TryFrom in both directions

## How does it work?
`Giftwrap` exposes two derive macros, `Wrap` and `Unwrap` that derive `impl From<inner_type> for your_type` and `impl From<your_type> for inner_type` (or `TryFrom<your_type>` in the case of enums) respectively.
It works for any struct or enum variant that holds only a single type, and don't worry variants with multiple types or with types you want to convert yourself can be easily ignored with `#[noWrap]` and `#[noUnwrap]`.

## Todo
`Giftwrap` does not yet support:
- [ ] Generics / Lifetimes
