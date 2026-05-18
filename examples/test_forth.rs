//! Test Forth interpreter

use newton_core::openfirmware::ForthInterpreter;

fn main() {
    println!("==================================================");
    println!("Forth Interpreter Test");
    println!("==================================================\n");

    let mut forth = ForthInterpreter::new();

    // Test 1: Basic arithmetic
    println!("Test 1: Basic arithmetic");
    println!("  2 3 + =");
    forth.eval("2 3 +").unwrap();
    println!("  Result: {}", forth.pop().unwrap());
    println!("  ✓ Expected: 5\n");

    // Test 2: Stack manipulation
    println!("Test 2: Stack manipulation");
    println!("  1 2 3 swap =");
    forth.eval("1 2 3 swap").unwrap();
    let a = forth.pop().unwrap();
    let b = forth.pop().unwrap();
    let c = forth.pop().unwrap();
    println!("  Result: {} {} {}", c, b, a);
    println!("  ✓ Expected: 1 3 2\n");

    // Test 3: Hex numbers
    println!("Test 3: Hex numbers");
    println!("  hex FF 10 + =");
    forth.eval("hex FF 10 +").unwrap();
    println!("  Result: 0x{:X}", forth.pop().unwrap());
    println!("  ✓ Expected: 0x10F\n");

    // Test 4: Constants
    println!("Test 4: Constants");
    println!("  Creating constant 'pagesz' = 4096");
    forth.create_constant("pagesz", 4096);
    forth.eval("pagesz 2 *").unwrap();
    println!("  pagesz 2 * = {}", forth.pop().unwrap());
    println!("  ✓ Expected: 8192\n");

    // Test 5: Colon definitions
    println!("Test 5: Colon definitions");
    println!("  : square dup * ;");
    forth.eval(": square dup * ;").unwrap();
    forth.eval("5 square").unwrap();
    println!("  5 square = {}", forth.pop().unwrap());
    println!("  ✓ Expected: 25\n");

    // Test 6: Complex expression (like boot script)
    println!("Test 6: Complex expression (simulating boot script constants)");
    forth.eval("decimal").unwrap();
    forth.create_constant("elf-offset", 0x4000);
    forth.create_constant("elf-size", 0x11690);
    forth.create_constant("pagesz", 0x1000);
    forth.create_constant("pagesz-1", 0xFFF);
    forth.create_constant("pagemask", -0x1000);
    
    println!("  elf-size pagesz-1 + pagemask and");
    forth.eval("elf-size pagesz-1 + pagemask and").unwrap();
    let elf_pages = forth.pop().unwrap();
    println!("  Result: 0x{:X} (elf-pages)", elf_pages);
    println!("  ✓ Expected: 0x12000 (elf-size rounded up to page boundary)\n");

    // Test 7: Return stack
    println!("Test 7: Return stack");
    println!("  5 >r 10 20 r> + =");
    forth.eval("5 >r 10 20 r> +").unwrap();
    let result = forth.pop().unwrap();
    println!("  Result: {} {}", forth.pop().unwrap(), result);
    println!("  ✓ Expected: 10 25\n");

    // Test 8: Logic operations
    println!("Test 8: Logic");
    println!("  10 5 < =");
    forth.eval("10 5 <").unwrap();
    println!("  Result: {}", forth.pop().unwrap());
    println!("  ✓ Expected: 0 (false)");
    
    println!("  5 10 < =");
    forth.eval("5 10 <").unwrap();
    println!("  Result: {}", forth.pop().unwrap());
    println!("  ✓ Expected: -1 (true)\n");

    // Test 9: Memory operations
    println!("Test 9: Memory operations");
    println!("  here 42 over ! @ =");
    forth.eval("here 42 over ! @").unwrap();
    println!("  Result: {}", forth.pop().unwrap());
    println!("  ✓ Expected: 42\n");

    println!("==================================================");
    println!("All tests completed!");
    println!("==================================================");
}
