# Migrating from Solidity to Bend-PVM

## Introduction

This guide is designed to help Solidity developers transition to Bend-PVM. While both languages target smart contract development, Bend-PVM offers several advantages:

- More precise resource model
- Functional programming features
- Pattern matching
- Type safety improvements
- Better error handling with monads

This guide will walk you through the equivalent Bend-PVM concepts and syntax for common Solidity patterns.

## Basic Syntax Comparison

### Contract Structure

**Solidity**
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleStorage {
    uint256 private value;
    
    function setValue(uint256 newValue) public {
        value = newValue;
    }
    
    function getValue() public view returns (uint256) {
        return value;
    }
}
```

**Bend-PVM**
```bend
# Simple storage contract

def set_value(new_value: u24) -> u24:
    with IO:
        storage_set("value", new_value)
        return 1

def get_value() -> u24:
    with IO:
        return storage_get("value")

def main() -> u24:
    with IO:
        selector = call_data_selector()
        if selector == "set":
            value = parse_u24(1)
            return set_value(value)
        elif selector == "get":
            return_value(u24_to_string(get_value()))
            return 1
        else:
            revert("Unknown function selector")
```

### Data Types

| Solidity      | Bend-PVM       | Notes                                    |
|---------------|----------------|------------------------------------------|
| `bool`        | `u24`          | 0 = false, non-zero = true               |
| `uint8-256`   | `u24`          | 24-bit (0 to 16,777,215)                 |
| `int8-256`    | `i24`          | 24-bit (-8,388,608 to 8,388,607)         |
| `address`     | `String`       | Represented as a string                  |
| `string`      | `String`       | UTF-8 encoded string                     |
| `bytes`       | `List(u24)`    | List of bytes                            |
| `mapping`     | Storage prefix | Implemented using key prefixes           |
| `struct`      | `object`       | Defined using the `object` keyword       |
| `enum`        | Custom type    | Defined using custom type variants       |

### Functions

**Solidity**
```solidity
function transfer(address to, uint256 amount) public returns (bool) {
    require(balances[msg.sender] >= amount, "Insufficient balance");
    balances[msg.sender] -= amount;
    balances[to] += amount;
    emit Transfer(msg.sender, to, amount);
    return true;
}
```

**Bend-PVM**
```bend
def transfer(to: String, amount: u24) -> Result:
    with IO:
        from = caller()
        from_balance = get_balance(from)
        if from_balance < amount:
            return Result/Err { reason: "Insufficient balance" }
        set_balance(from, from_balance - amount)
        to_balance = get_balance(to)
        set_balance(to, to_balance + amount)
        emit_event("Transfer", [from, to, u24_to_string(amount)])
        return Result/Ok { value: 1 }
```

### Error Handling

**Solidity**
```solidity
function withdraw(uint256 amount) public {
    require(balances[msg.sender] >= amount, "Insufficient balance");
    require(amount > 0, "Amount must be positive");
    
    balances[msg.sender] -= amount;
    (bool success, ) = msg.sender.call{value: amount}("");
    require(success, "Transfer failed");
}
```

**Bend-PVM**
```bend
def withdraw(amount: u24) -> Result:
    with IO:
        from = caller()
        from_balance = get_balance(from)
        
        if from_balance < amount:
            return Result/Err { reason: "Insufficient balance" }
        if amount == 0:
            return Result/Err { reason: "Amount must be positive" }
        
        set_balance(from, from_balance - amount)
        
        transfer_result = transfer_native(from, amount)
        match transfer_result:
            case Result/Ok:
                return Result/Ok { value: 1 }
            case Result/Err:
                # Revert the balance change
                set_balance(from, from_balance)
                return Result/Err { reason: "Transfer failed" }
```

### Events

**Solidity**
```solidity
event Transfer(address indexed from, address indexed to, uint256 amount);

function transfer(address to, uint256 amount) public {
    // ... transfer logic
    emit Transfer(msg.sender, to, amount);
}
```

**Bend-PVM**
```bend
def transfer(to: String, amount: u24) -> Result:
    with IO:
        # ... transfer logic
        emit_event("Transfer", [caller(), to, u24_to_string(amount)])
        return Result/Ok { value: 1 }
```

### Custom Types

**Solidity**
```solidity
enum State { Created, Locked, Release, Inactive }

struct Payment {
    uint256 amount;
    address payable recipient;
    bool completed;
}
```

**Bend-PVM**
```bend
type State:
    Created
    Locked
    Released
    Inactive

object Payment {
    amount: u24,
    recipient: String,
    completed: u24  # 0 = false, 1 = true
}
```

## Storage Management

### Solidity Storage

**Solidity**
```solidity
contract Storage {
    mapping(address => uint256) private balances;
    mapping(address => mapping(address => uint256)) private allowances;
    
    function getBalance(address account) public view returns (uint256) {
        return balances[account];
    }
    
    function getAllowance(address owner, address spender) public view returns (uint256) {
        return allowances[owner][spender];
    }
}
```

**Bend-PVM**
```bend
def get_balance(account: String) -> u24:
    with IO:
        balance = storage_get("balance:" + account)
        return balance

def get_allowance(owner: String, spender: String) -> u24:
    with IO:
        allowance = storage_get("allowance:" + owner + ":" + spender)
        return allowance
```

### Storage Keys

In Bend-PVM, storage is accessed using string keys. A common pattern is to use prefixes to create namespaced storage similar to Solidity mappings:

```bend
def token_balance_key(token: String, owner: String) -> String:
    return "token_balance:" + token + ":" + owner

def set_token_balance(token: String, owner: String, amount: u24) -> None:
    with IO:
        storage_set(token_balance_key(token, owner), amount)

def get_token_balance(token: String, owner: String) -> u24:
    with IO:
        return storage_get(token_balance_key(token, owner))
```

## Advanced Patterns

### Using Options (Nullable Values)

**Solidity**
```solidity
struct Account {
    uint256 balance;
    bool exists;
}

mapping(address => Account) private accounts;

function getAccount(address user) public view returns (uint256, bool) {
    Account memory account = accounts[user];
    return (account.balance, account.exists);
}
```

**Bend-PVM**
```bend
type Account:
    None
    Some { balance: u24 }

def get_account(user: String) -> Account:
    with IO:
        balance = storage_get("account:" + user)
        if balance == 0:
            return Account/None
        else:
            return Account/Some { balance: balance }
```

### Using Results (Error Handling)

**Solidity**
```solidity
function safeTransfer(address to, uint256 amount) public returns (bool success, string memory errorMessage) {
    if (balances[msg.sender] < amount) {
        return (false, "Insufficient balance");
    }
    
    balances[msg.sender] -= amount;
    balances[to] += amount;
    
    return (true, "");
}
```

**Bend-PVM**
```bend
type TransferResult:
    Ok { value: u24 }
    Err { reason: String }

def safe_transfer(to: String, amount: u24) -> TransferResult:
    with IO:
        from = caller()
        from_balance = get_balance(from)
        
        if from_balance < amount:
            return TransferResult/Err { reason: "Insufficient balance" }
        
        set_balance(from, from_balance - amount)
        to_balance = get_balance(to)
        set_balance(to, to_balance + amount)
        
        return TransferResult/Ok { value: 1 }
```

### Interface Implementation

**Solidity**
```solidity
interface IERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
}

contract MyToken is IERC20 {
    function totalSupply() external view override returns (uint256) {
        return _totalSupply;
    }
    
    function balanceOf(address account) external view override returns (uint256) {
        return _balances[account];
    }
    
    function transfer(address to, uint256 amount) external override returns (bool) {
        // Transfer implementation
        return true;
    }
}
```

**Bend-PVM**

Bend-PVM doesn't have explicit interfaces, but you can implement standard function signatures to achieve compatibility:

```bend
def total_supply() -> u24:
    with IO:
        return storage_get("total_supply")

def balance_of(account: String) -> u24:
    with IO:
        return storage_get("balance:" + account)

def transfer(to: String, amount: u24) -> Result:
    with IO:
        # Transfer implementation
        return Result/Ok { value: 1 }

def main() -> u24:
    with IO:
        selector = call_data_selector()
        
        if selector == "totalSupply":
            return_value(u24_to_string(total_supply()))
            return 1
        elif selector == "balanceOf":
            account = parse_address(1)
            return_value(u24_to_string(balance_of(account)))
            return 1
        elif selector == "transfer":
            to = parse_address(1)
            amount = parse_u24(2)
            result = transfer(to, amount)
            match result:
                case Result/Ok:
                    return_value(u24_to_string(1))
                    return 1
                case Result/Err:
                    revert(result.reason)
                    return 0
```

## Security Considerations

When migrating from Solidity to Bend-PVM, keep these security considerations in mind:

1. **Resource Limits**: Bend-PVM has explicit resource limits for ref_time, proof_size, and storage_deposit.
2. **Type Safety**: Leverage Bend-PVM's stronger type system to prevent errors.
3. **Pattern Matching**: Use pattern matching to handle all possible cases.
4. **Error Handling**: Use Result types instead of reverting for better error handling.
5. **Storage Keys**: Be careful with storage key construction to avoid collisions.

## Best Practices

### 1. Use Explicit Types

```bend
def transfer(to: String, amount: u24) -> Result:
    # ...
```

### 2. Use Pattern Matching

```bend
match result:
    case Result/Ok:
        return result.value
    case Result/Err:
        return 0
```

### 3. Use Monadic Composition

```bend
def transfer(to: String, amount: u24) -> Result:
    with IO:
        # IO operations...
```

### 4. Document Function Behavior

```bend
# Transfers tokens from the caller to the recipient
# Returns Ok(1) on success or Err with a reason on failure
def transfer(to: String, amount: u24) -> Result:
    # ...
```

### 5. Use Helper Functions for Storage

```bend
def balance_key(owner: String) -> String:
    return "balance:" + owner

def get_balance(owner: String) -> u24:
    with IO:
        return storage_get(balance_key(owner))
```

## Conclusion

Migrating from Solidity to Bend-PVM offers numerous benefits in terms of type safety, resource management, and functional programming features. While the syntax is different, the core concepts of smart contract development remain the same.

By leveraging Bend-PVM's powerful features like monadic composition, pattern matching, and algebraic data types, you can write more robust and maintainable smart contracts with fewer bugs and better resource efficiency.