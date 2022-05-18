//! A gemtext parser
//!
//! This library will parse gemtext into various [Nodes](GemtextNode)

#[derive(Debug, Eq, PartialEq, Clone)]
/// A singular gemtext node.
pub enum GemtextNode {
    /// A pure text block. The string contained within is the entire line of text
    Text(String),
    /// A link.
    ///
    /// A link is found by a line starting with the characters "=>" followed by a space
    ///
    /// The first string contained is the link itself and the second string is an optional
    /// descriptor
    Link(String, Option<String>),
    /// A prompt (Spartan only)
    ///
    /// A prompt is a line starting with the characters "=:" followed by a space
    /// The first string is the link and any further characters make up the optional
    /// display string
    Prompt(String, Option<String>),
    /// A heading
    /// A heading starts with a singular # with a space following.
    /// The string contained is the text that follows the heading marker
    Heading(String),
    /// A subheading
    ///
    /// A subheading starts with the characters "##" with a space following.
    /// The string contained is the text that folllows the subheading marker.
    SubHeading(String),
    /// A subsubheading
    ///
    /// A subsubheading starts with the characters "###" with a space following.
    /// The string contained is the text that follows the subheading marker
    SubSubHeading(String),
    /// A list item
    ///
    /// A list item starts with the character "*".
    /// Unlike markdown, '-' is not allowed to start a list item
    ///
    /// The string contained is the text that follows the list item marker.
    ListItem(String),
    /// A block quote
    ///
    /// A blockquote starts with the character ">".
    ///
    /// The string contained is the text that follows the blockquote marker
    Blockquote(String),
    /// A block of preformatted text.
    ///
    /// A preformatted text block starts with the characters "\`\`\`"
    ///
    /// The first string contained is the text within the preformatted text (newlines and all). The second string is an optional formatting tag for the preformatted text a la Markdown. It's worth noting that this is more clearly listed as an "alt text" as opposed to a formatting tag.
    Preformatted(String, Option<String>),
    /// A singular empty line
    EmptyLine,
}

impl core::fmt::Display for GemtextNode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            GemtextNode::EmptyLine => write!(f, ""),
            GemtextNode::Text(s) => write!(f, "{}", s),
            GemtextNode::Link(s, None) => {
                write!(f, "=> {}", s)
            }
            GemtextNode::Link(s, Some(d)) => {
                write!(f, "=> {} {}", s, d)
            }
            Self::Prompt(s, None) => {
                write!(f, "=: {}", s)
            }
            Self::Prompt(s, Some(d)) => {
                write!(f, "=: {} {}", s, d)
            }
            GemtextNode::Heading(s) => write!(f, "# {}", s),
            GemtextNode::SubHeading(s) => write!(f, "## {}", s),
            GemtextNode::SubSubHeading(s) => write!(f, "### {}", s),
            GemtextNode::ListItem(s) => write!(f, "* {}", s),
            GemtextNode::Blockquote(s) => write!(f, "> {}", s),
            GemtextNode::Preformatted(s, None) => {
                write!(f, "```\n{}\n```", s)
            }
            GemtextNode::Preformatted(s, Some(d)) => {
                write!(f, "```{}\n{}\n```", d, s)
            }
        }
    }
}

#[derive(Debug)]
enum ParseState {
    Searching,
    Text,
    FirstLinkChar,
    SecondLinkChar,
    SecondPromptChar,
    LinkLink,
    PromptLink,
    LinkDesc,
    PromptDesc,
    ListWaitForSpace,
    ListItem,
    FirstTick,
    SecondTick,
    PreformattedTextType,
    HeadingStart,
    Heading,
    SubHeadingStart,
    SubHeading,
    SubSubHeadingStart,
    SubSubHeading,
    BlockquoteStart,
    Blockquote,
}

#[must_use]
/// Parse gemtext into a vector of [`GemtextNode`]s
///
/// This will take a [`&str`] and return a vector of [`GemtextNode`]s. Because of the nature of the way
/// gemtext works, this parsing step cannot fail. It can only return garbage.
///
/// # Example:
/// ```
/// # use gemview::scheme::gemini::parser;
/// # fn main() {
/// let text = r#"# A test page!
/// Hello! This is a test page!"#;
/// let gemtext_nodes = parser::parse_gemtext(text);
/// if let parser::GemtextNode::Heading(s) = &gemtext_nodes[0] {
///     assert_eq!(s, "A test page!");
/// } else {
///     panic!("Incorrect type!");
/// }
/// # }
///
pub fn parse_gemtext(text: &str) -> Vec<GemtextNode> {
    // Let's define our parsing flags
    let mut is_in_preformatted = false;
    let mut preformatted_text_has_type = false;

    let mut nodes: Vec<GemtextNode> = Vec::new();
    let mut preformatted_text: String = String::new();
    let mut preformatted_text_type: String = String::new();

    for line in text.lines() {
        if is_in_preformatted {
            if line.starts_with("```") {
                if preformatted_text_has_type {
                    nodes.push(GemtextNode::Preformatted(
                        preformatted_text.clone(),
                        Some(preformatted_text_type.clone()),
                    ));
                } else {
                    nodes.push(GemtextNode::Preformatted(preformatted_text.clone(), None));
                }
                is_in_preformatted = false;
                preformatted_text.clear();
            } else {
                preformatted_text.push_str(line);
                preformatted_text.push('\n');
            }
            continue;
        }
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            nodes.push(GemtextNode::EmptyLine);
            continue;
        }
        // A simple enum to keep our parsing state
        let mut current_parse_state: ParseState = ParseState::Searching;
        let mut temp1 = String::new();
        let mut temp2 = String::new();
        // Go character by character and set our state accordingly
        for c in line.chars() {
            match current_parse_state {
                ParseState::Searching => match c {
                    '=' => current_parse_state = ParseState::FirstLinkChar,
                    '*' => current_parse_state = ParseState::ListWaitForSpace,
                    '`' => current_parse_state = ParseState::FirstTick,
                    '#' => current_parse_state = ParseState::HeadingStart,
                    '>' => current_parse_state = ParseState::BlockquoteStart,
                    _ => {
                        current_parse_state = ParseState::Text;
                    }
                },
                //=====
                //Text parsing
                //=====
                ParseState::Text => break,
                //=====
                //Link parsing
                //=====
                ParseState::FirstLinkChar => match c {
                    '>' => current_parse_state = ParseState::SecondLinkChar,
                    ':' => current_parse_state = ParseState::SecondPromptChar,
                    _ => {
                        current_parse_state = ParseState::Text;
                    }
                },
                ParseState::SecondLinkChar => {
                    if !c.is_whitespace() {
                        current_parse_state = ParseState::LinkLink;
                        temp1.push(c);
                    }
                }
                ParseState::SecondPromptChar => {
                    if !c.is_whitespace() {
                        current_parse_state = ParseState::PromptLink;
                        temp1.push(c);
                    }
                }
                ParseState::LinkLink => {
                    if c.is_whitespace() {
                        current_parse_state = ParseState::LinkDesc;
                    } else {
                        temp1.push(c);
                    }
                }
                ParseState::PromptLink => {
                    if c.is_whitespace() {
                        current_parse_state = ParseState::PromptDesc;
                    } else {
                        temp1.push(c);
                    }
                }
                ParseState::LinkDesc | ParseState::PromptDesc => temp2.push(c),
                //=====
                //List parsing
                //=====
                ParseState::ListWaitForSpace => {
                    if c.is_whitespace() {
                        current_parse_state = ParseState::ListItem;
                    } else {
                        current_parse_state = ParseState::Text;
                    }
                }
                ParseState::ListItem
                | ParseState::Heading
                | ParseState::SubHeading
                | ParseState::Blockquote
                | ParseState::SubSubHeading => temp1.push(c),
                //======
                //Preformatted text
                //======
                ParseState::FirstTick => {
                    if c == '`' {
                        current_parse_state = ParseState::SecondTick;
                    } else {
                        current_parse_state = ParseState::Text;
                    }
                }
                ParseState::SecondTick => {
                    if c == '`' {
                        current_parse_state = ParseState::PreformattedTextType;
                        preformatted_text_type.clear();
                    } else {
                        current_parse_state = ParseState::Text;
                    }
                }
                ParseState::PreformattedTextType => preformatted_text_type.push(c),
                //=====
                //Headings
                //=====
                ParseState::HeadingStart => {
                    if c == '#' {
                        current_parse_state = ParseState::SubHeadingStart;
                    } else if !c.is_whitespace() {
                        current_parse_state = ParseState::Text;
                    } else {
                        current_parse_state = ParseState::Heading;
                    }
                }
                ParseState::SubHeadingStart => {
                    if c == '#' {
                        current_parse_state = ParseState::SubSubHeadingStart;
                    } else if !c.is_whitespace() {
                        current_parse_state = ParseState::Text;
                    } else {
                        current_parse_state = ParseState::SubHeading;
                    }
                }
                ParseState::SubSubHeadingStart => {
                    if c == '#' {
                        current_parse_state = ParseState::SubSubHeading;
                    } else if !c.is_whitespace() {
                        current_parse_state = ParseState::Text;
                    } else {
                        current_parse_state = ParseState::SubSubHeading;
                    }
                }
                ParseState::BlockquoteStart => {
                    if c.is_whitespace() {
                        current_parse_state = ParseState::Blockquote;
                    } else {
                        current_parse_state = ParseState::Text;
                    }
                }
            }
        }
        // Clean up any parse state we are in
        match current_parse_state {
            ParseState::Text => nodes.push(GemtextNode::Text(line.to_string())),

            ParseState::SecondLinkChar | ParseState::SecondPromptChar => {
                nodes.push(GemtextNode::Text("=".to_string()));
            }
            ParseState::LinkLink | ParseState::PromptLink => {
                nodes.push(GemtextNode::Link(temp1, None));
            }
            ParseState::LinkDesc => {
                if temp2.is_empty() {
                    nodes.push(GemtextNode::Link(temp1, None));
                } else {
                    nodes.push(GemtextNode::Link(temp1, Some(temp2)));
                }
            }
            ParseState::PromptDesc => {
                if temp2.is_empty() {
                    nodes.push(GemtextNode::Prompt(temp1, None));
                } else {
                    nodes.push(GemtextNode::Prompt(temp1, Some(temp2)));
                }
            }

            ParseState::ListItem => nodes.push(GemtextNode::ListItem(temp1)),

            ParseState::FirstTick => nodes.push(GemtextNode::Text("`".to_string())),
            ParseState::SecondTick => nodes.push(GemtextNode::Text("``".to_string())),
            ParseState::PreformattedTextType => {
                is_in_preformatted = true;
                if preformatted_text_type.is_empty() {
                    preformatted_text_has_type = false;
                } else {
                    preformatted_text_has_type = true;
                }
            }
            ParseState::Heading => nodes.push(GemtextNode::Heading(temp1)),
            ParseState::HeadingStart => nodes.push(GemtextNode::Text("#".to_string())),
            ParseState::SubHeading => nodes.push(GemtextNode::SubHeading(temp1)),
            ParseState::SubHeadingStart => nodes.push(GemtextNode::Text("##".to_string())),
            ParseState::SubSubHeading => nodes.push(GemtextNode::SubSubHeading(temp1)),
            ParseState::SubSubHeadingStart => nodes.push(GemtextNode::Text("###".to_string())),
            ParseState::Blockquote => nodes.push(GemtextNode::Blockquote(temp1)),
            ParseState::BlockquoteStart => nodes.push(GemtextNode::Blockquote("".to_string())),
            s => panic!("Invalid state: {:?}", s),
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    macro_rules! test_prelude {
        ($n:ident, $c:tt) => {
            #[test]
            fn $n() {
                use $crate::gemini::parser::*;
                $c
            }
        };
    }
    //
    //====
    //
    test_prelude!(display_test, {
        // Text
        assert_eq!(
            GemtextNode::Text(String::from("This is a test")).to_string(),
            "This is a test"
        );
        // Link
        assert_eq!(
            GemtextNode::Link(String::from("gemini://link_test"), None).to_string(),
            "=> gemini://link_test"
        );
        assert_eq!(
            GemtextNode::Link(
                String::from("gemini://link_test"),
                Some(String::from("A test lol"))
            )
            .to_string(),
            "=> gemini://link_test A test lol"
        );
        // Heading
        assert_eq!(
            GemtextNode::Heading(String::from("A test heading")).to_string(),
            "# A test heading"
        );
        // Subheading
        assert_eq!(
            GemtextNode::SubHeading(String::from("A test subheading")).to_string(),
            "## A test subheading"
        );
        // Subsubheading
        assert_eq!(
            GemtextNode::SubSubHeading(String::from("A test subsubheading")).to_string(),
            "### A test subsubheading"
        );
        // List Item
        assert_eq!(
            GemtextNode::ListItem(String::from("A list item")).to_string(),
            "* A list item"
        );
        // Blockquote
        assert_eq!(
            GemtextNode::Blockquote(String::from("A blockquote test")).to_string(),
            "> A blockquote test"
        );
        // Preformatted
        assert_eq!(
            GemtextNode::Preformatted(String::from("A preformatted block"), None).to_string(),
            "```\nA preformatted block\n```"
        );
        assert_eq!(
            GemtextNode::Preformatted(
                String::from("A preformatted block"),
                Some(String::from("with alt text"))
            )
            .to_string(),
            "```with alt text\nA preformatted block\n```"
        );
        // Empty line
        assert_eq!(GemtextNode::EmptyLine.to_string(), "");
    });
    //
    //===
    //
    test_prelude!(parse_gemtext, {
        let test_article = r#"# Hello!
This is a test article for using to test the parsing of the gemtext stuff! For example, the next thing is a link!
=> gemini://a_test_link
And next is a link with some alt text
=> gemini://a_test_link some alt text
And now we'll get a subheading in here. And why not? We'll throw an empty line before it!

## A subheading
We'll also do a subsubheading
### A subsubheading
Then we'll do some list items
* list item 1
* list item 2
* list item 3
And a blockquote
> Just do it!
And we'll do some preformatted text with no alt text
```
fn main() {
    println!("Hello world!");
}
```
And some preformatted text with alt text
```rust
fn main() {
    println!("Goodbye world!");
}
```"#;
        let test_article_parsed = vec![GemtextNode::Heading(String::from("Hello!")),
        GemtextNode::Text(String::from("This is a test article for using to test the parsing of the gemtext stuff! For example, the next thing is a link!")),
        GemtextNode::Link(String::from("gemini://a_test_link"), None),
        GemtextNode::Text(String::from("And next is a link with some alt text")),
        GemtextNode::Link(String::from("gemini://a_test_link"), Some(String::from("some alt text"))),
        GemtextNode::Text(String::from("And now we'll get a subheading in here. And why not? We'll throw an empty line before it!")),
        GemtextNode::EmptyLine,
        GemtextNode::SubHeading(String::from("A subheading")),
        GemtextNode::Text(String::from("We'll also do a subsubheading")),
        GemtextNode::SubSubHeading(String::from("A subsubheading")),
        GemtextNode::Text(String::from("Then we'll do some list items")),
        GemtextNode::ListItem(String::from("list item 1")),
        GemtextNode::ListItem(String::from("list item 2")),
        GemtextNode::ListItem(String::from("list item 3")),
        GemtextNode::Text(String::from("And a blockquote")),
        GemtextNode::Blockquote(String::from("Just do it!")),
        GemtextNode::Text(String::from("And we'll do some preformatted text with no alt text")),
        GemtextNode::Preformatted(String::from(r#"fn main() {
    println!("Hello world!");
}
"#), None),
        GemtextNode::Text(String::from("And some preformatted text with alt text")),
        GemtextNode::Preformatted(String::from(r#"fn main() {
    println!("Goodbye world!");
}
"#), Some(String::from("rust")))
        ];
        // Parse the article
        let actual_parsed_article = parse_gemtext(test_article);
        for (actual_article_node, test_article_node) in
            actual_parsed_article.iter().zip(test_article_parsed.iter())
        {
            assert_eq!(actual_article_node, test_article_node);
        }
    });
}
