#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use printpdf::{
	PdfDocument, Image, 
	Mm, ImageTransform, ImageXObject, ColorBits, ColorSpace, Px,
};
// Use the version of the "image" crate that comes 
// with printpdf to avoid mismatching identifiers.
use printpdf::image_crate::*;
use std::fs::File;
use std::io::Write;

// TODO: make this configurable.
const OUT_DIR: &'static str = "./out";

const API_ENDPT: &'static str = "https://api.mangadex.org/at-home/server";
const EXAMPLE_CHAPTER_ID: &'static str = "a54c491c-8e4c-4e97-8873-5b79e59da210";

const PX_TO_MM_FACTOR: f64 = 0.2645833;


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


fn get_pages(client: Client, pages: Vec<String>, label: &str) -> Result<(), Box<dyn std::error::Error>> {
	println!("Saving {} pages for {}..", pages.len(), label);

	let (doc, mut curr_page, mut curr_layer) = PdfDocument::new(label, Mm(10.), Mm(10.), "Layer 0");

	for (i, page) in pages.iter().enumerate() {
		println!("Working on page {}:", i+1);
		let img_bytes = client
			.get(page)
			.send()?
			.bytes()?;
		
		// let len = page.len();
		// let filename = format!("{}/{}_{}.{}", OUT_DIR, label, i, &page[len-3..]);
		// img.save(&filename)?;
		// println!("\t {} saved.", filename);

		println!("\t bytes to dynimg");

		println!("\t Creating new page and layer.");
		
		println!("------");

		let dyn_img: DynamicImage = load_from_memory(&img_bytes)?;
		
		let ctype = dyn_img.color();
		let data = dyn_img.as_bytes().to_vec();


		let width_px: u32 = dyn_img.width();
		let height_px: u32 = dyn_img.height();

		let width_mm: f64 = dyn_img.width() as f64 * PX_TO_MM_FACTOR;
		let height_mm: f64 = dyn_img.height() as f64 * PX_TO_MM_FACTOR;

		(curr_page, curr_layer) = doc.add_page(Mm(width_mm), Mm(height_mm), format!("Layer {}", i+1));

		// https://github.com/fschutt/printpdf/blob/v0.5.0/src/xobject.rs#L223
		let imgx = ImageXObject {
			// TODO: might need to insert a factor here because 
			// the library changes Px weird...  
			width: Px(width_px as usize),
			height: Px(height_px as usize),
			color_space: ColorSpace::from(ctype),
			bits_per_component: ColorBits::from(ctype),
			image_data: data,
			interpolate: true,
			image_filter: None,
			clipping_bbox: None,
		};

		println!("w:{} h:{}, ", width_px, height_px);
		dbg!(Px(width_px as usize));
		dbg!(Px(height_px as usize));

		let imgp = Image::from(imgx);

		println!("\t Adding image to PDF.");
		let layer = doc.get_page(curr_page).get_layer(curr_layer);
		imgp.add_to_layer(layer.clone(), ImageTransform::default());

		
	}

	
	let pdf_bytes = doc.save_to_bytes()?;

	println!("Writing pdf file.");
	let mut file = File::create(format!("{}/{}.pdf", OUT_DIR, label))?;
	file.write_all(&pdf_bytes)?;

	Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();
	
	let client = Client::new();

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
	
	let pages: Vec<String> = imgs.iter().map(|img| format!(
		"{}/{}/{}/{}",
		response.baseUrl, quality, response.chapter.hash, img
	)).collect();

	dbg!(&pages);



	get_pages(client, pages, "tmp")?;

	Ok(())
}
