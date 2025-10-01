use askama::Template;
use rss::Channel;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Template)]
#[template(path = "full.html")]
struct FullTemplate {
    items: Vec<TemplateItem>,
}

#[derive(Template)]
#[template(path = "partial.html")]
struct PartialTemplate {
    items: Vec<TemplateItem>,
}

#[derive(Clone)]
struct TemplateItem {
    title: String,
    description: Option<String>,
    url: String,
}

fn fetch_feed() -> Result<Channel, Box<dyn Error>> {
    let url = "https://links.poxar.net/feeds/shared";
    let mut response = reqwest::blocking::get(url)?;

    let mut rss_file = File::create("reading.rss")?;
    std::io::copy(&mut response, &mut rss_file)?;

    let rss_file = File::open("reading.rss")?;
    let channel = Channel::read_from(BufReader::new(rss_file))?;
    Ok(channel)
}

fn main() {
    let channel = fetch_feed().unwrap_or_else(|err| {
        eprintln!("Failed to fetch feed");
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let items: Vec<TemplateItem> = channel
        .items()
        .iter()
        .flat_map(|item| {
            match (item.title(), item.description(), item.link()) {
                (Some(title), description, Some(url)) => {
                    return Some(TemplateItem {
                        title: title.to_string(),
                        description: description.map(|d| d.to_string()),
                        url: url.to_string(),
                    })
                }
                _ => return None,
            };
        })
        .collect();

    let partial = PartialTemplate {
        items: items.iter().take(5).cloned().collect(),
    };
    let partial_render = partial.render().unwrap();
    let mut partial_file = File::create("reading_partial.html").unwrap();
    partial_file.write_all(partial_render.as_bytes()).unwrap();

    let full = FullTemplate {
        items: items.iter().take(30).cloned().collect(),
    };
    let full_render = full.render().unwrap();
    let mut full_file = File::create("reading_full.html").unwrap();
    full_file.write_all(full_render.as_bytes()).unwrap();
}
