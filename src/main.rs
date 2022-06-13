mod bind;
mod parse;

use bind::compile_chapter;
use parse::{Args, MDClient, get_chapter_pages};


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse_args();
	let client = MDClient::new();

	/* 
	TODO:
	- query the swagger api for the chapters of a manga, given its ID
	- then we simply use that ID in the code below
	*/ 

	let pages = get_chapter_pages(&client, &args, "a54c491c-8e4c-4e97-8873-5b79e59da210")?;
	compile_chapter(&client, pages, "chapter name")?;

	Ok(())
}
