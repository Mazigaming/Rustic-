# Rustic++

A programming language where C, C++, and Rust libraries are first-class citizens.

## The Big Idea

Rustic++ automatically generates bindings for C, C++, and Rust libraries, allowing you to import and use them as if they were native code:

```rustic
import "raylib.h" as raylib;

fn main() -> Int {
    raylib.init_window(800, 600, "Hello");
    return 0;
}
```

No hand-written `#[extern "C"]` glue. No manual binding code. Just import and use.

## Status

**Phase 1: Lexer, Parser, AST** ✅ Complete

The foundation is built:
- Hand-written lexer with full token support
- Recursive descent parser with operator precedence
- AST representation for all language constructs
- 20+ example programs parsing successfully

## Project Structure

```
rustic-plusplus/
├── compiler/       # lexer, parser, type checker, IR
├── interp/         # tree-walking interpreter (Phase 2)
├── codegen/        # native x86-64 backend (Phase 5)
├── runtime/        # GC, FFI bridge, dynamic loader
├── bindgen-c/      # C header → bindings (Phase 4)
├── bindgen-cpp/    # C++ class → shim generator (Phase 6)
├── bindgen-rust/   # Rust source → extern "C" wrapper (Phase 7)
├── stdlib/         # standard library
├── examples/       # one example program per phase
└── docs/           # documentation
```

## Building

```bash
cargo build --package rustic-compiler
./target/debug/rcpp examples/phase1/basic_math.rcpp
```

## Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| 0 | Proof of concept (trilang-poc) | ✅ |
| 1 | Lexer, Parser, AST | ✅ |
| 2 | Tree-walking interpreter | 🔄 |
| 3 | Type system & semantic analysis | ⏳ |
| 4 | C interop | ⏳ |
| 5 | Native codegen | ⏳ |
| 6 | C++ interop | ⏳ |
| 7 | Rust interop | ⏳ |
| 8 | Garbage collector & memory model | ⏳ |
| 9 | Tooling & build system | ⏳ |
| 10 | Flagship demo & v1.0 | ⏳ |

## License

MIT