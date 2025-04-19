# Loglan WASM Parser App

This application provides a web-based interface to parse Loglan text using the `camxes-rs` library compiled to WebAssembly (WASM).

## Prerequisites

*   **Rust Toolchain:** Ensure you have Rust installed (https://rustup.rs/).
*   **`wasm-pack`:** Install `wasm-pack` for building Rust-generated WebAssembly:
    ```bash
    cargo install wasm-pack
    ```
*   **Simple HTTP Server:** You'll need a way to serve the static files locally. Python's built-in server or `basic-http-server` are good options.
    *   Python 3: `python -m http.server`
    *   Install `basic-http-server`: `cargo install basic-http-server` then run `basic-http-server`

## Building the Application

1.  Navigate to the `loglan-wasm-app` directory in your terminal:
    ```bash
    cd loglan-wasm-app
    ```
2.  Build the WASM package using `wasm-pack`. This command compiles the Rust code to WASM, generates JavaScript bindings, and places the output in a `pkg/` directory.
    ```bash
    wasm-pack build --target web --out-dir pkg
    ```
    *   `--target web`: Specifies the build target suitable for web browsers.
    *   `--out-dir pkg`: Specifies the output directory.

## Running the Application

1.  **Copy Static Files:** Copy the `static/index.html` file into the `pkg/` directory created by `wasm-pack`:
    ```bash
    cp static/index.html pkg/
    ```
    *(On Windows, use `copy static\index.html pkg\`)*

2.  **Serve the `pkg` Directory:** Navigate into the `pkg` directory and start a local web server.
    ```bash
    cd pkg
    # Using Python 3's server (adjust port if needed)
    python -m http.server 8080
    # Or using basic-http-server
    # basic-http-server --addr 127.0.0.1:8080 .
    ```

3.  **Open in Browser:** Open your web browser and navigate to the address provided by the server (e.g., `http://localhost:8080` or `http://127.0.0.1:8080`).

You should now see the Loglan parser interface. Enter text and click "Parse" to see the results.
