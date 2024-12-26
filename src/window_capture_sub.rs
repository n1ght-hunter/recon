use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use iced::widget::image;
use tracing::info;
use window_capture::{
    capture::{WindowsCaptureHandler, WindowsCaptureSettings},
    frame::{Frame, BGRA},
    monitor::Monitor,
};

use crate::Message;

struct Capture {
    fps: usize,
    last_output: Instant,
    last_frame: Instant,
    sender: Sender<(u32, u32, Vec<BGRA>)>,
}

impl WindowsCaptureHandler for Capture {
    type Flags = Sender<(u32, u32, Vec<BGRA>)>;

    fn new(sender: Self::Flags) -> Self {
        Self {
            fps: 0,
            last_output: Instant::now(),
            last_frame: Instant::now(),
            sender,
        }
    }

    fn on_frame_arrived(&mut self, frame: &Frame) {
        self.fps += 1;

        if self.last_output.elapsed().as_secs() >= 1 {
            println!("{}", self.fps);
            self.fps = 0;
            self.last_output = Instant::now();
        }

        if self.last_frame.elapsed().as_millis() > 1000 / 60 {
            let buffer = frame.buffer();
            if let Ok(buffer) = buffer {
                let pixels = buffer.get().to_vec();
                self.sender
                    .try_send((buffer.get_width(), buffer.get_height(), pixels));
            }
        }
    }

    fn on_closed(&mut self) {
        println!("Closed");
    }
}

struct Stopper(Arc<AtomicBool>);

impl Drop for Stopper {
    fn drop(&mut self) {
        info!("Stopping");
        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub fn create_window_capture_sub() -> iced::Subscription<Message> {
    iced::subscription::channel("window-capture", 100, |mut sender| async move {
        // let item = Window::from_window_name("Untitled - Notepad").unwrap();
        let item = Monitor::get_primary();
        let (raw_sender, mut raw_rev) =
            futures::channel::mpsc::channel::<(u32, u32, Vec<BGRA>)>(100);

        let stop = Arc::new(AtomicBool::new(false));

        let _stopper = Stopper(stop.clone());

        let settings = WindowsCaptureSettings {
            item: item.into(),
            capture_cursor: false,
            draw_border: false,
            flags: raw_sender,
        };

        tokio::task::spawn_blocking(move || {
            Capture::start(settings, stop).unwrap();
        });

        while let Some((width, height, pixels)) = raw_rev.next().await {
            
            let rgba = pixels
                .into_iter()
                .map(|bgra| {
                    let BGRA { b, g, r, a } = bgra;
                    [r, g, b, a]
                })
                .flatten()
                .collect::<Vec<u8>>();
            sender
                .send(Message::NewFrame(image::Handle::from_pixels(
                    width, height, rgba,
                )))
                .await
                .unwrap();
        }

        futures::future::pending().await
    })
}
