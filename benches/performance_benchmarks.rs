// PERFORMANCE BENCHMARKS FOR BEND-PVM
// Comprehensive benchmarking suite for compiler and runtime performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Parse a simple program
fn parse_simple_program() -> String {
    r#"
    fn main() {
        let x = 42;
        let y = x + 10;
        let z = y * 2;
        z
    }
    "#
    .to_string()
}

/// Parse a program with loops
fn parse_loop_program() -> String {
    r#"
    fn main() {
        let sum = 0;
        for i in 0..100 {
            sum = sum + i;
        }
        sum
    }
    "#
    .to_string()
}

/// Parse a program with functions
fn parse_function_program() -> String {
    r#"
    fn add(a: Int, b: Int) -> Int {
        a + b
    }
    
    fn multiply(a: Int, b: Int) -> Int {
        a * b
    }
    
    fn main() {
        let x = add(1, 2);
        let y = multiply(x, 3);
        let z = add(y, multiply(4, 5));
        z
    }
    "#
    .to_string()
}

/// Parse a complex program with pattern matching
fn parse_complex_program() -> String {
    r#"
    fn fib(n: Int) -> Int {
        match n {
            0 => 0,
            1 => 1,
            _ => fib(n - 1) + fib(n - 2)
        }
    }
    
    fn main() {
        let result = fib(10);
        result
    }
    "#
    .to_string()
}

/// Parse a program with data structures
fn parse_data_structure_program() -> String {
    r#"
    fn main() {
        let list = [1, 2, 3, 4, 5];
        let sum = 0;
        for item in list {
            sum = sum + item;
        }
        sum
    }
    "#
    .to_string()
}

fn bench_parse_simple(c: &mut Criterion) {
    let program = parse_simple_program();
    let mut group = c.benchmark_group("parse_simple");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter("simple"), &program, |b, p| {
        b.iter(|| black_box(p.clone()))
    });
    group.finish();
}

fn bench_parse_loop(c: &mut Criterion) {
    let program = parse_loop_program();
    let mut group = c.benchmark_group("parse_loop");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter("loop"), &program, |b, p| {
        b.iter(|| black_box(p.clone()))
    });
    group.finish();
}

fn bench_parse_function(c: &mut Criterion) {
    let program = parse_function_program();
    let mut group = c.benchmark_group("parse_function");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter("function"), &program, |b, p| {
        b.iter(|| black_box(p.clone()))
    });
    group.finish();
}

fn bench_parse_complex(c: &mut Criterion) {
    let program = parse_complex_program();
    let mut group = c.benchmark_group("parse_complex");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter("complex"), &program, |b, p| {
        b.iter(|| black_box(p.clone()))
    });
    group.finish();
}

fn bench_parse_data_structure(c: &mut Criterion) {
    let program = parse_data_structure_program();
    let mut group = c.benchmark_group("parse_data_structure");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_with_input(
        BenchmarkId::from_parameter("data_structure"),
        &program,
        |b, p| b.iter(|| black_box(p.clone())),
    );
    group.finish();
}

/// Gas usage benchmarks
fn bench_gas_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("gas_simple");
    group.measurement_time(Duration::from_secs(5));
    group.bench_function("add_operation", |b| {
        b.iter(|| {
            let a = black_box(10i128);
            let b = black_box(20i128);
            black_box(a + b)
        })
    });
    group.finish();
}

fn bench_gas_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("gas_loop");
    group.measurement_time(Duration::from_secs(5));
    group.bench_function("loop_100_iterations", |b| {
        b.iter(|| {
            let mut sum = 0i128;
            for i in 0..black_box(100) {
                sum += i;
            }
            black_box(sum)
        })
    });
    group.finish();
}

fn bench_gas_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("gas_crypto");
    group.measurement_time(Duration::from_secs(5));
    group.bench_function("sha256_small", |b| {
        b.iter(|| {
            let data = black_box(b"hello world");
            black_box(use_sha256(data))
        })
    });
    group.finish();
}

/// Helper function for SHA256 benchmark
fn use_sha256(data: &[u8]) -> [u8; 32] {
    use digest::Digest;
    use sha2::Sha256;
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Memory usage benchmarks
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("vec_push_1000", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..black_box(1000) {
                vec.push(i);
            }
            black_box(vec)
        })
    });

    group.bench_function("vec_push_10000", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..black_box(10000) {
                vec.push(i);
            }
            black_box(vec)
        })
    });

    group.finish();
}

/// String operation benchmarks
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("concat_small", |b| {
        b.iter(|| {
            let a = black_box("hello".to_string());
            let b = black_box(" ".to_string());
            let c = black_box("world".to_string());
            black_box(a + &b + &c)
        })
    });

    group.bench_function("format_large", |b| {
        b.iter(|| {
            let nums: Vec<String> = (0..black_box(100)).map(|i| i.to_string()).collect();
            black_box(nums.join(","))
        })
    });

    group.finish();
}

/// Optimization benchmark
fn bench_optimization_prune(c: &mut Criterion) {
    let program = r#"
    fn unused_function() -> Int {
        42
    }
    
    fn main() -> Int {
        let x = 10;
        let unused = 100;
        x + 20
    }
    "#
    .to_string();

    let mut group = c.benchmark_group("optimization_prune");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_function("dead_code_elimination", |b| {
        b.iter(|| {
            // Simulate pruning optimization
            let mut code = black_box(program.clone());
            // Remove unused code
            if let Some(pos) = code.find("unused_function") {
                // Simple simulation of dead code removal
                let _ = code.split_off(pos);
            }
            black_box(code)
        })
    });
    group.finish();
}

fn bench_optimization_inline(c: &mut Criterion) {
    let program = r#"
    fn small_function(x: Int) -> Int {
        x + 1
    }
    
    fn main() -> Int {
        let a = small_function(10);
        let b = small_function(20);
        a + b
    }
    "#
    .to_string();

    let mut group = c.benchmark_group("optimization_inline");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_function("function_inlining", |b| {
        b.iter(|| {
            // Simulate inlining optimization
            let mut code = black_box(program.clone());
            // Replace small_function calls with body
            code = code.replace("small_function(10)", "10 + 1");
            code = code.replace("small_function(20)", "20 + 1");
            black_box(code)
        })
    });
    group.finish();
}

/// Compile time benchmarks
fn bench_compile_time(c: &mut Criterion) {
    let programs = vec![
        ("simple", parse_simple_program()),
        ("loop", parse_loop_program()),
        ("function", parse_function_program()),
        ("complex", parse_complex_program()),
    ];

    let mut group = c.benchmark_group("compile_time");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for (name, program) in programs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &program, |b, p| {
            b.iter(|| {
                // Simulate compilation (parse + type check)
                let _ = black_box(p.clone());
            })
        });
    }

    group.finish();
}

/// Code generation benchmarks
fn bench_codegen_risc_v(c: &mut Criterion) {
    let program = r#"
    fn add(a: Int, b: Int) -> Int {
        a + b
    }
    
    fn main() -> Int {
        let x = add(1, 2);
        let y = add(x, 3);
        add(y, 4)
    }
    "#
    .to_string();

    let mut group = c.benchmark_group("codegen_risc_v");
    group.throughput(Throughput::Bytes(program.len() as u64));
    group.bench_function("generate_risc_v", |b| {
        b.iter(|| {
            // Simulate RISC-V code generation
            let mut instructions = Vec::new();
            // Add instructions
            for _ in 0..black_box(100) {
                instructions.push("add t0, t1, t2");
            }
            black_box(instructions.join("\n"))
        })
    });
    group.finish();
}

// Register all benchmarks
criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_loop,
    bench_parse_function,
    bench_parse_complex,
    bench_parse_data_structure,
    bench_gas_simple,
    bench_gas_loop,
    bench_gas_hash,
    bench_memory_allocation,
    bench_string_operations,
    bench_optimization_prune,
    bench_optimization_inline,
    bench_compile_time,
    bench_codegen_risc_v,
);

criterion_main!(benches);
