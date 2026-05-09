use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};

pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct CameraSource {
    camera: Camera,
}

impl CameraSource {
    pub fn new(_width: u32, _height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let index = CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        let mut camera = Camera::new(index, requested)?;
        camera.open_stream()?;

        Ok(Self { camera })
    }

    pub fn poll_frame(&mut self) -> Option<Frame> {
        match self.camera.frame() {
            Ok(frame) => {
                if let Ok(decoded) = frame.decode_image::<RgbFormat>() {
                    let w = decoded.width();
                    let h = decoded.height();
                    let data = decoded.into_raw();
                    Some(Frame {
                        data,
                        width: w,
                        height: h,
                    })
                } else {
                    None
                }
            }
            Err(e) => {
                eprintln!("Camera frame error: {:?}", e);
                None
            }
        }
    }
}
