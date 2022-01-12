# bsmart_downloader-rs
Download your books from bSmart as offline pdf in rust.
This project is based on [bSmart-downloader](https://github.com/Leone25/bSmart-downloader).

## How to use

### Usage

This guide assumes that you already have rust installed on your system.
If you don't, download it from [here](https://www.rust-lang.org/tools/install)

1. Download and extract this repo
2. Open a terminal window in the folder where you extracted the repo
3. Run 'cargo run'
4. Open the dev tools (F12) and go to the storage(Firefox) or application(Chromium) tab, there click on `Cookie`, then `https://my.bsmart.it`, then copy in the terminal the cookie called `_bsw_session_v1_production`
5. Input the id of the book you'd like to download, either from the list or from the url, after `/books/`. It's usually a 4 digit number
6. Press enter and the program will start working, a file will be saved in the folder of the repo

### Installation

If you want to install the package before using it:
1. Download and extract this repo
2. Open a terminal window in the folder where you extracted the repo
3. Run 'cargo install --path .'
4. Now you'll be able to use the tool from your terminal with the command 'bsmart_downloader-rs'
5. Open the dev tools (F12) and go to the storage(Firefox) or application(Chromium) tab, there click on `Cookie`, then `https://my.bsmart.it`, then copy in the terminal the cookie called `_bsw_session_v1_production`
6. Input the id of the book you'd like to download, either from the list or from the url, after `/books/`. It's usually a 4 digit number
7. Press enter and the program will start working, a file will be saved in the folder of the repo



ISC License

Copyright (c) 2022 Andrea Squitieri 

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
