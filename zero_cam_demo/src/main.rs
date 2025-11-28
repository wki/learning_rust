#[allow(dead_code)]
use std::time::Duration;
use libcamera::camera::CameraConfigurationStatus;
use libcamera::camera_manager::CameraManager;
use libcamera::framebuffer::AsFrameBuffer;
use libcamera::framebuffer_allocator::{FrameBuffer, FrameBufferAllocator};
use libcamera::framebuffer_map::MemoryMappedFrameBuffer;
use libcamera::pixel_format::PixelFormat;
use libcamera::stream::StreamRole;

// see https://libcamera.org/api-html/build_2include_2libcamera_2formats_8h_source.html
const PIXEL_FORMAT_MJPEG: PixelFormat = PixelFormat::new(u32::from_le_bytes([b'M', b'J', b'P', b'G']), 0);
const PIXEL_FORMAT_RGB565: PixelFormat = PixelFormat::new(u32::from_le_bytes([b'R', b'G', b'1', b'6']), 0);
const PIXEL_FORMAT_RGB888: PixelFormat = PixelFormat::new(u32::from_le_bytes([b'R', b'G', b'2', b'4']), 0);

fn main() {
    let mgr = CameraManager::new().unwrap();

    let cameras = mgr.cameras();
    let cam = cameras.get(0).expect("No cameras found");
    let mut cam = cam.acquire().expect("Unable to acquire camera");

    println!("Properties: {:#?}", cam.properties());
    // println!("Controls: {:#?}", cam.controls());

    // This will generate default configuration for each specified role
    let mut cfgs = cam.generate_configuration(&[StreamRole::ViewFinder]).unwrap();

    let view_finder_cfg = cfgs.get(0).unwrap();
    println!("Available formats: {:#?}", view_finder_cfg.formats());

    cfgs.get_mut(0).unwrap().set_pixel_format(PIXEL_FORMAT_RGB888);

    println!("Generated config: {cfgs:#?}");

    match cfgs.validate() {
        CameraConfigurationStatus::Valid => println!("Camera configuration valid!"),
        CameraConfigurationStatus::Adjusted => println!("Camera configuration was adjusted: {cfgs:#?}"),
        CameraConfigurationStatus::Invalid => panic!("Error validating camera configuration"),
    }

    // // Ensure that pixel format was unchanged
    // assert_eq!(
    //     cfgs.get(0).unwrap().get_pixel_format(),
    //     PIXEL_FORMAT_MJPEG,
    //     "MJPEG is not supported by the camera"
    // );

    cam.configure(&mut cfgs).unwrap();
    println!("Camera configuration complete!");

    let mut alloc = FrameBufferAllocator::new(&cam);

    // Allocate frame buffers for the stream
    let cfg = cfgs.get(0).unwrap();
    let stream = cfg.stream().unwrap();
    let buffers = alloc.alloc(&stream).unwrap();
    println!("Allocated {} buffers", buffers.len());

    // Convert FrameBuffer to MemoryMappedFrameBuffer, which allows reading &[u8]
    let buffers = buffers
        .into_iter()
        .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
        .collect::<Vec<_>>();

    // Create capture requests and attach buffers
    let mut reqs = buffers
        .into_iter()
        .map(|buf| {
            let mut req = cam.create_request(None).unwrap();
            req.add_buffer(&stream, buf).unwrap();
            req
        })
        .collect::<Vec<_>>();

    // Completed capture requests are returned as a callback
    let (tx, rx) = std::sync::mpsc::channel();
    cam.on_request_completed(move |req| {
        tx.send(req).unwrap();
    });

    cam.start(None).unwrap();

    // Multiple requests can be queued at a time, but for this example we just want a single frame.
    cam.queue_request(reqs.pop().unwrap()).unwrap();

    println!("Waiting for camera request execution");
    let req = rx.recv_timeout(Duration::from_secs(2)).expect("Camera request failed");

    println!("Camera request {req:?} completed!");
    println!("Metadata: {:#?}", req.metadata());
    // println!("Status: {:?}", req.status());

    // Get framebuffer for our stream
    let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> = req.buffer(&stream).unwrap();
    println!("FrameBuffer metadata: {:#?}", framebuffer.metadata());

    // MJPEG format has only one data plane containing encoded jpeg data with all the headers
    let planes = framebuffer.data();
    let jpeg_data = planes.first().unwrap();
    // Actual JPEG-encoded data will be smalled than framebuffer size, its length can be obtained from metadata.
    let jpeg_len = framebuffer.metadata().unwrap().planes().get(0).unwrap().bytes_used as usize;

    let filename = "image.jpg".to_string();
    std::fs::write(&filename, &jpeg_data[..jpeg_len]).unwrap();
    println!("Written {} bytes to {}", jpeg_len, &filename);
}

// use v4l::buffer::Type;
// use v4l::{Device, FourCC};
// use v4l::io::traits::CaptureStream;
// use v4l::prelude::MmapStream;
// use v4l::video::Capture;
//
// fn main() {
//     println!("Hello, world!");
//
//     let mut dev = Device::new(0).expect("Failed to open device");
//     let mut fmt = dev.format().expect("Failed to read format");
//     println!("Format: {:?}", fmt);
//     fmt.width = 640;
//     fmt.height = 480;
//     fmt.fourcc = FourCC::new(b"YUYV");
//     dev.set_format(&fmt).expect("Failed to set format");
//
//     // let mut stream = Stream::with_buffers(&mut dev, Type::VideoCapture, 4)
//     //     .expect("Failed to create buffer stream");
//     let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)
//         .expect("Failed to create buffer stream");
//
//     loop {
//         println!("reading stream...");
//         let (buf, meta) = stream.next().unwrap();
//         println!(
//             "Buffer size: {}, seq: {}, timestamp: {}",
//             buf.len(),
//             meta.sequence,
//             meta.timestamp
//         );
//
//         // To process the captured data, you can pass it somewhere else.
//         // If you want to modify the data or extend its lifetime, you have to
//         // copy it. This is a best-effort tradeoff solution that allows for
//         // zero-copy readers while enforcing a full clone of the data for
//         // writers.
//
//         break;
//     }
// }
