# Developer guide

This guide describes how to setup a development environment for building and running

- Nextclade CLI executable
- Nextalign CLI executable
- Nextclade Web and Nextclade WebAssembly module

## Developing locally

### Nextclade CLI and Nextalign CLI

Nextclade CLI and Nextalign CLI are written in Rust. The usual `rustup` & `cargo` workflow can be used:

```bash
# Clone Nextclade git repository
git clone https://github.com/nextstrain/nextclade
cd nextclade

# Install Rustup, the Rust version manager (https://www.rust-lang.org/tools/install)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Rust tools to the $PATH
export PATH="$PATH:$HOME/.cargo/bin"

# [Linux only] install openssl and pkgconfig. Example for Ubuntu:
sudo apt-get update
sudo apt-get install --yes libssl-dev pkg-config

# Prepare dotenv file with default values
cp .env.example .env

# Build and run Nextclade in debug mode (convenient for development, fast to build, slow to run, has debug info)
# Nextclade dataset is expected to be in ./data_dev/
# Refer to the user documentation for explanation of Nextclade CLI flags (https://docs.nextstrain.org/projects/nextclade/en/stable/)
cargo run --bin=nextclade -- run \
  --input-fasta=data_dev/sequences.fasta \
  --input-dataset=data_dev/ \
  --output-fasta='out/nextclade.aligned.fasta' \
  --output-tsv='out/nextclade.tsv' \
  --output-tree='out/nextclade.tree.json' \
  --in-order \
  --include-reference

# Build Nextclade in release mode (slow to build, fast to run, no debug info)
cargo build --release --bin=nextclade

# Run Nextclade release binary
./target/release/nextclade run \
  --input-fasta=data_dev/sequences.fasta \
  --input-dataset=data_dev/ \
  --output-fasta='out/nextclade.aligned.fasta' \
  --output-tsv='out/nextclade.tsv' \
  --output-tree='out/nextclade.tree.json' \
  --in-order \
  --include-reference

# Add -v flags to increase verbosity of output
# nextclade run ... -vv

# To build Nextalign, replace 'nextclade' with 'nextalign'
# cargo build --release --bin=nextalign

```

### Nextclade Web

Nextclade Web is a React Typescript application, which relies on Nextclade WebAssembly (wasm) module to perform the computation.
The WebAssembly module shares the algorithms Rust code with Nextclade CLI. So building Nextclade Web involves 2 steps: building WebAssembly module and building the app itself.

Install Node.js version 14+ (latest LTS release is recommended), by either downloading it from the official website: https://nodejs.org/en/download/, or by using [nvm](https://github.com/nvm-sh/nvm). We don't recommend using Node.js from the package manager of your operating system, and neither from conda or other sources.

Let's build the WebAssembly module:

```bash
# Clone Nextclade git repository
git clone https://github.com/nextstrain/nextclade
cd nextclade

# Install Rustup, the Rust version manager (https://www.rust-lang.org/tools/install)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Rust tools to the $PATH
export PATH="$PATH:$HOME/.cargo/bin"

# Install wasm-pack
cargo install wasm-pack

cd packages_rs/nextclade-web

# Install dependency packages
yarn install

# Run wasm-pack to build the WebAssembly module in release mode
yarn wasm-prod

# Alternatively, to build in debug mode (much much slower to run, but more debug info)
# yarn wasm-dev

```

The WebAssembly module and accompanying Typescript code should have been generated into `packages_rs/nextclade-web/src/gen/`,
and now the web app should be able to find it.

Let's build build and run the web app (it's convenient to do it in a separate terminal instance from WebAssembly module build):

```bash
# Build and run web app in dev mode
cd packages_rs/nextclade-web
yarn install
yarn dev
```

Open `http://localhost:3000/` in the browser. Code changes should trigger rebuild and fast refresh of the app.

The optimized production version of the web app can be built with

```bash
yarn prod:build
yarn prod:serve
```

Open `http://localhost:8080/` in the browser.

The resulting files should be available under `packages_rs/nextclade-web/.build/production/web`. They can be served by any static webserver or static file hosting (`https://clades.nextstrain.org` uses AWS S3 + Cloudfront). `yarn prod:serve` is just an example of a simple webserver that serves files locally.

### Linting (static analysis)

#### Linting Rust code

Rust code is linted with [Clippy](https://github.com/rust-lang/rust-clippy):

```bash
cargo clippy
```

Clippy is configured in `clippy.toml` and `.cargo/config.toml`.

#### Linting Typescript and JavaScript

The web app is linted using [eslint](https://github.com/eslint/eslint) and [tsc](https://www.typescriptlang.org/docs/handbook/compiler-options.html) as a part of development command, but the same lints also be ran separately:

```bash
cd packages_rs/nextclade-web
yarn lint
```

The `eslint` configuration is in `.eslintrc.js`. `tsc` configuration is in `tsconfig.json`.

### Formatting (code style)

```bash
cargo fmt --all
```

```bash
cd packages_rs/nextclade-web
yarn format:fix
```