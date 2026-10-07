//! A tolerant parser for Valve's KeyValues text format (`.vdf`, `.acf`).
//!
//! ```text
//! "AppState"
//! {
//!     "appid"   "570"
//!     "name"    "Dota 2"
//! }
//! ```

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Obj(Vec<(String, Value)>),
}

impl Value {
    /// Case-insensitive child lookup; Valve isn't consistent about casing.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(entries) => entries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Value::Str(_) => None,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Value::Str(s) => Some(s),
            Value::Obj(_) => None,
        }
    }

    pub fn entries(&self) -> &[(String, Value)] {
        match self {
            Value::Obj(entries) => entries,
            Value::Str(_) => &[],
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Open,
    Close,
}

fn tokenize(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => tokens.push(Token::Open),
            '}' => tokens.push(Token::Close),
            '"' => {
                let mut s = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => s.push(other),
                            None => break,
                        },
                        _ => s.push(c),
                    }
                }
                tokens.push(Token::Str(s));
            }
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            c if c.is_whitespace() => {}
            // Unquoted token (rare, but legal): read until whitespace or brace.
            _ => {
                let mut s = String::from(c);
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                tokens.push(Token::Str(s));
            }
        }
    }
    tokens
}

/// Parses a document into an object holding its top-level pairs.
/// Malformed input yields whatever could be read rather than an error;
/// a half-written manifest shouldn't hide the rest of the library.
pub fn parse(src: &str) -> Value {
    let tokens = tokenize(src);
    let mut iter = tokens.into_iter();
    Value::Obj(parse_entries(&mut iter))
}

fn parse_entries(iter: &mut impl Iterator<Item = Token>) -> Vec<(String, Value)> {
    let mut entries = Vec::new();
    loop {
        let key = match iter.next() {
            Some(Token::Str(key)) => key,
            Some(Token::Close) | None => return entries,
            Some(Token::Open) => {
                // Stray brace: skip its block.
                parse_entries(iter);
                continue;
            }
        };
        match iter.next() {
            Some(Token::Str(value)) => entries.push((key, Value::Str(value))),
            Some(Token::Open) => entries.push((key, Value::Obj(parse_entries(iter)))),
            Some(Token::Close) | None => return entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
"AppState"
{
	"appid"		"291550"
	"name"		"Brawlhalla"
	// comments are allowed
	"installdir"		"Brawlhalla"
	"UserConfig"
	{
		"language"		"english"
	}
}
"#;

    #[test]
    fn parses_nested_objects() {
        let doc = parse(MANIFEST);
        let app = doc.get("appstate").unwrap();
        assert_eq!(app.str("appid"), Some("291550"));
        assert_eq!(app.str("NAME"), Some("Brawlhalla"));
        assert_eq!(
            app.get("UserConfig").unwrap().str("language"),
            Some("english")
        );
    }

    #[test]
    fn unescapes_windows_paths() {
        let doc = parse(r#""path" "E:\\Steam\\steamapps""#);
        assert_eq!(doc.str("path"), Some(r"E:\Steam\steamapps"));
    }

    #[test]
    fn survives_truncated_input() {
        let doc = parse(r#""a" { "b" "1" "c" { "d""#);
        assert_eq!(doc.get("a").unwrap().str("b"), Some("1"));
    }
}
