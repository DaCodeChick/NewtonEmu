//! Test OpenFirmware-specific Forth words

use newton_core::openfirmware::{ForthInterpreter, OpenFirmwareForthExt};

fn main() {
    println!("==================================================");
    println!("OpenFirmware Forth Words Test");
    println!("==================================================\n");

    let mut forth = ForthInterpreter::new();
    forth.register_of_words();

    // Test 1: String comparison
    println!("Test 1: String comparison ($=)");
    println!("  Setting up test strings in data space...");
    forth.eval("here").unwrap();
    let addr1 = forth.pop().unwrap();
    forth.eval("105 c, 77 c, 97 c, 99 c,").unwrap(); // "iMac" in ASCII
    forth.eval("here").unwrap();
    let addr2 = forth.pop().unwrap();
    forth.eval("105 c, 77 c, 97 c, 99 c,").unwrap(); // "iMac" in ASCII
    
    forth.push(addr1);
    forth.push(4);
    forth.push(addr2);
    forth.push(4);
    forth.eval("$=").unwrap();
    let result = forth.pop().unwrap();
    println!("  \"iMac\" $= \"iMac\" = {}", result);
    println!("  ✓ Expected: -1 (true)\n");

    // Test 2: encode-int
    println!("Test 2: encode-int");
    println!("  h# 12345678 encode-int");
    forth.eval("hex 12345678 encode-int").unwrap();
    let len = forth.pop().unwrap();
    let addr = forth.pop().unwrap();
    println!("  Result: addr=0x{:X}, len={}", addr, len);
    println!("  ✓ Expected: len=4\n");

    // Test 3: encode+
    println!("Test 3: encode+ (concatenate encoded values)");
    println!("  1000 encode-int 2000 encode-int encode+");
    forth.eval("decimal 1000 encode-int 2000 encode-int encode+").unwrap();
    let len = forth.pop().unwrap();
    let _addr = forth.pop().unwrap();
    println!("  Result: combined length = {}", len);
    println!("  ✓ Expected: 8 bytes\n");

    // Test 4: ?dup (duplicate if non-zero)
    println!("Test 4: ?dup (duplicate if non-zero)");
    forth.eval("5 ?dup").unwrap();
    let depth = forth.depth();
    println!("  5 ?dup -> stack depth = {}", depth);
    println!("  ✓ Expected: 2 (value duplicated)");
    
    forth.eval("drop drop 0 ?dup").unwrap();
    let depth = forth.depth();
    println!("  0 ?dup -> stack depth = {}", depth);
    println!("  ✓ Expected: 1 (not duplicated)\n");

    // Test 5: between
    println!("Test 5: between");
    forth.eval("5 1 10 between").unwrap();
    println!("  5 between 1 and 10 = {}", forth.pop().unwrap());
    println!("  ✓ Expected: -1 (true)");
    
    forth.eval("15 1 10 between").unwrap();
    println!("  15 between 1 and 10 = {}", forth.pop().unwrap());
    println!("  ✓ Expected: 0 (false)\n");

    // Test 6: bounds
    println!("Test 6: bounds (for DO loops)");
    forth.eval("100 20 bounds").unwrap();
    let start = forth.pop().unwrap();
    let end = forth.pop().unwrap();
    println!("  100 20 bounds = end:{} start:{}", end, start);
    println!("  ✓ Expected: end:120 start:100\n");

    // Test 7: fill
    println!("Test 7: fill (memory fill)");
    forth.eval("here dup 8 88 fill").unwrap(); // 88 is 'X'
    let addr = forth.pop().unwrap();
    println!("  Filled 8 bytes with 'X' at addr 0x{:X}", addr);
    println!("  ✓ Filled successfully\n");

    // Test 8: Boot script constants simulation
    println!("Test 8: Boot script constants (like in real ROM boot)");
    forth.eval("hex").unwrap();
    forth.create_constant("pagesz", 0x1000);
    forth.eval("pagesz 1- ").unwrap();
    let pagesz_1 = forth.pop().unwrap();
    println!("  pagesz-1 = 0x{:X}", pagesz_1);
    
    forth.create_constant("elf-size", 0x11690);
    forth.eval("elf-size pagesz 1- + pagesz negate and").unwrap();
    let elf_pages_bytes = forth.pop().unwrap();
    println!("  elf-size rounded to page = 0x{:X}", elf_pages_bytes);
    println!("  ✓ Expected: 0x12000 (rounded up to 4KB boundary)\n");

    // Test 9: find-package (stub)
    println!("Test 9: find-package (stub)");
    forth.eval("here").unwrap();
    let addr = forth.pop().unwrap();
    forth.push(addr);
    forth.push(1);  // length
    forth.eval("find-package").unwrap();
    let phandle = forth.pop().unwrap();
    println!("  find-package -> phandle = {}", phandle);
    println!("  ✓ Returns: {} (stub returns root)\n", phandle);

    // Test 10: Simulated boot script fragment
    println!("Test 10: Simulated boot script fragment");
    forth.create_constant("mem-base", 0x400000);
    println!("  Defining: : claim-mem mem-base ;");
    forth.eval(": claim-mem mem-base ;").unwrap();
    println!("  Executing: claim-mem");
    forth.eval("claim-mem").unwrap();
    let claimed = forth.pop().unwrap();
    println!("  Result: 0x{:X}", claimed);
    println!("  ✓ User-defined word executed successfully\n");

    println!("==================================================");
    println!("All OpenFirmware Forth word tests completed!");
    println!("==================================================");
}
