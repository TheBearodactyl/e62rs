# e62rs

A hyper-customizable in-terminal e621/e926 browser and downloader

---

# Features

- 80+ available [configuration options](https://github.com/TheBearodactyl/e62rs/wiki/Configuration), and counting
- A fully in-terminal image viewer with no quality loss (via [icy_sixel](https://github.com/mkrueger/icy_sixel/))
- A completely offline downloads re-organizer
- A downloads browser, available in both CLI and Web flavors (also completely offline)
- A really fucking fast batch post downloader
- Automatic metadata storing (saves to `<imgpath>.json` on Unix like systems, and `<imgpath>:metadata` on Windows systems)
- Full support for [DText](https://e621.net/help/dtext.html) when viewing post info

---

# TODOs

## Bugs/Problems

- [x] Fix menu flow bugs (e.g. pressing back doesn't actually go back)
- [x] Finish the on-disk cache implementation

## Features

- [ ] Add GIF support for viewing downloaded images in-terminal (will require a really janky setup)
- [x] Add more filters to post and pool searching
- [x] Add more configuration options

## Code Stuff

- [x] Refactor the whole thing into something that isn't held together with duct tape and a dream
- [ ] Add unit tests and documentation for like... everything
  - [ ] Unit tests
  - [x] Documentation
