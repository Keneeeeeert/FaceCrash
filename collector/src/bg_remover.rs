use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn remove_background_python(
    input_rgb: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut child = Command::new("python")
        .args(["-c", REMBG_SCRIPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(&width.to_le_bytes())?;
        stdin.write_all(&height.to_le_bytes())?;
        stdin.write_all(input_rgb)?;
        stdin.flush()?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rembg failed: {}", stderr).into());
    }

    Ok(output.stdout)
}

pub fn next_filename(dir: &Path) -> String {
    let mut max_num = 2u32;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".png") {
                if let Ok(num) = name[..name.len() - 4].parse::<u32>() {
                    if num > max_num {
                        max_num = num;
                    }
                }
            }
        }
    }
    format!("{}.png", max_num + 1)
}

const REMBG_SCRIPT: &str = r#"
import sys, struct
import numpy as np
from PIL import Image
from rembg import remove
import cv2

w = struct.unpack('<I', sys.stdin.buffer.read(4))[0]
h = struct.unpack('<I', sys.stdin.buffer.read(4))[0]
size = w * h * 3
rgb = sys.stdin.buffer.read(size)

img = np.frombuffer(rgb, dtype=np.uint8).reshape((h, w, 3))
img_bgr = cv2.cvtColor(img, cv2.COLOR_RGB2BGR)
result = remove(img_bgr)
result_rgb = cv2.cvtColor(result, cv2.COLOR_BGRA2RGBA)
sys.stdout.buffer.write(result_rgb.tobytes())
"#;
