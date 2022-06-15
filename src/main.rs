mod bind;
mod parse;

use bind::bind_pages;
use parse::{Args, MDClient, get_chapter_pages, get_manga_chapters, get_chapter_id, get_label};


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse_args();
	let client = MDClient::new();

	// Get manga chapters from the swagger API.
	let chapters = get_manga_chapters(&client, "21f54bc1-aefd-4be1-8284-5858b1df0e55")?;

	dbg!(&chapters.len());

	// Grab a specific chapter we want to bind.
	// Chapter is specified in args!
	let id = get_chapter_id(&client, &args, chapters)?;
	
	dbg!(&id);

	if id != "err" {
		// Retrieve this chapter's pages from the at-home API.
		let pages = get_chapter_pages(&client, &args, id)?;
		// Bind them to a PDF.
		bind_pages(&client, pages, get_label(&args))?;
	}

	Ok(())
}
