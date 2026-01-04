pub mod core;
pub mod crypto;
pub mod math;

use self::core::StdlibCore;
use self::crypto::CryptoFunctions;
use self::math::{BigIntMath, BitwiseMath, MathFunctions, Percentage, Random, SafeMath};

/// Initialize the standard library
pub fn init_stdlib() -> StdlibCore {
    StdlibCore::new()
}

/// Get the standard library core
pub fn get_stdlib() -> StdlibCore {
    StdlibCore::new()
}

/// Get the math functions
pub fn get_math_functions() -> MathFunctions {
    MathFunctions::new()
}

/// Get the crypto functions
pub fn get_crypto_functions() -> CryptoFunctions {
    CryptoFunctions::new()
}

/// Get safe math operations
pub fn get_safe_math() -> SafeMath {
    SafeMath
}

/// Get big integer operations
pub fn get_bigint_math() -> BigIntMath {
    BigIntMath
}

/// Get bitwise operations
pub fn get_bitwise_math() -> BitwiseMath {
    BitwiseMath
}

/// Get random number generator
pub fn get_random() -> Random {
    Random
}

/// Get percentage calculations
pub fn get_percentage() -> Percentage {
    Percentage
}
