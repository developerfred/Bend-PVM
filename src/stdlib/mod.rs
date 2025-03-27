pub mod core;
pub mod math;
pub mod crypto;

use self::core::StdlibCore;
use self::math::MathFunctions;
use self::crypto::CryptoFunctions;

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