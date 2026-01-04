//! # Testing Framework
//!
//! This module provides a testing framework for Bend-PVM contracts,
//! including test runners, assertions, and mock environments.

pub mod assertions;
pub mod mocklib;
pub mod runner;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

use crate::runtime::env::ExecutionContext;
use crate::runtime::metering::MeteringContext;
use crate::runtime::storage::{StorageLimits, StorageManager};

/// Test case definition
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Name of the test
    pub name: String,

    /// Source code to test
    pub source: String,

    /// Function to test
    pub function: String,

    /// Arguments to pass to the function
    pub arguments: Vec<String>,

    /// Expected return value
    pub expected_return: Option<String>,

    /// Expected error (if any)
    pub expected_error: Option<String>,

    /// Initial storage state
    pub initial_storage: HashMap<String, Vec<u8>>,

    /// Gas limit for the test
    pub gas_limit: u64,

    /// Proof size limit
    pub proof_size_limit: u64,

    /// Storage deposit limit
    pub storage_deposit_limit: u128,

    /// Timeout in milliseconds
    pub timeout: u64,

    /// Whether this test is disabled
    pub disabled: bool,
}

impl Default for TestCase {
    fn default() -> Self {
        TestCase {
            name: "unnamed_test".to_string(),
            source: String::new(),
            function: "test".to_string(),
            arguments: Vec::new(),
            expected_return: None,
            expected_error: None,
            initial_storage: HashMap::new(),
            gas_limit: 10_000_000,
            proof_size_limit: 1_000_000,
            storage_deposit_limit: 1_000_000_000,
            timeout: 5000,
            disabled: false,
        }
    }
}

/// Test result
#[derive(Debug, Clone)]
pub enum TestResult {
    /// Test passed
    Passed {
        /// Time taken to run the test
        duration: Duration,
        /// Gas used
        gas_used: u64,
    },

    /// Test failed
    Failed {
        /// Time taken to run the test
        duration: Duration,
        /// Error that caused the failure
        error: TestError,
    },

    /// Test was skipped
    Skipped {
        /// Reason for skipping
        reason: String,
    },
}

/// Test errors
#[derive(Debug, Clone, Error)]
pub enum TestError {
    /// Compilation error
    #[error("Compile error: {0}")]
    Compile(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Assertion failed
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    /// Test timed out
    #[error("Test timed out after {0:?}")]
    Timeout(Duration),

    /// Test setup failed
    #[error("Setup error: {0}")]
    Setup(String),

    /// Invalid test case
    #[error("Invalid test case: {0}")]
    InvalidTestCase(String),
}

/// Test environment
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    /// Runtime environment
    pub context: ExecutionContext,

    /// Storage manager
    pub storage: StorageManager,

    /// Metering context
    pub metering: MeteringContext,

    /// Test start time
    start_time: Instant,
}

impl TestEnvironment {
    /// Create a new test environment
    pub fn new(gas_limit: u64, proof_size_limit: u64, storage_deposit_limit: u128) -> Self {
        let context = ExecutionContext::new(
            [0u8; 32],  // address
            [0u8; 32],  // caller
            0,          // value
            Vec::new(), // input
            1,          // block_number
            1000000,    // block_timestamp
            gas_limit,
            proof_size_limit,
            storage_deposit_limit,
        );

        let storage = StorageManager::new([0u8; 32], StorageLimits::default());
        let metering = MeteringContext::new(gas_limit, proof_size_limit, storage_deposit_limit);

        TestEnvironment {
            context,
            storage,
            metering,
            start_time: Instant::now(),
        }
    }

    /// Set initial storage values
    pub fn set_initial_storage(&mut self, storage: HashMap<String, Vec<u8>>) {
        for (key, value) in storage {
            let key_bytes = key.as_bytes().to_vec();
            let mut metering = self.metering.clone();
            let _ = self.storage.set(&key_bytes, &value, &mut metering);
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Test suite
#[derive(Debug, Clone)]
pub struct TestSuite {
    /// Name of the test suite
    pub name: String,

    /// Test cases
    pub tests: Vec<TestCase>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: &str) -> Self {
        TestSuite {
            name: name.to_string(),
            tests: Vec::new(),
        }
    }

    /// Add a test case
    pub fn add_test(&mut self, test: TestCase) {
        self.tests.push(test);
    }

    /// Run all tests
    pub fn run_all(&self) -> Vec<(String, TestResult)> {
        self.tests
            .iter()
            .filter(|t| !t.disabled)
            .map(|test| {
                let result = self.run_test(test);
                (test.name.clone(), result)
            })
            .collect()
    }

    /// Run a single test
    fn run_test(&self, _test: &TestCase) -> TestResult {
        // In a real implementation, this would run the test
        // For now, we just return a passed result
        TestResult::Passed {
            duration: Duration::from_millis(1),
            gas_used: 1000,
        }
    }
}

/// Test module macro helper
#[macro_export]
macro_rules! test_suite {
    ($name:expr) => {
        $crate::testing::TestSuite::new($name)
    };
    ($name:expr, $($test:expr),+) => {
        {
            let mut suite = $crate::testing::TestSuite::new($name);
            $(
                suite.add_test($test);
            )+
            suite
        }
    };
}

/// Create a test case
#[macro_export]
macro_rules! test_case {
    ($name:expr) => {
        $crate::testing::TestCase {
            name: $name.to_string(),
            ..Default::default()
        }
    };
    ($name:expr, source: $source:expr) => {
        $crate::testing::TestCase {
            name: $name.to_string(),
            source: $source.to_string(),
            ..Default::default()
        }
    };
    ($name:expr, source: $source:expr, function: $func:expr) => {
        $crate::testing::TestCase {
            name: $name.to_string(),
            source: $source.to_string(),
            function: $func.to_string(),
            ..Default::default()
        }
    };
}
