# Dolores

[The Lox Programming Language](https://www.craftinginterpreters.com/the-lox-language.html), implemented in relatively idiomatic Rust.

## Features

⚠️ This project is still a work in progress.

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
    - [ ] Static methods\*
    - [ ] Inheritance

\* : Syntax extension

<!--

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

### Prerequisites

The things you need before installing the software.

- You need this
- And you need this
- Oh, and don't forget this

### Installation

A step by step guide that will tell you how to get the development environment up and running.

```
$ First step
$ Another step
$ Final step
```

## Usage

A few examples of useful commands and/or tasks.

```
$ First example
$ Second example
$ And keep this in mind
```

-->
