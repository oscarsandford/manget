#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};

// TODO: make this configurable.
const OUT_DIR: &'static str = "./out";

const API_ENDPT: &'static str = "https://api.mangadex.org/at-home/server";
const EXAMPLE_CHAPTER_ID: &'static str = "a54c491c-8e4c-4e97-8873-5b79e59da210";



/// A CLI tool for binding manga from MangaDex.
#[derive(Parser)]
#[clap(author = "oes", version, about, long_about = None)]
struct Args {
	/// The name of the manga
	manga_name: String,

	/// Chapter #
	#[clap(short, long)]
	chapter: Option<u16>,

	/// Language
	#[clap(short, long, default_value = "en")]
	language: String,

	/// Faster at the cost of image quality
	#[clap(short, long)]
	fast: bool,

	/// Increase verbosity in stdout
	#[clap(long)]
	verbose: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChapterData {
	result: String,
	baseUrl: String,
	chapter: Chapter,
} 

#[derive(Serialize, Deserialize, Debug)]
struct Chapter {
	hash: String,
	data: Vec<String>,
	dataSaver: Vec<String>,
}


fn save_pages(pages: Vec<String>, label: &str) -> Result<(), Box<dyn std::error::Error>> {
	println!("Saving {} pages for {}..", pages.len(), label);
	for (i, page) in pages.iter().enumerate() {
		// TODO: use the client from earlier.
		let img_bytes = reqwest::blocking::get(page)?.bytes()?;
		let img = image::load_from_memory(&img_bytes)?;
		let len = page.len();
		let filename = format!("{}/{}_{}.{}", OUT_DIR, label, i, &page[len-3..]);
		img.save(&filename)?;
		println!("\t {} saved.", filename);
	}
	Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();
	
	let client = reqwest::blocking::Client::new();

	// TODO: User can search for the chapter id's somehow?
	
	// This request is blocking.
	let response = client
		.get(format!("{}/{}", API_ENDPT, EXAMPLE_CHAPTER_ID))
		.send()?
		.json::<ChapterData>()?;
	
	//dbg!(&response);
	
	// Grab the low-res image if desired.
	let (imgs, quality) = if args.fast { 
		(response.chapter.dataSaver, "data-saver")
	} else { 
		(response.chapter.data, "data")
	};
	
	let pages: Vec<String> = imgs.into_iter().map(|img| format!(
		"{}/{}/{}/{}",
		response.baseUrl, quality, response.chapter.hash, img
	)).collect();

	dbg!(&pages);

	save_pages(pages, "tmp")?;

	Ok(())
}
