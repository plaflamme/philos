use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::pin::Pin;
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::{AtomicWaker, Context, Poll};
use futures_util::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};

// like lazy_static! but will not initialize within interrupts
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue not initialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("unable to initialize scancode queue");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let q = SCANCODE_QUEUE.get().unwrap();

        if let Some(code) = q.pop() {
            return Poll::Ready(Some(code));
        }

        WAKER.register(&cx.waker());

        match q.pop() {
            Some(code) => {
                WAKER.take();
                Poll::Ready(Some(code))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut stream = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        pc_keyboard::layouts::Us104Key,
        ScancodeSet1,
        HandleControl::Ignore,
    );

    while let Some(scancode) = stream.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
