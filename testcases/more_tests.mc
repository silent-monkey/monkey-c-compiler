extern fn printf(format: *const u8, ...) -> i32;

fn main() -> i32 {
    test_void_return();
    test_nested_loops();
    test_param_passing();
    return 0;
}

// Test void return
fn test_void_return() {
    let x: u32 = 0;
    if (x == 0) {
        printf("void return test: x is 0\n");
        return;
    }
    printf("BUG: should not reach here\n");
}

// Test nested loops with break
fn test_nested_loops() {
    let i: u32 = 0;
    let count: u32 = 0;

    while (i < 5) {
        let j: u32 = 0;
        while (j < 3) {
            count += 1;
            j += 1;
        }
        i += 1;
    }

    // 5 * 3 = 15
    printf("nested loops: count = %u (expected 15)\n", count);
}

// Test function with multiple params
fn test_param_passing() {
    let result: u32 = add_three(10, 20, 30);
    printf("param passing: 10+20+30 = %u (expected 60)\n", result);
}

fn add_three(a: u32, b: u32, c: u32) -> u32 {
    return a + b + c;
}
