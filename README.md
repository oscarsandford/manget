# manget

A CLI tool for binding digital pages of manga from MangaDex to PDFs for offline reading.

## Setup

### Troubleshooting

If you get an error while building like [this](https://github.com/rust-lang/rls/issues/250), then try
```
sudo apt-get install pkg-config
```
and build again.


## Usage

The following example shows how to bind chapter 5 of the test manga in the Polish language to a PDF. Note that as of now, the value "test" is arbitrary and set in the program. In the future, we will have an option to search for a manga title and then select which chapters the user wants from manga.
```
manget test -c 5 -l "pl"
```


## Acknowledgements 

Credit to the [MangaDex API](https://api.mangadex.org/docs/).

Please support the scanlation groups that scan and translate the manga you might read while using this application. Also, support the authors whenever possible.

