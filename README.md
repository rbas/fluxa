# Fluxa

Fluxa is a lightweight and easy-to-use monitoring tool designed to check whether a given URL returns a successful HTTP status code. It continuously monitors the availability of web resources by sending periodic requests and alerts users if any issues are detected. Perfect for developers and system administrators looking for a straightforward solution to ensure their web services remain operational, Fluxa is built with simplicity in mind, making it easy to set up and use while maintaining reliability and effectiveness.

![fluxa >](https://raw.githubusercontent.com/rbas/fluxa/main/assets/fluxa.webp)

## Motivation

My motivation for creating Fluxa stemmed from the need for a lightweight, meta-monitoring tool designed to check whether other monitoring tools are up and running. With a focus on minimal memory footprint—just a few MB—Fluxa is built to be simple yet effective, capable of running even on resource-constrained devices like routers. The goal was to create a solution that could cross-monitor both main and advanced monitoring tools, ensuring seamless oversight without adding unnecessary complexity or overhead. Additionally, I integrated support for Pushover notifications to ensure timely alerts when services go down, making Fluxa not only a reliable monitoring tool but also a practical way to maintain uptime across critical systems.

## Usage

Copy `config.sample.toml` and create `config.local.toml`. Define the services that you want to monitor and add pushover configuration. Then save the file next to the `fluxa` binary file.

```
RUST_LOG=debug fluxa
```
It will start fluxa if there is not configuration problem and will start monitoring the services. Fluxa also have running http server on `localhost:8080` that resopnd with http status 200 for cross monitoring.

## Installation

### Compile from source
To compile `fluxa` from the source, you need [Rust installed]((https://www.rust-lang.org/tools/install)) on your system. First, clone this repository and navigate to its root directory. Then, run the following command to build the project:

```
cargo build --release
```
Rust will compile `fluxa`, and the resulting executable binary will be located at `target/release/fluxa`.


## Contribute
You are welcomed to fork the project and create a branch for each new feature you wish to add. Ensure that you write necessary tests and run the current tests before making your pull request.


## License: MIT
© 2025 Martin Voldrich
This work is licensed under [MIT license](https://github.com/rbas/fluxa/blob/main/LICENSE).
