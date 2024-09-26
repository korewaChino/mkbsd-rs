//! mkbsd-rs
//! 
//! A parallelized and even more complete version of the mkbsd tool, written in Rust.
//! 
//! This tool is designed to download all the wallpapers from MKBHD's Panels GCP storage bucket
//! 
//! There seems to be a standardized layout for the images, so we are going to iterate through all the repos and rip everything

use std::collections::HashMap;
use rayon::iter::IntoParallelRefIterator;
use serde::Deserialize;
use rayon::iter::ParallelIterator;
const DATE: &str = "20240916";
// const API_URL: &str = "https://storage.googleapis.com/panels-api/data/20240916/media-1a-i-p~s";
// const SPEC_URL: &str = "https://storage.googleapis.com/panels-api/data/20240916/spec.json";
const SPEC_URL: &str = "https://storage.googleapis.com/panels-api/data";
const DOWNLOADS_DIR: &str = "downloads";

// update: There are even more wallpapers and an API spec here
// https://storage.googleapis.com/panels-api/data/20240916/spec.json

// The API spec is basically this:
// media: {
//  root: "<base url>",
//  p: ["<id>", "<id>", ...],
//  b: ["<id>", "<id>", ...]
// }
// 
// to get all the images, we basically iterate and concat p and b in this format:
// 
// <base url>-<p>-<b>
// 
// rinse and repeat for ALL the repos in the GCP bucket until we have absolutely everything
// 

// todo: an API for the metadata and search for the images would be nice since they do have data in there

#[derive(Deserialize, Debug)]
pub struct Spec {
    content: String,
    search: String,
    pub media: PanelMedia,
}

impl Spec {
    pub async fn fetch() -> Result<Self, reqwest::Error> {
        let res = reqwest::get(format!("{SPEC_URL}/{DATE}/spec.json")).await?;
        let spec = res.json::<Spec>().await?;
        Ok(spec)
    }
}

#[derive(Deserialize, Debug)]
pub struct PanelMedia {
    pub root: String,
    pub p: Vec<String>,
    pub b: Vec<String>,
}

impl PanelMedia {
    pub fn iterate_all(&self) -> Vec<String> {
        self.p.iter().flat_map(|p| {
            self.b.iter().map(move |b| {
                format!("{root}-{p}-{b}", root = self.root, p = p, b = b)
            })
        }).collect()
    }
}



#[derive(Deserialize, Debug)]
pub struct Repo {
    #[serde(skip_deserializing)]
    repo: String,
    data: HashMap<String, Image>,
}

impl Repo {
    pub async fn new(repo: &str) -> Result<Self, reqwest::Error> {
        
        println!("Fetching repo {}", repo);
        let res = reqwest::get(format!("{repo}")).await?;
        
        let repo_url_parsed = url::Url::parse(repo).unwrap();
        let repo = repo_url_parsed.path_segments().unwrap().last().unwrap();
        
        
        // println!("Got response: {:#?}", res);

        let panels = res.json::<Repo>().await?;
        Ok(Self {
            repo: repo.to_string(),
            data: panels.data,
        })
    }
    
    // pub async fn fetch() -> Result<Self, reqwest::Error> {
    //     let res = reqwest::get(API_URL).await?;
    //     let panels = res.json::<Repo>().await?;
    //     Ok(panels)
    // }
    
    pub fn get_image(&self, id: &str) -> Option<&Image> {
        self.data.get(id)
    }
    
    pub fn get_image_url(&self, id: &str, form_factor: &str) -> Option<&String> {
        self.get_image(id).and_then(|image| image.get_url(form_factor))
    }
    
    pub async fn download_image(&self, id: &str, form_factor: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading image {} for form factor {}", id, form_factor);
        let repo_name = &self.repo;
        let repo_dir = format!("{DOWNLOADS_DIR}/{repo_name}");
        tokio::fs::create_dir_all(&repo_dir).await?;
        let url = self.get_image_url(id, form_factor).unwrap();
        let res = reqwest::get(url).await?;
        let bytes = res.bytes().await?;
        println!("Downloaded {} bytes ({url})", bytes.len());
        let fmt = file_format::FileFormat::from_bytes(&bytes);
        let ext = fmt.extension();
        let filename = format!("{repo_dir}/{}-{}.{ext}", id, form_factor);
        tokio::fs::write(filename, bytes).await?;
        Ok(())
    }
}

// The image is basically this:
// {
//  "<id>": {
//     "<form factor>": "<url>"
//   }
// }
// 
// so we are going to just make use of maps to serialize this
#[derive(Deserialize, Debug)]
#[serde(tag = "id")]
pub struct Image {
    #[serde(flatten)]
    // every field is a form factor
    image: HashMap<String, String>,
}

impl Image {
    pub fn get_url(&self, form_factor: &str) -> Option<&String> {
        self.image.get(form_factor)
    }
}

#[derive(Debug)]
struct ImageDownload {
    pub id: String,
    pub repo_id: String,
    pub form_factor: String,
    pub url: String,
}

impl ImageDownload {
    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading image for repo {}, form factor {} from {}", self.repo_id, self.form_factor, self.url);
        let repo_id = &self.repo_id;
        let repo_dir = format!("{DOWNLOADS_DIR}/{repo_id}");
        tokio::fs::create_dir_all(&repo_dir).await?;
        let res = reqwest::get(&self.url).await?;
        let bytes = res.bytes().await?;
        println!("Downloaded {} bytes", bytes.len());
        let fmt = file_format::FileFormat::from_bytes(&bytes);
        let ext = fmt.extension();
        let filename = format!("{repo_dir}/{id}-{form_factor}.{ext}", id = self.id, form_factor = self.form_factor);
        tokio::fs::write(filename, bytes).await?;
        Ok(())
    }
}

// rewrite of above function but accepts a vec of all images flattened

async fn download_images_flat(img: Vec<ImageDownload>) {
    
    img.par_iter().for_each(|image| {
        let res = tokio::runtime::Runtime::new().unwrap().block_on(image.download());
        if let Err(e) = res {
            eprintln!("Error downloading image: {:?}", e);
        }
    });
    
    // img.par_iter().for_each(|(id, form_factor)| {
    //     let res = tokio::runtime::Runtime::new().unwrap().block_on(images.download_image(id, form_factor));
    //     if let Err(e) = res {
    //         eprintln!("Error downloading image: {:?}", e);
    //     }
    // });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("Hello, world!");
    
    let spec = Spec::fetch().await?;
    
    // println!("{spec:#?}");
    
    let repos = spec.media.iterate_all();
    
    // println!("{repos:#?}");
    
    
    println!("Iterating through repos...");
    let repos_iter = repos.par_iter().map(|repo| {
            tokio::runtime::Runtime::new().unwrap().block_on(Repo::new(&repo))
        }).filter_map(Result::ok).collect::<Vec<_>>();
    
    
    // let mut futures = vec![];
    

    
    // flatten into vec of (repo_id, link) tuples
    
    let images = repos_iter.iter().flat_map(|repo| {
        repo.data.iter().flat_map(move |(id, image)| {
            image.image.iter().map(move |(form_factor, url)| {
                ImageDownload {
                    id: id.clone(),
                    repo_id: repo.repo.clone(),
                    form_factor: form_factor.clone(),
                    url: url.clone(),
                }
            })
        })
    }).collect::<Vec<_>>();
    
    // println!("{:#?}", images);
    
    download_images_flat(images).await;
    
    
    // iterate through all the repos
    
    // let panels = Repo::fetch().await?;
    
    // println!("{:#?}", panels);
    // download_image(&panels).await;
    
    
    Ok(())
}