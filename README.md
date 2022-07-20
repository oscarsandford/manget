# manget

A CLI tool for binding digital pages of manga from MangaDex to PDFs for offline reading.


## Guide and General Usage

Unfortunately, MangaDex has made it difficult to scrape their search page, so the search functionality will have to wait. You must navigate to [mangadex.org](https://mangadex.org/) yourself and find a suitable manga to download yourself. We are mainly interested in the **manga ID**, or the highlighted part in the example below:

<p style="text-align: center;"><span>https://mangadex.org/title/<strong>21f54bc1-aefd-4be1-8284-5858b1df0e55</strong>/initial-d<span></p>

Peruse the page for chapters you would like to read, and note if the language you want them in is available. Then, consider the technical usage of the tool below.

```
manget 1.1.0
Oscar Sandford
A CLI tool for binding manga from MangaDex

USAGE:
    manget [OPTIONS] <ID>

ARGS:
    <ID>    The id of the manga

OPTIONS:
    -c, --chapter <CHAPTER>      Chapter number or closed interval (e.g. 1-5) [default: 1]
    -f, --fast                   Get compressed images instead of original quality
    -h, --help                   Print help information
    -i, --images                 Output individual images instead of a single PDF
    -l, --language <LANGUAGE>    The translated language [default: en]
    -o, --output <OUTPUT>        Specify an alternative base file name (no extension) [default: bound]
    -v, --verbose                Increase verbosity in console output
    -V, --version                Print version information
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
You can do something similar for Windows or MacOS by following the instructions [here](https://zwbetz.com/how-to-add-a-binary-to-your-path-on-macos-linux-windows/#windows-cli).

## Examples

We will use the Manga ID for [*Initial D*](https://mangadex.org/title/21f54bc1-aefd-4be1-8284-5858b1df0e55/initial-d) as above in these examples.

Bind a single chapter and choose a language instead of English to retrieve.
```sh
# Bind the Polish translation of chapter 5 of Initial D in "fast" mode.
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 5 -l "pl" -f
```

Specify the chapters you want in a range.
```sh
# Bind the English translation of chapters 1, 2, 3, 4, and 5 of Initial D to a file called `initial_d.pdf`. Verbose mode is enabled.
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 1-5 -o initial_d -v
```

Alternatively, simply dump the images in the current working directory instead of binding them to a PDF. Note that the `-f` option will save them as JPGs instead of PNGs.
```sh
# Save images of the pages from the English translation of chapters 1 and 2 of Initial D to files `imag_1.jpg`, `imag_2.jpg`, ...
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 1-2 -o imag -i
```


<hr>

## Acknowledgements 

Credit to the [MangaDex API](https://api.mangadex.org/docs/).

Please support the scanlation groups that scan and translate the manga you might read while using this application. Also, support the authors whenever possible.
