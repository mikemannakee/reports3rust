#[macro_use]
extern crate rocket;

use std::ffi::OsStr;
use std::{process};
use image::{GenericImageView, DynamicImage};
use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page::CaptureScreenshotFormatOption};
use rocket::{Request, State};
use rocket::http::uri::Host;

#[get("/chart/<id>")]
async fn chart(id: &str, host: &Host<'_>, browser: &State<Browser>) -> Result<String, rocket::http::Status> {
	// Bail out if the request is not from reports3.hrstapp.com
	println!("Host: {}", host);
	if host != "reports3.hrstapp.com" && host != "127.0.0.1:8000" {
		return Err(rocket::http::Status::Unauthorized);
	}
	println!("Received request for chart with ID: {}", &id);

	let tab = browser.new_tab().map_err(|_| rocket::http::Status::InternalServerError)?;
	println!("New tab created");

	// Browse to the Report URL and wait for the page to load
	let mut path = "https://reports3.hrstapp.com/report_svg-".to_owned();
	path.push_str(&id);
	path.push_str(".php");
	
	println!("Navigating to {}", &path);
	
	// Detect if we are running on Windows or Linux
	#[cfg(windows)]
	let mut filename = "D:\\htdocs\\Larry\\HRST\\Reports3Rust\\".to_string();
	#[cfg(not(windows))]
	let mut filename = "/home/reports3/public_html/".to_owned();
	filename.push_str(&id);
	filename.push_str(".png");
	let png_data = tab
		.navigate_to(&path)
		.map_err(|_| rocket::http::Status::InternalServerError)?
		.wait_for_element("svg")
		.map_err(|_| rocket::http::Status::InternalServerError)?
		.capture_screenshot(CaptureScreenshotFormatOption::Png)
		.map_err(|_| rocket::http::Status::InternalServerError)?;
	println!("Screenshot captured");

	// Load the image and crop white borders
	let img = image::load_from_memory(&png_data).map_err(|_| rocket::http::Status::InternalServerError)?;
	let cropped_img = crop_white_borders(img);
	
	// Save the cropped image
	cropped_img.save(&filename).map_err(|_| rocket::http::Status::InternalServerError)?;

	println!("Screenshots successfully created.");
	
	Ok("image saved".to_string())
}

#[catch(500)]
fn internal_server_error(_req: &Request<'_>) -> () {
	// Shut down the process 
	process::exit(1);
}

#[launch]
fn rocket() -> _ {
	eprintln!("Starting browser");
	// Add the --no-sandbox flag to the launch options
	let options = LaunchOptions::default_builder()
		.args(vec![OsStr::new("--no-sandbox")])
		.build()
		.expect("Couldn't find appropriate Chrome binary.");
	let browser = Browser::new(options).unwrap();

	// Clean out the /tmp directory using a command line command
	let google = process::Command::new("rm").args(&["-rf", "/tmp/.com.google*"]).output();
	eprintln!("Google Chrome cache cleaned: {:?}", google);

	let rust = process::Command::new("rm").args(&["-rf", "/tmp/rust-headless*"]).output();
	eprintln!("Rust headless cache cleaned: {:?}", rust);

	
	rocket::build()
		.manage(browser)
		.mount("/", routes![chart])
		.register("/", catchers![internal_server_error])

}

fn crop_white_borders(img: DynamicImage) -> DynamicImage {
	let (width, height) = img.dimensions();
	let mut left = 0;
	let mut right = width;
	let mut top = 0;
	let mut bottom = height;

	// Find left border
	'outer: for x in 0..width {
		for y in 0..height {
			let pixel = img.get_pixel(x, y);
			if pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250 {
				left = x;
				break 'outer;
			}
		}
	}

	// Find right border
	'outer: for x in (0..width).rev() {
		for y in 0..height {
			let pixel = img.get_pixel(x, y);
			if pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250 {
				right = x + 1;
				break 'outer;
			}
		}
	}

	// Find top border
	'outer: for y in 0..height {
		for x in 0..width {
			let pixel = img.get_pixel(x, y);
			if pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250 {
				top = y;
				break 'outer;
			}
		}
	}

	// Find bottom border
	'outer: for y in (0..height).rev() {
		for x in 0..width {
			let pixel = img.get_pixel(x, y);
			if pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250 {
				bottom = y + 1;
				break 'outer;
			}
		}
	}

	img.crop_imm(left, top, right - left, bottom - top)
}