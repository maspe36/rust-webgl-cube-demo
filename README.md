# Rust WebGL Cube Demo
Implementation of the [WebGL Cube demo](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Creating_3D_objects_using_WebGL)
using Rust + WASM. This code is _very_ messy. I didn't try to write idiomatic rust but instead provide as close to a 1:1 implementation of the original demo.

## Dependencies
Follow the offical [rustwasm book](https://rustwasm.github.io/docs/book/game-of-life/setup.html) for good instructions on getting your environment configured 

## Running
1. Clone the project
   ```
   git clone https://github.com/maspe36/rust-webgl-cube-demo.git
   ```

2. Compile the Rust code into a wasm package

   ```
   cd rust-webgl-cube-demo/
   wasm-pack build
   ```

3. Install dependencies 

   ```
   cd www/
   npm install
   ```
   
4. Start the local server

   ```
   npm run start
   ```

   `CTRL + C` to stop
4. Navigate to the URL NPM is serving. Defaults to http://localhost:8080/


## Author
[Sam Privett](mailto:sam@privett.dev)
