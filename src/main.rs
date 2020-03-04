mod ceo;
mod spider;
mod utils;

use chrono;
use clap::{Arg, App, SubCommand};
use fern;
use fern::colors::{Color, ColoredLevelConfig};
use std::result::Result;
use std::fs;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use tokio::runtime::Runtime;

use crate::ceo::{CatalogueKeywordsScrapingHook, read_keywords_from_file};
use crate::spider::Spider;
use crate::utils::SpiderError;

// #[macro_use] extern crate fstrings;


#[allow(dead_code)]
fn setup_logger() -> Result<(), fern::InitError> {
    let mut colors = ColoredLevelConfig::new()
        // use builder methods
        .info(Color::Green);
    // or access raw fields
    colors.warn = Color::Magenta;

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

fn main() -> Result<(), SpiderError> {
    // let _ = setup_logger();

    let matches = App::new("Web tools")
        .version("0.1.0")
        .author("Max Plutonium <plutonium.max@gmail.com>")
        .about("Web tools")
        .subcommand(SubCommand::with_name("ceo")
            .about("CEO commands")
            .subcommand(SubCommand::with_name("keywords")
                .about("Search keywords in web page")
                .arg(Arg::with_name("keywords_file")
                    .short("i")
                    .long("in")
                    .takes_value(true)
                    .help("Input file with keywords"))
                .arg(Arg::with_name("output_file")
                    .short("o")
                    .long("out")
                    .takes_value(true)
                    .help("Output file with result JSON"))
                .arg(Arg::with_name("max_pages")
                    .short("m")
                    .long("max_pages")
                    .takes_value(true)
                    .help("Max pages count for scraping"))
                .arg(Arg::with_name("site")
                    .help("Site url"))
            ))
        .get_matches();

    if let Some(ceo_matches) = matches.subcommand_matches("ceo") {
        if let Some(catalogue_matches) = ceo_matches.subcommand_matches("keywords") {
            let kw_file = catalogue_matches.value_of("keywords_file").expect("Need filename");
            let out_file = catalogue_matches.value_of("output_file").expect("Need output filename");
            let url = catalogue_matches.value_of("site").expect("Need URL");
            let max_pages: Option<u32> = match catalogue_matches.value_of("max_pages") {
                Some(max_pages) => match max_pages.parse::<u32>() {
                    Ok(value) => Some(value),
                    Err(_) => None
                },
                None => None
            };
            let kw = read_keywords_from_file(kw_file);

            let mut rt = Runtime::new()?;
            let mut spider = Spider::new(&url, max_pages)?;

            let hook = Rc::new(RefCell::new(CatalogueKeywordsScrapingHook::new(kw.clone())));
            spider.add_hook(hook.clone());

            let _num_pages =  rt.block_on(spider.scrape_links())?;
            let res = hook.as_ref().borrow().result();

            match fs::write(out_file, res) {
                Err(why) => panic!("couldn't write to file {}: {}", out_file,  why.description()),
                Ok(_) => print!("Output write to file {}\n", out_file),
            }
        }
    }

    Ok(())
}
