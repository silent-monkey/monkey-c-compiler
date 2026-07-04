#!/bin/bash
# Run all mcc tests

MCC="./target/debug/mcc"
TESTS=(
    test_return:42
    test_simple_var:42
    test_vars:30
    test_arith:40
    test_if:1
    test_while:45
    test_call:7
    test_precedence:13
    test_nested_if:2
    test_lt:10
    test_eq:0
    test_prec2:25
    test_fact:120
    test_neg:214
    test_div:33
    test_mod:10
    test_and:2
    test_or:1
    test_complex_if:100
    test_bitnot:255
    test_negvar:246
)

PASS=0
FAIL=0

for entry in "${TESTS[@]}"; do
    test="${entry%%:*}"
    expected="${entry##*:}"
    
    echo -n "  $test ... "
    
    if ! "$MCC" "$test.c" 2>/dev/null; then
        echo "FAIL (mcc compilation error)"
        FAIL=$((FAIL + 1))
        continue
    fi
    
    if ! riscv64-linux-gnu-as -o "/tmp/$test.o" "$test.s" 2>/dev/null; then
        echo "FAIL (assembler error)"
        FAIL=$((FAIL + 1))
        continue
    fi
    
    if ! riscv64-linux-gnu-ld -o "/tmp/$test.riscv" "/tmp/$test.o" 2>/dev/null; then
        echo "FAIL (linker error)"
        FAIL=$((FAIL + 1))
        continue
    fi
    
    qemu-riscv64-static "/tmp/$test.riscv" 2>/dev/null
    actual=$?
    
    if [ "$actual" = "$expected" ]; then
        echo "OK (ret=$actual)"
        PASS=$((PASS + 1))
    else
        echo "FAIL (expected $expected, got $actual)"
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "$PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
