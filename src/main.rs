mod bind;
mod parse;

use bind::bind_pages;
use parse::{Args, MDClient, get_label};


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse_args();
	let client = MDClient::new();

	// Get manga chapters from the swagger API.
	let chapters = client.get_manga_chapters(&args)?;

	dbg!(&chapters.len());

	// Grab a specific chapter we want to bind.
	// Chapter is specified in args!
	let id = client.get_chapter_id(&args, chapters)?;
	
	dbg!(&id);

	if id != "err" {
		// Retrieve this chapter's pages from the at-home API.
		let pages = client.get_chapter_pages(&args, id)?;
		// Bind them to a PDF.
		bind_pages(&client, pages, get_label(&args))?;
	}

	Ok(())
}
