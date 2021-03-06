extern crate argparse;
extern crate image;
extern crate rand;

use argparse::{ArgumentParser, Store, StoreTrue};

use rand::prelude::*;

use std::process::Command;
use std::f64;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
struct Vertex {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, PartialEq)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl FromStr for RGB {
    type Err = std::num::ParseIntError;

    // Parses a color hex code of the form '#rRgGbB..' into an
    // instance of 'RGB'
    fn from_str(hex_code: &str) -> Result<Self, Self::Err> {
    
        // u8::from_str_radix(src: &str, radix: u32) converts a string
        // slice in a given base to u8
        let r: u8 = u8::from_str_radix(&hex_code[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex_code[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex_code[5..7], 16)?;

        Ok(RGB { r, g, b })
    }
}

fn parse_line(line: &String) -> Vertex {
    let mut iter = line.split_whitespace();

    Vertex {
        x: iter.next().unwrap().parse::<f64>().unwrap(),
        y: iter.next().unwrap().parse::<f64>().unwrap(),
        z: iter.next().unwrap().parse::<f64>().unwrap(),
    }
}

fn find_bounds(path: &String) -> (Vertex, Vertex) {
    let file = std::fs::File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut in_body = false;

    let mut min = Vertex {
        x: f64::MAX,
        y: f64::MAX,
        z: f64::MAX,
    };
    let mut max = Vertex {
        x: f64::MIN,
        y: f64::MIN,
        z: f64::MIN,
    };

    for line in reader.lines() {
        let line = line.unwrap();

        if in_body {
            let pt = parse_line(&line);

            if pt.x < min.x {
                min.x = pt.x;
            } else if pt.x > max.x {
                max.x = pt.x;
            }

            if pt.y < min.y {
                min.y = pt.y;
            } else if pt.y > max.y {
                max.y = pt.y;
            }

            if pt.z < min.z {
                min.z = pt.z;
            } else if pt.z > max.z {
                max.z = pt.z;
            }
        }

        if in_body == false && line == "end_header" {
            in_body = true;
        }
    }

    (min, max)
}


fn main() {
    let mut verbose = false;
    let mut path = "".to_string();
    let mut scale = 0.01_f64;
    let mut angle = 0_f64;
    let mut outfile = "scan.png".to_string();
    let mut color = "#000000".to_string();
    let mut bright = 0_i8;

    // this block limits scope of borrows by ap.refer() method
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Turn a PLY file into a PNG.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        ap.refer(&mut path)
            .add_option(&["--path"], Store, "Path to the PLY file");
        ap.refer(&mut scale)
            .add_option(&["--scale"], Store, "Size of each output pixel (in cm)");
        ap.refer(&mut angle)
            .add_option(&["--angle"], Store, "Amount to rotate the image (in degrees)");
        ap.refer(&mut color)
            .add_option(&["--color"], Store, "Color of the image, in form '#RRGGBB' (default black)");
        ap.refer(&mut bright)
            .add_option(&["--bright"], Store, "Brightness change on the final image (-100 to 100)");
        ap.refer(&mut outfile)
            .add_option(&["--out"], Store, "Output filename");
        ap.parse_args_or_exit();
    }

    let (min, max) = find_bounds(&path);

    println!("");
    println!("X: {} to {}", min.x, max.x);
    println!("Y: {} to {}", min.y, max.y);
    println!("Z: {} to {}", min.z, max.z);

    let xmax = (1.0 + (max.x - min.x) / scale).round() as u32;
    let ymax = (1.0 + (max.y - min.y) / scale).round() as u32;
    println!("Image Size : {} {}", xmax, ymax);
    println!("Angle : {}", angle);

    let mut image: image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::new(xmax, ymax);

    let rgb = RGB::from_str(&color).unwrap(); 

    // Iterate over the coordinates and pixels of the image
    // and set all pixels to the transparent color specified
    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        *pixel = image::Rgba([rgb.r,rgb.g,rgb.b,0]);
    }

    let file = std::fs::File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut in_body = false;

    for line in reader.lines() {
        let line = line.unwrap();

        if in_body {
            let pt = parse_line(&line);

            let x = xmax - ((pt.x - min.x) / scale).round() as u32;
            let y = ((pt.y - min.y) / scale).round() as u32;

            let pixel = image.get_pixel(x, y);
            let mut a = (*pixel as image::Rgba<u8>).data[3];

            let rand: u8 = random();
            if a < 250 && rand > a {
                a = a + 10;
            }

            image.put_pixel(x, y, image::Rgba([rgb.r,rgb.g,rgb.b,a]));
        }

        if in_body == false && line == "end_header" {
            in_body = true;
        }
    }

    image.save("tmp.png").unwrap();

    let _output = Command::new("convert")
        .arg("tmp.png")
        .arg("-channel")
        .arg("rgba")
        .arg("-background")
        .arg("transparent")
        .arg("-normalize")
        .arg("-brightness-contrast")
        .arg(format!("{}", bright))
        .arg("-rotate")
        .arg(format!("{}", angle))

        .arg(outfile)
        .output()
        .expect("Failed to transform output image");

    let _output = Command::new("rm")
        .arg("tmp.png")
        .output()
        .expect("Failed to delete temporary file");
}
