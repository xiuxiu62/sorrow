use crate::graphics::gop::writer::{Direction, TextWriter};
use alloc::format;
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
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

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
            .try_init_once(|| ArrayQueue::new(100))
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

pub async fn handle_keypresses(console: &mut TextWriter) {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                handle_keypress(console, key).await;
            }
        }
    }
}

async fn handle_keypress(console: &mut TextWriter, key: DecodedKey) {
    match key {
        DecodedKey::Unicode(key) => match key {
            '\n' => console.newline(),
            '\t' => console.write_str("    "),
            '\u{8}' => console.clear_last(),
            _ => console.write_char(key),
        },
        DecodedKey::RawKey(key) => match key {
            KeyCode::ArrowLeft => console.move_cursor(Direction::Left),
            KeyCode::ArrowRight => console.move_cursor(Direction::Right),
            KeyCode::ArrowUp => console.move_cursor(Direction::Up),
            KeyCode::ArrowDown => console.move_cursor(Direction::Down),
            _ => console.write_str(format!("{key:?}").as_str()),
        },
    }
}
