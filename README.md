# dolores

> "... light of my life."

[The Lox Programming Language](https://www.craftinginterpreters.com/the-lox-language.html) implemented in idiomatic Rust.

Ported from the original Java implementation `jlox`, but with no visitor patterns, no `null` values, no subclasses and implicit conversions, just pure `enum`s.

~~`Arc<T>` is a necessary evil...~~

---

## Contents

- [dolores](#dolores)
  - [Contents](#contents)
  - [Features](#features)
  - [Try it out!](#try-it-out)

---

## Features

- [x] AST-walking interpreter
  - [x] Lexer
  - [x] Parser
  - [x] Basic types
  - [x] Floating point arithmetics
  - [x] Logic expressions
  - [x] Control flow
    - [x] Jumps: `break`/`continue`\*
      - [x] Semantic analysis: jumping out of loops
  - [x] Functions
    - [x] Lambdas\*
    - [x] Semantic analysis: returning out of functions
    - [x] Semantic analysis: static closure captures
  - [x] Classes
    - [x] Instances
    - [x] Instance methods
      - [x] `this`
        - [x] Semantic analysis: `this` out of classes
      - [x] Initializers
        - [x] Semantic analysis: `return`ing values in initializers
    - [x] Inheritance
      - [x] `super`
        - [x] Semantic analysis: `super` out of subclasses

\* : Syntax extension

## Try it out!

With the latest [Rust toolchain](https://www.rust-lang.org/tools/install) installed, just execute:

```bash
cargo run
```
