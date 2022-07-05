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
	#[clap(short, long, default_value = "1")]
	chapter: String,

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
	volume: String,
	name: String,
	ids: Vec<String>,
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

	/*
	Use the reqwest client to retrieve the pages of a manga chapter given its ID.
	We can tell it which server to pull from based on the Args.
	*/
	pub fn get_chapter_pages(&self, args: &Args, chapter_id: String) -> Result<Vec<String>, reqwest::Error> {
		// This request is blocking.
		let response = self
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
	Given a vector of Chapters, retrieve the chapter ID with the given number and target language.
	*/
	pub fn get_chapter_id(&self, args: &Args, chapters: Vec<Chapter>) -> Result<String, reqwest::Error> {
		for chp in chapters.into_iter() {
			if chp.name == args.chapter {
				dbg!(&chp);
				for id in chp.ids {
					let response = self
						.fetch(&format!("{}/chapter/{}", API_URL, id))?
						.json::<serde_json::Value>()?;
		
					if let Some(lang) = response["data"]["attributes"]["translatedLanguage"].as_str() {
						dbg!(&lang);
						if lang.to_string() == args.language {
							return Ok(id);
						}
					};
				}
			}
		}
		// Replace this ASAP.
		Ok("err".to_string())
	}

	/*
	Retrieve the manga chapters for a given manga ID from the swagger API.
	*/
	pub fn get_manga_chapters(&self, args: &Args) -> Result<Vec<Chapter>, reqwest::Error> {
		let response = self
			.fetch(&format!("{}/manga/{}/aggregate", API_URL, args.manga_name))?
			.json::<serde_json::Value>()?;

		let mut chapters = Vec::<Chapter>::new();
		if let Some(volumes) = response["volumes"].as_object() { 
			for (vol, chp_data) in volumes {
				if let Some(chapters_data) = chp_data["chapters"].as_object() {
					for (chp, data) in chapters_data {
						// println!("\t{}: {}", chp, data);
		
						// TODO: see if we can do this better without mutability.
						let mut ids = vec![data["id"].as_str().unwrap().to_string()];
						for id in data["others"].as_array().unwrap() {
							ids.push(id.as_str().unwrap().to_string());
						}
		
						chapters.push(Chapter{
							volume: vol.to_string(),
							name: chp.to_string(),
							ids: ids,
						});	
					}
				}
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
}

/*
Returns a label for the file name of the output PDF.
This function is needed to preserve the privacy of Args members.
*/
pub fn get_label(args: &Args) -> String {
	format!("ch{}-{}", args.chapter, args.language)
}
