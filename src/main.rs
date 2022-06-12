#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use printpdf::{
	PdfDocument, PdfDocumentReference, PdfPageIndex, PdfLayerIndex, 
	Image, ImageTransform, ImageRotation, ImageXObject, 
	ColorBits, ColorSpace, Px, Mm
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
const PAGE_SCALE_FACTOR: f64 = 3.125; // Found this by trial and error around ~3.0.


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

struct MangaImage {
	bytes: Vec<u8>,
	color: ColorType,
	width_px: u32,
	height_px: u32,
	width_mm: f64,
	height_mm: f64,
}

/* 
Embed a mange image into a given PDF doc on a given page and layer.
*/
fn embed_image(img: MangaImage, doc: &PdfDocumentReference, page: PdfPageIndex, layer: PdfLayerIndex) -> () {
	// ImageXObject contruction based on this:
	// https://github.com/fschutt/printpdf/blob/v0.5.0/src/xobject.rs#L223
	let imgx = ImageXObject {
		width: Px(img.width_px as usize),
		height: Px(img.height_px as usize),
		color_space: ColorSpace::from(img.color),
		bits_per_component: ColorBits::from(img.color),
		image_data: img.bytes,
		interpolate: true,
		image_filter: None,
		clipping_bbox: None,
	};
	let pdf_img = Image::from(imgx);

	let layer = doc.get_page(page).get_layer(layer);

	// We just have to do this crazy ImageTransform instead of the 
	// default beause the only solution I have to the small images 
	// right now is to simply upscale them in the transform.
	// This totally works, but is worth revisiting.
	pdf_img.add_to_layer(layer.clone(), ImageTransform {
		translate_x: Option::<Mm>::from(Mm(0.)),
		translate_y: Option::<Mm>::from(Mm(0.)),
		rotate: Option::<ImageRotation>::from(ImageRotation{
			angle_ccw_degrees: 0., 
			rotation_center_x: Px(0), 
			rotation_center_y: Px(0)
		}),
		scale_x: Option::<f64>::from(PAGE_SCALE_FACTOR),
		scale_y: Option::<f64>::from(PAGE_SCALE_FACTOR),
		dpi: Option::<f64>::from(300.0),
	});
}

/*
Create and return a manga image from the image at the link reqwested.
*/
fn create_manga_image(client: &Client, link: &String) -> Result<MangaImage, Box<dyn std::error::Error>> {
	let img_bytes = client
			.get(link)
			.send()?
			.bytes()?;
	let img: DynamicImage = load_from_memory(&img_bytes)?;
	Ok(MangaImage {
		bytes: img.as_bytes().to_vec(),
		color: img.color(),
		width_px: img.width(),
		height_px: img.height(),
		width_mm: img.width() as f64 * PX_TO_MM_FACTOR,
		height_mm: img.height() as f64 * PX_TO_MM_FACTOR,
	})
}

/*
Compile a chapter. This involves reqwesting each page from the 
client, binding these pages in a PDF doc, and writing it to disk.
*/
fn compile_chapter(client: &Client, pages: Vec<String>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
	println!("Compiling a chapter of {} pages called {}:", pages.len(), filename);

	println!("> Working on page 1.");
	let img = create_manga_image(client, &pages[0])?;
	let (doc, mut page, mut layer) = PdfDocument::new(filename, Mm(img.width_mm), Mm(img.height_mm), "");
	embed_image(img, &doc, page, layer);

	for i in 1..pages.len() {
		println!("> Working on page {}.", i+1);
		
		let img = create_manga_image(client, &pages[i])?;
		(page, layer) = doc.add_page(Mm(img.width_mm), Mm(img.height_mm), "");
		embed_image(img, &doc, page, layer);
	}

	println!("Saving chapter as pdf file.");
	let pdf_bytes = doc.save_to_bytes()?;
	let mut pdf_file = File::create(format!("{}/{}.pdf", OUT_DIR, filename))?;
	pdf_file.write_all(&pdf_bytes)?;

	Ok(())
}

/*
Use the reqwest client to retrieve the pages of a manga chapter given its ID.
We can tell it which server to pull from based on the Args.
*/
fn get_chapter_pages(client: &Client, args: &Args, chapter_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	// This request is blocking.
	let response = client
		.get(format!("{}/{}", API_ENDPT, chapter_id))
		.send()?
		.json::<ChapterData>()?;
	
	let (imgs, quality) = if args.fast { 
		(response.chapter.dataSaver, "data-saver")
	} else { 
		(response.chapter.data, "data")
	};
	
	Ok(imgs.iter().map(|img| format!(
		"{}/{}/{}/{}",
		response.baseUrl, quality, response.chapter.hash, img
	)).collect())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();
	let client = Client::new();

	/* 
	TODO:
	- query the swagger api for the chapters of a manga, given its ID
	- then we simply use that ID in the code below
	*/ 

	let pages = get_chapter_pages(&client, &args, EXAMPLE_CHAPTER_ID)?;
	compile_chapter(&client, pages, "chapter name")?;

	Ok(())
}
