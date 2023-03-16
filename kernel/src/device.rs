use x86_64::instructions::port::Port;

struct PciDevice<'pci> {
    pub bus: &'pci PciBus<'pci>,
    pub num: u8,
}

struct PciBus<'pci> {
    pub pci: &'pci dyn ConfigAccess,
    pub num: u8,
}

impl<'pci> PciBus<'pci> {
    pub fn devices(&'pci self) -> PciBusIter<'pci> {
        PciBusIter::new(self)
    }

    pub unsafe fn read(&self, dev: u8, func: u8, offset: u16) -> u32 {
        self.pci.read(self.num, dev, func, offset)
    }

    pub unsafe fn write(&self, dev: u8, func: u8, offset: u16, value: u32) {
        self.pci.write(self.num, dev, func, offset, value)
    }
}

struct PciBusIter<'pci> {
    bus: &'pci PciBus<'pci>,
    num: u8,
}

impl<'pci> PciBusIter<'pci> {
    pub fn new(bus: &'pci PciBus<'pci>) -> Self {
        PciBusIter { bus, num: 0 }
    }
}

impl<'pci> Iterator for PciBusIter<'pci> {
    type Item = PciDevice<'pci>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.num < 32 {
            let dev = PciDevice {
                bus: self.bus,
                num: self.num,
            };
            self.num += 1;

            return Some(dev);
        }

        None
    }
}

pub trait ConfigAccess {
    unsafe fn read_nolock(&self, bus: u8, dev: u8, func: u8, offset: u16) -> u32;
    unsafe fn read(&self, bus: u8, dev: u8, func: u8, offset: u16) -> u32;

    unsafe fn write_nolock(&self, bus: u8, dev: u8, func: u8, offset: u16, value: u32);
    unsafe fn write(&self, bus: u8, dev: u8, func: u8, offset: u16, value: u32);
}

// fn pci_config_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
//     let bus = bus as u32;
//     let slot = slot as u32;
//     let func = func as u32;

//     let address = bus << 16 | slot << 11 | func << 8 | (offset as u32 & 0xFC) | (0x90000000);
//     unsafe { Port::new(0xCf8).write(address) };

//     return unsafe { (Port::<u32>::new(0xCFC).read() >> ((offset & 2) * 8) & 0xFFFF) as u16 };
// }

// fn pci_check_vendor( bus: u8,  slot: u8) -> u16 {
//     uint16_t vendor, device;
//     let vendor: u16;
//     let device: u16;

//     /* Try and read the first configuration register. Since there are no
//      * vendors that == 0xFFFF, it must be a non-existent device. */
//     if ((vendor = pciConfigReadWord(bus, slot, 0, 0)) != 0xFFFF) {
//        device = pciConfigReadWord(bus, slot, 0, 2);
//        . . .
//     } return (vendor);
// }

fn check_device(bus: u8, device: u8) {
    let function = 0;
    // let vendor_id = get_vendor_id();
}

fn check_all_buses() {
    (0..=255).for_each(|bus| (0..32).for_each(|device| check_device(bus, device)));
}
