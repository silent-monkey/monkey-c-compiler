extern fn printf(format: *const u8, ...) -> i32;

fn main() -> i32 {
    test_arithmetic();
    test_relational();
    test_factorial();
    return 0;
}

// Test basic arithmetic
fn test_arithmetic() {
    let a: u32 = 10;
    let b: u32 = 3;

    printf("add: 10+3 = %u (expected 13)\n", a + b);
    printf("sub: 10-3 = %u (expected 7)\n", a - b);
    printf("mul: 10*3 = %u (expected 30)\n", a * b);
    printf("div: 10/3 = %u (expected 3)\n", a / b);
    printf("rem: 10%%3 = %u (expected 1)\n", a % b);
}

// Test relational operators
fn test_relational() {
    let x: u32 = 10;
    let y: u32 = 20;

    if (x < y) {
        printf("rel: 10 < 20 is true\n");
    }
    if (y > x) {
        printf("rel: 20 > 10 is true\n");
    }
    if (x <= y) {
        printf("rel: 10 <= 20 is true\n");
    }
    if (y >= x) {
        printf("rel: 20 >= 10 is true\n");
    }
    if (x == x) {
        printf("rel: 10 == 10 is true\n");
    }
    if (x != y) {
        printf("rel: 10 != 20 is true\n");
    }
}

// Test function calling function (factorial)
fn test_factorial() {
    let n: u32 = 5;
    let result: u32 = factorial(n);
    printf("factorial(5) = %u (expected 120)\n", result);
}

fn factorial(n: u32) -> u32 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
