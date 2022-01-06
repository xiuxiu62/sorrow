use crate::io::{insb, insw, outb};

pub struct Drive(u32);

impl Drive {
    pub fn new(lba: u32) -> Self {
        Self(lba)
    }

    pub unsafe fn read_sector<'a>(
        &self,
        total: usize,
        buffer: &'a mut [u16],
    ) -> Result<(), &'a str> {
        let lba = self.0;

        // Prepare to read
        outb(0x1f6, ((lba >> 24) | 0xe0) as u8); // Send drive and head numbers
        outb(0x1f2, total as u8); // Send number of sectors
        outb(0x1f3, (lba & 0xff) as u8); // Send bits 0-7 of LBA
        outb(0x1f4, (lba >> 8) as u8); // Send bits 8-15 of LBA
        outb(0x1f5, (lba >> 16) as u8); // Send bits 16-23 of LBA
        outb(0x1f7, 0x20); // Command port

        (0..total).into_iter().for_each(|i| {
            // Wait for the buffer to be ready
            let mut byte = insb(0x1f7);
            while (byte & 0x08) != 0 {
                byte = insb(0x1f7);
            }

            // Copy from disk to memory
            let sector_size = 256;
            (0..sector_size)
                .into_iter()
                .for_each(|j| buffer[i * sector_size + j] = insw(0x1f0));
        });

        Ok(())
    }
}
