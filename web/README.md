# Web interface

This directory contains the web interface to the dots-and-boxes engine,
published at
[amdw.github.io/dotsandboxes/engine](https://amdw.github.io/dotsandboxes/engine/).
It runs the engine in the browser by compiling it to WebAssembly, so it
is a static site with no server.

## How it fits together

* ```engine-wasm``` is a Rust crate wrapping the engine (in
  ```../engine```) for the browser using
  [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen). Its API
  (in ```src/api.rs```) works on plain data, so it can be unit tested
  natively, and ```src/lib.rs``` exposes it to JavaScript.
* ```app``` is the web app itself, in TypeScript, using
  [Vite](https://vite.dev/) and [Preact](https://preactjs.com/). Quick
  operations such as making moves call the engine directly, while
  analysis (evaluation and Nimstring values), which can take a long
  time, runs in a Web Worker so the page stays responsive.
* The worker's engine keeps its caches of evaluated positions between
  analyses, so analysing the positions reached as a game goes on is
  usually almost instant.
* The engine can't be interrupted part-way through a calculation, so
  the worker does long analyses in chunks (each a limited number of
  new positions). Between chunks it reports progress, handles other
  requests, and notices if the analysis has been cancelled. Each chunk
  caches everything it finishes, so cancelling keeps the work done so
  far, and asking again carries on from there.

## Building and running

As well as the Rust toolchain, you will need
[Node.js](https://nodejs.org/), the WebAssembly target for Rust, and
the ```wasm-bindgen``` command-line tool. Its version must exactly
match the version of the ```wasm-bindgen``` crate in
```engine-wasm/Cargo.lock```:

* ```rustup target add wasm32-unknown-unknown```
* ```cargo install wasm-bindgen-cli --version 0.2.130 --locked```

Then, from the ```app``` directory:

* ```npm install``` to install the JavaScript dependencies
* ```npm run build:wasm``` to build the engine (re-run this after
  changing any Rust code)
* ```npm run dev``` to start a development server, then open the URL
  it prints
* ```npm test``` to run the unit tests (which use the real
  WebAssembly engine, so build it first), and ```npm run typecheck```
  to check the TypeScript
* ```npm run build``` to build a static site in ```app/dist```

To run the WebAssembly API's unit tests natively, run ```cargo test```
in the ```engine-wasm``` directory.

## Continuous integration and publishing

The ```Web app``` GitHub Actions workflow builds and tests all of this
for changes to ```engine``` or ```web```. The ```Pages``` workflow
publishes the website whenever master changes: ```../site``` holds the
website's top-level pages, and ```../site/assemble.sh``` puts them
together with the built web interface under ```engine/```.

To preview the website locally, build the web interface as above, then
from the root of the repository run ```site/assemble.sh _site``` and
serve the ```_site``` directory, e.g. with ```python3 -m http.server
--directory _site```.
