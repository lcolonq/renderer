use std::collections::HashMap;

use byteorder::ByteOrder;

pub type SharedTrackingState = std::sync::Arc<std::sync::Mutex<TrackingState>>;

pub struct TrackingState {
    pub expression_weights: HashMap<String, f32>,
    pub orientation: glam::Quat,
}

impl TrackingState {
    pub fn new() -> Self {
        Self {
            expression_weights: HashMap::from([
                ("happy".to_owned(), 0.0),
                ("angry".to_owned(), 0.0),
                ("sad".to_owned(), 0.0),
                ("relaxed".to_owned(), 0.0),
                ("surprised".to_owned(), 0.0),
                ("aa".to_owned(), 0.0),
                ("ih".to_owned(), 0.0),
                ("ou".to_owned(), 0.0),
                ("ee".to_owned(), 0.0),
                ("oh".to_owned(), 0.0),
                ("blink".to_owned(), 0.0),
            ]),
            orientation: glam::Quat::IDENTITY,
        }
    }

    pub fn run(sts: SharedTrackingState) {
        let blink_offset
            = 8 // timestamp
            + 4 // face id
            + 4 // width
            + 4 // height
            ;
        let quat_offset
            = blink_offset
            + 4 // left eye blink
            + 4 // right eye blink
            + 1 // success
            + 4 // pnp error
            ;
        let mouth_offset
            = quat_offset
            + 12 // euler
            + 12 // translation
            + (68 * (4 + 8 + 12)) // points
            + 16 // right gaze
            + 16 // left gaze
            + 4 // eye left
            + 4 // eye right
            + 4 // eyebrow steepness left
            + 4 // eyebrow updown left
            + 4 // eyebrow quirk left
            + 4 // eyebrow steepness right
            + 4 // eyebrow updown right
            + 4 // eyebrow quirk right
            + 4 // mouthcorner updown left
            + 4 // mouthcorner inout left
            + 4 // mouthcorner updown right
            + 4 // mouthcorner inout right
            ;
        std::thread::spawn(move || {
            let socket = std::net::UdpSocket::bind("127.0.0.1:11573").unwrap();
            let mut buf = [0; 65535];
            loop {
                socket.recv_from(&mut buf).unwrap();
                let left_blinking = byteorder::LittleEndian::read_f32(&buf[blink_offset..]);
                let right_blinking = byteorder::LittleEndian::read_f32(&buf[blink_offset+4..]);
                let mouth_open = byteorder::LittleEndian::read_f32(&buf[mouth_offset..]);
                let new = glam::Quat::from_xyzw(
                    byteorder::LittleEndian::read_f32(&buf[quat_offset..]),
                    -byteorder::LittleEndian::read_f32(&buf[quat_offset+4..]),
                    -byteorder::LittleEndian::read_f32(&buf[quat_offset+8..]),
                    byteorder::LittleEndian::read_f32(&buf[quat_offset+12..]),
                ).mul_quat(glam::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    std::f32::consts::PI,
                    0.0,
                    std::f32::consts::PI / 2.0,
                ));
                sts.lock().unwrap().orientation = new;
                sts.lock().unwrap().expression_weights.insert("blink".to_owned(), 1.0 - (left_blinking + right_blinking) / 2.0);
                sts.lock().unwrap().expression_weights.insert("oh".to_owned(), mouth_open.clamp(0.0, 1.0));
            }
        });
    }
}
