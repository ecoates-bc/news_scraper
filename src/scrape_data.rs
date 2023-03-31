use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
struct Article {
    title: String,
    contents: Vec<String>,
    date: String,
}

fn get_date_from_timestamp(doc: &Html) -> Result<String> {
    let time_selector = Selector::parse("time.timeStamp").expect("Unable to create selector");
    let time_elem = doc.select(&time_selector).next();

    if let Some(time_elem) = time_elem {
        let date_regex = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2}).*")?;
        let pub_date = time_elem
            .value()
            .attr("datetime")
            .ok_or(anyhow!("Cannot find"))?;
        let pub_date = date_regex.replace_all(pub_date, "$d-$m-$y");

        Ok(String::from(pub_date))
    } else {
        Err(anyhow!("Cannot find timestamp."))?
    }
}

fn get_date_from_div(doc: &Html) -> Result<String> {
    let date_selector =
        Selector::parse("span.published-date__since").expect("Unable to create selector");
    let date_elem = doc.select(&date_selector).next();

    if let Some(date_elem) = date_elem {
        let date_regex =
            Regex::new(r"Published (?P<m>[A-Za-z]{3}) (?P<d>\d\d?), (?P<y>\d{4})").unwrap();
        let pub_date = date_elem.text().collect::<Vec<_>>()[0];
        let pub_date = date_regex.replace_all(pub_date, "$d-$m-$y");
        let pub_date = pub_date.replace("Mar", "3");

        Ok(pub_date.to_string())
    } else {
        Err(anyhow!("Cannot find date."))?
    }
}

fn get_article_from_span(doc: &Html) -> Result<String> {
    let date_selector =
        Selector::parse("span.article__published-date").expect("Unable to create selector");
    let date_elem = doc.select(&date_selector).next();

    if let Some(date_elem) = date_elem {
        let date_regex = Regex::new(r"...\., (?P<m>[A-Za-z]+) (?P<d>\d\d?), (?P<y>\d{4})").unwrap();
        let pub_date = date_elem.text().collect::<Vec<_>>()[0];
        let pub_date = date_regex.replace_all(pub_date, "$d-$m-$y");

        Ok(pub_date.to_string())
    } else {
        Err(anyhow!("Cannot find date."))?
    }
}

fn get_story_div(doc: &Html, story_attr: String) -> Result<ElementRef> {
    let story_selector = Selector::parse(&story_attr).expect("Unable to construct selector.");
    let story = doc.select(&story_selector).next();

    match story {
        Some(story) => Ok(story),
        None => Err(anyhow!("Cannot find story div")),
    }
}

fn get_article_elem(doc: &Html) -> Result<ElementRef> {
    let article_selector =
        Selector::parse("article.article-content-story").expect("Unable to construct selector.");
    let article = doc.select(&article_selector).next();

    match article {
        Some(article) => Ok(article),
        None => Err(anyhow!("Cannot find article element")),
    }
}

fn parse_article(href: &str, site: &NewsSite) -> Result<Article> {
    // make request
    let request = get(href);
    let article_text = match request {
        Ok(res) => res.text().expect("Unable to get request text."),
        Err(error) => return Err(anyhow!(error)),
    };

    let doc = Html::parse_document(&article_text);

    // get title
    let title_selector = Selector::parse("title").expect("unable to construct selector.");
    let title_elem = doc.select(&title_selector).next();
    if let None = title_elem {
        return Err(anyhow!("Couldn't find title."));
    }
    let title_text = title_elem.unwrap().text().collect::<Vec<_>>()[0];

    // get date
    let mut pub_date = String::new();
    if let Ok(date) = get_date_from_timestamp(&doc) {
        pub_date = date;
    } else if let Ok(date) = get_date_from_div(&doc) {
        pub_date = date;
    } else if let Ok(date) = get_article_from_span(&doc) {
        pub_date = date;
    } else {
        return Err(anyhow!("Couldn't find timestamp."));
    }

    // get article contents
    let mut contents = Vec::new();

    if let Ok(elem) = get_story_div(&doc, site.body.clone()) {
        let story = elem;

        // get p elements
        let par_selector = Selector::parse(&site.par_class).expect("Unable to construct selector.");
        for par in story.select(&par_selector) {
            let par_text = par.text().collect::<Vec<_>>();
            if par_text.len() > 1 {
                let full_text = par_text.join("\n");
                let full_text = full_text.replace("Article content", "");
                let full_text = full_text.replace("\n ", "");
                contents.push(full_text.to_string());
            } else if par.inner_html().len() > 0 {
                let full_text = par.inner_html();
                let full_text = full_text.replace("Article content", "");
                let full_text = full_text.replace("\n ", "");
                contents.push(full_text.to_string());
            }
        }
    } else {
        return Err(anyhow!("Couldn't find article body."));
    }

    Ok(Article {
        title: String::from(title_text),
        contents: contents,
        date: String::from(pub_date),
    })
}

fn save_article(article: &Article, path_root: &Path) {
    let date_dir = path_root.join(&article.date);
    if !date_dir.exists() {
        create_dir_all(&date_dir).expect("Cannot create directory at {date_dir}.");
    }

    let filename = String::from(&article.title)
        .replace(" | CBC News", "")
        .replace(" | National Post", "")
        .replace(" | The Star", "")
        .replace(" ", "_")
        .to_lowercase();
    let filename = format!("{filename}.txt");
    let output_path = date_dir.join(filename);

    let output_file = File::create(output_path);
    if let Ok(mut file) = output_file {
        for par in &article.contents {
            file.write_all(par.as_bytes());
            file.write_all(b"\n\n");
        }
    }
}

fn get_news_links(href: &str, a_attr: &str, link_prefix: &str) -> Result<Vec<String>> {
    let response = get(href)?;
    let text = response.text()?;
    let doc = Html::parse_document(&text);

    let link_selector = Selector::parse(a_attr).expect("Couldn't construct a selector.");
    let links_elems = doc.select(&link_selector);

    let links = links_elems
        .into_iter()
        .filter_map(|elem| {
            let link_text = elem.value().attr("href")?;
            if link_text.contains("/news/") {
                let full_link = format!("https://{link_prefix}{link_text}");
                Some(full_link)
            } else {
                None
            }
        })
        .collect();

    Ok(links)
}

struct NewsSite {
    name: String,
    a_attr: String,
    body: String,
    news_href: String,
    link_prefix: String,
    par_class: String,
}

fn scrape_website(site: NewsSite) {
    let links = match get_news_links(&site.news_href, &site.a_attr, &site.link_prefix) {
        Ok(links) => links,
        Err(err) => panic!("{err}"),
    };

    let path_root = Path::new("scraped/").join(&site.name);

    for link in links {
        let parse_result = parse_article(&link, &site);
        if let Ok(article) = parse_result {
            println!("Saving article at {link}");
            save_article(&article, &path_root);
        } else {
            print!("Cannot parse article at {link}. Continuing...");
        }
    }
}

pub fn scrape_cbc() {
    let cbc = NewsSite {
        name: "cbc".into(),
        a_attr: "a.card".into(),
        body: "div.story".into(),
        news_href: "https://www.cbc.ca/news".into(),
        link_prefix: "cbc.ca".into(),
        par_class: "p".into(),
    };

    scrape_website(cbc);
}

pub fn scrape_np() {
    let np = NewsSite {
        name: "national_post".into(),
        a_attr: "a.article-card__link".into(),
        body: "section.article-content__content-group".into(),
        news_href: "https://nationalpost.com/category/news/".into(),
        link_prefix: "nationalpost.com".into(),
        par_class: "p.section.article-content__content-group".into(),
    };

    scrape_website(np);
}

pub fn scrape_star() {
    let star = NewsSite {
        name: "the_star".into(),
        a_attr: "a.c-mediacard".into(),
        body: "div.c-article-body__content".into(),
        news_href: "https://www.thestar.com/news/world".into(),
        link_prefix: "thestar.com".into(),
        par_class: "p.text-block-container".into(),
    };

    scrape_website(star);
}
