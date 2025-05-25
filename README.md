# Gaussian Blur Effect on RGB Image written in Rust

## Requirement
Install rust nightly
```
rustup toolchain install nightly
rustup default nightly
```

## BUILD
` cargo build --release `

copy the binary to i.e /user/local/bin

` sudo cp target/release/rust_gaussian_blur /usr/local/bin `

## Usage
` rust_gaussian_blur <PATH TO IMAGE>  `

example:

` rust_gaussian_blur /tmp/4kimg.jpg `

