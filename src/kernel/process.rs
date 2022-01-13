use core::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ProcessId(u64);

impl ProcessId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

enum ProcessState {
    Start,
    Ready,
    Waiting,
    Running,
    Exit,
}

struct Process<'a> {
    id: ProcessId,
    parent: &'a Process<'a>,
    state: ProcessState,
    privileges: u8,
}
