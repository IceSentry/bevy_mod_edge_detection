# bevy_mod_edge_detection

A simple plugin to add full screen edge detection.

![image](cornell_box.png)

## Implementation details

The implementation is mostly based on what is described in this article.

[https://alexanderameye.github.io/notes/rendering-outlines/#edge-detection](https://alexanderameye.github.io/notes/rendering-outlines/#edge-detection)

Essentially, it runs the sobel operator on the depth, normal and color textures. The sobel operator is able to determine discontinuity in those textures and the shader will simply draw those discontinuity.

## Getting Started

See the [examples/simples.rs](examples/simple.rs) example
