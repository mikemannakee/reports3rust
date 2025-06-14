use std::env;
use std::ffi::OsStr;
use image::{GenericImageView, DynamicImage};
use anyhow::Result;

use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page::CaptureScreenshotFormatOption};

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

fn main() -> Result<()> {
	let args = env::args().collect::<Vec<String>>();
	if args.len() != 2 {
		eprintln!("Usage: {} (argument) the name of the unique ID forming the name of the PHP file to fetch the image from. \nE.g. '684d9da4221e2' will fetch the image from 'report_svg-684d9da4221e2.php'", args[0]);
		return Ok(());
	}
	eprintln!("Starting browser");
	// Add the --no-sandbox flag to the launch options
	let options = LaunchOptions::default_builder()
		.args(vec![OsStr::new("--no-sandbox")])
		.build()
		.expect("Couldn't find appropriate Chrome binary.");
	let browser = Browser::new(options)?;
	let tab = browser.new_tab()?;
	
	// Browse to the Report URL and wait for the page to load
	let mut path = "https://reports3.hrstapp.com/report_svg-".to_owned();
	path.push_str(&args[1]);
	path.push_str(".php");
	
	eprintln!("Navigating to {}", &path);
	
	let mut filename = "/home/reports3/public_html/".to_owned();
	filename.push_str(&args[1]);
	filename.push_str(".png");
	let png_data = tab
		.navigate_to(&path)?
		.wait_for_element("svg")?
		.capture_screenshot(CaptureScreenshotFormatOption::Png)?;

	// Load the image and crop white borders
	let img = image::load_from_memory(&png_data)?;
	let cropped_img = crop_white_borders(img);
	
	// Save the cropped image
	cropped_img.save(&filename)?;

	println!("Screenshots successfully created.");
	Ok(())
}