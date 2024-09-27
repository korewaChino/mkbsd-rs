//! mkbsd-rs
//!
//! A parallelized and even more complete version of the mkbsd tool, written in Rust.
//!
//! This tool is designed to download all the wallpapers from MKBHD's Panels GCP storage bucket
//!
//! There seems to be a standardized layout for the images, so we are going to iterate through all the repos and rip everything

/*
    This program is free software. It comes without any warranty, to
    the extent permitted by applicable law. You can redistribute it
    and/or modify it under the terms of the Do What The Fuck You Want
    To Public License, Version 2, as published by Sam Hocevar. See
    http://www.wtfpl.net/ for more details.
*/

use clap::{Parser, ValueEnum};

mod simple;
mod verbose;

const DATE: &str = "20240916";
// const API_URL: &str = "https://storage.googleapis.com/panels-api/data/20240916/media-1a-i-p~s";
// const SPEC_URL: &str = "https://storage.googleapis.com/panels-api/data/20240916/spec.json";
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

#[derive(ValueEnum, Clone, Debug, Default)]
enum OperatingMode {
    /// The original mode, which downloads all the wallpapers
    /// in every crop and resolution available from the API
    ///
    /// This mode is heavy on the API, network and storage
    /// but will give you pre-cropped wallpapers for every form factor
    Verbose,
    /// A simplified mode that only downloads the wallpapers
    /// in its original form by using the CDN bucket
    ///
    /// This mode simply downloads the original wallpapers,
    /// recommended for those who just want the raw images
    /// so they can crop them themselves.
    #[default]
    Simple,
}

#[derive(clap::Parser)]
struct Cli {
    #[clap(short, long)]
    #[clap(value_enum)]
    #[arg(default_value_t)]
    mode: OperatingMode,

    #[clap(short, long, env = "DRY_RUN")]
    dry_run: bool,
}

impl Cli {
    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        // set dry run mode
        if self.dry_run {
            std::env::set_var("DRY_RUN", "true");
        }

        match self.mode {
            OperatingMode::Verbose => verbose::download_verbose().await,
            OperatingMode::Simple => simple::download_simple().await,
            // OperatingMode::Simple => verbose::download_verbose().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("Hello, world!");

    let cli = Cli::parse();

    cli.download().await?;

    Ok(())
}
