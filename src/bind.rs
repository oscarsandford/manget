// Various libraries needed for file read/write and PDF document mutation.
use std::fs::File;
use std::io::Write;
use printpdf::{
	PdfDocument, PdfDocumentReference, PdfPageIndex, PdfLayerIndex, 
	Image, ImageTransform, ImageRotation, ImageXObject, 
	ColorBits, ColorSpace, Px, Mm
};
use printpdf::image_crate::*;

use super::MDClient;


const PX_TO_MM_FACTOR: f64 = 0.2645833;
const PAGE_SCALE_FACTOR: f64 = 3.125; // Found this by trial and error around ~3.0.
const OUT_DIR: &'static str = "./out";

struct MangaImage {
	bytes: Vec<u8>,
	color: ColorType,
	width_px: u32,
	height_px: u32,
	width_mm: f64,
	height_mm: f64,
}

/*
Create and return a manga image from the image at the link reqwested.
*/
fn create_manga_image(client: &MDClient, link: &String) -> Result<MangaImage, Box<dyn std::error::Error>> {
	let img_bytes = client
			.fetch(link)?
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
Bind pages (of a chapter, for the most part). This involves reqwesting each page from the 
client, binding these pages in a PDF doc, and writing it to disk.
*/
pub fn bind_pages(client: &MDClient, pages: Vec<String>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
	println!("Binding {} pages called {}:", pages.len(), filename);

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

	println!("Saving bound pages as a pdf file.");
	let pdf_bytes = doc.save_to_bytes()?;
	let mut pdf_file = File::create(format!("{}/{}.pdf", OUT_DIR, filename))?;
	pdf_file.write_all(&pdf_bytes)?;

	Ok(())
}
