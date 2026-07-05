extern fn printf(format: *const u8, ...) -> i32;

fn main() -> i32 {
    let sum1: u32 = compute_sum_1();
    let sum2: u32 = compute_sum_2();

    // Sum of 0..9 = 45, should print 45 twice
    printf("sum1 = %u\n", sum1);
    printf("sum2 = %u\n", sum2);

    return 0;
}

fn compute_sum_1() -> u32 {
    let sum: u32 = 0;
    let i: u32 = 0;
    while (i < 10) {
        sum += i;
        i += 1;
    }
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
