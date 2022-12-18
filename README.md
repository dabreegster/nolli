# nolli

This is an experiment for me to try [Bevy](https://bevyengine.org) and play
with figure-ground maps, inspired by [Giambattista
Nolli](https://web.stanford.edu/group/spatialhistory/nolli/).

![Demo](demo.gif)

To try this:

1.  Install Rust and [Bevy dependencies](https://bevyengine.org/learn/book/getting-started/setup/)
2.  Get a GeoJSON file with some polygons in WGS84, using something like [Overpass](https://overpass-turbo.eu/s/Jk8)
3.  `cargo run --release path_to_polygons.geojson`
4.  Click and drag to pan, scroll to zoom, press space to start flooding from the cursor
