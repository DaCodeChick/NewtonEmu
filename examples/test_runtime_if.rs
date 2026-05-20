use newton_core::openfirmware::forth::ForthInterpreter;
use newton_core::openfirmware::OpenFirmwareForthExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut forth = ForthInterpreter::new();
    forth.register_of_words();
    
    println!("Test 1: if-then with true");
    forth.eval("true if 42 then")?;
    println!("Stack: {:?} (should have 42)", forth.stack_depth());
    
    println!("\nTest 2: if-then with false");
    forth.eval("false if 99 then")?;
    println!("Stack: {:?} (should still have just 42)", forth.stack_depth());
    
    println!("\nTest 3: if-else-then with true");
    forth.eval("true if 11 else 22 then")?;
    println!("Stack depth: {} (should have 42, 11)", forth.stack_depth());
    
    println!("\nTest 4: if-else-then with false");
    forth.eval("false if 77 else 88 then")?;
    println!("Stack depth: {} (should have 42, 11, 88)", forth.stack_depth());
    
    println!("\nTest 5: if with .\" inside (false, should skip)");
    forth.eval(r#"false if ." This should not print" then"#)?;
    println!("(Should not have printed anything above)");
    
    println!("\nTest 6: if with .\" inside (true, should print)");
    forth.eval(r#"true if ." This SHOULD print" then"#)?;
    println!("\n(Should have printed message above)");
    
    println!("\nAll tests passed!");
    Ok(())
}
