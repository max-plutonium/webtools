use std::collections::{HashSet, HashMap};
use url::Url;
use select::predicate::{And, Name, Class};
use select::document::Document;
use crate::spider::Hook;
use crate::utils::Id;
use calamine::{Xlsx, open_workbook, Reader};


pub struct CatalogueKeywordsScrapingHook {
    keywords: HashSet<String>,
    page_map: HashMap<String, Vec<String>>
}

impl CatalogueKeywordsScrapingHook {
    pub fn new(keywords: HashSet<String>) -> Self {
        Self { keywords, page_map: HashMap::new() }
    }

    pub fn result(&self) -> String {
        serde_json::to_string(&self.page_map).unwrap_or_default()
    }
}

impl Hook for CatalogueKeywordsScrapingHook {
    fn can_match(&self, url: &Url) -> bool {
        url.path().starts_with("/catalogue/")
    }

    fn hook(&mut self, url: &Url, doc: &Document) -> () {
        let mut res = String::new();
        doc.find(And(And(Name("div"), Id("panel1")), Class("tabs-panel")))
            .for_each(|node| res.push_str(&node.text()));

        for keyword in &self.keywords {
            if res.to_lowercase().contains(keyword) {
                let vec = self.page_map.entry(url.path().to_owned()).or_default();
                vec.push(keyword.to_owned());
            }
        }
    }
}

pub fn read_keywords_from_file(path: &str) -> HashSet<String> {
    let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot open file");

    let sheets = workbook.sheet_names().to_owned();
    assert!(sheets.len() >= 1);
    let sheet = workbook.worksheet_range_at(0)
        .expect("Need at least one sheet");

    let mut kw = HashSet::new();

    for s in sheet.iter() {
        for row in s.rows() {
            let item = &row[0];
            let keywords = item.get_string().unwrap_or("");
            kw.insert(keywords.to_lowercase());
        }
    }

    return kw;
}
