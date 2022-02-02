# GemView
Contents
========
[Introduction](#introduction)
[Usage](#usage)
## Introduction
GemView is a [gemini protocol](https://gemini.circumlunar.space/) browser widget
for gtk+ (version 4) implemented in Rust.
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
