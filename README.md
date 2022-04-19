# Sandbox 2D Simulation

Inspired by Sandspiel, Noita and countless other simulations, this repo implements falling sands mechanics. Unlike most of the similar projects it computes updates on GPU using render-to-texture approach. Code is borrowing block-cellular automata rules for sand gravity from Falling Turnip (https://github.com/tranma/falling-turnip) and adopting them to be run as part of WebGL 1.0 shader pipeline for maximum performance and portability.

Traditionally such simulations were limited to modest field area size on the web (such as 100x100 for single-threaded pure JS implementation, or 300x300 for single-threaded WASM implementation). Using block-cellular rules allows for parallel processing, which makes it possible to render very large simulation field area, leveraging massively parallel GPU compute model.

<p align="center">
    <img src="/docs/preview.gif">
</p>

## How it works

TBD: describe in details

1. Stepping grid and calculating index ids
2. Encoding rules as texture

## Limitations

While this approach allows to process sufficiently big canvases even on my old iPhone 6 (probably while draining quite a bit of battery), encoding rules as texture for plain search and substitute will make texture size increase exponentionally with more and more substance types added to the simulation. Obvious improvement could be to do this matching in hierarchy, if you have any ideas on how this can be done in a convenient way - please feel free to open an issue & start a discussion.

## References

1. Exploring the Tech and Design of Noita (https://www.youtube.com/watch?v=prXuyMCgbTc)
2. Making Sandspiel (https://maxbittker.com/making-sandspiel)
3. Falling Turnip (https://github.com/tranma/falling-turnip)
4. WebGL Fundamentals (https://webglfundamentals.org/)

## Development

To build

```make build```

To run

```make serve```

This will start a web server listening on `localhost:8888`, configured from `www/webpack.config.js`

Note: this project relies on some unpublished changes of another crate (https://github.com/wg-romank/glsmrs), make sure to check it out and update path to it in `Cargo.toml` if needed.
