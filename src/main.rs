mod bind;
mod parse;

use bind::bind_pages;
use parse::{Args, MDClient};

fn driver() -> Result<(), &'static str> {
	let args = Args::parse_args();
	let client = MDClient::new();

	// Get manga chapters from the swagger API.
	let all_chapters = match client.get_manga_chapters(&args.id) {
		Ok(res) => res,
		Err(_) => return Err("[!] Error aggregating all the chapters for this manga."),
	};
	if all_chapters.len() == 0 {
		return Err("[!] No manga chapters found. Could be an invalid ID.");
	}
	// Delimit on commas for the chapter(s) selected.
	let selection = args.chapter.split(",").collect();

	// Find chapter IDs to match the language we want.
	let ids = match client.get_chapter_ids(selection, &args.language, all_chapters) {
		Ok(res) => res,
		Err(_) => return Err("[!] Error finding chapter IDs for given language.")
	};
	
	let mut pages = Vec::<String>::new();

	// Grab the pages for each chapter and append them to a flat list.
	for id in ids {
		let mut cpages = match client.get_chapter_pages(&args, id) {
			Ok(res) => res,
			Err(_) => return Err("[!] Error while retrieving chapter pages.")
		};
		pages.append(&mut cpages);
	}
	if pages.len() == 0 {
		return Err("[!] No pages were able to be retrieved based on your chapter and language input.");
	}

	// Bind all the desired pages to a PDF.
	match bind_pages(&client, pages, format!("ch{}-{}", &args.chapter, &args.language)) {
		Ok(res) => res,
		Err(_) => return Err("[!] Error while binding PDF."),
	}
	
	Ok(())
}

fn main() {
	if let Err(e) = driver() {
		eprintln!("{}", e);
		std::process::exit(1);
	}
}
