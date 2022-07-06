# manget

A CLI tool for binding digital pages of manga from MangaDex to PDFs for offline reading.

## Installation

TODO


## Guide and General Usage

Unfortunately, MangaDex has made it difficult to scrape their search page, so the search functionality will have to wait. You must navigate to [mangadex.org](https://mangadex.org/) yourself and find a suitable manga to download yourself. We are mainly interested in the **manga ID**, or the highlighted part in the example below:

<p style="text-align: center;">https://mangadex.org/title/<strong>21f54bc1-aefd-4be1-8284-5858b1df0e55</strong>/initial-d</p>

Peruse the page for chapters you would like to read, and note if the language you want them in is available. Then, consider the technical usage of the tool below.

```
manget 0.1.0
oes
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
    -V, --version                Print version information
        --verbose                Increase verbosity in console output
```


## Examples

We will use the Manga ID for [*Initial D*](https://mangadex.org/title/21f54bc1-aefd-4be1-8284-5858b1df0e55/initial-d) as above in these examples.

```sh
# Bind the Polish translation of chapter 5 of Initial D in "fast" mode.
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 5 -l "pl" -f
```

Separate the chapters you want together with commas.
```sh
# Bind the English translation of chapters 1, 4, 5, and 7 of Initial D.
manget 21f54bc1-aefd-4be1-8284-5858b1df0e55 -c 1,4,5,7
```

When complete, the bound PDF will be saved in the current working directory.


<hr>

## Acknowledgements 

Credit to the [MangaDex API](https://api.mangadex.org/docs/).

Please support the scanlation groups that scan and translate the manga you might read while using this application. Also, support the authors whenever possible.
