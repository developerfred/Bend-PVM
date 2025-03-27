/// This module defines the host functions that the PolkaVM runtime provides to
/// contract executables. These functions allow the contract to interact with the
/// blockchain environment.

// Standard host functions provided to all contracts
#[repr(u32)]
pub enum HostFunction {
    // Storage operations
    StorageGet = 0,
    StorageSet = 1,
    StorageClear = 2,
    
    // Context information
    GetCaller = 10,
    GetCallValue = 11,
    GetBlockNumber = 12,
    GetBlockTimestamp = 13,
    
    // Contract interactions
    Call = 20,
    StaticCall = 21,
    DelegateCall = 22,
    Create = 23,
    Create2 = 24,
    
    // Crypto operations
    Keccak256 = 30,
    Blake2b256 = 31,
    Sha256 = 32,
    Ripemd160 = 33,
    EcdsaRecover = 34,
    
    // Debugging and logging
    Log = 40,
    Debug = 41,
    
    // Memory operations (for handling dynamic memory)
    MemoryAlloc = 50,
    MemoryFree = 51,
    
    // Misc
    Abort = 60,
    Return = 61,
    Revert = 62,
}

/// Generates bindings for host functions
/// 
/// These bindings allow the generated code to call the host functions provided
/// by the PolkaVM runtime.
pub fn generate_host_bindings() -> String {
    let mut bindings = String::new();
    
    // Add header
    bindings.push_str("# PolkaVM host function bindings\n\n");
    
    // Add storage operations
    bindings.push_str(".macro storage_get key_ptr key_len value_ptr value_len_ptr\n");
    bindings.push_str("    li a7, 0  # StorageGet\n");
    bindings.push_str("    mv a0, \\key_ptr\n");
    bindings.push_str("    mv a1, \\key_len\n");
    bindings.push_str("    mv a2, \\value_ptr\n");
    bindings.push_str("    mv a3, \\value_len_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro storage_set key_ptr key_len value_ptr value_len\n");
    bindings.push_str("    li a7, 1  # StorageSet\n");
    bindings.push_str("    mv a0, \\key_ptr\n");
    bindings.push_str("    mv a1, \\key_len\n");
    bindings.push_str("    mv a2, \\value_ptr\n");
    bindings.push_str("    mv a3, \\value_len\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro storage_clear key_ptr key_len\n");
    bindings.push_str("    li a7, 2  # StorageClear\n");
    bindings.push_str("    mv a0, \\key_ptr\n");
    bindings.push_str("    mv a1, \\key_len\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    // Add context information
    bindings.push_str(".macro get_caller result_ptr\n");
    bindings.push_str("    li a7, 10  # GetCaller\n");
    bindings.push_str("    mv a0, \\result_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro get_call_value result_ptr\n");
    bindings.push_str("    li a7, 11  # GetCallValue\n");
    bindings.push_str("    mv a0, \\result_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro get_block_number result_ptr\n");
    bindings.push_str("    li a7, 12  # GetBlockNumber\n");
    bindings.push_str("    mv a0, \\result_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro get_block_timestamp result_ptr\n");
    bindings.push_str("    li a7, 13  # GetBlockTimestamp\n");
    bindings.push_str("    mv a0, \\result_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    // Add contract interactions
    bindings.push_str(".macro call address_ptr value_ptr gas input_ptr input_len output_ptr output_len_ptr\n");
    bindings.push_str("    li a7, 20  # Call\n");
    bindings.push_str("    mv a0, \\address_ptr\n");
    bindings.push_str("    mv a1, \\value_ptr\n");
    bindings.push_str("    mv a2, \\gas\n");
    bindings.push_str("    mv a3, \\input_ptr\n");
    bindings.push_str("    mv a4, \\input_len\n");
    bindings.push_str("    mv a5, \\output_ptr\n");
    bindings.push_str("    mv a6, \\output_len_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro static_call address_ptr gas input_ptr input_len output_ptr output_len_ptr\n");
    bindings.push_str("    li a7, 21  # StaticCall\n");
    bindings.push_str("    mv a0, \\address_ptr\n");
    bindings.push_str("    mv a1, \\gas\n");
    bindings.push_str("    mv a2, \\input_ptr\n");
    bindings.push_str("    mv a3, \\input_len\n");
    bindings.push_str("    mv a4, \\output_ptr\n");
    bindings.push_str("    mv a5, \\output_len_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    // Add crypto operations
    bindings.push_str(".macro keccak256 input_ptr input_len output_ptr\n");
    bindings.push_str("    li a7, 30  # Keccak256\n");
    bindings.push_str("    mv a0, \\input_ptr\n");
    bindings.push_str("    mv a1, \\input_len\n");
    bindings.push_str("    mv a2, \\output_ptr\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    // Add logging and debugging
    bindings.push_str(".macro log topics_ptr topics_count data_ptr data_len\n");
    bindings.push_str("    li a7, 40  # Log\n");
    bindings.push_str("    mv a0, \\topics_ptr\n");
    bindings.push_str("    mv a1, \\topics_count\n");
    bindings.push_str("    mv a2, \\data_ptr\n");
    bindings.push_str("    mv a3, \\data_len\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro debug data_ptr data_len\n");
    bindings.push_str("    li a7, 41  # Debug\n");
    bindings.push_str("    mv a0, \\data_ptr\n");
    bindings.push_str("    mv a1, \\data_len\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    // Add result handling
    bindings.push_str(".macro finish result_ptr result_len\n");
    bindings.push_str("    li a7, 61  # Return\n");
    bindings.push_str("    mv a0, \\result_ptr\n");
    bindings.push_str("    mv a1, \\result_len\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings.push_str(".macro revert result_ptr result_len\n");
    bindings.push_str("    li a7, 62  # Revert\n");
    bindings.push_str("    mv a0, \\result_ptr\n");
    bindings.push_str("    mv a1, \\result_len\n");
    bindings.push_str("    ecall\n");
    bindings.push_str(".endm\n\n");
    
    bindings
}

/// Generate a helper function for ABI encoding function calls
pub fn generate_abi_helpers() -> String {
    let mut helpers = String::new();
    
    helpers.push_str("# ABI encoding helpers\n\n");
    
    // Add helper for encoding a function call
    helpers.push_str(".section .text\n");
    helpers.push_str(".global encode_function_call\n");
    helpers.push_str("# Encodes a function call with the given selector and arguments\n");
    helpers.push_str("# Arguments:\n");
    helpers.push_str("#   a0: pointer to function selector (4 bytes)\n");
    helpers.push_str("#   a1: pointer to arguments buffer\n");
    helpers.push_str("#   a2: length of arguments buffer\n");
    helpers.push_str("#   a3: pointer to output buffer\n");
    helpers.push_str("encode_function_call:\n");
    helpers.push_str("    # Save return address\n");
    helpers.push_str("    addi sp, sp, -4\n");
    helpers.push_str("    sw ra, 0(sp)\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Copy selector to output buffer\n");
    helpers.push_str("    lw t0, 0(a0)\n");
    helpers.push_str("    sw t0, 0(a3)\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Copy arguments to output buffer\n");
    helpers.push_str("    addi a3, a3, 4  # Offset by selector size\n");
    helpers.push_str("    beqz a2, .Lend  # If no arguments, skip loop\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Copy loop\n");
    helpers.push_str("    mv t0, a1  # Source pointer\n");
    helpers.push_str("    mv t1, a3  # Destination pointer\n");
    helpers.push_str("    mv t2, a2  # Remaining bytes\n");
    helpers.push_str(".Lcopy_loop:\n");
    helpers.push_str("    lb t3, 0(t0)\n");
    helpers.push_str("    sb t3, 0(t1)\n");
    helpers.push_str("    addi t0, t0, 1\n");
    helpers.push_str("    addi t1, t1, 1\n");
    helpers.push_str("    addi t2, t2, -1\n");
    helpers.push_str("    bnez t2, .Lcopy_loop\n");
    helpers.push_str("    \n");
    helpers.push_str(".Lend:\n");
    helpers.push_str("    # Return total size (selector + arguments)\n");
    helpers.push_str("    addi a0, a2, 4\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Restore return address and return\n");
    helpers.push_str("    lw ra, 0(sp)\n");
    helpers.push_str("    addi sp, sp, 4\n");
    helpers.push_str("    ret\n\n");
    
    // Add helper for decoding function return values
    helpers.push_str(".global decode_function_return\n");
    helpers.push_str("# Decodes a function return value\n");
    helpers.push_str("# Arguments:\n");
    helpers.push_str("#   a0: pointer to return data buffer\n");
    helpers.push_str("#   a1: length of return data buffer\n");
    helpers.push_str("#   a2: pointer to output buffer\n");
    helpers.push_str("decode_function_return:\n");
    helpers.push_str("    # Save return address\n");
    helpers.push_str("    addi sp, sp, -4\n");
    helpers.push_str("    sw ra, 0(sp)\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Check if return data is empty\n");
    helpers.push_str("    beqz a1, .Lend_decode\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Copy return data to output buffer\n");
    helpers.push_str("    mv t0, a0  # Source pointer\n");
    helpers.push_str("    mv t1, a2  # Destination pointer\n");
    helpers.push_str("    mv t2, a1  # Remaining bytes\n");
    helpers.push_str(".Lcopy_loop_decode:\n");
    helpers.push_str("    lb t3, 0(t0)\n");
    helpers.push_str("    sb t3, 0(t1)\n");
    helpers.push_str("    addi t0, t0, 1\n");
    helpers.push_str("    addi t1, t1, 1\n");
    helpers.push_str("    addi t2, t2, -1\n");
    helpers.push_str("    bnez t2, .Lcopy_loop_decode\n");
    helpers.push_str("    \n");
    helpers.push_str(".Lend_decode:\n");
    helpers.push_str("    # Return size copied\n");
    helpers.push_str("    mv a0, a1\n");
    helpers.push_str("    \n");
    helpers.push_str("    # Restore return address and return\n");
    helpers.push_str("    lw ra, 0(sp)\n");
    helpers.push_str("    addi sp, sp, 4\n");
    helpers.push_str("    ret\n");
    
    helpers
}

/// Generate prelude for all contracts
pub fn generate_prelude() -> String {
    let mut prelude = String::new();
    
    // Add host function bindings
    prelude.push_str(&generate_host_bindings());
    
    // Add ABI helpers
    prelude.push_str(&generate_abi_helpers());
    
    // Add common utility functions
    prelude.push_str("# Common utility functions\n\n");
    
    // Memory allocation wrapper (simplified)
    prelude.push_str(".section .text\n");
    prelude.push_str(".global malloc\n");
    prelude.push_str("# Allocates memory with the given size\n");
    prelude.push_str("# Arguments:\n");
    prelude.push_str("#   a0: size to allocate\n");
    prelude.push_str("# Returns:\n");
    prelude.push_str("#   a0: pointer to allocated memory or 0 if failed\n");
    prelude.push_str("malloc:\n");
    prelude.push_str("    li a7, 50  # MemoryAlloc\n");
    prelude.push_str("    ecall\n");
    prelude.push_str("    ret\n\n");
    
    prelude.push_str(".global free\n");
    prelude.push_str("# Frees memory at the given pointer\n");
    prelude.push_str("# Arguments:\n");
    prelude.push_str("#   a0: pointer to memory to free\n");
    prelude.push_str("free:\n");
    prelude.push_str("    li a7, 51  # MemoryFree\n");
    prelude.push_str("    ecall\n");
    prelude.push_str("    ret\n\n");
    
    prelude
}