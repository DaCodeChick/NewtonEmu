// Test to dump MDB raw bytes

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let mut file = File::open("disks/macos-922-uni.iso").unwrap();
    let hfs_offset = 329 * 512;
    
    // MDB is at offset 1024 from HFS start
    file.seek(SeekFrom::Start(hfs_offset + 1024)).unwrap();
    
    let mut mdb = vec![0u8; 512];
    file.read_exact(&mut mdb).unwrap();
    
    println!("MDB first 128 bytes:");
    for (i, chunk) in mdb[0..128].chunks(16).enumerate() {
        print!("{:04X}: ", i * 16);
        for b in chunk {
            print!("{:02X} ", b);
        }
        println!();
    }
    
    println!("\nMDB bytes 114-160 (extent overflow and catalog extents):");
    for (i, chunk) in mdb[114..160].chunks(16).enumerate() {
        print!("{:04X}: ", 114 + i * 16);
        for b in chunk {
            print!("{:02X} ", b);
        }
        println!();
    }
}
