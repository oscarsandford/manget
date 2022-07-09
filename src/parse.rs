#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;

const API_URL: &'static str = "https://api.mangadex.org";

/// A CLI tool for binding manga from MangaDex.
#[derive(Parser)]
#[clap(author = "Oscar Sandford", version, about, long_about = None)]
pub struct Args {
	/// The id of the manga
	pub id: String,

	/// Chapter number or numbers (e.g. 1,4,5,7)
	#[clap(short, long, default_value = "1")]
	pub chapter: String,

	/// The translated language
	#[clap(short, long, default_value = "en")]
	pub language: String,

	/// Specify an output file path
	#[clap(short, long, default_value = "./bound.pdf")]
	pub output: String,

	/// Get compressed images instead of original quality
	#[clap(short, long)]
	fast: bool,

	/// Increase verbosity in console output
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

pub struct Chapter {
	//volume: String,
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
	Given all the Chapters, retrieve the chapter IDs for the selected chapters in the given language.
	*/
	pub fn get_chapter_ids(&self, selected_chapters: Vec<&str>, language: &String, chapters: Vec<Chapter>) -> Result<Vec<String>, reqwest::Error> {
		let mut ids = Vec::<String>::new();

		for chp in chapters.into_iter() {
			if selected_chapters.contains(&chp.name.as_str()) {
				for id in chp.ids {
					let response = self
						.fetch(&format!("{}/chapter/{}", API_URL, id))?
						.json::<serde_json::Value>()?;
		
					if let Some(lang) = response["data"]["attributes"]["translatedLanguage"].as_str() {
						if &lang.to_string() == language {
							ids.push(id);
							println!("Chapter {} found in language {}.", chp.name, language);
							break;
						}
					};
				}
			}
		}
		Ok(ids)
	}

	/*
	Retrieve the manga chapters for a given manga ID from the swagger API.
	*/
	pub fn get_manga_chapters(&self, manga_id: &String) -> Result<Vec<Chapter>, reqwest::Error> {
		let response = self
			.fetch(&format!("{}/manga/{}/aggregate", API_URL, manga_id))?
			.json::<serde_json::Value>()?;

		let mut chapters = Vec::<Chapter>::new();
		if let Some(volumes) = response["volumes"].as_object() { 
			for (_vol, chp_data) in volumes {
				if let Some(chapters_data) = chp_data["chapters"].as_object() {
					for (chp, data) in chapters_data {
						// println!("\t{}: {}", chp, data);
		
						// TODO: see if we can do this better without mutability.
						let mut ids = vec![data["id"].as_str().unwrap().to_string()];
						for id in data["others"].as_array().unwrap() {
							ids.push(id.as_str().unwrap().to_string());
						}
		
						chapters.push(Chapter{
							//volume: vol.to_string(),
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
