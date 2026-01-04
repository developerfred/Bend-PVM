use bend_pvm::stdlib::init_stdlib;

#[test]
fn test_stdlib_initialization() {
    let stdlib = init_stdlib();
    let modules = stdlib.available_modules();
    
    assert!(modules.contains(&"String".to_string()));
    assert!(modules.contains(&"Math".to_string()));
    assert!(modules.contains(&"IO".to_string()));
    assert!(modules.contains(&"List".to_string()));
    assert!(modules.contains(&"Option".to_string()));
    assert!(modules.contains(&"Result".to_string()));
    assert!(modules.contains(&"Crypto".to_string()));
}

#[test]
fn test_load_math_module() {
    let stdlib = init_stdlib();
    let math = stdlib.load_module("Math").unwrap();
    
    // Check if some expected functions are present
    let has_sin = math.definitions.iter().any(|d| {
        match d {
            bend_pvm::compiler::parser::ast::Definition::FunctionDef { name, .. } => name == "Math/sin",
            _ => false,
        }
    });
    
    assert!(has_sin);
}

#[test]
fn test_load_crypto_module() {
    let stdlib = init_stdlib();
    let crypto = stdlib.load_module("Crypto").unwrap();
    
    let has_keccak = crypto.definitions.iter().any(|d| {
        match d {
            bend_pvm::compiler::parser::ast::Definition::FunctionDef { name, .. } => name == "Crypto/keccak256",
            _ => false,
        }
    });
    
    assert!(has_keccak);
}
