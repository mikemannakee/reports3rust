use std::fs;
use std::env;

use anyhow::Result;

use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page::CaptureScreenshotFormatOption};

fn main() -> Result<()> {
	let args = env::args().collect::<Vec<String>>();
	if args.len() != 2 {
		eprintln!("Usage: {} (argument) the name of the unique ID forming the name of the PHP file to fetch the image from. \nE.g. '684d9da4221e2' will fetch the image from 'report_svg-684d9da4221e2.php'", args[0]);
		return Ok(());
	}
	let options = LaunchOptions::default_builder()
		.build()
		.expect("Couldn't find appropriate Chrome binary.");
	let browser = Browser::new(options)?;
	let tab = browser.new_tab()?;
	
	// Browse to the Report URL and wait for the page to load
	let mut path = "https://reports3.hrstapp.com/report_svg-".to_owned();
	path.push_str(&args[1]);
	path.push_str(".php");
	let mut filename = args[1].to_string();
	filename.push_str(".png");
	let png_data = tab
		.navigate_to(&path)?
		.wait_for_element("svg")?
		.capture_screenshot(CaptureScreenshotFormatOption::Png)?;
	fs::write(filename, png_data)?;

	println!("Screenshots successfully created.");
	Ok(())
}