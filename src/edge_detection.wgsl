#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput

struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
};

struct Config {
    depth_threshold: f32,
    normal_threshold: f32,
    color_threshold: f32,
    edge_color: vec4<f32>,
    debug: u32,
    enabled: u32,
};

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;
@group(0) @binding(2)
var depth_prepass_texture: texture_depth_2d;
@group(0) @binding(3)
var normal_prepass_texture: texture_2d<f32>;
@group(0) @binding(4)
var<uniform> view: View;
@group(0) @binding(5)
var<uniform> config: Config;

// This function will give you the tex_coord of the screen texture for the current fragment position
fn get_screen_coord(in: FullscreenVertexOutput) -> vec2<f32> {
    let resolution = vec2<f32>(textureDimensions(screen_texture));
    let frag_coord = in.position.xy;
    let inverse_screen_size = 1.0 / resolution.xy;
    return frag_coord * inverse_screen_size;
}

fn coords_to_viewport_uv(position: vec2<f32>, viewport: vec4<f32>) -> vec2<f32> {
    return (position - viewport.xy) / viewport.zw;
}

fn prepass_depth(frag_coord: vec2<f32>) -> f32 {
    let depth_sample = textureLoad(depth_prepass_texture, vec2<i32>(frag_coord), 0);
    return depth_sample;
}

fn prepass_normal(frag_coord: vec2<f32>) -> vec3<f32> {
    let world_normal = textureLoad(normal_prepass_texture, vec2<i32>(frag_coord), 0).xyz;
    return world_normal;
}

var<private> sobel_x: array<f32, 9> = array<f32, 9>(
    1.0,
    0.0,
    -1.0,
    2.0,
    0.0,
    -2.0,
    1.0,
    0.0,
    -1.0,
);

var<private> sobel_y: array<f32, 9> = array<f32, 9>(
    1.0,
    2.0,
    1.0,
    0.0,
    0.0,
    0.0,
    -1.0,
    -2.0,
    -1.0,
);

var<private> neighbours: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
    vec2<f32>(-1.0, 1.0),  // 0. top left
    vec2<f32>(0.0, 1.0),   // 1. top center
    vec2<f32>(1.0, 1.0),   // 2. top right
    vec2<f32>(-1.0, 0.0),  // 3. center left
    vec2<f32>(0.0, 0.0),   // 4. center center
    vec2<f32>(1.0, 0.0),   // 5. center right
    vec2<f32>(-1.0, -1.0), // 6. bottom left
    vec2<f32>(0.0, -1.0),  // 7. bottom center
    vec2<f32>(1.0, -1.0),  // 8. bottom right
);

fn detect_edge_depth(frag_coord: vec2<f32>) -> f32 {
    let depth_modulation = 0.001;
    var samples = array<f32, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = depth_modulation / prepass_depth(frag_coord + neighbours[i]);
    }

    var horizontal = vec4<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += samples[i] * sobel_x[i];
    }

    var vertical = vec4<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += samples[i] * sobel_y[i];
    }

    var edge = sqrt(dot(horizontal, horizontal) + dot(vertical, vertical));
    if edge < config.depth_threshold {
        return 0.0;
    }
    return edge;
}

fn detect_edge_normal(frag_coord: vec2<f32>) -> f32 {
    var samples = array<vec3<f32>, 9>();
    for (var i = 0; i < 9; i++) {
        samples[i] = prepass_normal(frag_coord + neighbours[i]);
    }

    var horizontal = vec3<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += samples[i].xyz * sobel_x[i];
    }

    var vertical = vec3<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += samples[i].xyz * sobel_y[i];
    }

    var edge = sqrt(dot(horizontal, horizontal) + dot(vertical, vertical));
    if edge < config.normal_threshold {
        return 0.0;
    }
    return edge;
}

fn detect_edge_color(uv: vec2<f32>, resolution: vec2<f32>) -> f32 {
    var samples = array<vec4<f32>, 9>();
    for (var i = 0; i < 9; i++) {
        let offset = neighbours[i] / resolution;
        samples[i] = textureSample(screen_texture, texture_sampler, uv + offset);
    }

    var horizontal = vec3<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        horizontal += samples[i].xyz * sobel_x[i];
    }

    var vertical = vec3<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        vertical += samples[i].xyz * sobel_y[i];
    }

    var edge = sqrt(dot(horizontal, horizontal) + dot(vertical, vertical));
    if edge < config.color_threshold {
        return 0.0;
    }
    return edge;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(textureDimensions(screen_texture));
    let frag_coord = in.position.xy;

    let color = textureSample(screen_texture, texture_sampler, in.uv);

    if config.enabled == 1u {
        let edge_depth = detect_edge_depth(frag_coord);
        let edge_normal = detect_edge_normal(frag_coord);
        let edge_color = detect_edge_color(in.uv, resolution);
        let edge = max(edge_depth, max(edge_normal, edge_color));

        if config.debug == 1u {
            return vec4(edge_depth, edge_normal, edge_color, 1.0);
        }

        if edge > 0.01 {
            return config.edge_color;
        }
    }

    return color;
}