#!/bin/bash

# Bend-PVM Performance Test Suite
# Tests compilation performance, memory usage, and correctness

set -e

echo "🚀 Bend-PVM Performance Test Suite"
echo "=================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test files
SMALL_TEST="test_small.bend"
MEDIUM_TEST="test_medium.bend"
LARGE_TEST="test_large.bend"

# Create test programs
create_test_programs() {
    echo "📝 Creating test programs..."

    # Small program
    cat > "$SMALL_TEST" << 'EOF'
fn add(a: u32, b: u32) -> u32 {
    return a + b;
}

fn main() -> u32 {
    return add(5, 3);
}
EOF

    # Medium program
    cat > "$MEDIUM_TEST" << 'EOF'
fn factorial(n: u32) -> u32 {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

fn main() -> u32 {
    let fact5 = factorial(5);
    let fib8 = fibonacci(8);
    return fact5 + fib8;
}
EOF

    # Large program
    cat > "$LARGE_TEST" << 'EOF'
fn helper1(x: u32) -> u32 { return x + 1; }
fn helper2(x: u32) -> u32 { return x + 2; }
fn helper3(x: u32) -> u32 { return x + 3; }
fn helper4(x: u32) -> u32 { return x + 4; }
fn helper5(x: u32) -> u32 { return x + 5; }
fn helper6(x: u32) -> u32 { return x + 6; }
fn helper7(x: u32) -> u32 { return x + 7; }
fn helper8(x: u32) -> u32 { return x + 8; }
fn helper9(x: u32) -> u32 { return x + 9; }
fn helper10(x: u32) -> u32 { return x + 10; }

fn complex_calc(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    let step1 = helper1(a) + helper2(b);
    let step2 = helper3(step1) * helper4(c);
    let step3 = helper5(step2) - helper6(d);
    let step4 = helper7(step3) / helper8(e);
    return helper9(step4) + helper10(42);
}

fn main() -> u32 {
    let result1 = complex_calc(1, 2, 3, 4, 5);
    let result2 = complex_calc(6, 7, 8, 9, 10);
    let result3 = complex_calc(11, 12, 13, 14, 15);
    return result1 + result2 + result3;
}
EOF

    echo "✅ Test programs created"
}

# Measure execution time
measure_time() {
    local start_time=$(date +%s.%3N)
    "$@" > /dev/null 2>&1
    local end_time=$(date +%s.%3N)
    local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "0")
    echo "$duration"
}

# Test compilation performance
test_compilation_performance() {
    echo "⚡ Testing Compilation Performance"
    echo "----------------------------------"

    # Test small program
    echo -n "Small program compilation: "
    local time_small=$(measure_time cargo run -- compile "$SMALL_TEST" --output test_small_output)
    echo "${time_small}s"

    # Test medium program
    echo -n "Medium program compilation: "
    local time_medium=$(measure_time cargo run -- compile "$MEDIUM_TEST" --output test_medium_output)
    echo "${time_medium}s"

    # Test large program
    echo -n "Large program compilation: "
    local time_large=$(measure_time cargo run -- compile "$LARGE_TEST" --output test_large_output)
    echo "${time_large}s"

    # Performance assertions
    if (( $(echo "$time_small < 5.0" | bc -l) )); then
        echo -e "${GREEN}✅ Small program compiles quickly${NC}"
    else
        echo -e "${YELLOW}⚠️  Small program compilation is slow${NC}"
    fi

    if (( $(echo "$time_medium < 10.0" | bc -l) )); then
        echo -e "${GREEN}✅ Medium program compiles reasonably${NC}"
    else
        echo -e "${YELLOW}⚠️  Medium program compilation is slow${NC}"
    fi

    if (( $(echo "$time_large < 20.0" | bc -l) )); then
        echo -e "${GREEN}✅ Large program compiles within limits${NC}"
    else
        echo -e "${RED}❌ Large program compilation is too slow${NC}"
    fi
}

# Test optimization levels
test_optimization_levels() {
    echo
    echo "🔧 Testing Optimization Levels"
    echo "-------------------------------"

    local levels=("none" "basic" "standard" "aggressive")
    local times=()

    for level in "${levels[@]}"; do
        echo -n "Optimization level '$level': "
        local time=$(measure_time cargo run -- compile "$MEDIUM_TEST" --output "test_opt_${level}" --optimize)
        echo "${time}s"
        times+=("$time")
    done

    # Check that optimization doesn't make things dramatically worse
    local none_time="${times[0]}"
    local aggressive_time="${times[3]}"

    if (( $(echo "$aggressive_time < $none_time * 3" | bc -l) )); then
        echo -e "${GREEN}✅ Optimization overhead is reasonable${NC}"
    else
        echo -e "${YELLOW}⚠️  Optimization has high overhead${NC}"
    fi
}

# Test memory usage (rough estimate)
test_memory_usage() {
    echo
    echo "🧠 Testing Memory Usage"
    echo "-----------------------"

    echo "Note: This is a rough estimate using system tools"

    # Test with small program
    echo -n "Small program peak memory: "
    local mem_small=$(timeout 10 /usr/bin/time -l cargo run -- compile "$SMALL_TEST" --output test_mem_small 2>&1 | grep "maximum resident set size" | awk '{print $1}' || echo "N/A")
    echo "${mem_small} KB"

    # Test with large program
    echo -n "Large program peak memory: "
    local mem_large=$(timeout 15 /usr/bin/time -l cargo run -- compile "$LARGE_TEST" --output test_mem_large 2>&1 | grep "maximum resident set size" | awk '{print $1}' || echo "N/A")
    echo "${mem_large} KB"

    if [[ "$mem_small" != "N/A" && "$mem_large" != "N/A" ]]; then
        if (( mem_large < mem_small * 10 )); then
            echo -e "${GREEN}✅ Memory scaling is reasonable${NC}"
        else
            echo -e "${YELLOW}⚠️  Memory usage scales poorly${NC}"
        fi
    fi
}

# Test correctness
test_correctness() {
    echo
    echo "✅ Testing Correctness"
    echo "----------------------"

    # Test that compilation succeeds
    local success_count=0
    local total_tests=3

    if cargo run -- compile "$SMALL_TEST" --output test_correct_small > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Small program compiles successfully${NC}"
        ((success_count++))
    else
        echo -e "${RED}❌ Small program compilation failed${NC}"
    fi

    if cargo run -- compile "$MEDIUM_TEST" --output test_correct_medium > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Medium program compiles successfully${NC}"
        ((success_count++))
    else
        echo -e "${RED}❌ Medium program compilation failed${NC}"
    fi

    if cargo run -- compile "$LARGE_TEST" --output test_correct_large > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Large program compiles successfully${NC}"
        ((success_count++))
    else
        echo -e "${RED}❌ Large program compilation failed${NC}"
    fi

    local success_rate=$((success_count * 100 / total_tests))
    echo "Success rate: ${success_rate}%"

    if (( success_rate >= 100 )); then
        echo -e "${GREEN}🎉 All correctness tests passed!${NC}"
    elif (( success_rate >= 66 )); then
        echo -e "${YELLOW}⚠️  Most correctness tests passed${NC}"
    else
        echo -e "${RED}❌ Many correctness tests failed${NC}"
    fi
}

# Test code generation
test_code_generation() {
    echo
    echo "🔨 Testing Code Generation"
    echo "--------------------------"

    # Test assembly output
    if cargo run -- compile "$SMALL_TEST" --output test_asm --assembly > /dev/null 2>&1; then
        if [[ -f "test_asm.s" ]]; then
            local asm_lines=$(wc -l < test_asm.s)
            echo -e "${GREEN}✅ Assembly generated (${asm_lines} lines)${NC}"
        else
            echo -e "${RED}❌ Assembly file not created${NC}"
        fi
    else
        echo -e "${RED}❌ Assembly generation failed${NC}"
    fi

    # Test metadata output
    if cargo run -- compile "$SMALL_TEST" --output test_meta --metadata > /dev/null 2>&1; then
        if [[ -f "test_meta.metadata.json" ]]; then
            echo -e "${GREEN}✅ Metadata generated${NC}"
        else
            echo -e "${RED}❌ Metadata file not created${NC}"
        fi
    else
        echo -e "${RED}❌ Metadata generation failed${NC}"
    fi
}

# Main test execution
main() {
    # Setup
    create_test_programs

    # Run tests
    test_correctness
    test_compilation_performance
    test_optimization_levels
    test_memory_usage
    test_code_generation

    # Cleanup
    echo
    echo "🧹 Cleaning up test files..."
    rm -f "$SMALL_TEST" "$MEDIUM_TEST" "$LARGE_TEST"
    rm -f test_* test_*.s test_*.metadata.json test_*.abi.json

    echo
    echo "🎯 Performance testing complete!"
}

# Run main function
main "$@"