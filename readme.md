# Storyboard
GPU accelerated 2D framework using wgpu and winit

* See `storyboard` crate for framework.
* See `componenets` directory for provided draw components.
* See `crates` directory for individual parts of framework.

## Features
1. State based app. Easy to maintenance and extend.
2. Highly extendable render resource and component modularity system.
3. No runtime required.
4. Cross platform rendering including web (via WebGL, WebGPU).
5. Nonthreaded / Threaded rendering switching support for low latency app.

## Examples
See `examples` directory for example projects

## TODO
1. [ ] Layout system
2. [x] State system
3. [x] Component system
4. [x] Threaded rendering
5. [x] Primitive(Triangle, Rect) shape rendering
6. [x] Box rendering (Rect with border, rounded border, shadow, glow effects)
7. [ ] Path rendering module (in development)
8. [ ] Text rendering module (in development)
