use alloc::vec::Vec;
use crate::io;

pub const SECTOR_SIZE: usize = 256;

pub struct Drive {
    id: usize,
    sector_size: usize,
}

impl Drive {
    pub fn new(id: usize, lba: u32, sector_size: usize) -> Self {
        Self {
            id,
            sector_size
        }
    }

    pub unsafe fn read_sector<'a>(
        &self,
        lba: usize,
        total: usize,
    ) -> Result<Vec<u16>, &'a str> {
        // Prepare to read
        io::outb(0x1f6, ((lba >> 24) | 0xe0) as u8); // Send drive and head numbers
        io::outb(0x1f2, total as u8); // Send number of sectors
        io::outb(0x1f3, (lba & 0xff) as u8); // Send bits 0-7 of LBA
        io::outb(0x1f4, (lba >> 8) as u8); // Send bits 8-15 of LBA
        io::outb(0x1f5, (lba >> 16) as u8); // Send bits 16-23 of LBA
        io::outb(0x1f7, 0x20); // Command port


        let mut buffer = vec![0; total * SECTOR_SIZE];
        for i in 0..total {
            // Wait until ready
            let mut byte = io::inb(0x1f7);
            while (byte & 0x08) != 0 {
                byte = io::inb(0x1f7);
            }

            // Read from disk
            for j in 0..self.sector_size {
                buffer[i * self.sector_size + j] = io::inw(0x1f0); 
            }
        }

        Ok(buffer)
    }
}

impl Default for Drive {
    fn default() -> Self {
        Self::new(0, 0, SECTOR_SIZE) 
    }
}