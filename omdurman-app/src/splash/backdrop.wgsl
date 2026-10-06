// The title screen's and the lobby's backdrop, one full-screen UI node: the
// period map projected in perspective, blurred, dimmed and sepia-toned, the
// crossfade into the next map and the first map's fade-in, then the dark
// overlays over it (title: the horizontal fade and the top/bottom scrims;
// lobby: a scrim and the glow around the lobby's panel). Every layer is
// composited here, premultiplied, so the screen keeps one group opacity
// (`composite`) over the board behind it. The geometry and every number come
// from `backdrop.rs` (see `splash::map` and `splash::params` for the
// meanings).

#import bevy_ui::ui_vertex_output::UiVertexOutput

const MODE_WIDE: u32 = 0u;
const MODE_NARROW: u32 = 1u;
const MODE_LOBBY: u32 = 2u;

const STOPS_FADE: u32 = 0u;
const STOPS_SCRIM: u32 = 1u;
const STOPS_GLOW: u32 = 2u;

// How one slide's map is drawn: its pose (screen point, homogeneous -> image
// square fraction, homogeneous), the UV rect the image square shows, the
// texel size, and the separable blur kernel (x: offset in texels, y: weight).
struct MapSampling {
    to_square: mat3x3<f32>,
    uv_min: vec2<f32>,
    uv_size: vec2<f32>,
    texel: vec2<f32>,
    tap_count: u32,
    taps: array<vec4<f32>, 12>,
}

struct Params {
    // Node size in logical pixels: the screen.
    screen: vec2<f32>,
    composite: f32,
    mode: u32,
    // The map's clip rect: min.xy, max.xy.
    region: vec4<f32>,
    // Linear RGB.
    backdrop: vec4<f32>,
    fade_in: f32,
    progress: f32,
    incoming: u32,
    has_map: u32,
    current: MapSampling,
    next: MapSampling,
    // (x: position, y: value) stops of the fade, the scrims and the glow.
    fade_stops: array<vec4<f32>, 8>,
    scrim_stops: array<vec4<f32>, 8>,
    glow_stops: array<vec4<f32>, 8>,
    stop_counts: vec4<u32>,
    // Rows produce r, g, b (as CSS `sepia(1)`).
    sepia_matrix: mat3x3<f32>,
    brightness: f32,
    sepia: f32,
    blend_gamma: f32,
    narrow_scrim: f32,
    // The lobby's panel: min.xy, max.xy.
    panel: vec4<f32>,
    panel_radius: f32,
    glow_width: f32,
    lobby_scrim: f32,
}

@group(1) @binding(0) var<uniform> params: Params;
@group(1) @binding(1) var current_texture: texture_2d<f32>;
@group(1) @binding(2) var current_sampler: sampler;
@group(1) @binding(3) var next_texture: texture_2d<f32>;
@group(1) @binding(4) var next_sampler: sampler;

fn stop(list: u32, i: u32) -> vec2<f32> {
    switch list {
        case STOPS_FADE: {
            return params.fade_stops[i].xy;
        }
        case STOPS_SCRIM: {
            return params.scrim_stops[i].xy;
        }
        default: {
            return params.glow_stops[i].xy;
        }
    }
}

// Piecewise-linear value of a stop list at `at`, held flat beyond its ends
// (`map::stops_at`).
fn stops_at(list: u32, at: f32) -> f32 {
    let count = params.stop_counts[list];
    if count == 0u {
        return 0.0;
    }
    let first = stop(list, 0u);
    if at <= first.x {
        return first.y;
    }
    for (var i = 1u; i < count; i++) {
        let a = stop(list, i - 1u);
        let b = stop(list, i);
        if at <= b.x {
            var t = 1.0;
            if b.x > a.x {
                t = (at - a.x) / (b.x - a.x);
            }
            return a.y + (b.y - a.y) * t;
        }
    }
    return stop(list, count - 1u).y;
}

// The drawn opacity of a dark overlay of CSS opacity `alpha`
// (`map::css_overlay_alpha`).
fn css_overlay(alpha: f32) -> f32 {
    return 1.0 - pow(1.0 - clamp(alpha, 0.0, 1.0), params.blend_gamma);
}

// `map::map_alpha`.
fn map_alpha(fade: f32, composite: f32) -> f32 {
    let covered = 1.0 - fade * composite;
    if covered <= 1e-6 {
        return 0.0;
    }
    return composite * (1.0 - fade) / covered;
}

// `color` at `alpha` over the premultiplied `acc`.
fn over(acc: vec4<f32>, color: vec3<f32>, alpha: f32) -> vec4<f32> {
    let a = clamp(alpha, 0.0, 1.0);
    return vec4(color * a, a) + acc * (1.0 - a);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3(2.4));
    return select(high, low, c <= vec3(0.04045));
}

// Dim to `brightness`, then mix `sepia` of the way toward the sepia tone, on
// encoded sRGB values (CSS `brightness() sepia()`; the order matters where the
// sepia matrix clips at white).
fn tone(encoded: vec3<f32>) -> vec3<f32> {
    let dimmed = encoded * params.brightness;
    let toned = min(params.sepia_matrix * dimmed, vec3(1.0));
    return mix(dimmed, toned, params.sepia);
}

// Map `which` (0: current, 1: incoming) at image square fraction `f`: blurred
// in texture space, toned, as linear RGB. The textures hold encoded sRGB
// values sampled as data, so the blur and tone see what the CSS filters do.
fn map_color(which: u32, f: vec2<f32>) -> vec3<f32> {
    var m = params.current;
    if which == 1u {
        m = params.next;
    }
    let uv = m.uv_min + f * m.uv_size;
    var sum = vec3(0.0);
    for (var j = 0u; j < m.tap_count; j++) {
        let ty = m.taps[j];
        for (var i = 0u; i < m.tap_count; i++) {
            let tx = m.taps[i];
            let at = uv + vec2(tx.x, ty.x) * m.texel;
            var texel: vec4<f32>;
            if which == 0u {
                texel = textureSampleLevel(current_texture, current_sampler, at, 0.0);
            } else {
                texel = textureSampleLevel(next_texture, next_sampler, at, 0.0);
            }
            sum += tx.y * ty.y * texel.rgb;
        }
    }
    return srgb_to_linear(tone(sum));
}

// `map::crossfade_alphas`: (under, over).
fn crossfade(alpha: f32, progress: f32) -> vec2<f32> {
    let over_alpha = alpha * progress;
    var under = 0.0;
    if 1.0 - over_alpha > 1e-6 {
        under = alpha * (1.0 - progress) / (1.0 - over_alpha);
    }
    return vec2(under, over_alpha);
}

// Where screen point `pos` falls on slide `which`'s image square (xy, as
// fractions of its side), and whether it does (z: 1, or 0 off the square or
// behind the eye).
fn square_point(which: u32, pos: vec2<f32>) -> vec3<f32> {
    var to_square = params.current.to_square;
    if which == 1u {
        to_square = params.next.to_square;
    }
    let q = to_square * vec3(pos, 1.0);
    if q.z <= 0.0 {
        return vec3(0.0);
    }
    let f = q.xy / q.z;
    let on = all(f >= vec2(0.0)) && all(f <= vec2(1.0));
    return vec3(f, select(0.0, 1.0, on));
}

// Signed distance from the outline of the rounded rect `rect` (min.xy,
// max.xy) with corner `radius`; negative inside.
fn rounded_rect_distance(p: vec2<f32>, rect: vec4<f32>, radius: f32) -> f32 {
    let centre = (rect.xy + rect.zw) * 0.5;
    let half_size = (rect.zw - rect.xy) * 0.5;
    let r = clamp(radius, 0.0, min(half_size.x, half_size.y));
    let q = abs(p - centre) - (half_size - vec2(r));
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let pos = in.uv * params.screen;
    let backdrop = params.backdrop.rgb;
    if params.has_map == 0u {
        // Before the maps are ready: the plain backdrop.
        return vec4(backdrop, params.composite);
    }
    // The backdrop layer's opacity over the map, before the group opacity.
    var fade = 0.0;
    if params.mode == MODE_WIDE {
        fade = css_overlay(stops_at(STOPS_FADE, pos.x / params.screen.x));
    } else if params.mode == MODE_NARROW {
        fade = params.narrow_scrim;
    }

    var acc = vec4(0.0);
    let in_region = all(pos >= params.region.xy) && all(pos <= params.region.zw);
    let current = square_point(0u, pos);
    if in_region && current.z > 0.0 {
        // The map's opacity as one group with the backdrop layer over it.
        let group = map_alpha(fade, params.composite);
        // The first map dissolves in from the plain backdrop, in its shape.
        let shown = crossfade(group, params.fade_in);
        if params.fade_in < 1.0 {
            acc = over(acc, backdrop, shown.x);
        }
        var progress = 0.0;
        if params.incoming != 0u {
            progress = params.progress;
        }
        // Both slides cover the whole region (`no_map_edge_shows...`), so
        // the crossfade keeps the group opacity everywhere.
        let maps = crossfade(shown.y, progress);
        acc = over(acc, map_color(0u, current.xy), maps.x);
        let next = square_point(1u, pos);
        if params.incoming != 0u && next.z > 0.0 {
            acc = over(acc, map_color(1u, next.xy), maps.y);
        }
    }

    if params.mode == MODE_LOBBY {
        acc = over(acc, backdrop, params.lobby_scrim);
        let outside = max(rounded_rect_distance(pos, params.panel, params.panel_radius), 0.0);
        acc = over(acc, backdrop, stops_at(STOPS_GLOW, outside / max(params.glow_width, 1e-3)));
    } else {
        // The fade spans the whole width: left of the map region it is the
        // solid backdrop of the menu column.
        acc = over(acc, backdrop, fade * params.composite);
        acc = over(
            acc,
            backdrop,
            css_overlay(stops_at(STOPS_SCRIM, pos.y / params.screen.y)) * params.composite,
        );
    }
    if acc.a <= 0.0 {
        return vec4(0.0);
    }
    return vec4(acc.rgb / acc.a, acc.a);
}
