use url::Url;
use select::document::Document;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use crate::utils::SpiderError;
use select::predicate::Name;


pub trait Hook {
    fn can_match(&self, url: &Url) -> bool;
    fn hook(&mut self, url: &Url, doc: &Document) -> ();
}

pub struct Spider {
    base_url: Url,
    links_visited: HashSet<Url>,
    page_hooks: Vec<Rc<RefCell<dyn Hook>>>,
    num_pages: Option<u32>
}

impl Spider {
    pub fn new(base_url: &str, num_pages: Option<u32>) -> std::result::Result<Self, SpiderError> {
        let base_url = Url::parse(base_url)?;
        Ok(Self {
            base_url,
            links_visited: HashSet::new(),
            page_hooks: Vec::new(),
            num_pages
        })
    }

    pub fn add_hook(&mut self, hook: Rc<RefCell<dyn Hook>>) -> () {
        self.page_hooks.push(hook);
    }

    pub async fn scrape_links(&mut self) -> std::result::Result<u32, SpiderError> {
        let mut queue = VecDeque::<Url>::new();
        let mut num_pages = 0u32;
        queue.push_back(self.base_url.clone());

        while let Some(url) = queue.pop_front() {
            if !self.links_visited.contains(&url) {
                let links: HashSet<Url> = self._visit_page(&url).await?
                    .into_iter()
                    .filter(|x| !self.links_visited.contains(&x))
                    .collect();
                queue.extend(links);

                num_pages += 1;
                if let Some(max_pages) = self.num_pages {
                    if num_pages >= max_pages { break; }
                }
            }
        }

        Ok(num_pages)
    }

    async fn _visit_page(&mut self, page: &Url)
                         -> std::result::Result<HashSet<Url>, SpiderError> {
        let res = reqwest::get(page.as_str()).await?;
        let body: String = res.text().await?;
        let base_parser = Url::options().base_url(Some(&self.base_url));

        match Document::from_read(body.as_bytes()) {
            Ok(document) => {
                self.links_visited.insert(page.clone());
                for hook in &self.page_hooks {
                    if hook.borrow().can_match(&page) {
                        hook.borrow_mut().hook(&page, &document)
                    }
                }

                let links: HashSet<Url> = document
                    .find(Name("a"))
                    .filter_map(|n| n.attr("href"))
                    .filter(|link| link.starts_with("/"))
                    .filter_map(|link| base_parser.parse(link).ok())
                    .collect();

                Ok(links)
            },
            Err(error) => Err(error.into())
        }
    }
}
