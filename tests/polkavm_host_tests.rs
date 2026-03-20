//! Tests for the PolkaVM host module

use bend_pvm::compiler::polkavm::host::*;

#[cfg(test)]
mod tests {
    use super::*;

    mod host_function_tests {
        use super::*;

        #[test]
        fn test_storage_operations() {
            assert_eq!(HostFunction::StorageGet as u32, 0);
            assert_eq!(HostFunction::StorageSet as u32, 1);
            assert_eq!(HostFunction::StorageClear as u32, 2);
        }

        #[test]
        fn test_context_operations() {
            assert_eq!(HostFunction::GetCaller as u32, 10);
            assert_eq!(HostFunction::GetCallValue as u32, 11);
            assert_eq!(HostFunction::GetBlockNumber as u32, 12);
            assert_eq!(HostFunction::GetBlockTimestamp as u32, 13);
        }

        #[test]
        fn test_contract_operations() {
            assert_eq!(HostFunction::Call as u32, 20);
            assert_eq!(HostFunction::StaticCall as u32, 21);
            assert_eq!(HostFunction::DelegateCall as u32, 22);
            assert_eq!(HostFunction::Create as u32, 23);
            assert_eq!(HostFunction::Create2 as u32, 24);
        }

        #[test]
        fn test_crypto_operations() {
            assert_eq!(HostFunction::Keccak256 as u32, 30);
            assert_eq!(HostFunction::Blake2b256 as u32, 31);
            assert_eq!(HostFunction::Sha256 as u32, 32);
            assert_eq!(HostFunction::Ripemd160 as u32, 33);
            assert_eq!(HostFunction::EcdsaRecover as u32, 34);
        }

        #[test]
        fn test_debug_operations() {
            assert_eq!(HostFunction::Log as u32, 40);
            assert_eq!(HostFunction::Debug as u32, 41);
        }

        #[test]
        fn test_memory_operations() {
            assert_eq!(HostFunction::MemoryAlloc as u32, 50);
            assert_eq!(HostFunction::MemoryFree as u32, 51);
        }

        #[test]
        fn test_misc_operations() {
            assert_eq!(HostFunction::Abort as u32, 60);
            assert_eq!(HostFunction::Return as u32, 61);
            assert_eq!(HostFunction::Revert as u32, 62);
        }
    }

    mod host_bindings_tests {
        use super::*;

        #[test]
        fn test_generate_host_bindings_returns_non_empty() {
            let bindings = generate_host_bindings();
            assert!(!bindings.is_empty());
        }

        #[test]
        fn test_generate_host_bindings_contains_header() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains("PolkaVM host function bindings"));
        }

        #[test]
        fn test_generate_host_bindings_contains_storage_macros() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains("storage_get"));
            assert!(bindings.contains("storage_set"));
            assert!(bindings.contains("storage_clear"));
        }

        #[test]
        fn test_generate_host_bindings_contains_context_macros() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains("get_caller"));
            assert!(bindings.contains("get_call_value"));
            assert!(bindings.contains("get_block_number"));
            assert!(bindings.contains("get_block_timestamp"));
        }

        #[test]
        fn test_generate_host_bindings_contains_call_macros() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains(".macro call"));
            assert!(bindings.contains(".macro static_call"));
        }

        #[test]
        fn test_generate_host_bindings_contains_crypto_macros() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains("keccak256"));
        }

        #[test]
        fn test_generate_host_bindings_contains_log_macros() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains(".macro log"));
            assert!(bindings.contains(".macro debug"));
        }

        #[test]
        fn test_generate_host_bindings_contains_result_macros() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains(".macro finish"));
            assert!(bindings.contains(".macro revert"));
        }

        #[test]
        fn test_generate_host_bindings_uses_ecall() {
            let bindings = generate_host_bindings();
            assert!(bindings.contains("ecall"));
        }
    }

    mod abi_helpers_tests {
        use super::*;

        #[test]
        fn test_generate_abi_helpers_returns_non_empty() {
            let helpers = generate_abi_helpers();
            assert!(!helpers.is_empty());
        }

        #[test]
        fn test_generate_abi_helpers_contains_encode_function_call() {
            let helpers = generate_abi_helpers();
            assert!(helpers.contains("encode_function_call"));
        }

        #[test]
        fn test_generate_abi_helpers_contains_decode_function_return() {
            let helpers = generate_abi_helpers();
            assert!(helpers.contains("decode_function_return"));
        }

        #[test]
        fn test_generate_abi_helpers_contains_helper_section() {
            let helpers = generate_abi_helpers();
            assert!(helpers.contains(".section .text"));
        }
    }

    mod prelude_tests {
        use super::*;

        #[test]
        fn test_generate_prelude_returns_non_empty() {
            let prelude = generate_prelude();
            assert!(!prelude.is_empty());
        }

        #[test]
        fn test_generate_prelude_contains_host_bindings() {
            let prelude = generate_prelude();
            assert!(prelude.contains("PolkaVM host function bindings"));
        }

        #[test]
        fn test_generate_prelude_contains_abi_helpers() {
            let prelude = generate_prelude();
            assert!(prelude.contains("encode_function_call"));
        }

        #[test]
        fn test_generate_prelude_contains_malloc() {
            let prelude = generate_prelude();
            assert!(prelude.contains(".global malloc"));
            assert!(prelude.contains("MemoryAlloc"));
        }

        #[test]
        fn test_generate_prelude_contains_free() {
            let prelude = generate_prelude();
            assert!(prelude.contains(".global free"));
            assert!(prelude.contains("MemoryFree"));
        }

        #[test]
        fn test_generate_prelude_contains_utility_section() {
            let prelude = generate_prelude();
            assert!(prelude.contains("Common utility functions"));
        }
    }
}
