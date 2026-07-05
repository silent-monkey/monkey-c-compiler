extern fn printf(format: *const u8, ...) -> i32;

fn main() -> i32 {
    test_if();
    test_if_else();
    test_break_continue();
    return 0;
}

// Test if statement
fn test_if() {
    let x: u32 = 10;
    if (x < 20) {
        printf("if test: x < 20 is true (x=%u)\n", x);
    }

    let y: u32 = 30;
    if (y < 20) {
        printf("BUG: should not print\n");
    }
    printf("if test: y < 20 is false (y=%u)\n", y);
}

// Test if/else
fn test_if_else() {
    let a: u32 = 50;
    if (a < 30) {
        printf("BUG: a < 30\n");
    } else {
        printf("if/else test: a >= 30 (a=%u)\n", a);
    }

    let b: u32 = 5;
    if (b < 10) {
        printf("if/else test: b < 10 (b=%u)\n", b);
    } else {
        printf("BUG: b >= 10\n");
    }
}

// Test break and continue
fn test_break_continue() {
    let i: u32 = 0;
    let sum: u32 = 0;

    loop {
        if (i >= 5) {
            break;
        }

        // skip i=2 (continue test)
        if (i == 2) {
            i += 1;
            continue;
        }

        sum += i;
        i += 1;
    }

    // sum should be 0+1+3+4 = 8 (skip 2)
    printf("break/continue test: sum = %u (expected 8)\n", sum);
}
