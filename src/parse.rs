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
	id: String,

	/// Chapter number or numbers (e.g. 1,4,5,7)
	#[clap(short, long, default_value = "1")]
	chapter: String,

	/// The translated language
	#[clap(short, long, default_value = "en")]
	language: String,

	/// Get compressed images instead of original quality
	#[clap(short, long)]
	fast: bool,
	
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
	//volume: u16,
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
	pub fn get_chapter_ids(&self, chapters: Vec<Chapter>) -> Result<Vec<String>, reqwest::Error> {
		let selected_chapters: Vec<u16> = self.args.chapter
								.split(",")
								.map(|s| {match s.parse::<u16>() {
									Ok(x) => x,
									Err(_) => 0,
								}})
								.collect();
		let mut ids = Vec::<String>::new();
		self.status(format!("[get_chapter_ids:selected_chapters] {:#?}", &selected_chapters));

		for chp in chapters.into_iter() {
			if selected_chapters.contains(&chp.name) {
				for id in chp.ids {
					let req = format!("{}/chapter/{}", API_URL, id);
					self.status(format!("[get_chapter_ids:req] {:#?}", &req));

					let res = self
						.fetch(&req)?
						.json::<serde_json::Value>()?;
					// We should be able to safely do this with the translatedLanguage field, 
					// but we must avoid results where the externalUrl field is set.
					let lang = res["data"]["attributes"]["translatedLanguage"].as_str().unwrap();
					let ext_url = res["data"]["attributes"]["externalUrl"].as_str();
					
					if &lang.to_string() == &self.args.language && !ext_url.is_some() {
						ids.push(id);
						self.status(format!("[get_chapter_ids:res(matched)] {:#?}", &res));
						break;
					}
				}
			}
		}
		Ok(ids)
	}

	/*
	Retrieve the manga chapters for a given manga ID from the swagger API.
	*/
	pub fn get_manga_chapters(&self) -> Result<Vec<Chapter>, reqwest::Error> {
		let req = format!("{}/manga/{}/aggregate", API_URL, self.args.id);
		self.status(format!("[get_manga_chapters:req] {:#?}", &req));
		
		let res = self
			.fetch(&req)?
			.json::<serde_json::Value>()?;

		// TODO: change this to a HashMap? See longer comment below.
		let mut chapters = Vec::<Chapter>::new();

		if let Some(volumes) = res["volumes"].as_object() { 
			for (_vol, chp_data) in volumes {
				if let Some(chapters_data) = chp_data["chapters"].as_object() {
					for (chp, data) in chapters_data {
		
						// TODO: see if we can do this better without mutability.
						let mut ids = vec![data["id"].as_str().unwrap().to_string()];
						for id in data["others"].as_array().unwrap() {
							ids.push(id.as_str().unwrap().to_string());
						}
		
						let name = match chp.parse::<u16>() {
							Ok(x) => x,
							Err(_) => 0,
						};
						
						chapters.push(Chapter{
							name: name,
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
