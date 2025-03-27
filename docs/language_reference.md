# Bend-PVM Language Reference

## Introduction

Bend-PVM is a new smart contract programming language designed for the PolkaVM runtime. It focuses on safety, efficiency, and expressive power, making it ideal for developing secure and resource-efficient smart contracts.

The language combines functional programming principles with a familiar syntax, allowing developers to use pattern matching, algebraic data types, and monadic composition while maintaining readability.

## Syntax Basics

### Comments

Bend-PVM supports single-line and multi-line comments:

```bend
# This is a single-line comment

#{
  This is a multi-line comment
  that spans multiple lines
}#
```

### Data Types

Bend-PVM provides the following primitive data types:

- `u24`: Unsigned 24-bit integer (0 to 16,777,215)
- `i24`: Signed 24-bit integer (-8,388,608 to 8,388,607)
- `f24`: 24-bit floating point number
- `String`: UTF-8 string
- `List`: Homogeneous list of elements
- `Option`: Optional value (Some or None)
- `Result`: Success or error result (Ok or Err)

### Variables and Assignments

Variables are assigned using the `=` operator:

```bend
x = 42
name = "Alice"
result = calculate_value()
```

### Functions

Functions are defined using the `def` keyword:

```bend
def add(a: u24, b: u24) -> u24:
    return a + b
```

Functions can have type annotations for parameters and return values, but they are optional in some cases.

### Control Flow

#### If Statements

```bend
if x > 0:
    return "Positive"
else:
    return "Non-positive"
```

#### Match Statements

```bend
match result:
    case Result/Ok:
        return result.value
    case Result/Err:
        return 0
```

#### Bend Statements

The `bend` statement is a unique control flow construct for iterative computations:

```bend
bend counter = 0:
    when counter < 10:
        counter = counter + 1
        # Do something
    else:
        # Done
```

### Type Definitions

Custom types are defined using the `type` keyword:

```bend
type Result:
    Ok { value: u24 }
    Err { reason: String }
```

### Object Definitions

Objects are defined using the `object` keyword:

```bend
object Point {
    x: f24,
    y: f24
}
```

## Resource Model

Bend-PVM uses a multi-dimensional resource model that tracks:

1. **ref_time**: Computational costs (similar to Ethereum gas)
2. **proof_size**: Size of validator state proofs
3. **storage_deposit**: Storage anti-bloat mechanism

This model provides more precise resource tracking and optimization opportunities.

## Monadic Composition

Bend-PVM supports monadic composition through the `with` statement:

```bend
def transfer(to: String, amount: u24) -> Result:
    with IO:
        from_balance = get_balance(caller())
        if from_balance < amount:
            return Result/Err { reason: "Insufficient balance" }
        set_balance(caller(), from_balance - amount)
        to_balance = get_balance(to)
        set_balance(to, to_balance + amount)
        return Result/Ok { value: 1 }
```

## Pattern Matching

Pattern matching is supported in `match` statements:

```bend
match value:
    case List/Nil:
        return 0
    case List/Cons:
        return 1 + length(value.tail)
```

## Libraries and Imports

### Import Statements

```bend
import Math

from String import concat, length
```

### Standard Library

Bend-PVM includes a standard library with the following modules:

- `String`: String manipulation functions
- `Math`: Mathematical functions
- `List`: List operations
- `Option`: Option type functions
- `Result`: Result type functions
- `Crypto`: Cryptographic functions
- `IO`: Input/output and blockchain interaction

## Contract Structure

A typical Bend-PVM contract consists of:

1. Type definitions
2. Helper functions
3. Public contract functions
4. Main entry point

Example:

```bend
# Type definitions
type TokenResult:
    Ok { value: u24 }
    Err { reason: String }

# Helper functions
def get_balance(owner: String) -> u24:
    return storage_get("balance:" + owner)

def set_balance(owner: String, amount: u24) -> None:
    storage_set("balance:" + owner, amount)

# Public contract functions
def transfer(to: String, amount: u24) -> TokenResult:
    with IO:
        from = caller()
        from_balance = get_balance(from)
        if from_balance < amount:
            return TokenResult/Err { reason: "Insufficient balance" }
        set_balance(from, from_balance - amount)
        to_balance = get_balance(to)
        set_balance(to, to_balance + amount)
        return TokenResult/Ok { value: 1 }

# Main entry point
def main() -> u24:
    with IO:
        selector = call_data_selector()
        if selector == "transfer":
            to = parse_address(1)
            amount = parse_u24(2)
            result = transfer(to, amount)
            return 1
        else:
            revert("Unknown function selector")
```

## Best Practices

1. **Type Safety**: Use type annotations to catch errors at compile time.
2. **Resource Efficiency**: Be mindful of storage, computation, and proof size costs.
3. **Pattern Matching**: Use pattern matching for clearer and safer code.
4. **Monadic Composition**: Use monads to separate pure and effectful code.
5. **Security**: Follow security best practices, especially for financial contracts.

## Advanced Features

### Recursive Types

Bend-PVM supports recursive types using the `~` symbol:

```bend
type Tree:
    Leaf { value: u24 }
    Node { value: u24, ~left: Tree, ~right: Tree }
```

### Higher-Order Functions

Functions can take other functions as arguments:

```bend
def map(list: List(T), f: T -> U) -> List(U):
    match list:
        case List/Nil:
            return List/Nil
        case List/Cons:
            return List/Cons(f(list.head), map(list.tail, f))
```

### Lambda Expressions

Lambda expressions create anonymous functions:

```bend
increment = lambda x: x + 1

doubled_values = map(values, lambda x: x * 2)
```

## Conclusion

Bend-PVM combines the safety and expressiveness of functional programming with a familiar syntax, making it an ideal choice for developing secure and efficient smart contracts on PolkaVM. By leveraging its resource model and type system, developers can write contracts that are both safe and cost-effective.