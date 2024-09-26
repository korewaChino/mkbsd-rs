# mkbsd-rs

A port of [mkbsd](https://github.com/nadimkobeissi/mkbsd) in (Multi-threaded) Rust

Rip Mark-ass Brownlee's wallpapers at blazingly fast speeds! 🚀

This Rust program makes use of [rayon](https://github.com/rayon-rs/rayon) to parallelize the process of downloading all the wallpapers from the Panels API, allowing all wallpapers
to be downloaded simultaneously.

This obviously eats a lot of bandwidth and may anger GCP's ratelimits, so use responsibly.

## Why?

1. the original code isn't fast enough for me I have a 1GbE connection
2. This thing is embarrassingly parallel
3. I love free shit
4. My 1GbE plan is cheaper than this app

## Aren't you stealing from artists?

yea, but so is piracy in general. if you really like what you see just pay for it, see `LICENSE` for my full opinion

## Usage

1. [Get Rust](https://rustup.rs)
2. Clone this project and enter the project directory
3. `cargo run --release`
4. Wait

All the images should appear in the `downloads` directory.

## License

Do whatever the fuck you want.
