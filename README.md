# manget

A CLI tool for binding digital pages of manga from MangaDex to PDFs for offline reading.


## Guide and General Usage

Unfortunately, MangaDex has made it difficult to scrape their search page, so the search functionality will have to wait. You must navigate to [mangadex.org](https://mangadex.org/) yourself and find a suitable manga to download yourself. We are mainly interested in the **manga ID**, or the highlighted part in the example below:

<p style="text-align: center;"><strong></strong>https://mangadex.org/title/<strong>21f54bc1-aefd-4be1-8284-5858b1df0e55</strong>/initial-d</p>

Peruse the page for chapters you would like to read, and note if the language you want them in is available. Then, consider the technical usage of the tool below.

```
manget 1.0.0
Oscar Sandford
A CLI tool for binding manga from MangaDex

USAGE:
    manget [OPTIONS] <ID>

ARGS:
    <ID>    The id of the manga

OPTIONS:
    -c, --chapter <CHAPTER>      Chapter number or numbers (e.g. 1,4,5,7) [default: 1] 
    -f, --fast                   Get compressed images instead of original quality     
    -h, --help                   Print help information
    -l, --language <LANGUAGE>    The translated language [default: en]
    -o, --output <OUTPUT>        Specify an output file path [default: ./bound.pdf]
    -V, --version                Print version information
        --verbose                Increase verbosity in console output
```


## Setup

You can build the project yourself in release mode by cloning this GitHub repository and running
```sh
cargo build --release --target x86_64-unknown-linux-gnu
```
Or for Windows MSVC (or any other target you want!):
```
cargo build --release --target x86_64-pc-windows-msvc
```
If you like convenience and are running Linux, you can move the executable `manget` to the user's local bin directory so you can run the `manget` command anywhere in a terminal:
```sh
chmod 777 manget
sudo cp manget /usr/local/bin
```
There will be a documented way to do this for Windows soon.

## Examples

We will use the Manga ID for [*Initial D*](https://mangadex.org/title/21f54bc1-aefd-4be1-8284-5858b1df0e55/initial-d) as above in these examples.

```sh
# Bind the Polish translation of chapter 5 of Initial D in "fast" mode.
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 5 -l "pl" -f
```

Separate the chapters you want together with commas.
```sh
# Bind the English translation of chapters 1, 4, 5, and 7 of Initial D to a file called initial_d.pdf.
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 1,4,5,7 -o ./out/initial_d.pdf
```

If an alternative output file path is not specified, the bound PDF will be saved in the current working directory.


<hr>

## Acknowledgements 

Credit to the [MangaDex API](https://api.mangadex.org/docs/).

Please support the scanlation groups that scan and translate the manga you might read while using this application. Also, support the authors whenever possible.
