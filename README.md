# GemView
Contents
========
- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
## Introduction
GemView is a [gemini protocol](https://gemini.circumlunar.space/) browser widget
for gtk+ (version 4) implemented in Rust.
## Features
- [x] Browse and render gemini gemtext content
- [ ] Display plain text over gemini
- [ ] Browse and render gopher and plain text over gopher
- [ ] Display images served over gemini/gopher
- [x] Open http(s) links in a *normal* browser
- [x] User customizable fonts
- [ ] User customizable colors
- [x] Back/forward list
- [ ] History

## Usage
```Yaml
[dependencies]
gemview = { git = "https://codeberg.org/jeang3nie/gemview" }

[dependencies.gtk]
version = "~0.4"
package = "gtk4"
```
```Rust
use gemview::GemView;
use gtk::prelude::*;

let browser = GemView::default();
let scroller = gtk::builders::ScrolledWindowBuilder::new()
    .child(&browser)
    .hexpand(true)
    .vexpand(true)
    .build();
let window = gtk::builders::WindowBuilder::new()
    .child(&scroller)
    .title("GemView")
    .build()
window.show();
browser.visit("gemini://gemini.circumlunar.space");
```
