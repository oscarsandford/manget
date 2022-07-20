mod bind;
mod parse;

use bind::{bind_pages, save_pages_as_images};
use parse::MDClient;

fn driver() -> Result<(), &'static str> {
	let client = MDClient::new();
	
	println!("(1/3) Searching MangaDex for the specified manga and chapters.");

	// Get manga chapters from the swagger API.
	let all_chapters = match client.get_manga_chapters() {
		Ok(res) => res,
		Err(_) => return Err("[!] Error aggregating all the chapters for this manga."),
	};
	if all_chapters.len() == 0 {
		return Err("[!] No manga chapters found. Likely an invalid ID.");
	}

	// Find chapter IDs to match the language we want.
	let ids = match client.get_chapter_lang_ids(all_chapters) {
		Ok(res) => res,
		Err(_) => return Err("[!] Error finding chapter IDs for given language.")
	};
	
	let mut pages = Vec::<String>::new();

	// Grab the pages for each chapter and append them to a flat list.
	for id in ids {
		if let Ok(mut res) = client.get_chapter_pages(id) {
			pages.append(&mut res);
		}
	}
	if pages.len() == 0 {
		return Err("[!] No pages were able to be retrieved based on your chapter and language input.");
	}

	if client.args.images {
		if let Err(_) = save_pages_as_images(&client, pages) {
			return Err("[!] Error while saving images.");
		}
	}
	else {
		// Bind all the desired pages to a PDF.
		if let Err(_) = bind_pages(&client, pages) {
			return Err("[!] Error while binding PDF.");
		}
	}
	
	
	Ok(())
}

fn main() {
	if let Err(e) = driver() {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}
