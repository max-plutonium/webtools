use select::predicate::Predicate;
use url::ParseError;
use select::node::Node;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Id<T>(pub T);

impl<'a> Predicate for Id<&'a str> {
    fn matches(&self, node: &Node) -> bool {
        node.attr("id") == Some(self.0)
    }
}

#[derive(Debug)]
pub enum SpiderError {
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Parse(ParseError)
}

impl From<std::io::Error> for SpiderError {
    fn from(err: std::io::Error) -> SpiderError {
        SpiderError::Io(err)
    }
}

impl From<reqwest::Error> for SpiderError {
    fn from(err: reqwest::Error) -> SpiderError {
        SpiderError::Reqwest(err)
    }
}

impl From<ParseError> for SpiderError {
    fn from(err: ParseError) -> SpiderError {
        SpiderError::Parse(err)
    }
}
