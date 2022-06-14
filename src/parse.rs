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
	volume: String,
	number: String,
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


pub fn get_chapter_id(client: &MDClient, chapters: Vec<Chapter>, number: &str, target_lang: String) -> Result<String, Box<dyn std::error::Error>> {
	for chp in chapters.into_iter() {
		if chp.number == number {
			dbg!(&chp);
			for id in chp.ids {
				let response = client
					.fetch(&format!("{}/chapter/{}", API_URL, id))?
					.json::<serde_json::Value>()?;
	
				let lang = response["data"]["attributes"]["translatedLanguage"].as_str().unwrap().to_string();

				dbg!(&lang);

				if lang == target_lang {
					return Ok(id);
				}
			}
		}
	}
	// Replace this ASAP.
	Ok("tmp".to_string())
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
			let mut ids = vec![data["id"].as_str().unwrap().to_string()];
			for id in data["others"].as_array().unwrap() {
				ids.push(id.as_str().unwrap().to_string());
			}

			chapters.push(Chapter{
				volume: vol.to_string(),
				number: chp.to_string(),
				ids: ids,
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

	// Lastly, the method of only retrieving the first chapter ID does not guarentee it 
	// will be the English (or other desireable language) version.
	// I have tried the manga/{id}/feed API endpoint in order to decipher the language 
	// mappings, but it seems to return fewer results than aggregate. 
	// Therefore, a possible but terrible solution is to simply bind the chapters for 
	// each language until we find the language desired.
	// OR we can maybe scrape using the chapter ID somehow in order to find the language.
	// IS THERE SOMETHING FOR THIS IN THE SWAGGER ENDPOINT LIST?

	// !!!

	// Yeah nevermind, we can just do chapter/{id} to fetch the single piece of important 
	// information: the translatedLanguage field.
	// We look at each chapter ID in this way and choose the one with the desired language match. 


	dbg!(&chapters);

	Ok(chapters)
}
