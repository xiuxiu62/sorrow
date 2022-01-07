use crate::{graphics::gop::Direction, move_cursor, print};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{
    layouts, DecodedKey, HandleControl, KeyCode, KeyboardLayout, ScancodeSet, ScancodeSet1,
};

static WAKER: AtomicWaker = AtomicWaker::new();
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
const SCANCODE_QUEUE_SIZE: usize = 100;

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {

            // println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        // println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        if let Some(scancode) = queue.pop() {
            WAKER.take();
            return Poll::Ready(Some(scancode));
        }

        return Poll::Pending;
    }
}

pub struct Keyboard<L: KeyboardLayout, S: ScancodeSet> {
    device: pc_keyboard::Keyboard<L, S>,
    stream: ScancodeStream,
}

impl<L: KeyboardLayout, S: ScancodeSet> Keyboard<L, S> {
    pub fn new(layout: L, scancode_set: S, handle_control: HandleControl) -> Self {
        Self {
            device: pc_keyboard::Keyboard::new(layout, scancode_set, handle_control),
            stream: ScancodeStream::new(),
        }
    }

    pub async fn listen(&mut self) {
        while let Some(scancode) = self.stream.next().await {
            if let Ok(Some(key_event)) = self.device.add_byte(scancode) {
                if let Some(key) = self.device.process_keyevent(key_event) {
                    self.handle_keypress(key).await;
                }
            }
        }
    }

    async fn handle_keypress(&self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode(key) => print!("{key}"),
            DecodedKey::RawKey(key) => match key {
                KeyCode::ArrowLeft => move_cursor!(Direction::Left),
                KeyCode::ArrowRight => move_cursor!(Direction::Right),
                KeyCode::ArrowUp => move_cursor!(Direction::Up),
                KeyCode::ArrowDown => move_cursor!(Direction::Down),
                _ => print!("{key:?}"),
            },
        }
    }
}

pub fn init_us_keyboard() -> Keyboard<layouts::Us104Key, ScancodeSet1> {
    Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
}
