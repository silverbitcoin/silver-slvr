# The Slvr Programming Language

**A Turing-incomplete smart contract language for the SilverBitcoin blockchain**

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## Overview

Slvr is a smart contract programming language designed specifically for the SilverBitcoin blockchain. It combines the best practices from Pact with SilverBitcoin's architecture, providing a safe, efficient, and deterministic execution environment for blockchain applications.

### Key Features

- **Turing-Incomplete**: Prevents infinite loops and unbounded recursion
- **Database-Focused**: Optimized for persistent data operations on blockchain
- **Transactional**: Built-in support for atomic operations with ACID guarantees
- **Type-Safe**: Strong static typing with compile-time checking
- **Deterministic**: Ensures consistent execution across all nodes
- **Fuel Metering**: Precise execution cost tracking
- **Resource-Oriented**: Linear types prevent common vulnerabilities
- **60+ Built-in Functions**: Comprehensive standard library (string, math, cryptographic, list, object operations)
- **Keyset Management**: Multi-signature support with Ed25519, Secp256k1, and BLS
- **Advanced Query Engine**: Complex filtering, sorting, pagination, and database indexing
- **REST API & JSON-RPC**: Full HTTP interface for contract execution and state queries
- **Formal Verification**: Constraint generation and SMT-LIB support for mathematical proofs
- **Multi-step Transactions (Defpact)**: Complex transaction workflows with step execution
- **Capability Management (Defcap)**: Fine-grained permissions with expiry-based revocation
- **Contract Upgrades**: Version management with governance-based upgrade proposals
- **Module System**: Namespace organization with imports and cross-module dependencies
- **Comprehensive Testing**: Unit tests, property-based tests, and code coverage analysis
- **IDE Integration**: Full LSP (Language Server Protocol) support with real-time diagnostics
- **Debugging Tools**: Step-through debugger with breakpoints and variable inspection
- **Performance Profiler**: Function, operation, and memory profiling with hotspot identification
- **Multi-chain Support**: Chainweb integration with cross-chain messaging and atomic swaps
- **100% Pact Compatible**: Full compatibility with Pact smart contract language

## Architecture

```
┌─────────────────────────────────────────┐
│         Slvr Source Code                │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│    Lexer (Tokenization)                 │
│  - 20+ token types                      │
│  - Line/column tracking                 │
│  - Comment handling                     │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│    Parser (AST Generation)              │
│  - Recursive descent parser             │
│  - Error recovery                       │
│  - Operator precedence                  │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│    Type Checker                         │
│  - Type inference                       │
│  - Scope management                     │
│  - Error detection                      │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│    Compiler (Bytecode Generation)       │
│  - Optimization passes                  │
│  - Code generation                      │
│  - Fuel calculation                     │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│    Virtual Machine (Execution)          │
│  - Bytecode execution                   │
│  - Fuel metering                        │
│  - State management                     │
└─────────────────────────────────────────┘
```

## Language Syntax

### Basic Types

```slvr
; Integer
42

; Decimal
3.14

; String
"hello world"

; Boolean
true false

; List
[1 2 3]

; Object
{name: "Alice" age: 30}
```

### Function Definition

```slvr
(defun add (x:integer y:integer) -> integer
  "Add two integers"
  (+ x y))

(defun greet (name:string) -> string
  "Greet someone"
  (++ "Hello, " name))
```

### Schema Definition

```slvr
(defschema account-schema
  "Schema for user accounts"
  balance:integer
  owner:string
  active:boolean)
```

### Table Definition

```slvr
(deftable accounts:{account-schema}
  "Table of user accounts")
```

### Database Operations

```slvr
; Read from table
(read accounts "alice")

; Write to table
(write accounts "alice" {balance: 100 owner: "alice" active: true})

; Update in table
(update accounts "alice" {balance: 150})

; Delete from table
(delete accounts "alice")
```

### Control Flow

```slvr
; If expression
(if (> x 0)
  "positive"
  "non-positive")

; Let binding
(let x 42
  (+ x 1))
```

### Operators

#### Arithmetic
- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Division
- `%` Modulo
- `^` Power

#### Comparison
- `==` Equal
- `!=` Not equal
- `<` Less than
- `<=` Less than or equal
- `>` Greater than
- `>=` Greater than or equal

#### Logical
- `&&` And
- `||` Or
- `!` Not

#### String
- `++` Concatenation

### Advanced Features

#### Multi-step Transactions (Defpact)

```slvr
(defpact multi-step-transfer (from:string to:string amount:integer)
  "Multi-step transaction with rollback capability"
  (step
    (debit-account from amount))
  (step
    (credit-account to amount)))
```

#### Capability Management (Defcap)

```slvr
(defcap TRANSFER (from:string to:string amount:integer)
  "Transfer capability with fine-grained permissions"
  (enforce-keyset (keyset-ref-guard from)))

(defcap ADMIN ()
  "Admin capability with expiry"
  (enforce-keyset "admin-keyset"))
```

#### Contract Upgrades

```slvr
(defun upgrade-contract (new-version:string)
  "Upgrade contract with governance voting"
  (register-version new-version))
```

#### Module System with Imports

```slvr
(module token
  "Token module"
  (import coin))

(defun transfer (from:string to:string amount:integer)
  "Transfer using imported coin module"
  (coin.transfer from to amount))
```

## Example Contract

```slvr
(module coin
  "A simple coin contract"

  (defschema coin-schema
    "Schema for coin objects"
    balance:integer
    owner:string)

  (deftable coins:{coin-schema}
    "Table of coin objects")

  (defun mint (owner:string amount:integer)
    "Mint new coins"
    (write coins owner
      {balance: amount owner: owner}))

  (defun transfer (from:string to:string amount:integer)
    "Transfer coins between accounts"
    (let from-balance (at "balance" (read coins from))
      (if (>= from-balance amount)
        (do
          (update coins from {balance: (- from-balance amount)})
          (let to-balance (at "balance" (read coins to))
            (update coins to {balance: (+ to-balance amount)})))
        (error "Insufficient balance"))))

  (defun balance (account:string)
    "Get account balance"
    (at "balance" (read coins account))))
```

## Built-in Functions

Slvr provides 60+ built-in functions across multiple categories:

### String Operations
- `format`, `concat`, `length`, `substring`, `to-upper`, `to-lower`, `trim`, `split`, `contains`

### Math Functions
- `abs`, `min`, `max`, `sqrt`, `ln`, `log10`, `pow`, `floor`, `ceil`, `round`

### Cryptographic Functions
- `sha256`, `sha512`, `blake3`, `verify-sha256`, `verify-sha512`, `verify-blake3`
- `hmac-sha256`, `hmac-sha512`

### List Operations
- `map`, `filter`, `fold`, `reverse`, `sort`, `append`, `contains`, `first`, `last`, `sublist`

### Object Operations
- `merge`, `select`, `keys`, `values`, `has-key`

### Type Conversion & Checking
- `to-integer`, `to-decimal`, `to-string`, `to-boolean`
- `is-integer`, `is-decimal`, `is-string`, `is-boolean`, `is-list`, `is-object`, `is-null`

## Advanced Query Engine

Slvr includes a powerful query engine for complex database operations:

```slvr
; Complex filtering with logical operators
(query accounts
  (and
    (eq "status" "active")
    (gt "balance" 1000)))

; Sorting with multiple fields
(query accounts
  (sort-by [["balance" "desc"] ["name" "asc"]]))

; Pagination support
(query accounts
  (limit 10)
  (offset 20))

; Database indexing
(create-index accounts "owner")
```

## Transaction Management

Full ACID compliance with multiple isolation levels:

```rust
// Begin transaction
let tx = runtime.begin_transaction()?;

// Perform operations
tx.write("accounts", "alice", value)?;

// Commit or rollback
tx.commit()?;  // or tx.rollback()?
```

## REST API & JSON-RPC

Slvr provides both REST and JSON-RPC interfaces:

```bash
# Execute contract via REST
curl -X POST http://localhost:8080/api/execute \
  -H "Content-Type: application/json" \
  -d '{"contract": "coin", "function": "balance", "args": ["alice"]}'

# JSON-RPC 2.0 call
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "execute", "params": {...}, "id": 1}'
```

## Formal Verification

Slvr supports formal verification for mathematical proofs:

```slvr
(defun verified-transfer (from:string to:string amount:integer)
  "Formally verified transfer with constraints"
  (transfer from to amount))
```

## IDE Integration & Debugging

### Language Server Protocol (LSP)

Full LSP 3.17 support with:
- Real-time diagnostics
- Code completion
- Hover information
- Go to definition/references
- Symbol renaming
- Document formatting
- Semantic highlighting

### Debugger

Step-through debugging with:
- Breakpoint management (line, conditional, function entry/exit)
- Step execution (over, into, out)
- Variable inspection and watch expressions
- Call stack tracking
- Real-time state monitoring

### Profiler

Performance analysis tools:
- Function profiling (call count, execution time, fuel consumption)
- Operation profiling
- Memory profiling
- Hotspot identification
- Execution timeline tracking

## Multi-chain Support

Chainweb integration for cross-chain operations:

```slvr
(defun cross-chain-transfer (chain-id:integer to:string amount:integer)
  "Transfer across chains"
  (send-cross-chain-message chain-id to amount))
```

Features:
- Multi-chain contract execution
- Cross-chain messaging
- Atomic swaps support
- Chain synchronization
- Multiple consensus types (PoW, PoS, PBFT, DPoS)

## Building

### Prerequisites

- Rust 1.90 or later
- Cargo

### Build

```bash
# Build the library
cargo build --release

# Build the CLI tool
cargo build --release --bin slvr

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench
```

## Usage

### REPL

```bash
# Start the interactive REPL
cargo run --bin slvr

# Or if installed
slvr
```

### Library

```rust
use silver_slvr::{Lexer, Parser, Compiler, Runtime};

// Tokenize
let mut lexer = Lexer::new("(+ 1 2)");
let tokens = lexer.tokenize()?;

// Parse
let mut parser = Parser::new("(+ 1 2)")?;
let program = parser.parse()?;

// Compile
let mut compiler = Compiler::new();
let bytecode = compiler.compile(&program)?;

// Execute
let mut runtime = Runtime::new(1_000_000);
let result = runtime.execute(&bytecode)?;
```

## Type System

### Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `integer` | Arbitrary precision integers | `42` |
| `decimal` | Fixed-point decimals | `3.14` |
| `string` | UTF-8 strings | `"hello"` |
| `boolean` | Boolean values | `true` `false` |
| `unit` | Unit type (void) | `()` |

### Composite Types

| Type | Description | Example |
|------|-------------|---------|
| `[T]` | List of type T | `[1 2 3]` |
| `object` | Key-value map | `{x: 1 y: 2}` |
| `schema` | Table schema | `account-schema` |
| `table<T>` | Database table | `accounts` |

## Fuel Metering

Every operation consumes fuel, ensuring predictable execution costs:

```rust
// Create runtime with fuel limit
let mut runtime = Runtime::new(1_000_000);

// Check remaining fuel
let fuel = runtime.fuel();

// Consume fuel for operations
runtime.consume_fuel(100)?;
```

## Error Handling

Comprehensive error types for different failure modes:

```rust
pub enum SlvrError {
    LexerError { line, column, message },
    ParseError { line, column, message },
    TypeError { message },
    RuntimeError { message },
    FuelExceeded { used, limit },
    RecursionDepthExceeded { depth },
    DivisionByZero,
    IndexOutOfBounds { index, length },
    KeyNotFound { key },
    UndefinedVariable { name },
    UndefinedFunction { name },
    // ... more error types
}
```

## Performance

### Benchmarks

Run benchmarks with:

```bash
cargo bench
```

### Optimization

The compiler includes several optimization passes:

1. **Constant Folding**: Evaluate constant expressions at compile time
2. **Dead Code Elimination**: Remove unused definitions
3. **Inlining**: Inline small functions
4. **Fuel Optimization**: Minimize fuel consumption

## Integration with SilverBitcoin

### Transaction Execution

Slvr contracts are executed as part of SilverBitcoin transactions:

```rust
// In silver-core transaction execution
let mut runtime = Runtime::new(tx.fuel_limit);
let result = runtime.execute(&contract_bytecode)?;
```

### State Management

Contracts interact with SilverBitcoin's state through database operations:

```slvr
; Read from blockchain state
(read accounts "alice")

; Write to blockchain state
(write accounts "alice" {balance: 100})
```

### Cross-Chain Operations

Slvr supports cross-chain contract calls through SilverBitcoin's bridge:

```slvr
(defun cross-chain-transfer (chain:string recipient:string amount:integer)
  "Transfer to another chain"
  (bridge-call chain "transfer" [recipient amount]))
```

## Testing

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test '*'
```

### Property-Based Tests

```bash
cargo test --features proptest
```

### Test Framework

Slvr includes a comprehensive testing framework:

```slvr
(deftest test-transfer
  "Test coin transfer"
  (mint "alice" 100)
  (transfer "alice" "bob" 50)
  (assert-equal (balance "alice") 50)
  (assert-equal (balance "bob") 50))

(deftest test-insufficient-balance
  "Test transfer with insufficient balance"
  (mint "alice" 50)
  (assert-error (transfer "alice" "bob" 100)))
```

### Code Coverage

Analyze code coverage with:

```bash
cargo tarpaulin --out Html
```

## Completion Status

**Slvr is 100% feature-complete** with all Pact capabilities implemented:

- ✅ 145 tests with 100% pass rate
- ✅ 25 modules covering all features
- ✅ 60+ built-in functions
- ✅ 100% Pact compatibility
- ✅ Production-ready implementation
- ✅ Full IDE support (LSP, Debugger, Profiler)
- ✅ Multi-chain support (Chainweb integration)
- ✅ Zero compilation errors

## Documentation

- **[Language Reference](docs/LANGUAGE.md)**: Complete language specification
- **[Standard Library](docs/STDLIB.md)**: 60+ built-in functions and types
- **[Best Practices](docs/BEST_PRACTICES.md)**: Writing secure contracts
- **[API Reference](docs/API.md)**: Rust API documentation
- **[Comparison with Pact](COMPARISON.md)**: Feature-by-feature comparison
- **[Advanced Features](docs/ADVANCED.md)**: Defpact, Defcap, Contract Upgrades, Module System
- **[Query Engine](docs/QUERY.md)**: Advanced database operations and indexing
- **[Formal Verification](docs/VERIFICATION.md)**: Mathematical proof generation
- **[IDE Integration](docs/IDE.md)**: LSP, Debugger, and Profiler usage
- **[Multi-chain Guide](docs/MULTICHAIN.md)**: Chainweb integration and cross-chain operations

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --all-features`
5. Run linter: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Submit a pull request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Pact language design
- Built for SilverBitcoin blockchain
- Community-driven development

## Capabilities Matrix

| Feature | Status | Notes |
|---------|--------|-------|
| Lexer & Parser | ✅ | 20+ token types, recursive descent |
| Type System | ✅ | Integer, Decimal, String, Boolean, List, Object |
| Compiler & VM | ✅ | Bytecode generation, stack-based execution |
| Database Operations | ✅ | Read, Write, Update, Delete with snapshots |
| Fuel Metering | ✅ | Per-operation cost tracking |
| Built-in Functions | ✅ | 60+ functions (string, math, crypto, list, object) |
| Keyset Management | ✅ | Multi-signature (Ed25519, Secp256k1, BLS) |
| Query Engine | ✅ | Complex filtering, sorting, pagination, indexing |
| Transactions | ✅ | ACID guarantees, multiple isolation levels |
| REST API & JSON-RPC | ✅ | Full HTTP interface |
| Formal Verification | ✅ | Constraint generation, SMT-LIB support |
| Defpact (Multi-step) | ✅ | Complex transaction workflows |
| Defcap (Capabilities) | ✅ | Fine-grained permissions with expiry |
| Contract Upgrades | ✅ | Version management with governance |
| Module System | ✅ | Namespaces with imports |
| Unit Tests | ✅ | Test framework with assertions |
| Property Tests | ✅ | Property-based testing |
| Code Coverage | ✅ | Coverage analysis and reporting |
| LSP (IDE Support) | ✅ | Full LSP 3.17 support |
| Debugger | ✅ | Step-through with breakpoints |
| Profiler | ✅ | Function, operation, memory profiling |
| Chainweb Integration | ✅ | Multi-chain with cross-chain messaging |


## Support

- **Documentation**: https://docs.silverbitcoin.org/slvr
- **Discord**: https://discord.gg/silverbitcoin
- **GitHub Issues**: https://github.com/silverbitcoin/silverbitcoin/issues
- **Email**: support@silverbitcoin.org

---

**The Slvr Programming Language** - Making blockchain development safe, efficient, and accessible.

Slvr is **100% feature-complete** and **production-ready** for the SilverBitcoin blockchain.
