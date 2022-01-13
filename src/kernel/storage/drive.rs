use crate::io;
use alloc::vec::Vec;

pub const SECTOR_SIZE: usize = 256;

pub struct Drive {
    _id: usize,
    sector_size: usize,
}

impl Drive {
    pub fn new(_id: usize, sector_size: usize) -> Self {
        Self { _id, sector_size }
    }

    /// Reads `total` sectors, starting from `lba` and returning a vector.
    pub fn read(&self, lba: usize, total: usize) -> Vec<u16> {
        let mut buffer = vec![0_u16; total * self.sector_size];
        self.read_to_buffer(lba, total, &mut buffer);
        buffer
    }

    /// Reads `total` sectors, starting from `lba` and reading into the provided buffer.
    pub fn read_to_buffer(&self, lba: usize, total: usize, buffer: &mut [u16]) {
        // Prepare to read
        unsafe {
            io::outb(0x1f6, ((lba >> 24) | 0xe0) as u8); // Send drive and head numbers
            io::outb(0x1f2, total as u8); // Send number of sectors
            io::outb(0x1f3, (lba & 0xff) as u8); // Send bits 0-7 of LBA
            io::outb(0x1f4, (lba >> 8) as u8); // Send bits 8-15 of LBA
            io::outb(0x1f5, (lba >> 16) as u8); // Send bits 16-23 of LBA
            io::outb(0x1f7, 0x20); // Command port
        };

        for i in 0..total {
            // Wait until ready
            let mut byte = unsafe { io::inb(0x1f7) };
            while (byte & 0x08) != 0 {
                byte = unsafe { io::inb(0x1f7) };
            }

            // Read from disk
            for j in 0..self.sector_size {
                buffer[i * self.sector_size + j] = unsafe { io::inw(0x1f0) };
            }
        }
    }
}

impl Default for Drive {
    fn default() -> Self {
        Self::new(0, SECTOR_SIZE)
    }
}
