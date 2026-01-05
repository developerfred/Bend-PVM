//! Advanced Build System for Bend-PVM
//!
//! This module provides a comprehensive build system that orchestrates
//! the entire compilation pipeline with advanced features like incremental
//! compilation, dependency management, and build caching.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::compiler::analyzer::type_checker::TypeChecker;
use crate::compiler::codegen::risc_v::RiscVCodegen;
use crate::compiler::optimizer::passes::{create_default_manager, OptimizationLevel};
use crate::compiler::parser::parser::Parser;
use crate::error::{BendError, BendResult, ErrorReporter};
use crate::security::{SecurityConfig, SecurityManager};

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Source files to compile
    pub source_files: Vec<PathBuf>,

    /// Output directory
    pub output_dir: PathBuf,

    /// Optimization level
    pub optimization_level: OptimizationLevel,

    /// Enable debugging
    pub debug: bool,

    /// Target platform
    pub target: TargetPlatform,

    /// Security level
    pub security_level: SecurityLevel,

    /// Enable incremental compilation
    pub incremental: bool,

    /// Maximum number of parallel jobs
    pub jobs: usize,
}

impl Default for BuildConfig {
    fn default() -> Self {
        BuildConfig {
            source_files: Vec::new(),
            output_dir: PathBuf::from("build"),
            optimization_level: OptimizationLevel::Standard,
            debug: false,
            target: TargetPlatform::PolkaVM,
            security_level: SecurityLevel::Enhanced,
            incremental: true,
            jobs: num_cpus::get(),
        }
    }
}

/// Target platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    /// PolkaVM (primary target)
    PolkaVM,

    /// WebAssembly
    Wasm,

    /// RISC-V assembly
    RiscV,

    /// x86-64 native
    Native,
}

/// Security levels for build
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// No security checks
    None,

    /// Basic security validation
    Basic,

    /// Enhanced security with static analysis
    Enhanced,

    /// Maximum security with fuzzing
    Maximum,
}

/// Build artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// Source file path
    pub source_path: PathBuf,

    /// Output file path
    pub output_path: PathBuf,

    /// Compilation timestamp
    pub timestamp: u64,

    /// File hash for change detection
    pub hash: String,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Build status
    pub status: BuildStatus,
}

/// Build status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    /// Build pending
    Pending,

    /// Currently building
    Building,

    /// Build successful
    Success,

    /// Build failed
    Failed,

    /// Build skipped (cached)
    Skipped,
}

/// Build cache for incremental compilation
#[derive(Debug)]
struct BuildCache {
    /// Cache file path
    cache_path: PathBuf,

    /// Cached artifacts
    artifacts: HashMap<String, BuildArtifact>,
}

impl BuildCache {
    /// Create a new build cache
    fn new(cache_path: PathBuf) -> Self {
        BuildCache {
            cache_path,
            artifacts: HashMap::new(),
        }
    }

    /// Load cache from disk
    fn load(&mut self) -> BendResult<()> {
        if self.cache_path.exists() {
            let data = fs::read_to_string(&self.cache_path)?;
            self.artifacts = serde_json::from_str(&data)?;
        }
        Ok(())
    }

    /// Save cache to disk
    fn save(&self) -> BendResult<()> {
        let data = serde_json::to_string_pretty(&self.artifacts)?;
        fs::create_dir_all(self.cache_path.parent().unwrap())?;
        fs::write(&self.cache_path, data)?;
        Ok(())
    }

    /// Check if a file needs rebuild
    fn needs_rebuild(&self, source_path: &Path, current_hash: &str) -> bool {
        match self
            .artifacts
            .get(&source_path.to_string_lossy().to_string())
        {
            Some(artifact) => artifact.hash != current_hash,
            None => true, // New file
        }
    }

    /// Update artifact in cache
    fn update_artifact(&mut self, artifact: BuildArtifact) {
        self.artifacts
            .insert(artifact.source_path.to_string_lossy().to_string(), artifact);
    }
}

/// Dependency graph for build ordering
#[derive(Debug)]
struct DependencyGraph {
    /// Dependencies for each file
    dependencies: HashMap<String, HashSet<String>>,

    /// Reverse dependencies
    reverse_deps: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    fn new() -> Self {
        DependencyGraph {
            dependencies: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// Add a dependency
    fn add_dependency(&mut self, file: &str, dependency: &str) {
        self.dependencies
            .entry(file.to_string())
            .or_default()
            .insert(dependency.to_string());

        self.reverse_deps
            .entry(dependency.to_string())
            .or_default()
            .insert(file.to_string());
    }

    /// Get build order (topological sort)
    fn build_order(&self, files: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        fn visit(
            file: &str,
            graph: &DependencyGraph,
            result: &mut Vec<String>,
            visited: &mut HashSet<String>,
            visiting: &mut HashSet<String>,
        ) {
            if visited.contains(file) {
                return;
            }

            if visiting.contains(file) {
                // Cycle detected - for now, just continue
                return;
            }

            visiting.insert(file.to_string());

            if let Some(deps) = graph.dependencies.get(file) {
                for dep in deps {
                    visit(dep, graph, result, visited, visiting);
                }
            }

            visiting.remove(file);
            visited.insert(file.to_string());
            result.push(file.to_string());
        }

        for file in files {
            visit(file, self, &mut result, &mut visited, &mut visiting);
        }

        result
    }
}

/// Advanced build system
pub struct BuildSystem {
    /// Build configuration
    config: BuildConfig,

    /// Build cache
    cache: BuildCache,

    /// Dependency graph
    dependencies: DependencyGraph,

    /// Error reporter
    error_reporter: ErrorReporter,

    /// Build statistics
    stats: BuildStats,
}

/// Build statistics
#[derive(Debug, Default, Clone)]
pub struct BuildStats {
    /// Total files processed
    pub files_processed: usize,

    /// Files compiled
    pub files_compiled: usize,

    /// Files skipped (cached)
    pub files_skipped: usize,

    /// Total compilation time (ms)
    pub total_time_ms: u128,

    /// Peak memory usage (bytes)
    pub peak_memory: usize,

    /// Errors encountered
    pub errors: usize,

    /// Warnings encountered
    pub warnings: usize,
}

impl BuildSystem {
    /// Create a new build system
    pub fn new(config: BuildConfig) -> BendResult<Self> {
        let cache_path = config.output_dir.join("build_cache.json");
        let mut cache = BuildCache::new(cache_path);

        if config.incremental {
            cache.load().unwrap_or(());
        }

        Ok(BuildSystem {
            config,
            cache,
            dependencies: DependencyGraph::new(),
            error_reporter: ErrorReporter::new(),
            stats: BuildStats::default(),
        })
    }

    /// Build all source files
    pub fn build(&mut self) -> BendResult<BuildResult> {
        let start_time = std::time::Instant::now();

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Collect all source files
        let source_files = self.collect_source_files()?;

        // Analyze dependencies
        self.analyze_dependencies(&source_files)?;

        // Determine build order
        let build_order = self.dependencies.build_order(&source_files);

        // Build files in parallel
        let results = self.build_parallel(build_order)?;

        // Update cache
        if self.config.incremental {
            self.cache.save()?;
        }

        // Update statistics
        self.stats.total_time_ms = start_time.elapsed().as_millis();

        Ok(BuildResult {
            artifacts: results,
            stats: self.stats.clone(),
            success: self.error_reporter.error_count() == 0,
        })
    }

    /// Collect all source files to build
    fn collect_source_files(&self) -> BendResult<Vec<String>> {
        let mut files = Vec::new();

        for source_file in &self.config.source_files {
            if source_file.is_dir() {
                self.collect_from_directory(source_file, &mut files)?;
            } else if source_file.extension().is_some_and(|ext| ext == "bend") {
                files.push(source_file.to_string_lossy().to_string());
            }
        }

        Ok(files)
    }

    /// Collect all Bend source files from a directory
    #[allow(clippy::only_used_in_recursion)]
    fn collect_from_directory(&self, dir: &Path, files: &mut Vec<String>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.collect_from_directory(&path, files)?;
            } else if path.extension().is_some_and(|ext| ext == "bend") {
                files.push(path.to_string_lossy().to_string());
            }
        }
        Ok(())
    }

    /// Analyze dependencies between source files
    fn analyze_dependencies(&mut self, files: &[String]) -> BendResult<()> {
        for file in files {
            let content = fs::read_to_string(file)?;
            let deps = self.extract_dependencies(&content);
            for dep in deps {
                self.dependencies.add_dependency(file, &dep);
            }
        }
        Ok(())
    }

    /// Extract dependencies from source content
    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut deps = Vec::new();

        // Simple regex-based dependency extraction
        // In a real implementation, this would use the parser
        for line in content.lines() {
            if line.trim().starts_with("import") || line.trim().starts_with("from") {
                // Extract module names (simplified)
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let module = &line[start + 1..start + 1 + end];
                        deps.push(module.to_string());
                    }
                }
            }
        }

        deps
    }

    /// Build files in parallel
    fn build_parallel(&mut self, files: Vec<String>) -> BendResult<Vec<BuildArtifact>> {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        let config = Arc::new(self.config.clone());

        // Spawn worker threads
        let mut handles = Vec::new();
        let files_per_thread = files.len().div_ceil(self.config.jobs);

        for chunk in files.chunks(files_per_thread) {
            let chunk = chunk.to_vec();
            let tx = tx.clone();
            let config = Arc::clone(&config);

            let handle = thread::spawn(move || {
                for file in chunk {
                    let result = Self::build_single_file(&config, &file);
                    let _ = tx.send((file, result));
                }
            });

            handles.push(handle);
        }

        drop(tx); // Close the sender

        // Collect results
        let mut results = Vec::new();
        for (_file, result) in rx {
            match result {
                Ok(artifact) => {
                    self.cache.update_artifact(artifact.clone());
                    results.push(artifact);
                    self.stats.files_processed += 1;
                }
                Err(error) => {
                    self.error_reporter.error(error, Default::default());
                    self.stats.errors += 1;
                }
            }
        }

        // Wait for all threads
        for handle in handles {
            let _ = handle.join();
        }

        Ok(results)
    }

    /// Build a single file
    fn build_single_file(config: &BuildConfig, source_path: &str) -> BendResult<BuildArtifact> {
        let source_path_buf = PathBuf::from(source_path);
        let content = fs::read_to_string(&source_path_buf)?;
        let hash = Self::calculate_hash(&content);

        // Determine output path
        let file_name = source_path_buf.file_stem().unwrap().to_string_lossy();
        let output_path = match config.target {
            TargetPlatform::PolkaVM => config.output_dir.join(format!("{}.polkavm", file_name)),
            TargetPlatform::Wasm => config.output_dir.join(format!("{}.wasm", file_name)),
            TargetPlatform::RiscV => config.output_dir.join(format!("{}.s", file_name)),
            TargetPlatform::Native => config.output_dir.join(format!("{}.o", file_name)),
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Parse source
        let mut parser = Parser::new(&content);
        let program = parser.parse_program().map_err(|e| {
            BendError::Compilation(crate::error::CompilationError::Parse(e.to_string()))
        })?;

        // Type check
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program).map_err(|e| {
            BendError::Compilation(crate::error::CompilationError::Type(e.to_string()))
        })?;

        // Security check
        if config.security_level != SecurityLevel::None {
            let security_config = SecurityConfig {
                gas_limit: 10_000_000,
                max_call_depth: 100,
                enable_access_control: config.security_level as u8 >= SecurityLevel::Basic as u8,
                enable_reentrancy_guard: config.security_level as u8
                    >= SecurityLevel::Enhanced as u8,
                enable_input_validation: config.security_level as u8 >= SecurityLevel::Basic as u8,
                enable_static_analysis: config.security_level as u8
                    >= SecurityLevel::Enhanced as u8,
                enable_fuzz_testing: config.security_level as u8 >= SecurityLevel::Maximum as u8,
            };

            let mut security_manager = SecurityManager::new(security_config);
            security_manager.validate_program(&program).map_err(|e| {
                BendError::Compilation(crate::error::CompilationError::Security(e.to_string()))
            })?;
        }

        // Optimize
        let mut optimizer = create_default_manager();
        optimizer.set_level(config.optimization_level);
        let optimized_program = optimizer.optimize(program)?;

        // Generate code
        let mut codegen = RiscVCodegen::new();
        let instructions = codegen.generate(&optimized_program).map_err(|e| {
            BendError::Compilation(crate::error::CompilationError::Codegen(e.to_string()))
        })?;

        // Write output (simplified - in real implementation would compile to target)
        let output_content = instructions
            .iter()
            .map(|inst| inst.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&output_path, output_content)?;

        Ok(BuildArtifact {
            source_path: source_path_buf,
            output_path,
            timestamp,
            hash,
            dependencies: Vec::new(), // Would be populated with actual dependencies
            status: BuildStatus::Success,
        })
    }

    /// Calculate file hash for change detection
    fn calculate_hash(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get build statistics
    pub fn stats(&self) -> &BuildStats {
        &self.stats
    }

    /// Get error reporter
    pub fn errors(&self) -> &ErrorReporter {
        &self.error_reporter
    }

    /// Clean build artifacts
    pub fn clean(&self) -> BendResult<()> {
        if self.config.output_dir.exists() {
            fs::remove_dir_all(&self.config.output_dir)?;
        }
        Ok(())
    }
}

/// Build result
#[derive(Debug)]
pub struct BuildResult {
    /// Generated artifacts
    pub artifacts: Vec<BuildArtifact>,

    /// Build statistics
    pub stats: BuildStats,

    /// Whether build was successful
    pub success: bool,
}

/// Convenience functions for common build operations
pub mod build_functions {
    use super::*;

    /// Create a default build configuration
    pub fn default_config() -> BuildConfig {
        BuildConfig::default()
    }

    /// Create a release build configuration
    pub fn release_config() -> BuildConfig {
        BuildConfig {
            optimization_level: OptimizationLevel::Aggressive,
            debug: false,
            security_level: SecurityLevel::Maximum,
            incremental: false,
            ..Default::default()
        }
    }

    /// Create a debug build configuration
    pub fn debug_config() -> BuildConfig {
        BuildConfig {
            optimization_level: OptimizationLevel::None,
            debug: true,
            security_level: SecurityLevel::Basic,
            incremental: true,
            ..Default::default()
        }
    }

    /// Build a single file
    pub fn build_file(source: &Path, output: &Path) -> BendResult<BuildArtifact> {
        let config = BuildConfig {
            source_files: vec![source.to_path_buf()],
            output_dir: output.parent().unwrap().to_path_buf(),
            ..Default::default()
        };

        let mut builder = BuildSystem::new(config)?;
        let result = builder.build()?;

        if result.success && !result.artifacts.is_empty() {
            Ok(result.artifacts[0].clone())
        } else {
            Err(BendError::Generic("Build failed".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_file(content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let file_name = format!(
            "test_{}.bend",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = temp_dir.join(file_name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_build_config_defaults() {
        let config = BuildConfig::default();
        assert_eq!(config.optimization_level, OptimizationLevel::Standard);
        assert!(!config.debug);
        assert_eq!(config.security_level, SecurityLevel::Enhanced);
        assert!(config.incremental);
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        graph.add_dependency("main.bend", "utils.bend");
        graph.add_dependency("main.bend", "math.bend");
        graph.add_dependency("utils.bend", "math.bend");

        let order = graph.build_order(&[
            "main.bend".to_string(),
            "utils.bend".to_string(),
            "math.bend".to_string(),
        ]);

        // math.bend should come before utils.bend, which should come before main.bend
        assert!(
            order.iter().position(|x| x == "math.bend").unwrap()
                < order.iter().position(|x| x == "utils.bend").unwrap()
        );
        assert!(
            order.iter().position(|x| x == "utils.bend").unwrap()
                < order.iter().position(|x| x == "main.bend").unwrap()
        );
    }

    #[test]
    fn test_build_cache() {
        let temp_dir = std::env::temp_dir();
        let cache_path = temp_dir.join("test_cache.json");

        let mut cache = BuildCache::new(cache_path.clone());

        // Test cache operations
        let artifact = BuildArtifact {
            source_path: PathBuf::from("test.bend"),
            output_path: PathBuf::from("test.out"),
            timestamp: 12345,
            hash: "abcd".to_string(),
            dependencies: vec!["dep1".to_string()],
            status: BuildStatus::Success,
        };

        cache.update_artifact(artifact);
        assert!(!cache.needs_rebuild(Path::new("test.bend"), "abcd")); // Cache hit - same hash
        assert!(cache.needs_rebuild(Path::new("test.bend"), "efgh")); // Cache miss - different hash

        // Cleanup
        let _ = fs::remove_file(&cache_path);
    }

    #[test]
    fn test_build_stats() {
        let mut stats = BuildStats::default();
        stats.files_processed = 5;
        stats.files_compiled = 3;
        stats.files_skipped = 2;
        stats.errors = 0;
        stats.warnings = 1;

        assert_eq!(stats.files_processed, 5);
        assert_eq!(stats.files_compiled, 3);
        assert_eq!(stats.files_skipped, 2);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.warnings, 1);
    }

    #[test]
    fn test_build_functions() {
        let config = build_functions::default_config();
        assert_eq!(config.optimization_level, OptimizationLevel::Standard);

        let release_config = build_functions::release_config();
        assert_eq!(
            release_config.optimization_level,
            OptimizationLevel::Aggressive
        );

        let debug_config = build_functions::debug_config();
        assert_eq!(debug_config.optimization_level, OptimizationLevel::None);
        assert!(debug_config.debug);
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();

        reporter.error(
            BendError::Generic("Test error".to_string()),
            Default::default(),
        );
        reporter.warning(
            BendError::Generic("Test warning".to_string()),
            Default::default(),
        );

        assert_eq!(reporter.error_count(), 1);
        assert_eq!(reporter.warning_count(), 1);
        assert!(reporter.has_errors());
        assert!(reporter.has_warnings());
    }
}
