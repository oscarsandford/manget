#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;

const API_URL: &'static str = "https://api.mangadex.org";

/// A CLI tool for binding manga from MangaDex.
#[derive(Parser)]
#[clap(author = "oes", version, about, long_about = None)]
pub struct Args {
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

	/// The name of the output pdf
	#[clap(short, long)]
	output: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChapterPagesData {
	result: String,
	baseUrl: String,
	chapter: ChapterPages,
} 

#[derive(Serialize, Deserialize, Debug)]
struct ChapterPages {
	hash: String,
	data: Vec<String>,
	dataSaver: Vec<String>,
}

#[derive(Debug)]
pub struct Chapter {
	vol_name: String,
	chp_name: String,
	chp_id_main_lang: String,
}

pub struct MDClient {
	pub client: Client,
}

impl Args {
	pub fn parse_args() -> Args {
		Args::parse()
	}
}

impl MDClient {
	pub fn new() -> MDClient {
		MDClient { client: Client::new() }
	}
	// Use references here because a primary use case is passing Vector elements as indexed.
	// Also we almost never want to consume the client.
	// This fetch request is blocking.
	pub fn fetch(&self, query: &String) -> Result<reqwest::blocking::Response, reqwest::Error> {
		self.client.get(query).send()
	}
	// TODO: maybe make specific methods for fetching chapter or manga itself? To make interface cleaner.
}


/*
Use the reqwest client to retrieve the pages of a manga chapter given its ID.
We can tell it which server to pull from based on the Args.
*/
pub fn get_chapter_pages(client: &MDClient, args: &Args, chapter_id: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	// This request is blocking.
	let response = client
		.fetch(&format!("{}/at-home/server/{}", API_URL, chapter_id))?
		.json::<ChapterPagesData>()?;
	
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

/*
Returns the ID of a Chapter in a list, given a String query for the chapter name.
*/
pub fn get_chapter_id(chapters: Vec<Chapter>, id: &str) -> Option<String> {
	for chp in chapters.into_iter() {
		if chp.chp_name == id {
			return Some(chp.chp_id_main_lang);
		}
	}
	None
}

/*
Retrieve the manga chapters for a given manga ID from the swagger API.
*/
pub fn get_manga_chapters(client: &MDClient, manga_id: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
	let response = client.
		fetch(&format!("{}/manga/{}/aggregate", API_URL, manga_id))?
		.json::<serde_json::Value>()?;

	let mut chapters = Vec::<Chapter>::new();
	for (vol, chp_data) in response["volumes"].as_object().unwrap() {
		for (chp, data) in chp_data["chapters"].as_object().unwrap() {
			// println!("\t{}: {}", chp, data);
			chapters.push(Chapter{
				vol_name: vol.to_string(),
				chp_name: chp.to_string(),
				// TODO:
				// Besides the fact that the line of code below to format the 
				// the chapter ID is messy and error-prone, is that the "id" 
				// field may not always be the ID for the English version.
				chp_id_main_lang: data["id"].as_str().unwrap().to_string(),
			});
		}
	}

	// Note that the chapters will not be in the correct order in the vector, so we 
	// will need to either sort them beforehand if we want to bind a whole volume, or 
	// do that when merging the PDF docs (if that's how we decide to do that).

	// Further, while the use of a vector is a good generic start, it means that we have 
	// to run the length of the vector in order to find a chapter in the worst case.
	// A HashMap would help solve this problem, but then we need to decide whether the 
	// key will be a volume or a chapter.
	// A decision like this is better made down the line, so for now, we will just 
	// query the vector for a Chapter with a given ID, or the Chapters of a given volume.

	// dbg!(&chapters);

	Ok(chapters)
}
