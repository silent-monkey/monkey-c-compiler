#!/bin/bash
# Build and run all Monkey C test cases
set -e

MCC="cargo run --quiet --"
MC_DIR="testcases"
BUILD_DIR="testcases/build"

mkdir -p "$BUILD_DIR"

run_test() {
    local name="$1"
    local expected_output="$2"
    local extra_mattr="${3:-}"

    echo -n "Testing $name... "

    # Compile .mc -> .s
    $MCC "$MC_DIR/$name.mc" -o "$BUILD_DIR/$name.s" 2>/dev/null

    # Assemble .s -> .o
    llvm-mc -triple=riscv64 -filetype=obj "$BUILD_DIR/$name.s" \
        -o "$BUILD_DIR/$name.o" --mattr=+d,+m$extra_mattr 2>/dev/null

    # Link .o -> executable
    clang --target=riscv64-linux-gnu --sysroot=/usr/riscv64-linux-gnu \
        -static "$BUILD_DIR/$name.o" -o "$BUILD_DIR/$name" 2>/dev/null

    # Run
    actual_output=$(qemu-riscv64-static "$BUILD_DIR/$name" 2>&1)
    actual_exit=$?

    if [ "$actual_output" = "$expected_output" ] && [ "$actual_exit" -eq 0 ]; then
        echo "PASS"
    else
        echo "FAIL"
        echo "  Expected output: '$expected_output'"
        echo "  Actual output:   '$actual_output'"
        echo "  Exit code: $actual_exit"
        exit 1
    fi
}

run_test "hello" "Hello, world!"
run_test "sum" "sum1 = 45
sum2 = 45"
run_test "control_flow" "if test: x < 20 is true (x=10)
if test: y < 20 is false (y=30)
if/else test: a >= 30 (a=50)
if/else test: b < 10 (b=5)
break/continue test: sum = 8 (expected 8)"
run_test "arithmetic" "add: 10+3 = 13 (expected 13)
sub: 10-3 = 7 (expected 7)
mul: 10*3 = 30 (expected 30)
div: 10/3 = 3 (expected 3)
rem: 10%3 = 1 (expected 1)
rel: 10 < 20 is true
rel: 20 > 10 is true
rel: 10 <= 20 is true
rel: 20 >= 10 is true
rel: 10 == 10 is true
rel: 10 != 20 is true
factorial(5) = 120 (expected 120)"

run_test "more_tests" "void return test: x is 0
nested loops: count = 15 (expected 15)
param passing: 10+20+30 = 60 (expected 60)"

echo ""
echo "All tests passed!"
