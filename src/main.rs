use image::{ImageBuffer, Rgb, RgbImage};
use lerp::Lerp;
use std::env;
use std::fs;
use std::io::{stdout, Write};
use std::path::Path;
use std::process::Command;

fn main() {
    const ESCAPE_RADIUS: f64 = 4.0;
    const ZOOM_FACTOR: f64 = 0.995;
    const MAX_ITERATIONS: u16 = 500;
    const C_R: f64 =  -1.74910052372728893139257190897519162350086705009084014932879952538357036521448572336655187497147882098099098703936165441385416738949265;
    const C_I: f64 = -0.00034781681040860848288201654149008579426487640620291842473412498756596692238612317351024836599060137028311952298078246601736366418880955;
    const COLOR_0: (f32, f32, f32) = (16.0, 25.0, 53.0);
    const COLOR_1: (f32, f32, f32) = (194.0, 30.0, 86.0);
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    const FRAMERATE: u64 = 24;
    const FRAMES: u64 = 600;
    const TEMP_DIR_NAME: &str = "julia-demo-temp";
    const OUTPUT_FILE: &str = "julia-demo.mp4";

    // verify that ffmpeg is installed
    if Command::new("ffmpeg").output().is_err() {
        println!("missing dependency : ffmpeg");
        return;
    }

    let mut max_x = 2.0;
    let mut max_y = max_x * HEIGHT as f64 / WIDTH as f64;
    let mut x;
    let mut y;
    let mut iteration;
    let temp_dir = env::current_dir()
        .expect("error accessing current directory")
        .as_path()
        .join(TEMP_DIR_NAME);

    // verify that output file doesn't exist
    if Path::new(OUTPUT_FILE).is_file() {
        eprintln!("{}", format!("file {} already exists", OUTPUT_FILE));
        return;
    }

    // verify that temp directory doesn't exist
    if temp_dir.is_dir() {
        eprintln!("{}", format!("directory {} already exists", TEMP_DIR_NAME));
        return;
    } else {
        fs::create_dir(TEMP_DIR_NAME).expect("error creating directory");
    }

    // for each frame
    for frame_index in 0..FRAMES {
        let mut frame: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

        // apply zoom
        max_x *= ZOOM_FACTOR;
        max_y *= ZOOM_FACTOR;

        // for each pixel in the image
        for (pixel_x, pixel_y, pixel) in frame.enumerate_pixels_mut() {
            // map x and y to the right coordinates
            x = max_x * (2.0 * (pixel_x as f64 + 1.0) / WIDTH as f64 - 1.0);
            y = max_y * (2.0 * (pixel_y as f64 + 1.0) / HEIGHT as f64 - 1.0);

            iteration = 0;

            // the actual program is 2 lines lmao
            while x * x + y * y < ESCAPE_RADIUS && iteration < MAX_ITERATIONS {
                (x, y) = (x * x - y * y + C_R, 2.0 * x * y + C_I);
                iteration += 1;
            }

            // set the color of the pixel depending
            // on the number of iterations
            *pixel = Rgb([
                COLOR_0
                    .0
                    .lerp(COLOR_1.0, iteration as f32 / MAX_ITERATIONS as f32)
                    as u8,
                COLOR_0
                    .1
                    .lerp(COLOR_1.1, iteration as f32 / MAX_ITERATIONS as f32)
                    as u8,
                COLOR_0
                    .2
                    .lerp(COLOR_1.2, iteration as f32 / MAX_ITERATIONS as f32)
                    as u8,
            ]);
        }

        // write file to temp folder
        loop {
            if let Ok(_) = frame.save(temp_dir.join(format!("{}.png", frame_index))) {
                break;
            } else {
                eprintln!("error writing temporary file, retrying...");
            }
        }

        // print progress
        print!("\r{}%", (frame_index as f32 / FRAMES as f32 * 100.0) as u8);
        let _ = stdout().flush();
    }

    println!("\rsaving video...");

    // quick and dirty hack to save video with ffmpeg
    let _ = Command::new("ffmpeg")
        .arg("-framerate")
        .arg(format!("{}", FRAMERATE))
        .arg("-crf")
        .arg("0")
        .arg(OUTPUT_FILE)
        .arg("-i")
        .arg(temp_dir.join("%d.png"))
        .output()
        .is_err();

    // delete temp directory
    fs::remove_dir_all(temp_dir).expect(&format!(
        "couldn't delete temporary directory : {}",
        TEMP_DIR_NAME
    ));
}
