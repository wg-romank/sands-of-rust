# Sandbox 2D Simulation

Inspired by Sandspiel, Noita and countless other simulations, this repo implements falling sands mechanics. Unlike most of the similar projects it computes updates on GPU using render-to-texture approach. Code is borrowing block-cellular automata rules for sand gravity from Falling Turnip (https://github.com/tranma/falling-turnip) and adopting them to be run as part of WebGL 1.0 shader pipeline for maximum performance and portability.

Traditionally such simulations were limited to modest field area size on the web (such as 100x100 for single-threaded pure JS implementation, or 300x300 for single-threaded WASM implementation). Using block-cellular rules allows for parallel processing, which makes it possible to render a very large simulation field area, leveraging a massively parallel GPU compute model.

<p align="center">
    <img src="/docs/preview.png">
</p>

## How it works

In order to make field data available on the GPU we pack the field into a texture, where each texel represents a single cell of a simulation.

Classic implementation of falling sands physics (see details in references) applies rules for each cell in sequence. Using the block-cellular automata approach we instead use `2 x 2` neighborhoods of each cell to determine its state on the next step.

Here's the example of block rules for particles that behave 'like sand' in traditional simulation. For each of those rules we need to create a companion rule that would be symmetric about the Y-axis. Simulation is built in a way that we create those companion rules automatically so we don't have to spend time manually mirroring each rule.

<img src="/docs/rules-illustration.svg">

Rules are packed into two additional textures, such that the result of rule application is stored on the same offset as the pattern of that rule. Storing rules in texture makes querying them straightforward in shader code. Texture size is matched with the next power of 2 to ensure texture coordinates behave. Extra space is filled with padding.

Rules are applied in a stepping grid where we alternate between odd and even columns and rows, stepping grid could be illustrated in the image below.

<img src="/docs/stepping-grid.svg">

Here on each time step `t` we

1. Query its left, upper and upper-left neighbor, if `t mod 4 == 0`
2. Query its right, upper and upper-right neighbor, if `t mod 4 == 1`
3. Query its right, lower and lower-right neighbor, if `t mod 4 == 2`
4. Query its left, lower, and lower-left neighbor, if `t mod 4 == 3`

To compute state of next iteration we

1. Decide what kind of neighborhood we want to query based on time step `t`
2. Query relative values for each pixel and pack them into neighborhood
3. Scan rules texture until we find first match with current neighborhood
4. Substitute current neighborhood with values from yet another texture, that contains rule application result at the same offset as original rule

Since we compute values per pixel we do some extra work here, but packing the whole neighborhood value would not work directly with the stepping grid. Another room for optimization is here.

Pipeline is first rendering from state to new state using render-to-texture method. Resulting state is then rendered with another pass to display. The display shader can have its own handling of state values for presentation, decoupled from state update logic.

## Limitations

While this approach allows processing of sufficiently big canvases even on my old iPhone 6 (probably while draining quite a bit of battery), encoding rules as texture for plain search and substitute will make texture size increase exponentially with more and more substance types added to the simulation. Obvious improvement could be to do this matching in hierarchy, if you have any ideas on how this can be done in a convenient way - please feel free to open an issue & start a discussion.

## References

1. Exploring the Tech and Design of Noita (https://www.youtube.com/watch?v=prXuyMCgbTc)
2. Making Sandspiel (https://maxbittker.com/making-sandspiel)
3. Falling Turnip (https://github.com/tranma/falling-turnip)
4. WebGL Fundamentals (https://webglfundamentals.org/)

## Development

Check `Setup` section of Rust WebAssembly book for prerequisite installs (https://rustwasm.github.io/docs/book/game-of-life/setup.html)

To build

```make build```

To run

```make serve```

This will start a web server listening on `localhost:8888`, configured from `www/webpack.config.js`

Note: this project relies on some unpublished changes of another crate (https://github.com/wg-romank/glsmrs), make sure to check it out and update the path to it in `Cargo.toml` if needed.
