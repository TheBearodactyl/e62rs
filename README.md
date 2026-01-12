# e62rs

A hyper-customizable in-terminal e621/e926 browser and downloader

---

# Features

- 100+ available [configuration options](https://github.com/TheBearodactyl/e62rs/wiki/Configuration), and counting
- A fully in-terminal image viewer with animation support and no quality loss (via [icy_sixel](https://github.com/mkrueger/icy_sixel/))
- A completely offline downloads re-organizer
- A downloads browser, available in both CLI and Web flavors (also completely offline)
- A really fucking fast batch post downloader
- Automatic metadata storing (saves to `<imgpath>.json` on Unix like systems, and `<imgpath>:metadata` on Windows systems)
- Full support for [DText](https://e621.net/help/dtext.html) when viewing post info
- Support for other languages (currently Spanish and Japanese)

---

# TODOs

## Localization

- [x] **ENGLISH**
- [x] Labels: 60/60 (100.00%)
- [x] Descriptions: 60/60 (100.00%)

- [ ] **SPANISH**
- [ ] Labels: 34/60 (56.67%)
- [ ] Descriptions: 32/60 (53.33%)

- [ ] **JAPANESE**
- [ ] Labels: 1/60 (1.67%)
- [ ] Descriptions: 1/60 (1.67%)

## Documentation

- [x] Short crate level documentation
- [ ] Long crate level documentation
- [x] Short descriptions for methods/functions
- [ ] Long descriptions for methods/functions
- [x] Short struct documentation
- [x] Short enum documentation
- [ ] Long struct documentation
- [ ] Long enum documentation
- [ ] Errors section for methods/functions returning a `Result`
- [ ] Arguments section for methods/functions with parameters
- [x] Struct field documentation
- [x] Enum variant documentation

## Bugs/Problems

- [x] Fix menu flow bugs (e.g. pressing back doesn't actually go back)
- [x] Finish the on-disk cache implementation

## Features

- [x] Add GIF/WebP support for viewing downloaded images in-terminal
- [x] Add more filters to post and pool searching
- [x] Add more configuration options

## Code Stuff

- [x] Refactor the whole thing into something that isn't held together with duct tape and a dream
- [ ] Add unit tests and documentation for like... everything
    - [ ] Unit tests
    - [x] Documentation
