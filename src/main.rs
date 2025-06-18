use std::fs;
use std::ffi::OsStr;
use image::{GenericImageView, DynamicImage};
use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
use anyhow::Result;
use std::sync::mpsc;
use std::path::Path;

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
	let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
	let mut watcher = recommended_watcher(tx)?;
	// let root_path = "D:\\htdocs\\Larry\\HRST\\Reports3Rust\\".to_string();
	let watched_file = "/home/reports3/rust/ids".to_string();
	let file_save_path = "/home/reports3/public_html/".to_string();
	watcher.watch(Path::new(&watched_file), RecursiveMode::Recursive)?;
	eprintln!("Watching for changes in the current directory");

	eprintln!("Starting browser");
	// Add the --no-sandbox flag to the launch options
	let options = LaunchOptions::default_builder()
		.args(vec![OsStr::new("--no-sandbox")])
		.build()
		.expect("Couldn't find appropriate Chrome binary.");
	let browser = Browser::new(options).unwrap();

	let mut handled_ids: Vec<String> = Vec::new();
	let mut clearing_file = false;

	for event in rx {
		match event {
			Ok(event) => {
				eprintln!("Event: {:?}", event);

				// Read in the ids.txt file
				let ids = fs::read_to_string(&watched_file)?;
				eprintln!("IDs: {}", &ids);

				// Go through the ids and check if they have been handled
				for id in ids.split("\n") {
					let id = id.trim().to_string();
					if id.is_empty() || clearing_file {
						continue;
					}
					if !handled_ids.contains(&id) {
						
						eprintln!("Handling ID: {}", &id);

						// Navigate to the report URL and wait for the page to load
						let mut path = "https://reports3.hrstapp.com/report_svg-".to_owned();
						path.push_str(&id);
						path.push_str(".php");

						eprintln!("Navigating to {}", &path);

						let mut filename = file_save_path.clone();
						filename.push_str(&id);
						filename.push_str(".png");

						let tab = browser.new_tab()?;
						
						let png_data = tab
							.navigate_to(&path)?
							.wait_for_element("svg")?
							.capture_screenshot(CaptureScreenshotFormatOption::Png)?;

						let img = image::load_from_memory(&png_data)?;
						let cropped_img = crop_white_borders(img);
						
						// Save the cropped image
						cropped_img.save(&filename)?;
						
						println!("Screenshot successfully created. Saved to {}", &filename);

						// Close the tab
						tab.close(false)?;

						// Add the id to the handled_ids vector
						handled_ids.push(id.clone());

						// Clear out the ids file
						if !clearing_file {
							clearing_file = true;
							fs::write(&watched_file, "")?;
						}
					}
				}
			}
			Err(e) => {
				eprintln!("Error: {:?}", e);
			}
		}
	}

	Ok(())
}