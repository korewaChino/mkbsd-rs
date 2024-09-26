use std::collections::HashMap;
use rayon::iter::IntoParallelRefIterator;
use serde::Deserialize;
const API_URL: &str = "https://storage.googleapis.com/panels-api/data/20240916/media-1a-i-p~s";
const DOWNLOADS_DIR: &str = "downloads";
use rayon::iter::ParallelIterator;
// the funny panels wallpaper API:
// Contains a JSON with the following data:
// - data: an array of objects, each object representing an image
//   an image object has an ID and links to the image
#[derive(Deserialize, Debug)]
pub struct Panels {
    data: HashMap<String, Image>,
}

impl Panels {
    pub async fn fetch() -> Result<Self, reqwest::Error> {
        let res = reqwest::get(API_URL).await?;
        let panels = res.json::<Panels>().await?;
        Ok(panels)
    }
    
    pub fn get_image(&self, id: &str) -> Option<&Image> {
        self.data.get(id)
    }
    
    pub fn get_image_url(&self, id: &str, form_factor: &str) -> Option<&String> {
        self.get_image(id).and_then(|image| image.get_url(form_factor))
    }
    
    pub async fn download_image(&self, id: &str, form_factor: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading image {} for form factor {}", id, form_factor);
        tokio::fs::create_dir_all(DOWNLOADS_DIR).await?;
        let url = self.get_image_url(id, form_factor).unwrap();
        let res = reqwest::get(url).await?;
        let bytes = res.bytes().await?;
        let filename = format!("{}/{}-{}.jpg", DOWNLOADS_DIR, id, form_factor);
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

async fn download_image(images: &Panels) {
    // flatten the images into a vector of (id, form_factor) tuples
    let img = images.data.iter().flat_map(|(id, image)| {
        image.image.iter().map(move |(form_factor, _)| (id, form_factor))
    }).collect::<Vec<_>>();

    
    img.par_iter().for_each(|(id, form_factor)| {
        // let _ = images.download_image(id, form_factor);
        
        let res = tokio::runtime::Runtime::new().unwrap().block_on(images.download_image(id, form_factor));
        if let Err(e) = res {
            eprintln!("Error downloading image: {:?}", e);
        }
        
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    
    let panels = Panels::fetch().await?;
    
    // println!("{:#?}", panels);
    
    download_image(&panels).await;
    
    Ok(())
}