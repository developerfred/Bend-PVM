#{
  ERC-20 Token Implementation in Bend-PVM
  
  This contract implements the ERC-20 token standard with the following features:
  - Token transfers
  - Allowance-based transfers
  - Minting (restricted to the owner)
  - Burning
  - Event emission
}#

# Define result types
type Result:
  Ok { value: u24 }
  Err { reason: String }

# Storage keys
def total_supply_key() -> String:
  return "total_supply"

def balance_key(owner: String) -> String:
  return "balance:" + owner

def allowance_key(owner: String, spender: String) -> String:
  return "allowance:" + owner + ":" + spender

def owner_key() -> String:
  return "owner"

def name_key() -> String:
  return "name"

def symbol_key() -> String:
  return "symbol"

def decimals_key() -> String:
  return "decimals"

# Storage functions
def get_storage(key: String) -> u24:
  with IO:
    value = IO/storage_get(key)
    match value:
      case Result/Ok:
        return String/to_u24(value.value)
      case Result/Err:
        return 0

def set_storage(key: String, value: u24) -> None:
  with IO:
    * <- IO/storage_set(key, String/from_u24(value))

# Token information functions
def name() -> String:
  with IO:
    value = IO/storage_get(name_key())
    match value:
      case Result/Ok:
        return value.value
      case Result/Err:
        return ""

def symbol() -> String:
  with IO:
    value = IO/storage_get(symbol_key())
    match value:
      case Result/Ok:
        return value.value
      case Result/Err:
        return ""

def decimals() -> u24:
  return get_storage(decimals_key())

def total_supply() -> u24:
  return get_storage(total_supply_key())

def balance_of(owner: String) -> u24:
  return get_storage(balance_key(owner))

def allowance(owner: String, spender: String) -> u24:
  return get_storage(allowance_key(owner, spender))

# Check if caller is the owner
def is_owner() -> u24:
  with IO:
    caller <- IO/get_caller()
    owner = IO/storage_get(owner_key())
    
    match owner:
      case Result/Ok:
        if owner.value == caller:
          return 1
        else:
          return 0
      case Result/Err:
        return 0

# Transfer tokens
def transfer(to: String, amount: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    from_balance = balance_of(caller)
    
    if from_balance < amount:
      return Result/Err { reason: "Insufficient balance" }
    
    # Update balances
    set_storage(balance_key(caller), from_balance - amount)
    to_balance = balance_of(to)
    set_storage(balance_key(to), to_balance + amount)
    
    # Emit Transfer event
    * <- IO/emit_event("Transfer", [caller, to, String/from_u24(amount)])
    
    return Result/Ok { value: 1 }

# Approve spender to transfer tokens
def approve(spender: String, amount: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    
    # Set allowance
    set_storage(allowance_key(caller, spender), amount)
    
    # Emit Approval event
    * <- IO/emit_event("Approval", [caller, spender, String/from_u24(amount)])
    
    return Result/Ok { value: 1 }

# Transfer tokens from one account to another (with allowance)
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
    set_storage(allowance_key(from, caller), allowed_amount - amount)
    
    # Update balances
    set_storage(balance_key(from), from_balance - amount)
    to_balance = balance_of(to)
    set_storage(balance_key(to), to_balance + amount)
    
    # Emit Transfer event
    * <- IO/emit_event("Transfer", [from, to, String/from_u24(amount)])
    
    return Result/Ok { value: 1 }

# Mint new tokens (only owner)
def mint(to: String, amount: u24) -> Result:
  with IO:
    # Check if caller is owner
    if is_owner() == 0:
      return Result/Err { reason: "Only owner can mint tokens" }
    
    # Update total supply
    supply = total_supply()
    set_storage(total_supply_key(), supply + amount)
    
    # Update balance
    balance = balance_of(to)
    set_storage(balance_key(to), balance + amount)
    
    # Emit Transfer event (from zero address)
    caller <- IO/get_caller()
    * <- IO/emit_event("Transfer", ["0x0000000000000000000000000000000000000000", to, String/from_u24(amount)])
    
    return Result/Ok { value: 1 }

# Burn tokens
def burn(amount: u24) -> Result:
  with IO:
    caller <- IO/get_caller()
    balance = balance_of(caller)
    
    if balance < amount:
      return Result/Err { reason: "Insufficient balance" }
    
    # Update balance
    set_storage(balance_key(caller), balance - amount)
    
    # Update total supply
    supply = total_supply()
    set_storage(total_supply_key(), supply - amount)
    
    # Emit Transfer event (to zero address)
    * <- IO/emit_event("Transfer", [caller, "0x0000000000000000000000000000000000000000", String/from_u24(amount)])
    
    return Result/Ok { value: 1 }

# Initialize the token (called only once during deployment)
def initialize(token_name: String, token_symbol: String, token_decimals: u24, initial_supply: u24) -> Result:
  with IO:
    # Check if already initialized
    supply = total_supply()
    if supply > 0:
      return Result/Err { reason: "Already initialized" }
    
    # Set token information
    * <- IO/storage_set(name_key(), token_name)
    * <- IO/storage_set(symbol_key(), token_symbol)
    set_storage(decimals_key(), token_decimals)
    
    # Set owner
    caller <- IO/get_caller()
    * <- IO/storage_set(owner_key(), caller)
    
    # Mint initial supply to deployer
    mint_result = mint(caller, initial_supply)
    match mint_result:
      case Result/Ok:
        return Result/Ok { value: 1 }
      case Result/Err:
        return mint_result
    
    return Result/Ok { value: 1 }

# Entry point
def main() -> u24:
  with IO:
    # Get calldata
    calldata <- IO/get_calldata()
    
    # Extract function selector (first 4 bytes)
    selector = String/slice(calldata, 0, 4)
    
    # Dispatch to the appropriate function
    if selector == "init": # initialize(string,string,uint8,uint256)
      # Parse parameters
      name_offset = String/to_u24(String/slice(calldata, 4, 32))
      symbol_offset = String/to_u24(String/slice(calldata, 36, 32))
      decimals = String/to_u24(String/slice(calldata, 68, 32))
      initial_supply = String/to_u24(String/slice(calldata, 100, 32))
      
      # Extract strings
      name_length = String/to_u24(String/slice(calldata, name_offset, 32))
      name = String/slice(calldata, name_offset + 32, name_length)
      
      symbol_length = String/to_u24(String/slice(calldata, symbol_offset, 32))
      symbol = String/slice(calldata, symbol_offset + 32, symbol_length)
      
      # Initialize
      result = initialize(name, symbol, decimals, initial_supply)
      match result:
        case Result/Ok:
          return 1
        case Result/Err:
          * <- IO/revert(result.reason)
          return 0
    
    elif selector == "name": # name()
      token_name = name()
      * <- IO/return_value(token_name)
      return 1
    
    elif selector == "symb": # symbol()
      token_symbol = symbol()
      * <- IO/return_value(token_symbol)
      return 1
    
    elif selector == "deci": # decimals()
      token_decimals = decimals()
      * <- IO/return_value(String/from_u24(token_decimals))
      return 1
    
    elif selector == "ttsp": # totalSupply()
      supply = total_supply()
      * <- IO/return_value(String/from_u24(supply))
      return 1
    
    elif selector == "balc": # balanceOf(address)
      owner = String/slice(calldata, 4, 32)
      balance = balance_of(owner)
      * <- IO/return_value(String/from_u24(balance))
      return 1
    
    elif selector == "tran": # transfer(address,uint256)
      to = String/slice(calldata, 4, 32)
      amount = String/to_u24(String/slice(calldata, 36, 32))
      
      result = transfer(to, amount)
      match result:
        case Result/Ok:
          * <- IO/return_value(String/from_u24(1))
          return 1
        case Result/Err:
          * <- IO/revert(result.reason)
          return 0
    
    elif selector == "aprv": # approve(address,uint256)
      spender = String/slice(calldata, 4, 32)
      amount = String/to_u24(String/slice(calldata, 36, 32))
      
      result = approve(spender, amount)
      match result:
        case Result/Ok:
          * <- IO/return_value(String/from_u24(1))
          return 1
        case Result/Err:
          * <- IO/revert(result.reason)
          return 0
    
    elif selector == "alow": # allowance(address,address)
      owner = String/slice(calldata, 4, 32)
      spender = String/slice(calldata, 36, 32)
      
      amount = allowance(owner, spender)
      * <- IO/return_value(String/from_u24(amount))
      return 1
    
    elif selector == "trfm": # transferFrom(address,address,uint256)
      from = String/slice(calldata, 4, 32)
      to = String/slice(calldata, 36, 32)
      amount = String/to_u24(String/slice(calldata, 68, 32))
      
      result = transfer_from(from, to, amount)
      match result:
        case Result/Ok:
          * <- IO/return_value(String/from_u24(1))
          return 1
        case Result/Err:
          * <- IO/revert(result.reason)
          return 0
    
    elif selector == "mint": # mint(address,uint256)
      to = String/slice(calldata, 4, 32)
      amount = String/to_u24(String/slice(calldata, 36, 32))
      
      result = mint(to, amount)
      match result:
        case Result/Ok:
          * <- IO/return_value(String/from_u24(1))
          return 1
        case Result/Err:
          * <- IO/revert(result.reason)
          return 0
    
    elif selector == "burn": # burn(uint256)
      amount = String/to_u24(String/slice(calldata, 4, 32))
      
      result = burn(amount)
      match result:
        case Result/Ok:
          * <- IO/return_value(String/from_u24(1))
          return 1
        case Result/Err:
          * <- IO/revert(result.reason)
          return 0
    
    else:
      * <- IO/revert("Unknown function selector")
      return 0