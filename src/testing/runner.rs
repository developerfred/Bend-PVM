use std::time::{Duration, Instant};

use crate::compiler::parser::parser::Parser;
use crate::compiler::analyzer::type_checker::TypeChecker;
use crate::compiler::optimizer::passes::OptimizationManager;
use crate::compiler::codegen::risc_v::RiscVCodegen;
use crate::compiler::polkavm::bridge::compile_to_polkavm;
use crate::runtime::env::{Environment, ExecutionContext, ExecutionResult};
use crate::testing::{TestCase, TestError, TestEnvironment};

/// Test runner for running test cases
pub struct TestRunner {
    /// Test environment
    environment: TestEnvironment,
    
    /// Compiled code
    code: Vec<u8>,
    
    /// Test timeout
    timeout: Duration,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        // Create a default test environment
        let environment = TestEnvironment::new(
            10_000_000, // 10M gas
            1_000_000, // 1M proof size
            1_000_000_000, // 1 token
        );
        
        TestRunner {
            environment,
            code: Vec::new(),
            timeout: Duration::from_secs(5),
        }
    }
    
    /// Get a reference to the execution context
    pub fn context(&self) -> &ExecutionContext {
        &self.environment.context
    }
    
    /// Set up the test runner
    pub fn setup(&mut self, test_case: &TestCase) -> Result<(), TestError> {
        // Set timeout
        self.timeout = Duration::from_millis(test_case.timeout);
        
        // Set up test environment
        self.environment = TestEnvironment::new(
            test_case.gas_limit,
            test_case.proof_size_limit,
            test_case.storage_deposit_limit,
        );
        
        // Set initial storage
        self.environment.set_initial_storage(test_case.initial_storage.clone());
        
        // Compile the test code
        self.compile(&test_case.source)?;
        
        Ok(())
    }
    
    /// Compile the test code
    fn compile(&mut self, source: &str) -> Result<(), TestError> {
        // Parse the source
        let mut parser = Parser::new(source);
        let mut program = parser.parse_program()
            .map_err(|e| TestError::Compile(e.to_string()))?;
        
        // Type check the program
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)
            .map_err(|e| TestError::Compile(e.to_string()))?;
        
        // Optimize the program
        let mut optimizer = OptimizationManager::new();
        program = optimizer.optimize(program)
            .map_err(|e| TestError::Compile(e.to_string()))?;
        
        // Generate code
        let mut codegen = RiscVCodegen::new();
        let instructions = codegen.generate(&program)
            .map_err(|e| TestError::Compile(e.to_string()))?;
        
        // Compile to PolkaVM
        let module = compile_to_polkavm(&instructions, None)
            .map_err(|e| TestError::Compile(e.to_string()))?;
        
        // Get the binary
        if let Some(binary) = module.binary {
            self.code = binary;
        } else {
            return Err(TestError::Compile("Failed to generate binary".to_string()));
        }
        
        Ok(())
    }
    
    /// Run the test
    pub fn run(&mut self) -> Result<(), TestError> {
        let start_time = Instant::now();
        
        // Create an environment
        let mut env = Environment::new(self.environment.context.clone());
        
        // Import the initial storage
        for (key, value) in self.environment.storage.entries() {
            env.storage.insert(key, value);
        }
        
        // Run the contract with a timeout
        let result = match env.execute(&self.code) {
            Ok(result) => result,
            Err(err) => return Err(TestError::Runtime(err.to_string())),
        };
        
        // Check for timeout
        if start_time.elapsed() > self.timeout {
            return Err(TestError::Timeout(self.timeout));
        }
        
        // Check the result
        match result {
            ExecutionResult::Success { .. } => {
                // Update the context with gas and storage deposit used
                self.environment.context.gas_used = env.context.gas_used;
                self.environment.context.storage_deposit_used = env.context.storage_deposit_used;
                
                Ok(())
            },
            ExecutionResult::Failure { reason, .. } => {
                Err(TestError::Runtime(reason))
            },
            ExecutionResult::Revert { .. } => {
                Err(TestError::Runtime("Contract execution reverted".to_string()))
            },
        }
    }
}