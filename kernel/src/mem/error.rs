use x86_64::{
    structures::paging::{mapper::MapToError, page_table, Size4KiB},
    VirtAddr,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Frame(FrameError),
    Map(MapToError<Size4KiB>),
    PhysicalMemoryOffset(PhysicalMemoryOffsetError),
}

impl From<FrameError> for Error {
    fn from(value: FrameError) -> Self {
        Self::Frame(value)
    }
}

impl From<MapToError<Size4KiB>> for Error {
    fn from(value: MapToError<Size4KiB>) -> Self {
        Self::Map(value)
    }
}

impl From<PhysicalMemoryOffsetError> for Error {
    fn from(value: PhysicalMemoryOffsetError) -> Self {
        Self::PhysicalMemoryOffset(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Frame(err) => write!(f, "{err}"),
            Self::Map(err) => write!(f, "{err:#?}"),
            Self::PhysicalMemoryOffset(err) => write!(f, "{err}"),
        }
    }
}

impl core::error::Error for Error {}

#[derive(Debug)]
pub struct FrameError {
    error: page_table::FrameError,
    address: VirtAddr,
}

impl FrameError {
    pub fn new(error: page_table::FrameError, address: VirtAddr) -> Self {
        Self { error, address }
    }
}

impl From<(page_table::FrameError, VirtAddr)> for FrameError {
    fn from((error, address): (page_table::FrameError, VirtAddr)) -> Self {
        Self { error, address }
    }
}

impl core::fmt::Display for FrameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let address = self.address.as_u64();
        let description = match self.error {
            page_table::FrameError::FrameNotPresent => "frame not present",
            page_table::FrameError::HugeFrame => "huge pages not supported",
        };

        write!(f, "Frame error at virtual address {address}: {description}")
    }
}

impl core::error::Error for FrameError {}

#[derive(Debug)]
pub struct PhysicalMemoryOffsetError;

impl core::fmt::Display for PhysicalMemoryOffsetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Physical memory offset not set")
    }
}

impl core::error::Error for PhysicalMemoryOffsetError {}
