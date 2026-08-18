## Component Role
`wm-macros` is a dedicated procedural macro crate providing custom derive macros (`SubEnum`, `EnumFromInner`) for AST manipulation and boilerplate elimination across workspace enum types.

## Dependency Graph
- **Inherited (`workspace = true`)**: None
- **External**:
  - `syn` = "2.0.103" (default features)
  - `quote` = "1.0" (default features)
  - `proc-macro2` = "1.0" (default features)
- **Local Workspace Peers**:
  - Consumed by `wm` (`packages/wm`) via `[workspace.dependencies]`

## Public API
```rust
// Crate-level attributes
#![feature(proc_macro_diagnostic)]

// Exported Procedural Derive Macros
#[proc_macro_derive(SubEnum, attributes(subenum))]
pub fn sub_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream;

#[proc_macro_derive(EnumFromInner)]
pub fn enum_from_inner(input: proc_macro::TokenStream) -> proc_macro::TokenStream;
```

## Architecture & Modules
```text
wm-macros/src/
├── lib.rs                       # Proc-macro entry points, compiler feature gates, and root prelude
├── common/                      # Reusable AST parsing, token stream combinators, and error utilities
│   ├── mod.rs                   # Module declarations and keyword macro re-exports
│   ├── attributes.rs            # Trait `FindAttributes` for attribute filtering on `Vec<syn::Attribute>` / `&[syn::Attribute]`
│   ├── branch.rs                # Parsing combinators (`Ordered`, `Unordered`, `IfElse`, `Optional`) and `ParseableTuple` / `PeekableTuple`
│   ├── error_handling.rs        # Diagnostics & error traits (`ThenError`, `ToError`, `ToSpanError`, `EmitError`)
│   ├── named_parameter.rs       # Key-value AST parser `NamedParameter<Name, Param>` (`name = value`)
│   ├── parenthesized.rs         # Delimiter wrapper `Parenthesized<T>`
│   ├── peekable.rs              # Generic lookahead system (`Peekable`, `SynPeek`, `PeekableStream`, `TPeek`)
│   └── spanned_string.rs        # Span-preserving string wrapper `SpannedString`
├── enum_from_inner/             # Derive implementation for `EnumFromInner`
│   └── mod.rs                   # Generates `From<T>`, `TryFrom<Enum>`, and `TryFrom<&Enum>` for single-field variants
└── subenum/                     # Derive implementation for `SubEnum`
    ├── mod.rs                   # Variant extraction, sub-enum generation, cross-enum `From` / `TryFrom` generators
    ├── enum_attrs.rs            # Parser for `#[subenum(...)]` declarations and `defaults` attribute blocks
    └── variant_attr.rs          # Parser for variant-level subenum assignments and single-field validations
```

### Internal Type Signatures & Traits

#### `common::attributes`
- `pub trait FindAttributes`:
  - `fn find_attrs(&self, name: &str) -> impl Iterator<Item = &syn::Attribute>`
  - `fn find_list_attrs(&self, name: &str) -> impl Iterator<Item = &syn::MetaList>`

#### `common::branch`
- `pub trait ParseableTuple: Sized`:
  - `type FirstItem: syn::parse::Parse`
  - `fn parse_tuple<Sep: syn::parse::Parse>(stream: syn::parse::ParseStream) -> syn::Result<Self>`
- `pub trait PeekableTuple: Sized`:
  - `fn peek_parse_tuple<Sep: syn::parse::Parse + Peekable>(stream: syn::parse::ParseStream) -> syn::Result<Self>`
- `pub struct Ordered<T: ParseableTuple, Sep: syn::parse::Parse> { pub items: T, ... }`
- `pub struct Unordered<T: PeekableTuple, Sep: syn::parse::Parse + Peekable> { pub items: T, ... }`
- `pub enum IfElse<If: syn::parse::Parse + Peekable, Else: syn::parse::Parse> { If(If), Else(Else) }`
- `pub enum Optional<T: syn::parse::Parse + Peekable> { Some(T), None }`

#### `common::error_handling`
- `pub trait ThenError<E>: Sized { fn then_error(self, error: E) -> Result<Self, E>; }`
- `pub trait ToError { fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error; }`
- `pub trait ToSpanError { fn serror<D: core::fmt::Display>(&self, message: D) -> syn::Error; }`
- `pub trait EmitError { fn emit_warning<D: Into<String>>(&self, message: D); fn emit_help<D: Into<String>>(&self, message: D); fn emit_note<D: Into<String>>(&self, message: D); }`

#### `common::peekable`
- `pub trait PeekableStream { fn is_empty(&self) -> bool; fn peek<T: syn::parse::Peek>(&self, token: T) -> bool; }`
- `pub trait Peekable { fn peek<T: PeekableStream>(stream: T) -> bool; fn display() -> &'static str; }`
- `pub trait SynPeek { fn peekable() -> impl syn::parse::Peek; fn display() -> &'static str; }`
- `pub trait TPeek<'a> { fn tpeek<T: Peekable>(&'a self) -> bool; }`

#### `common::spanned_string`
- `pub struct SpannedString { pub value: String, pub span: proc_macro2::Span }`

#### `subenum::enum_attrs`
- `pub enum Subenum { Defaults(proc_macro2::TokenStream), Declaration(SubenumDeclaration) }`
- `pub struct SubenumDeclaration { pub name: syn::Ident, pub attrs: proc_macro2::TokenStream }`

#### `subenum::variant_attr`
- `pub struct SubenumVariant { pub name: syn::Ident, pub contained: syn::Type, pub enums: Vec<syn::Ident> }`
- `pub fn parse_variant(variant: &syn::Variant) -> syn::Result<SubenumVariant>`

## Execution Context
- **Execution Boundary**: Compile-time proc-macro execution only (executed synchronously by `rustc` during compilation).
- **Runtime System**: None; zero runtime dependencies or async executors (no Tokio, no I/O polling).
- **Toolchain Requirement**: Nightly Rust required due to crate-level `#![feature(proc_macro_diagnostic)]` used for emitting inline compiler diagnostics in `EmitError`.
- **State & Concurrency**: Stateless and immutable macro expansion pipelines; operates entirely via purely functional AST transformations from `proc_macro::TokenStream` to `proc_macro::TokenStream`.
