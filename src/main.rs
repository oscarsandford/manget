mod bind;
mod parse;

use bind::bind_pages;
use parse::{Args, MDClient, get_chapter_pages, get_manga_chapters, get_chapter_id};

const EX_MANGA_ID: &'static str = "67e7453b-9ee5-4ae5-9316-215b03e4a71d";
// const EX_CHAPTER_ID: &'static str = "a54c491c-8e4c-4e97-8873-5b79e59da210";


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse_args();
	let client = MDClient::new();


	// Get manga chapters from the swagger API.
	// TODO: We need to fix the problem where the chapter IDs are not the English version.
	let chapters = get_manga_chapters(&client, "21f54bc1-aefd-4be1-8284-5858b1df0e55")?;
	// Grab a specific chapter we want to bind.
	let chp_id = get_chapter_id(chapters, "707");

	match chp_id {
		Some(id) => {
			// Retrieve this chapter's pages from the at-home API.
			let pages = get_chapter_pages(&client, &args, id)?;
			// Bind them to a PDF.
			bind_pages(&client, pages, "chapter name")?;
		},
		None => eprintln!("No such chapter was found."),
	}

	Ok(())
}
