use bend_pvm::compiler::analyzer::type_checker::TypeChecker;
use bend_pvm::compiler::codegen::risc_v::RiscVCodegen;
use bend_pvm::compiler::optimizer::passes::{create_default_manager, OptimizationLevel};
use bend_pvm::compiler::parser::parser::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create_test_program(size: usize) -> String {
    let mut source = String::new();
    source.push_str("fn main() -> u32 {\n");

    for i in 0..size {
        source.push_str(&format!("    let var{} = {};\n", i, i));
    }

    source.push_str("    return var0");

    for i in 1..size {
        source.push_str(&format!(" + var{}", i));
    }

    source.push_str(";\n}\n");
    source
}

fn create_complex_program(num_functions: usize) -> String {
    let mut source = String::new();

    // Add helper functions
    for i in 0..num_functions {
        source.push_str(&format!("fn func{}(x: u32) -> u32 {{\n", i));
        source.push_str(&format!("    return x + {};\n", i));
        source.push_str("}\n\n");
    }

    // Main function calling all others
    source.push_str("fn main() -> u32 {\n");
    source.push_str("    let result = 0");

    for i in 0..num_functions {
        source.push_str(&format!(" + func{}(1)", i));
    }

    source.push_str(";\n    return result;\n}\n");
    source
}

fn bench_parsing_small(c: &mut Criterion) {
    let source = create_test_program(5);

    c.bench_function("parse_small_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&source));
            let _ = parser.parse_program().unwrap();
        })
    });
}

fn bench_parsing_medium(c: &mut Criterion) {
    let source = create_test_program(50);

    c.bench_function("parse_medium_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&source));
            let _ = parser.parse_program().unwrap();
        })
    });
}

fn bench_parsing_large(c: &mut Criterion) {
    let source = create_test_program(200);

    c.bench_function("parse_large_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&source));
            let _ = parser.parse_program().unwrap();
        })
    });
}

fn bench_type_checking_small(c: &mut Criterion) {
    let source = create_test_program(5);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();

    c.bench_function("type_check_small_program", |b| {
        b.iter(|| {
            let mut type_checker = TypeChecker::new();
            let _ = type_checker.check_program(black_box(&program)).unwrap();
        })
    });
}

fn bench_type_checking_medium(c: &mut Criterion) {
    let source = create_test_program(50);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();

    c.bench_function("type_check_medium_program", |b| {
        b.iter(|| {
            let mut type_checker = TypeChecker::new();
            let _ = type_checker.check_program(black_box(&program)).unwrap();
        })
    });
}

fn bench_codegen_small(c: &mut Criterion) {
    let source = create_test_program(5);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();

    c.bench_function("codegen_small_program", |b| {
        b.iter(|| {
            let mut codegen = RiscVCodegen::new();
            let _ = codegen.generate(black_box(&program)).unwrap();
        })
    });
}

fn bench_codegen_medium(c: &mut Criterion) {
    let source = create_test_program(50);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();

    c.bench_function("codegen_medium_program", |b| {
        b.iter(|| {
            let mut codegen = RiscVCodegen::new();
            let _ = codegen.generate(black_box(&program)).unwrap();
        })
    });
}

fn bench_optimization_small(c: &mut Criterion) {
    let source = create_test_program(5);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();
    let manager = create_default_manager();

    c.bench_function("optimize_small_program", |b| {
        b.iter(|| {
            let _ = manager.optimize(black_box(program.clone())).unwrap();
        })
    });
}

fn bench_optimization_medium(c: &mut Criterion) {
    let source = create_test_program(50);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();
    let manager = create_default_manager();

    c.bench_function("optimize_medium_program", |b| {
        b.iter(|| {
            let _ = manager.optimize(black_box(program.clone())).unwrap();
        })
    });
}

fn bench_full_compilation_small(c: &mut Criterion) {
    let source = create_test_program(5);

    c.bench_function("full_compilation_small", |b| {
        b.iter(|| {
            let _ = bend_pvm::compile_from_source(black_box(&source), Default::default()).unwrap();
        })
    });
}

fn bench_full_compilation_medium(c: &mut Criterion) {
    let source = create_test_program(50);

    c.bench_function("full_compilation_medium", |b| {
        b.iter(|| {
            let _ = bend_pvm::compile_from_source(black_box(&source), Default::default()).unwrap();
        })
    });
}

fn bench_complex_parsing(c: &mut Criterion) {
    let source = create_complex_program(10);

    c.bench_function("parse_complex_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&source));
            let _ = parser.parse_program().unwrap();
        })
    });
}

fn bench_complex_compilation(c: &mut Criterion) {
    let source = create_complex_program(10);

    c.bench_function("compile_complex_program", |b| {
        b.iter(|| {
            let _ = bend_pvm::compile_from_source(black_box(&source), Default::default()).unwrap();
        })
    });
}

fn bench_optimization_levels(c: &mut Criterion) {
    let source = create_test_program(20);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().unwrap();

    let levels = vec![
        ("none", OptimizationLevel::None),
        ("basic", OptimizationLevel::Basic),
        ("standard", OptimizationLevel::Standard),
        ("aggressive", OptimizationLevel::Aggressive),
    ];

    for (name, level) in levels {
        let mut manager = create_default_manager();
        manager.set_level(level);

        c.bench_function(&format!("optimize_level_{}", name), |b| {
            b.iter(|| {
                let _ = manager.optimize(black_box(program.clone())).unwrap();
            })
        });
    }
}

fn bench_memory_usage(c: &mut Criterion) {
    // Test memory usage patterns
    c.bench_function("memory_allocation_test", |b| {
        b.iter(|| {
            let source = create_test_program(100);
            let mut parser = Parser::new(&source);
            let program = parser.parse_program().unwrap();

            let mut codegen = RiscVCodegen::new();
            let _instructions = codegen.generate(&program).unwrap();

            // Force some allocations
            let _cloned = program.clone();
            drop(_cloned);
        })
    });
}

criterion_group!(
    benches,
    bench_parsing_small,
    bench_parsing_medium,
    bench_parsing_large,
    bench_type_checking_small,
    bench_type_checking_medium,
    bench_codegen_small,
    bench_codegen_medium,
    bench_optimization_small,
    bench_optimization_medium,
    bench_full_compilation_small,
    bench_full_compilation_medium,
    bench_complex_parsing,
    bench_complex_compilation,
    bench_optimization_levels,
    bench_memory_usage,
);

criterion_main!(benches);
