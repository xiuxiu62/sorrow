use core::fmt::Display;
use thiserror::Error;
use x86_64::structures::paging::{mapper, page_table, PageSize, Size4KiB};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Frame(FrameError),
    #[error(transparent)]
    MapTo(MapToError<Size4KiB>),
    #[error(transparent)]
    PhysicalMemoryOffset(#[from] PhysicalMemoryOffsetError),
}

impl From<page_table::FrameError> for Error {
    fn from(value: page_table::FrameError) -> Self {
        Self::Frame(FrameError::from(value))
    }
}

impl From<mapper::MapToError<Size4KiB>> for Error {
    fn from(value: mapper::MapToError<Size4KiB>) -> Self {
        Self::MapTo(MapToError::from(value))
    }
}

#[derive(Debug, Error)]
pub struct FrameError(page_table::FrameError);

impl From<page_table::FrameError> for FrameError {
    fn from(value: page_table::FrameError) -> Self {
        Self(value)
    }
}

impl Display for FrameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let description = match self.0 {
            page_table::FrameError::FrameNotPresent => "frame not present",
            page_table::FrameError::HugeFrame => "huge pages not supported",
        };

        write!(f, "Frame error: {description}")
    }
}

#[derive(Debug, Error)]
#[error("{0:?}")]
pub struct MapToError<S: PageSize>(mapper::MapToError<S>);

impl<S: PageSize> From<mapper::MapToError<S>> for MapToError<S> {
    fn from(value: mapper::MapToError<S>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Error)]
#[error("Physical memory offset not set")]
pub struct PhysicalMemoryOffsetError;
