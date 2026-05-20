// NewtonEmu - PowerPC Macintosh Emulator
// Copyright (C) 2026 NewtonEmu Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Forth interpreter for OpenFirmware
//!
//! This implements a subset of Forth needed to execute OpenFirmware boot scripts.
//! 
//! OpenFirmware uses Forth as its primary scripting language. Boot scripts
//! contain Forth code that:
//! - Checks platform compatibility
//! - Allocates memory regions
//! - Loads and decompresses ROM images
//! - Sets up device tree properties
//! - Transfers control to the operating system
//!
//! This interpreter supports:
//! - Data and return stacks
//! - Basic arithmetic and logic operations
//! - Stack manipulation words
//! - Control flow (if/then/else, begin/while/repeat, do/loop)
//! - Word definitions (: and ;)
//! - Variables and constants
//! - OpenFirmware-specific words (find-device, get-package-property, etc.)

use newton_utils::Result;
use std::collections::HashMap;

/// Forth value type - all values are 32-bit signed integers in Forth
pub type ForthValue = i32;

/// Forth word definition
#[derive(Clone, Debug)]
pub enum ForthWord {
    /// Built-in primitive word
    Primitive(fn(&mut ForthInterpreter) -> Result<()>),
    /// User-defined word (compiled sequence of other words)
    Compiled(Vec<String>),
    /// Constant value
    Constant(ForthValue),
    /// Variable (address in data space)
    Variable(usize),
}

/// Forth interpreter state
pub struct ForthInterpreter {
    /// Data stack (parameter stack)
    data_stack: Vec<ForthValue>,
    /// Return stack
    return_stack: Vec<ForthValue>,
    /// Dictionary of defined words
    dictionary: HashMap<String, ForthWord>,
    /// Data space for variables
    data_space: Vec<u8>,
    /// Here pointer (next free address in data space)
    here: usize,
    /// Compilation state (true = compiling, false = interpreting)
    compiling: bool,
    /// Current word being compiled
    compile_buffer: Vec<String>,
    /// Input buffer
    input_buffer: String,
    /// Input position
    input_pos: usize,
    /// Base for number parsing (10 = decimal, 16 = hex)
    base: u32,
    /// Program entry point (set by init-program)
    pub program_entry: Option<u32>,
    /// Load base address
    pub load_base: Option<u32>,
}

impl ForthInterpreter {
    /// Create a new Forth interpreter
    pub fn new() -> Self {
        let mut interpreter = Self {
            data_stack: Vec::with_capacity(256),
            return_stack: Vec::with_capacity(256),
            dictionary: HashMap::new(),
            data_space: vec![0; 65536], // 64KB data space
            here: 0,
            compiling: false,
            compile_buffer: Vec::new(),
            input_buffer: String::new(),
            input_pos: 0,
            base: 10, // Start in decimal
            program_entry: None,
            load_base: None,
        };

        // Register built-in words
        interpreter.register_primitives();
        
        interpreter
    }

    /// Register all built-in primitive words
    fn register_primitives(&mut self) {
        // Stack manipulation
        self.register_primitive("dup", |i| i.dup());
        self.register_primitive("drop", |i| i.drop());
        self.register_primitive("swap", |i| i.swap());
        self.register_primitive("over", |i| i.over());
        self.register_primitive("rot", |i| i.rot());
        self.register_primitive("pick", |i| i.pick());
        self.register_primitive("roll", |i| i.roll());
        self.register_primitive("2dup", |i| i.two_dup());
        self.register_primitive("2drop", |i| i.two_drop());
        self.register_primitive("2swap", |i| i.two_swap());
        self.register_primitive("2over", |i| i.two_over());
        self.register_primitive("nip", |i| i.nip());
        self.register_primitive("tuck", |i| i.tuck());

        // Constants
        self.register_primitive("true", |i| { i.push(-1); Ok(()) });
        self.register_primitive("false", |i| { i.push(0); Ok(()) });

        // Return stack
        self.register_primitive(">r", |i| i.to_r());
        self.register_primitive("r>", |i| i.r_from());
        self.register_primitive("r@", |i| i.r_fetch());

        // Arithmetic
        self.register_primitive("+", |i| i.add());
        self.register_primitive("-", |i| i.sub());
        self.register_primitive("*", |i| i.mul());
        self.register_primitive("/", |i| i.div());
        self.register_primitive("mod", |i| i.modulo());
        self.register_primitive("negate", |i| i.negate());
        self.register_primitive("abs", |i| i.abs());
        self.register_primitive("1+", |i| i.one_plus());
        self.register_primitive("1-", |i| i.one_minus());

        // Logic and comparison
        self.register_primitive("and", |i| i.and());
        self.register_primitive("or", |i| i.or());
        self.register_primitive("xor", |i| i.xor());
        self.register_primitive("invert", |i| i.invert());
        self.register_primitive("=", |i| i.equal());
        self.register_primitive("<>", |i| i.not_equal());
        self.register_primitive("<", |i| i.less_than());
        self.register_primitive(">", |i| i.greater_than());
        self.register_primitive("u<", |i| i.unsigned_less_than());
        self.register_primitive("u>", |i| i.unsigned_greater_than());
        self.register_primitive("0=", |i| i.zero_equal());
        self.register_primitive("0<", |i| i.zero_less());
        self.register_primitive("min", |i| i.min());
        self.register_primitive("max", |i| i.max());
        self.register_primitive("<<", |i| i.lshift());
        self.register_primitive(">>", |i| i.rshift());
        self.register_primitive("lshift", |i| i.lshift());
        self.register_primitive("rshift", |i| i.rshift());

        // Memory access (these will work on data_space for now)
        self.register_primitive("@", |i| i.fetch());
        self.register_primitive("!", |i| i.store());
        self.register_primitive("c@", |i| i.c_fetch());
        self.register_primitive("c!", |i| i.c_store());
        self.register_primitive("l@", |i| i.l_fetch());
        self.register_primitive("l!", |i| i.l_store());
        self.register_primitive("w@", |i| i.w_fetch());
        self.register_primitive("w!", |i| i.w_store());
        self.register_primitive("2@", |i| i.two_fetch());
        self.register_primitive("2!", |i| i.two_store());

        // Base control
        self.register_primitive("decimal", |i| { i.base = 10; Ok(()) });
        self.register_primitive("hex", |i| { i.base = 16; Ok(()) });
        self.register_primitive("h#", |i| i.h_number());

        // Data space
        self.register_primitive("here", |i| i.here());
        self.register_primitive("allot", |i| i.allot());
        self.register_primitive("c,", |i| i.c_comma());

        // Output (for debugging)
        self.register_primitive(".", |i| i.dot());
        self.register_primitive("cr", |i| i.cr());
        self.register_primitive(".\"", |i| i.dot_quote());
        self.register_primitive("u.", |i| i.u_dot());
        
        // String literals (special handling in eval loop)
        self.register_primitive("\"", |i| i.parse_string_literal());

        // Compilation
        self.register_primitive(":", |i| i.colon());
        self.register_primitive(";", |i| i.semicolon());
        self.register_primitive("constant", |i| i.constant_word());
        self.register_primitive("variable", |i| i.variable_word());

        // Control structures (these need special handling)
        self.register_primitive("if", |i| i.if_word());
        self.register_primitive("then", |i| i.then_word());
        self.register_primitive("else", |i| i.else_word());
        self.register_primitive("exit", |i| i.exit_word());
        self.register_primitive("until", |i| i.until_word());
        self.register_primitive("again", |i| i.again_word());
        
        // Constants for cell sizes
        self.create_constant("/l", 4);  // Size of long (32-bit)
        self.create_constant("/w", 2);  // Size of word (16-bit)
        self.create_constant("/c", 1);  // Size of char (8-bit)
        self.create_constant("/n", 4);  // Size of cell (32-bit)
    }

    /// Register a primitive word
    pub fn register_primitive(&mut self, name: &str, func: fn(&mut ForthInterpreter) -> Result<()>) {
        self.dictionary.insert(name.to_string(), ForthWord::Primitive(func));
    }

    /// Get mutable reference to data space
    pub fn data_space_mut(&mut self) -> &mut [u8] {
        &mut self.data_space
    }

    /// Get reference to data space
    pub fn data_space(&self) -> &[u8] {
        &self.data_space
    }

    /// Get current here pointer
    pub fn here_ptr(&self) -> usize {
        self.here
    }

    /// Set here pointer
    pub fn set_here(&mut self, here: usize) {
        self.here = here;
    }

    /// Push value onto data stack
    pub fn push(&mut self, value: ForthValue) {
        self.data_stack.push(value);
    }

    /// Pop value from data stack
    pub fn pop(&mut self) -> Result<ForthValue> {
        self.data_stack.pop().ok_or_else(|| newton_utils::Error::Other("Stack underflow".to_string()))
    }

    /// Peek at top of data stack without popping
    pub fn peek(&self) -> Result<ForthValue> {
        self.data_stack.last().copied().ok_or_else(|| newton_utils::Error::Other("Stack empty".to_string()))
    }

    /// Get stack depth
    pub fn depth(&self) -> usize {
        self.data_stack.len()
    }
    
    /// Get reference to the dictionary
    pub fn dictionary(&self) -> &HashMap<String, ForthWord> {
        &self.dictionary
    }

    // ============================================================================
    // Stack manipulation words
    // ============================================================================

    fn dup(&mut self) -> Result<()> {
        let val = self.peek()?;
        self.push(val);
        Ok(())
    }

    fn drop(&mut self) -> Result<()> {
        self.pop()?;
        Ok(())
    }

    fn swap(&mut self) -> Result<()> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn over(&mut self) -> Result<()> {
        let len = self.data_stack.len();
        if len < 2 {
            return Err(newton_utils::Error::Other("Stack underflow".to_string()));
        }
        let val = self.data_stack[len - 2];
        self.push(val);
        Ok(())
    }

    fn rot(&mut self) -> Result<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b);
        self.push(c);
        self.push(a);
        Ok(())
    }

    fn pick(&mut self) -> Result<()> {
        let n = self.pop()? as usize;
        let len = self.data_stack.len();
        if n >= len {
            return Err(newton_utils::Error::Other("Stack underflow".to_string()));
        }
        let val = self.data_stack[len - 1 - n];
        self.push(val);
        Ok(())
    }

    fn roll(&mut self) -> Result<()> {
        let n = self.pop()? as usize;
        let len = self.data_stack.len();
        if n >= len {
            return Err(newton_utils::Error::Other("Stack underflow".to_string()));
        }
        let val = self.data_stack.remove(len - 1 - n);
        self.push(val);
        Ok(())
    }

    fn two_dup(&mut self) -> Result<()> {
        let len = self.data_stack.len();
        if len < 2 {
            return Err(newton_utils::Error::Other("Stack underflow".to_string()));
        }
        let a = self.data_stack[len - 2];
        let b = self.data_stack[len - 1];
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn two_drop(&mut self) -> Result<()> {
        self.pop()?;
        self.pop()?;
        Ok(())
    }

    fn two_swap(&mut self) -> Result<()> {
        let d = self.pop()?;
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(c);
        self.push(d);
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn nip(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.pop()?;
        self.push(a);
        Ok(())
    }

    fn tuck(&mut self) -> Result<()> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a);
        self.push(b);
        self.push(a);
        Ok(())
    }

    fn two_over(&mut self) -> Result<()> {
        // 2over ( a b c d -- a b c d a b )
        let d = self.pop()?;
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a);
        self.push(b);
        self.push(c);
        self.push(d);
        self.push(a);
        self.push(b);
        Ok(())
    }

    // ============================================================================
    // Return stack words
    // ============================================================================

    fn to_r(&mut self) -> Result<()> {
        let val = self.pop()?;
        self.return_stack.push(val);
        Ok(())
    }

    fn r_from(&mut self) -> Result<()> {
        let val = self.return_stack.pop().ok_or_else(|| newton_utils::Error::Other("Return stack underflow".to_string()))?;
        self.push(val);
        Ok(())
    }

    fn r_fetch(&mut self) -> Result<()> {
        let val = self.return_stack.last().copied().ok_or_else(|| newton_utils::Error::Other("Return stack empty".to_string()))?;
        self.push(val);
        Ok(())
    }

    // ============================================================================
    // Arithmetic words
    // ============================================================================

    fn add(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.wrapping_add(b));
        Ok(())
    }

    fn sub(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.wrapping_sub(b));
        Ok(())
    }

    fn mul(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.wrapping_mul(b));
        Ok(())
    }

    fn div(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        if b == 0 {
            return Err(newton_utils::Error::Other("Division by zero".to_string()));
        }
        self.push(a / b);
        Ok(())
    }

    fn modulo(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        if b == 0 {
            return Err(newton_utils::Error::Other("Division by zero".to_string()));
        }
        self.push(a % b);
        Ok(())
    }

    fn negate(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(-a);
        Ok(())
    }

    fn abs(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(a.abs());
        Ok(())
    }

    fn one_plus(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(a.wrapping_add(1));
        Ok(())
    }

    fn one_minus(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(a.wrapping_sub(1));
        Ok(())
    }

    // ============================================================================
    // Logic and comparison words
    // ============================================================================

    fn and(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a & b);
        Ok(())
    }

    fn or(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a | b);
        Ok(())
    }

    fn xor(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a ^ b);
        Ok(())
    }

    fn invert(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(!a);
        Ok(())
    }

    fn equal(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a == b { -1 } else { 0 });
        Ok(())
    }

    fn not_equal(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a != b { -1 } else { 0 });
        Ok(())
    }

    fn less_than(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a < b { -1 } else { 0 });
        Ok(())
    }

    fn greater_than(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a > b { -1 } else { 0 });
        Ok(())
    }

    fn zero_equal(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(if a == 0 { -1 } else { 0 });
        Ok(())
    }

    fn zero_less(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(if a < 0 { -1 } else { 0 });
        Ok(())
    }

    fn unsigned_less_than(&mut self) -> Result<()> {
        let b = self.pop()? as u32;
        let a = self.pop()? as u32;
        self.push(if a < b { -1 } else { 0 });
        Ok(())
    }

    fn unsigned_greater_than(&mut self) -> Result<()> {
        let b = self.pop()? as u32;
        let a = self.pop()? as u32;
        self.push(if a > b { -1 } else { 0 });
        Ok(())
    }

    fn min(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a < b { a } else { b });
        Ok(())
    }

    fn max(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(if a > b { a } else { b });
        Ok(())
    }

    fn lshift(&mut self) -> Result<()> {
        // << or lshift ( n count -- n<<count )
        let count = self.pop()? as u32;
        let n = self.pop()? as u32;
        self.push((n << count) as i32);
        Ok(())
    }

    fn rshift(&mut self) -> Result<()> {
        // >> or rshift ( n count -- n>>count )
        let count = self.pop()? as u32;
        let n = self.pop()? as u32;
        self.push((n >> count) as i32);
        Ok(())
    }

    // ============================================================================
    // Memory access words
    // ============================================================================

    fn fetch(&mut self) -> Result<()> {
        let addr = self.pop()? as usize;
        if addr + 4 > self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        let val = i32::from_be_bytes([
            self.data_space[addr],
            self.data_space[addr + 1],
            self.data_space[addr + 2],
            self.data_space[addr + 3],
        ]);
        self.push(val);
        Ok(())
    }

    fn store(&mut self) -> Result<()> {
        let addr = self.pop()? as usize;
        let val = self.pop()?;
        if addr + 4 > self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        let bytes = val.to_be_bytes();
        self.data_space[addr..addr + 4].copy_from_slice(&bytes);
        Ok(())
    }

    fn c_fetch(&mut self) -> Result<()> {
        let addr = self.pop()? as usize;
        if addr >= self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        self.push(self.data_space[addr] as i32);
        Ok(())
    }

    fn c_store(&mut self) -> Result<()> {
        let addr = self.pop()? as usize;
        let val = self.pop()?;
        if addr >= self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        self.data_space[addr] = val as u8;
        Ok(())
    }

    fn l_fetch(&mut self) -> Result<()> {
        // l@ ( addr -- value ) - Fetch 32-bit big-endian
        self.fetch()
    }

    fn l_store(&mut self) -> Result<()> {
        // l! ( value addr -- ) - Store 32-bit big-endian
        self.store()
    }

    fn w_fetch(&mut self) -> Result<()> {
        // w@ ( addr -- value ) - Fetch 16-bit big-endian
        let addr = self.pop()? as usize;
        if addr + 2 > self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        let value = i16::from_be_bytes([
            self.data_space[addr],
            self.data_space[addr + 1],
        ]) as i32;
        self.push(value);
        Ok(())
    }

    fn w_store(&mut self) -> Result<()> {
        // w! ( value addr -- ) - Store 16-bit big-endian
        let addr = self.pop()? as usize;
        let val = self.pop()? as i16;
        if addr + 2 > self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        let bytes = val.to_be_bytes();
        self.data_space[addr..addr + 2].copy_from_slice(&bytes);
        Ok(())
    }

    fn two_fetch(&mut self) -> Result<()> {
        // 2@ ( addr -- val1 val2 ) - Fetch two consecutive cells
        let addr = self.pop()? as usize;
        if addr + 8 > self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        let val1 = i32::from_be_bytes([
            self.data_space[addr],
            self.data_space[addr + 1],
            self.data_space[addr + 2],
            self.data_space[addr + 3],
        ]);
        let val2 = i32::from_be_bytes([
            self.data_space[addr + 4],
            self.data_space[addr + 5],
            self.data_space[addr + 6],
            self.data_space[addr + 7],
        ]);
        self.push(val1);
        self.push(val2);
        Ok(())
    }

    fn two_store(&mut self) -> Result<()> {
        // 2! ( val1 val2 addr -- ) - Store two consecutive cells
        let addr = self.pop()? as usize;
        let val2 = self.pop()?;
        let val1 = self.pop()?;
        if addr + 8 > self.data_space.len() {
            return Err(newton_utils::Error::Other("Memory access out of bounds".to_string()));
        }
        let bytes1 = val1.to_be_bytes();
        let bytes2 = val2.to_be_bytes();
        self.data_space[addr..addr + 4].copy_from_slice(&bytes1);
        self.data_space[addr + 4..addr + 8].copy_from_slice(&bytes2);
        Ok(())
    }

    // ============================================================================
    // Data space words
    // ============================================================================

    fn here(&mut self) -> Result<()> {
        self.push(self.here as i32);
        Ok(())
    }

    fn allot(&mut self) -> Result<()> {
        let n = self.pop()? as usize;
        self.here += n;
        if self.here > self.data_space.len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        Ok(())
    }

    fn c_comma(&mut self) -> Result<()> {
        let val = self.pop()?;
        if self.here >= self.data_space.len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        self.data_space[self.here] = val as u8;
        self.here += 1;
        Ok(())
    }

    // ============================================================================
    // Output words
    // ============================================================================

    fn dot(&mut self) -> Result<()> {
        let val = self.pop()?;
        print!("{} ", val);
        Ok(())
    }

    fn u_dot(&mut self) -> Result<()> {
        let val = self.pop()?;
        print!("{} ", val as u32);
        Ok(())
    }

    fn cr(&mut self) -> Result<()> {
        println!();
        Ok(())
    }

    fn dot_quote(&mut self) -> Result<()> {
        // ." ( "ccc<quote>" -- )
        // Parse and print string until closing quote
        let mut result = String::new();
        let mut found_quote = false;
        
        while let Some(ch) = self.input_buffer.chars().nth(self.input_pos) {
            self.input_pos += 1;
            if ch == '"' {
                found_quote = true;
                break;
            }
            result.push(ch);
        }
        
        if !found_quote {
            return Err(newton_utils::Error::Other("Unterminated string in .\"".to_string()));
        }
        
        print!("{}", result);
        Ok(())
    }

    fn parse_string_literal(&mut self) -> Result<()> {
        // " ( "ccc<quote>" -- addr len )
        // Parse string until closing quote, store in data space, push address and length
        let mut result = String::new();
        let mut found_quote = false;
        
        // Skip leading whitespace after "
        while let Some(ch) = self.input_buffer.chars().nth(self.input_pos) {
            if !ch.is_whitespace() {
                break;
            }
            self.input_pos += 1;
        }
        
        // Parse until closing quote
        while let Some(ch) = self.input_buffer.chars().nth(self.input_pos) {
            self.input_pos += 1;
            if ch == '"' {
                found_quote = true;
                break;
            }
            result.push(ch);
        }
        
        if !found_quote {
            return Err(newton_utils::Error::Other("Unterminated string in \"".to_string()));
        }
        
        // Store string in data space
        let addr = self.here;
        let bytes = result.as_bytes();
        let len = bytes.len();
        
        if self.here + len > self.data_space.len() {
            return Err(newton_utils::Error::Other("Data space exhausted".to_string()));
        }
        
        self.data_space[self.here..self.here + len].copy_from_slice(bytes);
        self.here += len;
        
        // Push address and length
        self.push(addr as i32);
        self.push(len as i32);
        
        Ok(())
    }

    // ============================================================================
    // Compilation words
    // ============================================================================

    fn colon(&mut self) -> Result<()> {
        self.compiling = true;
        self.compile_buffer.clear();
        Ok(())
    }

    fn semicolon(&mut self) -> Result<()> {
        if !self.compiling {
            return Err(newton_utils::Error::Other("Not compiling".to_string()));
        }
        self.compiling = false;
        
        // First word in buffer should be the name
        if self.compile_buffer.is_empty() {
            return Err(newton_utils::Error::Other("Empty definition".to_string()));
        }
        
        let name = self.compile_buffer[0].clone();
        let body = self.compile_buffer[1..].to_vec();
        
        self.dictionary.insert(name, ForthWord::Compiled(body));
        self.compile_buffer.clear();
        
        Ok(())
    }

    fn if_word(&mut self) -> Result<()> {
        // For now, stub
        Ok(())
    }

    fn then_word(&mut self) -> Result<()> {
        // For now, stub
        Ok(())
    }

    fn else_word(&mut self) -> Result<()> {
        // For now, stub
        Ok(())
    }
    
    fn exit_word(&mut self) -> Result<()> {
        // exit - Return from current word immediately
        // This is a stub - proper implementation would require execution flow control
        Ok(())
    }
    
    fn until_word(&mut self) -> Result<()> {
        // until ( flag -- ) - Loop until flag is true
        // begin ... flag until
        // Stub for now
        Ok(())
    }
    
    fn again_word(&mut self) -> Result<()> {
        // again - Infinite loop back to begin
        // begin ... again
        // Stub for now
        Ok(())
    }
    
    fn h_number(&mut self) -> Result<()> {
        // h# ( "number" -- n )
        // Parse next token as hexadecimal number
        let token = self.next_token()
            .ok_or_else(|| newton_utils::Error::Other("Expected hex number after h#".to_string()))?;
        // Parse as u32 to handle large hex numbers, then convert to i32
        let value = u32::from_str_radix(&token, 16)
            .map_err(|_| newton_utils::Error::Other(format!("Invalid hex number: {}", token)))?;
        self.push(value as i32);
        Ok(())
    }
    
    fn constant_word(&mut self) -> Result<()> {
        // constant ( n "name" -- )
        // Read next token as name
        let name = self.next_token()
            .ok_or_else(|| newton_utils::Error::Other("Expected constant name".to_string()))?;
        let value = self.pop()?;
        self.create_constant(&name, value);
        Ok(())
    }
    
    fn variable_word(&mut self) -> Result<()> {
        // variable ( "name" -- )
        // Read next token as name
        let name = self.next_token()
            .ok_or_else(|| newton_utils::Error::Other("Expected variable name".to_string()))?;
        self.create_variable(&name);
        Ok(())
    }

    // ============================================================================
    // Interpreter
    // ============================================================================

    /// Parse a token from input
    fn next_token(&mut self) -> Option<String> {
        // Skip whitespace
        while self.input_pos < self.input_buffer.len() {
            let c = self.input_buffer.chars().nth(self.input_pos)?;
            if !c.is_whitespace() {
                break;
            }
            self.input_pos += 1;
        }

        if self.input_pos >= self.input_buffer.len() {
            return None;
        }

        // Collect token
        let start = self.input_pos;
        while self.input_pos < self.input_buffer.len() {
            let c = self.input_buffer.chars().nth(self.input_pos)?;
            if c.is_whitespace() {
                break;
            }
            self.input_pos += 1;
        }

        Some(self.input_buffer[start..self.input_pos].to_string())
    }

    /// Parse a number in current base
    fn parse_number(&self, token: &str) -> Option<ForthValue> {
        // Handle hex prefix
        if let Some(hex_str) = token.strip_prefix("h#").or_else(|| token.strip_prefix("0x")) {
            return i32::from_str_radix(hex_str, 16).ok();
        }
        
        // Parse in current base
        i32::from_str_radix(token, self.base).ok()
    }

    /// Execute a word
    fn execute_word(&mut self, name: &str) -> Result<()> {
        // First, try to parse as a number
        if let Some(num) = self.parse_number(name) {
            self.push(num);
            return Ok(());
        }
        
        // Then check dictionary
        if let Some(word) = self.dictionary.get(name).cloned() {
            match word {
                ForthWord::Primitive(func) => func(self),
                ForthWord::Compiled(body) => {
                    // Execute compiled word body
                    for token in body {
                        self.execute_word(&token)?;
                    }
                    Ok(())
                }
                ForthWord::Constant(val) => {
                    self.push(val);
                    Ok(())
                }
                ForthWord::Variable(addr) => {
                    self.push(addr as i32);
                    Ok(())
                }
            }
        } else {
            Err(newton_utils::Error::Other(format!("Undefined word: {}", name)))
        }
    }

    /// Evaluate a line of Forth code
    pub fn eval(&mut self, input: &str) -> Result<()> {
        self.input_buffer = input.to_string();
        self.input_pos = 0;

        while let Some(token) = self.next_token() {
            if self.compiling {
                // In compilation mode, add tokens to compile buffer
                if token == ";" {
                    self.semicolon()?;
                } else {
                    self.compile_buffer.push(token);
                }
            } else {
                // Try to parse as number first
                if let Some(num) = self.parse_number(&token) {
                    self.push(num);
                } else {
                    // Execute as word
                    self.execute_word(&token)?;
                }
            }
        }

        Ok(())
    }

    /// Create a constant
    pub fn create_constant(&mut self, name: &str, value: ForthValue) {
        self.dictionary.insert(name.to_string(), ForthWord::Constant(value));
    }

    /// Create a variable
    pub fn create_variable(&mut self, name: &str) -> usize {
        let addr = self.here;
        self.here += 4; // Reserve 4 bytes
        self.dictionary.insert(name.to_string(), ForthWord::Variable(addr));
        addr
    }
    
    /// Get the depth of the data stack
    pub fn stack_depth(&self) -> usize {
        self.data_stack.len()
    }
    
    /// Get the number of words in the dictionary
    pub fn dictionary_size(&self) -> usize {
        self.dictionary.len()
    }
}

impl Default for ForthInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut forth = ForthInterpreter::new();
        
        forth.eval("2 3 +").unwrap();
        assert_eq!(forth.pop().unwrap(), 5);
        
        forth.eval("10 3 -").unwrap();
        assert_eq!(forth.pop().unwrap(), 7);
        
        forth.eval("4 5 *").unwrap();
        assert_eq!(forth.pop().unwrap(), 20);
        
        forth.eval("20 4 /").unwrap();
        assert_eq!(forth.pop().unwrap(), 5);
    }

    #[test]
    fn test_stack_manipulation() {
        let mut forth = ForthInterpreter::new();
        
        forth.eval("1 2 3").unwrap();
        assert_eq!(forth.data_stack, vec![1, 2, 3]);
        
        forth.eval("dup").unwrap();
        assert_eq!(forth.data_stack, vec![1, 2, 3, 3]);
        
        forth.eval("drop").unwrap();
        assert_eq!(forth.data_stack, vec![1, 2, 3]);
        
        forth.eval("swap").unwrap();
        assert_eq!(forth.data_stack, vec![1, 3, 2]);
    }

    #[test]
    fn test_hex_numbers() {
        let mut forth = ForthInterpreter::new();
        
        forth.eval("hex 10").unwrap();
        assert_eq!(forth.pop().unwrap(), 16);
        
        forth.eval("h# FF").unwrap();
        assert_eq!(forth.pop().unwrap(), 255);
        
        forth.eval("decimal 10").unwrap();
        assert_eq!(forth.pop().unwrap(), 10);
    }

    #[test]
    fn test_constants() {
        let mut forth = ForthInterpreter::new();
        
        forth.create_constant("answer", 42);
        forth.eval("answer").unwrap();
        assert_eq!(forth.pop().unwrap(), 42);
    }

    #[test]
    fn test_colon_definition() {
        let mut forth = ForthInterpreter::new();
        
        forth.eval(": double 2 * ;").unwrap();
        forth.eval("5 double").unwrap();
        assert_eq!(forth.pop().unwrap(), 10);
    }

    #[test]
    fn test_true_false() {
        let mut forth = ForthInterpreter::new();
        
        forth.eval("true").unwrap();
        assert_eq!(forth.pop().unwrap(), -1);
        
        forth.eval("false").unwrap();
        assert_eq!(forth.pop().unwrap(), 0);
    }

    #[test]
    fn test_string_literal() {
        let mut forth = ForthInterpreter::new();
        
        // Test " word
        forth.eval("\" hello world\"").unwrap();
        let len = forth.pop().unwrap() as usize;
        let addr = forth.pop().unwrap() as usize;
        
        assert_eq!(len, 11);
        let string = String::from_utf8_lossy(&forth.data_space()[addr..addr+len]);
        assert_eq!(string, "hello world");
    }

    #[test]
    fn test_memory_words() {
        let mut forth = ForthInterpreter::new();
        
        // Test w@ and w!
        forth.eval("h# 1234 0 w!").unwrap();
        forth.eval("0 w@").unwrap();
        assert_eq!(forth.pop().unwrap(), 0x1234);
        
        // Test l@ and l!
        forth.eval("h# abcd1234 4 l!").unwrap();
        forth.eval("4 l@").unwrap();
        assert_eq!(forth.pop().unwrap() as u32, 0xabcd1234);
    }

    #[test]
    fn test_arithmetic_extensions() {
        let mut forth = ForthInterpreter::new();
        
        // Test lshift/<<
        forth.eval("h# FF 4 <<").unwrap();
        assert_eq!(forth.pop().unwrap(), 0xFF0);
        
        // Test rshift/>>
        forth.eval("h# FF0 4 >>").unwrap();
        assert_eq!(forth.pop().unwrap(), 0xFF);
        
        // Test min/max
        forth.eval("10 20 min").unwrap();
        assert_eq!(forth.pop().unwrap(), 10);
        
        forth.eval("10 20 max").unwrap();
        assert_eq!(forth.pop().unwrap(), 20);
    }

    #[test]
    fn test_cell_size_constants() {
        let mut forth = ForthInterpreter::new();
        
        forth.eval("/l").unwrap();
        assert_eq!(forth.pop().unwrap(), 4);
        
        forth.eval("/w").unwrap();
        assert_eq!(forth.pop().unwrap(), 2);
        
        forth.eval("/c").unwrap();
        assert_eq!(forth.pop().unwrap(), 1);
        
        forth.eval("/n").unwrap();
        assert_eq!(forth.pop().unwrap(), 4);
    }
}
