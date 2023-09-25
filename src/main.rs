use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;
use nokhwa::{pixel_format::RgbFormat, query, utils::{ApiBackend, RequestedFormat, RequestedFormatType}, CallbackCamera};
use show_image::{create_window, ImageInfo, ImageView};

#[show_image::main]
fn main() -> Result<(), String> {
    let cameras = query(ApiBackend::Auto).unwrap();
    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let first_camera = cameras.first().unwrap();
    let (tx, rx) = channel();
    let window = create_window("image", Default::default()).expect("Window creation failed.");

    window.context_proxy().run_function_wait(|handle| {
        handle.set_exit_with_last_window(true);
    });

    let mut threaded = CallbackCamera::new(first_camera.index().clone(), format, move |buffer| {
        tx.send(buffer).expect("Failed to send buffer");
    }).unwrap();

    let receiver = thread::spawn(move || {
        let mut count = 0;
        let mut start = Instant::now();
        while let Ok(value) = rx.recv() {
            let image = value.decode_image::<RgbFormat>().unwrap();
            let image = ImageView::new(ImageInfo::rgb8(1280, 720), image.iter().as_slice());

            window.set_image("iMAgE", image).unwrap_or_default();

            if start.elapsed().as_secs() > 1 {
                println!("Frames in the last sec : {:?}", count);
                start = Instant::now();
                count = 0;
            }

            count = count + 1;
        }
    });

    threaded.open_stream().unwrap();
    receiver.join().expect("The receiver thread has panicked");

    Ok(())
}