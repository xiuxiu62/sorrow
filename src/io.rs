use x86_64::instructions::port::{PortRead, PortWrite};

pub unsafe fn insb(port: u16) -> u8 {
    u8::read_from_port(port)
}

pub unsafe fn insw(port: u16) -> u16 {
    u16::read_from_port(port)
}

pub unsafe fn insl(port: u16) -> u32 {
    u32::read_from_port(port)
}

pub unsafe fn outb(port: u16, val: u8) {
    u8::write_to_port(port, val);
}

pub unsafe fn outw(port: u16, val: u16) {
    u16::write_to_port(port, val);
}

pub unsafe fn outl(port: u16, val: u32) {
    u32::write_to_port(port, val);
}

