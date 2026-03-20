//! Tests for the PolkaVM ABI module

use bend_pvm::compiler::polkavm::abi::*;

#[cfg(test)]
mod tests {
    use super::*;

    mod contract_abi_tests {
        use super::*;

        #[test]
        fn test_contract_abi_creation() {
            let abi = ContractABI {
                name: "TestContract".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![],
                events: vec![],
                errors: vec![],
                state_variables: vec![],
                types: vec![],
            };
            assert_eq!(abi.name, "TestContract");
            assert_eq!(abi.version, "1.0.0");
            assert!(abi.methods.is_empty());
        }

        #[test]
        fn test_contract_abi_with_methods() {
            let method = MethodABI {
                name: "testMethod".to_string(),
                selector: "0x12345678".to_string(),
                type_: MethodType::Function,
                inputs: vec![],
                outputs: vec![],
                state_mutability: StateMutability::NonPayable,
                payable: false,
            };

            let abi = ContractABI {
                name: "TestContract".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![method],
                events: vec![],
                errors: vec![],
                state_variables: vec![],
                types: vec![],
            };

            assert_eq!(abi.methods.len(), 1);
            assert_eq!(abi.methods[0].name, "testMethod");
        }
    }

    mod method_abi_tests {
        use super::*;

        #[test]
        fn test_method_abi_with_inputs_and_outputs() {
            let input = ParameterABI {
                name: "value".to_string(),
                type_: "uint256".to_string(),
                components: None,
                indexed: None,
            };

            let output = ParameterABI {
                name: "".to_string(),
                type_: "bool".to_string(),
                components: None,
                indexed: None,
            };

            let method = MethodABI {
                name: "setValue".to_string(),
                selector: "0xabcdef01".to_string(),
                type_: MethodType::Function,
                inputs: vec![input],
                outputs: vec![output],
                state_mutability: StateMutability::NonPayable,
                payable: false,
            };

            assert_eq!(method.inputs.len(), 1);
            assert_eq!(method.outputs.len(), 1);
            assert_eq!(method.inputs[0].name, "value");
        }

        #[test]
        fn test_selector_format() {
            let method = MethodABI {
                name: "test".to_string(),
                selector: "0xabcdef12".to_string(),
                type_: MethodType::Function,
                inputs: vec![],
                outputs: vec![],
                state_mutability: StateMutability::View,
                payable: false,
            };

            assert!(method.selector.starts_with("0x"));
            assert_eq!(method.selector.len(), 10); // "0x" + 8 hex chars
        }
    }

    mod parameter_abi_tests {
        use super::*;

        #[test]
        fn test_parameter_abi_simple() {
            let param = ParameterABI {
                name: "amount".to_string(),
                type_: "u256".to_string(),
                components: None,
                indexed: None,
            };

            assert_eq!(param.name, "amount");
            assert_eq!(param.type_, "u256");
            assert!(param.components.is_none());
        }

        #[test]
        fn test_parameter_abi_with_components() {
            let inner = ParameterABI {
                name: "inner".to_string(),
                type_: "uint256".to_string(),
                components: None,
                indexed: None,
            };

            let param = ParameterABI {
                name: "data".to_string(),
                type_: "tuple".to_string(),
                components: Some(vec![inner]),
                indexed: Some(true),
            };

            assert!(param.components.is_some());
            assert_eq!(param.components.as_ref().unwrap().len(), 1);
            assert_eq!(param.indexed, Some(true));
        }
    }

    mod method_type_tests {
        use super::*;

        #[test]
        fn test_method_type_variants() {
            assert_eq!(MethodType::Function as u32, 0);
            assert_eq!(MethodType::Constructor as u32, 1);
            assert_eq!(MethodType::Receive as u32, 2);
            assert_eq!(MethodType::Fallback as u32, 3);
        }
    }

    mod state_mutability_tests {
        use super::*;

        #[test]
        fn test_state_mutability_variants() {
            assert_eq!(StateMutability::Pure as u32, 0);
            assert_eq!(StateMutability::View as u32, 1);
            assert_eq!(StateMutability::NonPayable as u32, 2);
            assert_eq!(StateMutability::Payable as u32, 3);
        }
    }

    mod type_kind_tests {
        use super::*;

        #[test]
        fn test_type_kind_variants() {
            assert_eq!(TypeKind::Struct as u32, 0);
            assert_eq!(TypeKind::Enum as u32, 1);
            assert_eq!(TypeKind::Tuple as u32, 2);
        }
    }

    mod event_abi_tests {
        use super::*;

        #[test]
        fn test_event_abi_creation() {
            let event = EventABI {
                name: "Transfer".to_string(),
                inputs: vec![],
                anonymous: false,
            };

            assert_eq!(event.name, "Transfer");
            assert!(!event.anonymous);
        }
    }

    mod error_abi_tests {
        use super::*;

        #[test]
        fn test_error_abi_creation() {
            let error = ErrorABI {
                name: "InsufficientBalance".to_string(),
                inputs: vec![],
            };

            assert_eq!(error.name, "InsufficientBalance");
        }
    }

    mod state_variable_abi_tests {
        use super::*;

        #[test]
        fn test_state_variable_abi_creation() {
            let var = StateVariableABI {
                name: "owner".to_string(),
                type_: "address".to_string(),
                public: true,
                constant: false,
            };

            assert_eq!(var.name, "owner");
            assert!(var.public);
            assert!(!var.constant);
        }
    }

    mod type_abi_tests {
        use super::*;

        #[test]
        fn test_type_abi_creation() {
            let type_abi = TypeABI {
                name: "MyStruct".to_string(),
                kind: TypeKind::Struct,
                components: vec![],
            };

            assert_eq!(type_abi.name, "MyStruct");
            assert!(matches!(type_abi.kind, TypeKind::Struct));
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_serialize_abi_produces_valid_json() {
            let abi = ContractABI {
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![],
                events: vec![],
                errors: vec![],
                state_variables: vec![],
                types: vec![],
            };

            let json = serialize_abi(&abi);
            assert!(json.is_ok());
            let json_str = json.unwrap();
            assert!(json_str.contains("Test"));
            assert!(json_str.contains("1.0.0"));
        }

        #[test]
        fn test_parse_abi_from_json() {
            let json = r#"{
                "name": "TestContract",
                "version": "1.0.0",
                "methods": [],
                "events": [],
                "errors": [],
                "state_variables": [],
                "types": []
            }"#;

            let result = parse_abi(json);
            assert!(result.is_ok());
            let abi = result.unwrap();
            assert_eq!(abi.name, "TestContract");
        }

        #[test]
        fn test_serialize_deserialize_roundtrip() {
            let original = ContractABI {
                name: "MyContract".to_string(),
                version: "2.0.0".to_string(),
                methods: vec![MethodABI {
                    name: "test".to_string(),
                    selector: "0x12345678".to_string(),
                    type_: MethodType::Function,
                    inputs: vec![ParameterABI {
                        name: "x".to_string(),
                        type_: "u256".to_string(),
                        components: None,
                        indexed: None,
                    }],
                    outputs: vec![],
                    state_mutability: StateMutability::View,
                    payable: false,
                }],
                events: vec![],
                errors: vec![],
                state_variables: vec![],
                types: vec![],
            };

            let json = serialize_abi(&original).unwrap();
            let parsed = parse_abi(&json).unwrap();

            assert_eq!(parsed.name, original.name);
            assert_eq!(parsed.version, original.version);
            assert_eq!(parsed.methods.len(), original.methods.len());
        }
    }
}
