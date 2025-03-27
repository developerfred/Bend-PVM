#{
  Simple token contract written in Bend-PVM.
  This implements a basic ERC-20 compatible token with transfer, approve, and transferFrom functionality.
}

type Result:
  Ok { value: u24 }
  Err { reason: String }

# Storage keys
def TOTAL_SUPPLY_KEY() -> String:
  return "total_supply"

def BALANCE_KEY(owner: String) -> String:
  return "balance:" + owner

def ALLOWANCE_KEY(owner: String, spender: String) -> String:
  return "allowance:" + owner + ":" + spender

# Contract storage interface
def get_storage(key: String) -> u24:
  with IO:
    result <- IO/storage_get(key)
    match result:
      case Result/Ok:
        return u24/from_string(result.value)
      case Result/Err:
        return 0

def set_storage(key: String, value: u24) -> None:
  with IO:
    * <- IO/storage_set(key, u24/to_string(value))
    return wrap(*)

# Token implementation
def total_supply() -> u24:
  return get_storage(TOTAL_SUPPLY_KEY())

def balance_of(owner: String) -> u24:
  return get_storage(BALANCE_KEY(owner))

def allowance(owner: String, spender: String) -> u24:
  return get_storage(ALLOWANCE_KEY(owner, spender))

def transfer(to: String, amount: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    caller_balance = balance_of(caller)
    
    if caller_balance < amount:
      return Result/Err { reason: "Insufficient balance" }
    
    # Update balances
    set_storage(BALANCE_KEY(caller), caller_balance - amount)
    recipient_balance = balance_of(to)
    set_storage(BALANCE_KEY(to), recipient_balance + amount)
    
    # Emit Transfer event
    * <- IO/emit_event("Transfer", [caller, to, u24/to_string(amount)])
    
    return Result/Ok { value: 1 }

def approve(spender: String, amount: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    
    # Set allowance
    set_storage(ALLOWANCE_KEY(caller, spender), amount)
    
    # Emit Approval event
    * <- IO/emit_event("Approval", [caller, spender, u24/to_string(amount)])
    
    return Result/Ok { value: 1 }

def transfer_from(from: String, to: String, amount: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    
    # Check allowance
    allowed_amount = allowance(from, caller)
    if allowed_amount < amount:
      return Result/Err { reason: "Insufficient allowance" }
    
    # Check balance
    from_balance = balance_of(from)
    if from_balance < amount:
      return Result/Err { reason: "Insufficient balance" }
    
    # Update allowance
    set_storage(ALLOWANCE_KEY(from, caller), allowed_amount - amount)
    
    # Update balances
    set_storage(BALANCE_KEY(from), from_balance - amount)
    recipient_balance = balance_of(to)
    set_storage(BALANCE_KEY(to), recipient_balance + amount)
    
    # Emit Transfer event
    * <- IO/emit_event("Transfer", [from, to, u24/to_string(amount)])
    
    return Result/Ok { value: 1 }

# Initial setup - called only once when the contract is deployed
def initialize(initial_supply: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    
    # Check if already initialized
    if total_supply() > 0:
      return Result/Err { reason: "Already initialized" }
    
    # Set total supply
    set_storage(TOTAL_SUPPLY_KEY(), initial_supply)
    
    # Assign all tokens to the creator
    set_storage(BALANCE_KEY(caller), initial_supply)
    
    # Emit Transfer event (from zero address)
    * <- IO/emit_event("Transfer", ["0x0000000000000000000000000000000000000000", caller, u24/to_string(initial_supply)])
    
    return Result/Ok { value: 1 }

# Entry point - dispatches to the appropriate function based on the function selector
def main() -> IO(u24):
  with IO:
    # Get function selector from call data
    call_data <- IO/get_call_data()
    
    # Extract function selector (first 4 bytes)
    selector = String/slice(call_data, 0, 4)
    
    # Dispatch to the appropriate function
    if selector == "init":
      # Initialize(uint256)
      initial_supply = u24/from_string(String/slice(call_data, 4, 0))
      result = initialize(initial_supply)
      return wrap(1)
    
    elif selector == "ttsp":
      # TotalSupply()
      result = total_supply()
      * <- IO/return_value(u24/to_string(result))
      return wrap(1)
    
    elif selector == "balc":
      # BalanceOf(address)
      owner = String/slice(call_data, 4, 0)
      result = balance_of(owner)
      * <- IO/return_value(u24/to_string(result))
      return wrap(1)
    
    elif selector == "tran":
      # Transfer(address,uint256)
      to = String/slice(call_data, 4, 20)
      amount = u24/from_string(String/slice(call_data, 24, 0))
      result = transfer(to, amount)
      
      match result:
        case Result/Ok:
          * <- IO/return_value(u24/to_string(1))
          return wrap(1)
        case Result/Err:
          * <- IO/revert(result.reason)
          return wrap(0)
    
    elif selector == "aprv":
      # Approve(address,uint256)
      spender = String/slice(call_data, 4, 20)
      amount = u24/from_string(String/slice(call_data, 24, 0))
      result = approve(spender, amount)
      
      match result:
        case Result/Ok:
          * <- IO/return_value(u24/to_string(1))
          return wrap(1)
        case Result/Err:
          * <- IO/revert(result.reason)
          return wrap(0)
    
    elif selector == "alow":
      # Allowance(address,address)
      owner = String/slice(call_data, 4, 20)
      spender = String/slice(call_data, 24, 20)
      result = allowance(owner, spender)
      * <- IO/return_value(u24/to_string(result))
      return wrap(1)
    
    elif selector == "trfm":
      # TransferFrom(address,address,uint256)
      from = String/slice(call_data, 4, 20)
      to = String/slice(call_data, 24, 20)
      amount = u24/from_string(String/slice(call_data, 44, 0))
      result = transfer_from(from, to, amount)
      
      match result:
        case Result/Ok:
          * <- IO/return_value(u24/to_string(1))
          return wrap(1)
        case Result/Err:
          * <- IO/revert(result.reason)
          return wrap(0)
    
    else:
      # Unknown function
      * <- IO/revert("Unknown function selector")
      return wrap(0)