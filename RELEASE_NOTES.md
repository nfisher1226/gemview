Contents
========
* [0.5.2](#0.5.2-release)
* [0.5.0](#0.5.0-release)
* [0.4.0](#0.4.0-release)

## 0.5.2 release
Fix preformatted regression caused by new gemtext parser
* The old parser always inserted an empty newline at the end of every
  preformatted block
* This was being compensated for by truncating the block by one character
* Without the trailing newline, this was truncating the fina non-whitespace
  character of every preformatted block
* The fix removes the truncation.
  * Bonus - the variable is now immutable.

## 0.5.0 release
* add `connect_request_input_sensitive` method
* use colored icons next to links to show different protocols
* link rendering - common code moved into traits for re-use
* add initial support for Spartan protocol
* add `connect_request_upload` method
* gemini::parser - add Spartan prompt line support
* finish Spartan upload support
* Gemini/Spartan - double check file mime type and attempt to handle in
  application before falling back to downloading
* Gemini - rewrite of parser
  * Consecutive blockquote lines are grouped together
  * Reduced allocations by specifying lifetimes for &str's
  * main parser loop is significantly shorter, with no nested loop

## 0.4.0 release
* improved handling up "request-download" signal
* significant code cleanups
* Simplify imports
* Fix broken tests
