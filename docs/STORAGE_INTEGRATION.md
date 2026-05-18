# Storage Integration Plan

This document outlines the steps needed to integrate the storage system into the main emulator.

## Current Status

✅ **Completed:**
- Block device abstraction (`BlockDevice` trait)
- ISO9660 image support (read-only CD/DVD)
- Raw disk image support (read/write HDD)
- SCSI command emulation (7 commands)
- Storage bus manager (SCSI + IDE)
- Comprehensive test suite

## Integration Steps

### Phase 1: Basic Integration (Required for Boot)

#### 1.1 Add StorageBus to Emulator
**File:** `crates/newton-core/src/emulator.rs`

```rust
use newton_devices::storage::StorageBus;

pub struct Emulator {
    // ... existing fields ...
    storage_bus: StorageBus,
}
```

#### 1.2 Initialize Storage in Emulator::new()
```rust
impl Emulator {
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        // ... existing initialization ...
        
        let mut storage_bus = StorageBus::new();
        
        // Attach default devices if specified in config
        if let Some(iso_path) = &config.boot_cd {
            let iso = IsoImage::open(iso_path)?;
            storage_bus.attach_scsi(3, Arc::new(RwLock::new(iso)))?;
        }
        
        if let Some(disk_path) = &config.boot_disk {
            let disk = RawDiskImage::open(disk_path)?;
            storage_bus.attach_scsi(0, Arc::new(RwLock::new(disk)))?;
        }
        
        Ok(Self {
            // ... existing fields ...
            storage_bus,
        })
    }
}
```

#### 1.3 Add Storage Configuration
**File:** `crates/newton-core/src/config.rs`

```rust
pub struct EmulatorConfig {
    // ... existing fields ...
    
    /// Boot CD/DVD ISO path
    pub boot_cd: Option<PathBuf>,
    
    /// Boot hard disk image path
    pub boot_disk: Option<PathBuf>,
}
```

#### 1.4 Register Storage Devices in Device Tree
**File:** `crates/newton-core/src/openfirmware/device_tree.rs`

```rust
fn create_storage_nodes(&mut self, storage_bus: &StorageBus) -> Result<()> {
    // Create SCSI controller node
    self.create_node("/pci@f2000000/scsi@18")?;
    
    for id in 0..8 {
        if let Some(device) = storage_bus.scsi_device(id) {
            let device_lock = device.device.read().unwrap();
            let info = device_lock.info();
            
            let device_type = match info.device_type {
                DeviceType::CdRom => "cdrom",
                DeviceType::HardDisk => "disk",
                _ => "device",
            };
            
            let path = format!("/pci@f2000000/scsi@18/{}@{}", device_type, id);
            self.create_node(&path)?;
            
            // Add properties
            self.set_property(&path, "name", Property::String(device_type.to_string()))?;
            self.set_property(&path, "device_type", Property::String("block".to_string()))?;
            self.set_property(&path, "block-size", Property::Int(info.block_size as i32))?;
            
            // Add capacity in blocks
            let blocks = info.size / info.block_size as u64;
            self.set_property(&path, "capacity", Property::Int(blocks as i32))?;
        }
    }
    
    Ok(())
}
```

### Phase 2: SCSI Controller Emulation

#### 2.1 Implement SCSI Controller MMIO
**File:** `crates/newton-devices/src/storage/scsi_controller.rs` (new)

```rust
pub struct ScsiController {
    base_addr: u32,
    storage_bus: Arc<RwLock<StorageBus>>,
    
    // Controller state
    current_target: u8,
    command_buffer: Vec<u8>,
    data_buffer: Vec<u8>,
    status: u8,
}

impl MmioDevice for ScsiController {
    fn read(&self, addr: u32, size: usize) -> Result<u64> {
        // Read SCSI controller registers
        match addr - self.base_addr {
            0x00 => Ok(self.status as u64),      // Status register
            0x04 => Ok(self.current_target as u64), // Target ID
            0x08 => {
                // Read data from buffer
                // ...
            }
            _ => Ok(0)
        }
    }
    
    fn write(&mut self, addr: u32, value: u64, size: usize) -> Result<()> {
        // Write SCSI controller registers
        match addr - self.base_addr {
            0x00 => {
                // Command register
                self.execute_scsi_operation(value as u8)?;
            }
            0x04 => {
                // Set target ID
                self.current_target = value as u8;
            }
            0x08 => {
                // Write data to buffer
                // ...
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### 2.2 Register SCSI Controller in MMIO Bus
**File:** `crates/newton-core/src/emulator.rs`

```rust
// Register SCSI controller at standard address
let scsi_controller = ScsiController::new(0x80010000, Arc::clone(&storage_bus));
mmio_bus.register(0x80010000, 0x1000, Box::new(scsi_controller))?;
```

### Phase 3: IDE/ATA Controller Emulation

#### 3.1 Implement IDE Controller
**File:** `crates/newton-devices/src/storage/ide_controller.rs` (new)

```rust
pub struct IdeController {
    base_cmd: u32,       // Command block registers
    base_ctrl: u32,      // Control block registers
    storage_bus: Arc<RwLock<StorageBus>>,
    
    // IDE state per channel
    channels: [IdeChannel; 2],
}

struct IdeChannel {
    selected_device: u8,  // 0=master, 1=slave
    sector_buffer: Vec<u8>,
    lba: u64,
    sector_count: u16,
    status: u8,
    error: u8,
}

impl MmioDevice for IdeController {
    fn read(&self, addr: u32, size: usize) -> Result<u64> {
        // Implement IDE register reads
        // 0x1F0-0x1F7: Primary channel command block
        // 0x3F6: Primary channel control/alternate status
        // 0x170-0x177: Secondary channel command block
        // 0x376: Secondary channel control/alternate status
    }
    
    fn write(&mut self, addr: u32, value: u64, size: usize) -> Result<()> {
        // Implement IDE register writes
    }
}
```

### Phase 4: Interrupt Support

#### 4.1 Generate Completion Interrupts
```rust
// After SCSI command completion
interrupt_controller.assert_irq(SCSI_IRQ)?;

// After IDE command completion
interrupt_controller.assert_irq(IDE_PRIMARY_IRQ)?;
```

#### 4.2 Wire to PIC/VIC
Ensure storage interrupts are connected to the PowerPC interrupt controller.

### Phase 5: Testing & Validation

#### 5.1 Unit Tests
- Test SCSI controller register access
- Test IDE controller register access
- Test device detection in OpenFirmware
- Test boot script execution with storage

#### 5.2 Integration Tests
- Boot from CD-ROM ISO
- Boot from hard disk image
- Install Mac OS to virtual disk
- Read/write operations during boot

#### 5.3 Test Cases
```rust
#[test]
fn test_scsi_device_discovery() {
    let mut emu = Emulator::new(config)?;
    
    // Check device tree has SCSI nodes
    let of = emu.openfirmware();
    assert!(of.find_node("/pci@f2000000/scsi@18/cdrom@3").is_some());
}

#[test]
fn test_boot_from_cd() {
    let config = EmulatorConfig {
        boot_cd: Some("disks/macos9_install.iso".into()),
        ..Default::default()
    };
    
    let mut emu = Emulator::new(config)?;
    emu.reset()?;
    emu.run_cycles(1000000)?;
    
    // Verify bootloader accessed CD-ROM
    // Check for expected output
}
```

## Implementation Priority

### High Priority (Required for Boot)
1. ✅ Storage bus and devices (done)
2. Device tree node registration
3. SCSI controller MMIO (basic)
4. Boot from CD-ROM support

### Medium Priority (Required for Installation)
1. IDE controller MMIO
2. Write support testing
3. Interrupt generation
4. Error handling improvements

### Low Priority (Nice to Have)
1. DMA support
2. Advanced SCSI commands
3. ATAPI support
4. Performance optimizations
5. Additional disk formats (QCOW2, VMDK)

## Timeline Estimate

- **Phase 1 (Basic Integration):** 2-3 hours
- **Phase 2 (SCSI Controller):** 3-4 hours  
- **Phase 3 (IDE Controller):** 4-6 hours
- **Phase 4 (Interrupts):** 1-2 hours
- **Phase 5 (Testing):** 2-3 hours

**Total:** ~15-20 hours of development time

## Dependencies

- OpenFirmware device tree fully functional ✅
- MMIO bus operational ✅
- Interrupt controller available ❓
- PCI bus emulation ❓

## Potential Issues

1. **MMIO Address Conflicts:** Ensure storage controllers don't conflict with other devices
2. **Timing:** Storage operations may need cycle-accurate timing
3. **Endianness:** PowerPC is big-endian; ensure proper byte order in registers
4. **DMA:** May need DMA controller for efficient transfers
5. **Bootloader Expectations:** ROM may expect specific SCSI/IDE initialization

## Success Criteria

✅ Storage devices appear in OpenFirmware device tree  
✅ Bootloader can query storage devices  
✅ Can read boot blocks from CD-ROM  
✅ Can install Mac OS to virtual hard disk  
✅ Can boot Mac OS from virtual hard disk  

## References

- **SCSI-2 Specification:** Commands and status codes
- **ATA/ATAPI-4 Specification:** IDE interface
- **Mac Toolbox:** SCSI Manager, IDE Driver
- **OpenFirmware Standard:** Device tree requirements
