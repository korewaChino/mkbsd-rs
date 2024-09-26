use std::path::Path;

use crate::DOWNLOADS_DIR;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde_json::Value;
const DATE: &str = "20240730";
// https://storage.googleapis.com/panels-cdn/data/20240730/all.json
const CDN_URL: &str = "https://storage.googleapis.com/panels-cdn/data";

#[derive(Deserialize, Debug)]
struct Cdn {
    #[serde(flatten)]
    inner: serde_json::Value,
}

pub async fn download(url: &url::Url) -> Result<(), Box<dyn std::error::Error>> {
    // get the last two segments of the path

    let path = url.path_segments().unwrap().collect::<Vec<_>>();

    let filename = path[path.len() - 2..].join("/");

    let file_path = Path::new(DOWNLOADS_DIR).join(&filename);

    let dir = file_path.parent().unwrap();

    tokio::fs::create_dir_all(dir).await?;

    println!("Downloading {} to {}", url, filename);

    let res = reqwest::get(url.clone()).await?;
    let bytes = res.bytes().await?;
    println!("Downloaded {} bytes ({})", bytes.len(), &file_path.display());
    tokio::fs::write(&file_path, &bytes).await?;

    Ok(())
}

pub async fn download_simple() -> Result<(), Box<dyn std::error::Error>> {
    let spec = Cdn::fetch().await?;
    // println!("{:#?}", spec);
    //
    println!("Finding urls...");

    let urls = spec
        .find_urls()
        .into_iter()
        .map(|url| url::Url::parse(&url).unwrap())
        .collect::<Vec<_>>();

    let rt = tokio::runtime::Runtime::new().unwrap();
    urls.par_iter().for_each(|url| {
        let res = rt.block_on(download(url));
        if let Err(e) = res {
            eprintln!("Error downloading {}: {:?}", url, e);
        }
    });

    rt.shutdown_background();

    // println!("{:#?}", urls);
    Ok(())
}

impl Cdn {
    pub async fn fetch() -> Result<Self, reqwest::Error> {
        let res = reqwest::get(format!("{CDN_URL}/{DATE}/all.json")).await?;
        // println!("{:#?}", res);
        let spec = res.json::<Self>().await?;

        // println!("{:#?}", spec);
        Ok(spec)
    }

    pub fn find_urls(&self) -> Vec<String> {
        // find any key called "url" in the json recursively
        let mut urls = vec![];
        extract_urls(&self.inner, &mut urls);
        urls
    }
}

fn extract_urls(element: &Value, urls: &mut Vec<String>) {
    match element {
        Value::Object(map) => {
            map.iter().for_each(|(key, value)| {
                if key == "url" {
                    if let Some(url) = value.as_str() {
                        urls.push(url.to_string());
                    }
                } else {
                    extract_urls(value, urls);
                }
            });
        }
        Value::Array(arr) => {
            arr.iter().for_each(|item| extract_urls(item, urls));
        }
        _ => {}
    }
}
