/// Standard CHIP-8 font set (hex digits 0-F)
/// Each digit is 5 bytes representing an 8x5 pixel sprite
const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Memory address where font sprites are loaded
pub const FONT_START_ADDRESS: usize = 0x50;

const RAM_SIZE: usize = 4096;

/// Represents the CHIP-8's 4KB of RAM.
///
/// The memory map is as follows:
/// - `0x000-0x1FF`: Chip-8 interpreter (contains font set in emu)
/// - `0x050-0x0A0`: Used for the built in 4x5 pixel font set (0-F). See [FONT_SET].
/// - `0x200-0xFFF`: Program ROM and work RAM. See `crate::consts::ROM_START_ADDRESS`.
pub struct Memory {
    ram: [u8; RAM_SIZE],
}

#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("unrecoverable error: {0}")]
    Unrecoverable(String),
    #[error("out of memory")]
    OutOfMemory,
}

impl Memory {
    /// Creates a new `Memory` instance.
    ///
    /// This initializes the RAM with zeros and loads the font set into the appropriate memory region
    /// by calling [`Memory::load_font()`].
    ///
    /// # Errors
    ///
    /// Returns `MemoryError` if the font set cannot be loaded, though this is unlikely
    /// under normal circumstances as the font set and its location are fixed. See [MemoryError].
    pub fn try_new() -> Result<Self, MemoryError> {
        let mut mem = Memory { ram: [0; RAM_SIZE] };
        mem.load_font()?;
        Ok(mem)
    }

    /// Reads a single byte from a given memory address.
    ///
    /// # Parameters
    ///
    /// - `address`: The memory address to read from.
    ///
    /// # Returns
    ///
    /// Returns `Some(u8)` with the value if the address is valid, or `None` if the address
    /// is out of bounds.
    pub fn read_byte(&self, address: usize) -> Option<u8> {
        self.ram.get(address).copied()
    }

    pub fn read_word(&self, address: usize) -> Option<u16> {
        self.ram
            .get(address..address + 2)
            .map(|bytes| ((bytes[0] as u16) << 8) | bytes[1] as u16)
    }

    /// Writes a single byte to a given memory address.
    ///
    /// # Parameters
    ///
    /// - `address`: The memory address to write to.
    /// - `value`: The byte value to write.
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfMemory` if the address is out of bounds.
    pub fn write_byte(&mut self, address: usize, value: u8) -> Result<(), MemoryError> {
        if address >= RAM_SIZE {
            return Err(MemoryError::OutOfMemory);
        }
        self.ram[address] = value;
        Ok(())
    }

    /// Writes a slice of bytes to memory starting at a given offset.
    ///
    /// # Parameters
    ///
    /// - `buf`: The slice of bytes to write.
    /// - `offset`: The starting memory address to write to.
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::OutOfMemory` if writing the buffer would exceed the
    /// available RAM size ([RAM_SIZE]).
    pub fn write_at(&mut self, buf: &[u8], offset: usize) -> Result<(), MemoryError> {
        if offset + buf.len() > RAM_SIZE {
            return Err(MemoryError::OutOfMemory);
        }
        self.ram[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    /// Returns an immutable slice of memory.
    ///
    /// This method is a wrapper around [`slice::get()`].
    pub fn get(&self, index: impl std::slice::SliceIndex<[u8], Output = [u8]>) -> Option<&[u8]> {
        self.ram.get(index)
    }

    /// Loads the font set into memory.
    ///
    /// It writes the [FONT_SET] data to the [FONT_START_ADDRESS].
    fn load_font(&mut self) -> Result<(), MemoryError> {
        self.write_at(&FONT_SET, FONT_START_ADDRESS)
    }

    /// Returns the total size of the RAM, which is [RAM_SIZE].
    pub fn size(&self) -> usize {
        RAM_SIZE
    }

    /// Checks if a given address is within the valid memory bounds (less than [RAM_SIZE]).
    pub fn is_valid_address(&self, address: usize) -> bool {
        address < RAM_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_new_loads_font() {
        let memory = Memory::try_new().unwrap();
        // Check if a portion of the font set is loaded correctly.
        // FONT_SET for '0' is 0xF0, 0x90, 0x90, 0x90, 0xF0
        assert_eq!(
            memory.get(FONT_START_ADDRESS..FONT_START_ADDRESS + FONT_SET.len()),
            Some(FONT_SET.as_slice())
        );
    }

    #[test]
    fn test_read_and_write_byte() {
        let mut memory = Memory::try_new().unwrap();
        let addr = 0x200;
        let value = 0xAB;

        // Test successful write and read
        assert!(memory.write_byte(addr, value).is_ok());
        assert_eq!(memory.read_byte(addr), Some(value));

        // Test reading from an uninitialized address
        assert_eq!(memory.read_byte(addr + 1), Some(0x00));

        // Test writing to an out-of-bounds address
        let out_of_bounds_addr = RAM_SIZE;
        let result = memory.write_byte(out_of_bounds_addr, value);
        assert!(matches!(result, Err(MemoryError::OutOfMemory)));

        // Test reading from an out-of-bounds address
        assert_eq!(memory.read_byte(out_of_bounds_addr), None);
    }

    #[test]
    fn test_read_word() {
        let mut memory = Memory::try_new().unwrap();
        memory.write_byte(0x200, 0xAB).unwrap();
        memory.write_byte(0x201, 0xCD).unwrap();
        assert_eq!(memory.read_word(0x200), Some(0xABCD));
    }

    #[test]
    fn test_write_at() {
        let mut memory = Memory::try_new().unwrap();
        let offset = 0x300;
        let data = [0xDE, 0xAD, 0xBE, 0xEF];

        // Test successful write
        assert!(memory.write_at(&data, offset).is_ok());
        assert_eq!(
            memory.get(offset..offset + data.len()),
            Some(data.as_slice())
        );

        // Test writing out of bounds
        let large_data = vec![0u8; 10];
        let result = memory.write_at(&large_data, RAM_SIZE - 5);
        assert!(matches!(result, Err(MemoryError::OutOfMemory)));

        // Verify that the memory was not partially written
        assert_eq!(memory.read_byte(RAM_SIZE - 5), Some(0x00));
    }

    #[test]
    fn test_get() {
        let mut memory = Memory::try_new().unwrap();
        let addr = 0x500;
        let data = [1, 2, 3, 4];
        memory.write_at(&data, addr).unwrap();

        // Test get
        assert_eq!(memory.get(addr..addr + data.len()), Some(data.as_slice()));
        assert_eq!(memory.get(RAM_SIZE..), Some(&[] as &[u8]));
        assert_eq!(memory.get(RAM_SIZE + 1..), None);
        assert_eq!(memory.get(RAM_SIZE - 2..RAM_SIZE + 1), None);
    }

    #[test]
    fn test_size() {
        let memory = Memory::try_new().unwrap();
        assert_eq!(memory.size(), RAM_SIZE);
    }

    #[test]
    fn test_is_valid_address() {
        let memory = Memory::try_new().unwrap();
        assert!(memory.is_valid_address(0));
        assert!(memory.is_valid_address(RAM_SIZE - 1));
        assert!(!memory.is_valid_address(RAM_SIZE));
        assert!(!memory.is_valid_address(RAM_SIZE + 1));
    }
}
