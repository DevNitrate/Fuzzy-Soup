use std::{f32::{consts::PI}, fs::File, io::{BufRead, BufReader, Read}, path::{Path, PathBuf}};

use bevy::{math::{Quat, Vec3}};

#[derive(Debug)]
pub struct ColmapCamera {
    pub camera_id: u32,
    pub width: u64,
    pub height: u64,
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64
}

#[derive(Debug)]
pub struct ColmapImage {
    // pub image_id: u32,
    pub camera_id: u32,
    pub image_path: PathBuf,
    pub camera_orientation: Quat,
    pub camera_pos: Vec3,
}

#[derive(Debug)]
pub struct ColmapPoint {
    pub pos: Vec3
}

pub struct ColmapScene {
    pub images: Vec<ColmapImage>,
    pub cameras: Vec<ColmapCamera>,
    pub points: Vec<ColmapPoint>
}

impl ColmapScene {
    pub fn new(colmap: &str) -> Self {
        let colmap_path: &Path = Path::new(colmap);
        let sparse_path: PathBuf = colmap_path.join("sparse");
        let images_path: PathBuf = colmap_path.join("images");
        let f_cam: File = File::open(sparse_path.join("cameras.bin")).unwrap();
        let mut cam_reader = BufReader::new(f_cam);

        let mut cam_num_buf: [u8; 8] = [123; 8];
        cam_reader.read_exact(&mut cam_num_buf).unwrap();
        let cam_num: u64 = u64::from_le_bytes(cam_num_buf);

        let mut cameras: Vec<ColmapCamera> = Vec::new();

        for _ in 0..cam_num {
            let mut cam_buf: [u8; 56] = [123; 56];
            cam_reader.read_exact(&mut cam_buf).expect("cameras.bin corrupted or does not contain PINHOLE camera");
            
            let camera_id: u32 = u32::from_le_bytes(*(cam_buf[0..4].as_array().unwrap()));
            let model_id: i32 = i32::from_le_bytes(*(cam_buf[4..8].as_array().unwrap()));
            let width: u64 = u64::from_le_bytes(*(cam_buf[8..16].as_array().unwrap()));
            let height: u64 = u64::from_le_bytes(*(cam_buf[16..24].as_array().unwrap()));
            let fx: f64 = f64::from_le_bytes(*(cam_buf[24..32].as_array().unwrap()));
            let fy: f64 = f64::from_le_bytes(*(cam_buf[32..40].as_array().unwrap()));
            let cx: f64 = f64::from_le_bytes(*(cam_buf[40..48].as_array().unwrap()));
            let cy: f64 = f64::from_le_bytes(*(cam_buf[48..56].as_array().unwrap()));

            if model_id != 1 {
                panic!("cameras.bin must contain only PINHOLE cameras. consider undistorting the images")
            }

            cameras.push(ColmapCamera {
                camera_id,
                width,
                height,
                fx,
                fy,
                cx,
                cy
            });
        }

        let f_img: File = File::open(sparse_path.join("images.bin")).unwrap();
        let mut img_reader = BufReader::new(f_img);

        let mut img_num_buf: [u8; 8] = [123; 8];
        img_reader.read_exact(&mut img_num_buf).unwrap();
        let img_num: u64 = u64::from_le_bytes(img_num_buf);

        let mut images: Vec<ColmapImage> = Vec::new();

        let flip: Quat = Quat::from_rotation_x(PI);

        for _ in 0..img_num {
            let mut img_header_buf: [u8; 64] = [123; 64];
            img_reader.read_exact(&mut img_header_buf).unwrap();

            let _image_id: u32 = u32::from_le_bytes(*(img_header_buf[0..4].as_array().unwrap()));
            let qw: f32 = f64::from_le_bytes(*(img_header_buf[4..12].as_array().unwrap())) as f32;
            let qx: f32 = f64::from_le_bytes(*(img_header_buf[12..20].as_array().unwrap())) as f32;
            let qy: f32 = f64::from_le_bytes(*(img_header_buf[20..28].as_array().unwrap())) as f32;
            let qz: f32 = f64::from_le_bytes(*(img_header_buf[28..36].as_array().unwrap())) as f32;
            let tx: f64 = f64::from_le_bytes(*(img_header_buf[36..44].as_array().unwrap()));
            let ty: f64 = f64::from_le_bytes(*(img_header_buf[44..52].as_array().unwrap()));
            let tz: f64 = f64::from_le_bytes(*(img_header_buf[52..60].as_array().unwrap()));
            let camera_id: u32 = u32::from_le_bytes(*(img_header_buf[60..64].as_array().unwrap()));

            let mut img_path_buf: Vec<u8> = Vec::new();
            img_reader.read_until(0, &mut img_path_buf).unwrap();
            img_path_buf.pop();
            let img_path = String::from_utf8(img_path_buf).unwrap();
            let image_path = images_path.join(img_path).canonicalize().unwrap();
            
            let mut num_points_buf: [u8; 8] = [123; 8];
            img_reader.read_exact(&mut num_points_buf).unwrap();
            let num_points: u64 = u64::from_le_bytes(*(num_points_buf[0..8].as_array().unwrap()));

            let mut phony_buf: Vec<u8> = vec![0u8; (num_points as usize)*24];
            img_reader.read_exact(&mut phony_buf).unwrap();

            let r_w2c: Quat = Quat::from_xyzw(qx, qy, qz, qw);
            let r_c2w: Quat = r_w2c.conjugate();
            let t: Vec3 = Vec3::new(tx as f32, ty as f32, tz as f32);
            let cam_pos: Vec3 = -(r_c2w * t);

            let camera_orientation: Quat = flip * r_c2w * flip.conjugate();
            let camera_pos: Vec3 = flip * cam_pos;

            images.push(ColmapImage {
                camera_id,
                image_path,
                camera_orientation,
                camera_pos
            });
        }

        let mut min: Vec3 = Vec3::splat(f32::INFINITY);
        let mut max: Vec3 = Vec3::splat(f32::NEG_INFINITY);

        for img in &images {
            min = min.min(img.camera_pos);
            max = max.max(img.camera_pos);
        }

        let center: Vec3 = (min + max) * 0.5;

        let radius: f32 = images.iter().map(|img| (img.camera_pos - center).length()).fold(0.0f32, f32::max);

        for img in &mut images {
            img.camera_pos = (img.camera_pos - center) / radius;
        }

        let f_p: File = File::open(sparse_path.join("points3D.bin")).unwrap();
        let mut p_reader = BufReader::new(f_p);

        let mut p_num_buf: [u8; 8] = [123; 8];
        p_reader.read_exact(&mut p_num_buf).unwrap();
        let p_num: u64 = u64::from_le_bytes(p_num_buf);

        let mut points: Vec<ColmapPoint> = Vec::new();

        for _ in 0..p_num {
            let mut p_header_buf: [u8; 51] = [123; 51];
            p_reader.read_exact(&mut p_header_buf).unwrap();

            let x: f64 = f64::from_le_bytes(*(p_header_buf[8..16].as_array().unwrap()));
            let y: f64 = f64::from_le_bytes(*(p_header_buf[16..24].as_array().unwrap()));
            let z: f64 = f64::from_le_bytes(*(p_header_buf[24..32].as_array().unwrap()));
            let track_len: u64 = u64::from_le_bytes(*(p_header_buf[43..51].as_array().unwrap()));

            let mut phony_buf: Vec<u8> = vec![0u8; (track_len as usize)*8];
            p_reader.read_exact(&mut phony_buf).unwrap();

            let xn: f32 = ((x - center.x as f64) / radius as f64) as f32;
            let yn: f32 = ((-y - center.y as f64) / radius as f64) as f32;
            let zn: f32 = ((-z - center.z as f64) / radius as f64) as f32;

            let pos: Vec3 = Vec3::new(xn, yn, zn);

            points.push(ColmapPoint {
                pos
            });
        }

        ColmapScene {
            cameras,
            images,
            points
        }
    }
}
