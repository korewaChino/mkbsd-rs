use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use std::collections::HashMap;
const SPEC_URL: &str = "https://storage.googleapis.com/panels-api/data";
use crate::{DATE, DOWNLOADS_DIR};

pub async fn download_verbose() -> Result<(), Box<dyn std::error::Error>> {
    let spec = Spec::fetch().await?;
    let repos = spec.media.iterate_all();

    println!("Iterating through repos...");
    let repos_iter = repos
        .par_iter()
        .map(|repo| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(Repo::new(repo))
        })
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    // let mut futures = vec![];

    // flatten into vec of (repo_id, link) tuples

    let images = repos_iter
        .iter()
        .flat_map(|repo| {
            repo.data.iter().flat_map(move |(id, image)| {
                image
                    .image
                    .iter()
                    .map(move |(form_factor, url)| ImageDownload {
                        id: id.clone(),
                        repo_id: repo.repo.clone(),
                        form_factor: form_factor.clone(),
                        url: url.clone(),
                    })
            })
        })
        .collect::<Vec<_>>();

    // println!("{:#?}", images);

    download_images_flat(images).await;

    Ok(())
}

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
        self.p
            .iter()
            .flat_map(|p| {
                self.b
                    .iter()
                    .map(move |b| format!("{root}-{p}-{b}", root = self.root, p = p, b = b))
            })
            .collect()
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
        let res = reqwest::get(repo.to_string()).await?;

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


#[derive(Debug)]
struct ImageDownload {
    pub id: String,
    pub repo_id: String,
    pub form_factor: String,
    pub url: String,
}

impl ImageDownload {
    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Downloading image for repo {}, form factor {} from {}",
            self.repo_id, self.form_factor, self.url
        );
        let repo_id = &self.repo_id;
        let repo_dir = format!("{DOWNLOADS_DIR}/{repo_id}");
        tokio::fs::create_dir_all(&repo_dir).await?;
        let res = reqwest::get(&self.url).await?;
        let bytes = res.bytes().await?;
        println!("Downloaded {} bytes", bytes.len());
        let fmt = file_format::FileFormat::from_bytes(&bytes);
        let ext = fmt.extension();
        let filename = format!(
            "{repo_dir}/{id}-{form_factor}.{ext}",
            id = self.id,
            form_factor = self.form_factor
        );
        tokio::fs::write(filename, bytes).await?;
        Ok(())
    }
}

// rewrite of above function but accepts a vec of all images flattened

async fn download_images_flat(img: Vec<ImageDownload>) {
    img.par_iter().for_each(|image| {
        let res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(image.download());
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
