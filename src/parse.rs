#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};
use serde_json::json;
use reqwest::blocking::Client;

const API_URL: &'static str = "https://api.mangadex.org";

/// A CLI tool for binding manga from MangaDex.
#[derive(Parser)]
#[clap(author = "Oscar Sandford", version, about, long_about = None)]
pub struct Args {
	/// The id of the manga
	id: String,

	/// Chapter number or closed interval (e.g. 3 or 1-5)
	#[clap(short, long, default_value = "1")]
	chapter: String,

	/// The translated language
	#[clap(short, long, default_value = "en")]
	language: String,

	/// Get compressed images instead of original quality
	#[clap(short, long)]
	fast: bool,

	/// Output images into a folder instead of binding them
	#[clap(short, long)]
	images: bool,
	
	/// Specify an output file path
	#[clap(short, long, default_value = "bound.pdf")]
	pub output: String,

	/// Increase verbosity in console output
	#[clap(short, long)]
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
	name: u16,
	ids: Vec<String>,
}

pub struct MDClient {
	pub client: Client,
	pub args: Args,
}


impl MDClient {
	pub fn new() -> MDClient {
		MDClient { 
			client: Client::new(),
			args: Args::parse(),
		}
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
	pub fn get_chapter_pages(&self, chapter_id: String) -> Result<Vec<String>, reqwest::Error> {
		// This request is blocking.
		let req = format!("{}/at-home/server/{}", API_URL, chapter_id);
		self.status(format!("[get_chapter_pages:req] {:#?}", &req));
		let res = self
			.fetch(&req)?
			.json::<ChapterPagesData>()?;
		
		self.status(format!("[get_chapter_pages:res] {:#?}", &res));

		let (imgs, quality) = if self.args.fast { 
			(res.chapter.dataSaver, "data-saver")
		} else { 
			(res.chapter.data, "data")
		};
		
		Ok(imgs.iter().map(|img| format!(
			"{}/{}/{}/{}",
			res.baseUrl, quality, res.chapter.hash, img
		)).collect())
	}

	/*
	Given all the Chapters, retrieve the chapter IDs for the selected chapters in the given language.
	*/
	pub fn get_chapter_lang_ids(&self, chapters: Vec<Chapter>) -> Result<Vec<String>, reqwest::Error> {
		let sargs: Vec<u16> = self.args.chapter
								.split("-")
								.take(2)
								.map(|s| {match s.parse::<u16>() {
									Ok(x) => x,
									Err(_) => 1,
								}})
								.collect();

		let start: u16 = sargs[0];
		let end: u16 = if sargs.len() > 1 && sargs[0] < sargs[1] { sargs[1] } else { sargs[0] };
		self.status(format!("[get_chapter_ids:start->end] {:#?}->{:#?}", &start, &end));
		let mut ids = Vec::<String>::new();

		// Only need to iterate over the range of chapters we need.
		for chapter in chapters.iter().filter(|c| start <= c.name && c.name <= end) {
			for id in &chapter.ids {
				let req = format!("{}/chapter/{}", API_URL, id);
				self.status(format!("[get_chapter_ids:req] {:#?}", &req));
				let res = self
					.fetch(&req)?
					.json::<serde_json::Value>()?;
				// Avoid results where the externalUrl field is set - we cannot retrieve pages off-site.
				let lang: &str = res["data"]["attributes"]["translatedLanguage"].as_str().unwrap_or("");
				let ext_url: Option<&str> = res["data"]["attributes"]["externalUrl"].as_str();
				
				if &lang.to_string() == &self.args.language && !ext_url.is_some() {
					ids.push(id.to_string());
					self.status(format!("[get_chapter_ids:res(matched)] {:#?}", &res));
					println!(" Chapter {} retrieved successfully.", chapter.name);
					break;
				}
			}
		}
		Ok(ids)
	}

	/*
	Retrieve the manga chapters for a given manga ID from the MangaDex swagger API.
	*/
	pub fn get_manga_chapters(&self) -> Result<Vec<Chapter>, reqwest::Error> {
		let req = format!("{}/manga/{}/aggregate", API_URL, self.args.id);
		self.status(format!("[get_manga_chapters:req] {:#?}", &req));
		
		let res = self
			.fetch(&req)?
			.json::<serde_json::Value>()?;

		let mut chapters = Vec::<Chapter>::new();

		if let Some(volumes) = res["volumes"].as_object() { 
			for (_vol, chp_data) in volumes {
				if let Some(chapters_data) = chp_data["chapters"].as_object() {
					for (chp, data) in chapters_data {	

						let ids: Vec<String> = data["others"].as_array().unwrap_or(&vec![json!([])]).into_iter()
								.map(|id| id.as_str().unwrap_or("").to_string())
								.chain(std::iter::once(data["id"].as_str().unwrap_or("").to_string()))
								.collect();

						if let Ok(name) = chp.parse::<u16>() {
							chapters.push(Chapter{name, ids})
						};
					}
				}
			}
		}
		
		// Note that the chapters will (likely) not be in the correct order in the vector, 
		// so we must sort them by name (i.e. the chapter number) in ascending order.
		chapters.sort_by_key(|c| c.name);

		Ok(chapters)
	}

	/*
	Prints a given status message to the console when in verbose mode.
	There was a possibility to use a custom macro here, but to keep this 
	interface clean, we need to check the verbosity level from the Args.
	*/
	pub fn status(&self, msg: String) {
		if self.args.verbose {
			println!("{}", msg);
		}
	}
}
