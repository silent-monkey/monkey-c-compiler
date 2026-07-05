// Only line comments are supported in monkey C.

// extern functions shall not have bodies.
extern fn printf(format: *const u8, ...) -> i32;

// Non-extern functions shall have bodies.
fn main() -> i32 {
    // Functions could be used before their declaration/definition.
    printf_hello_world();

    // let defines a local variable.
    // The type annotation is compulsory.
    // The initialization expression is optional.
    let sum_1: u32 = compute_sum_1();
    let sum_2: u32 = compute_sum_2();

    return 0;
}

fn compute_sum_1() -> u32 {
    let sum: u32 = 0;
    let i: u32 = 0;
    while (i < 10) {
        sum += i;
        i += 1;
    }

    // Missing return - let me add one
    return sum;
}

fn compute_sum_2() -> u32 {
    let sum: u32 = 0;
    let i: u32 = 0;
    loop {
        sum += i;
        i += 1;

        if (i >= 10) {
            return sum;
        }
    }
}

fn printf_hello_world() {
    printf("Hello, world!\n");
}
