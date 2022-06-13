#![allow(non_snake_case)]

use clap::Parser;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;

const API_ENDPT: &'static str = "https://api.mangadex.org/at-home/server";

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
}

#[derive(Serialize, Deserialize, Debug)]
struct ChapterData {
	result: String,
	baseUrl: String,
	chapter: Chapter,
} 

#[derive(Serialize, Deserialize, Debug)]
struct Chapter {
	hash: String,
	data: Vec<String>,
	dataSaver: Vec<String>,
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
pub fn get_chapter_pages(client: &MDClient, args: &Args, chapter_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	// This request is blocking.
	let response = client
		.fetch(&format!("{}/{}", API_ENDPT, chapter_id))?
		.json::<ChapterData>()?;
	
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
