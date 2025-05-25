![Rust](https://img.shields.io/badge/-Rust-red?logo=rust&logoColor=white&style=plastic)
![TOML](https://img.shields.io/badge/-Toml-blue?logo=toml&style=plastic)

# Gaussian Blur Effect on RGB Image written in Rust

## Disclaimer & Description
This code using experimental features of Rust, so need Rust nightly builds.

### Code Description
This code will apply Gaussian blur effect on RGB images.

The effect will be executed using available CPU extension (SIMD) and compared to normal (scalar mode).

So, if your cpu has AVX2 then the code will optimize using that instruction to apply the effect.

There will be 2 images generated on the same location


### Requirement
Install rust nightly
```
rustup toolchain install nightly
rustup default nightly
```

### BUILD
` cargo build --release `

copy the binary to i.e /user/local/bin

` sudo cp target/release/rust_gaussian_blur /usr/local/bin `

### Usage
` rust_gaussian_blur <PATH TO IMAGE>  `

example:

` rust_gaussian_blur /tmp/4kimg.jpg `

