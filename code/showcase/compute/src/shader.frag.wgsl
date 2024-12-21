struct Light {
    light_position: vec3<f32>,
    light_color: vec3<f32>,
}

struct FragmentOutput {
    @location(0) f_color: vec4<f32>,
}

var<private> v_tex_coords_1: vec2<f32>;
var<private> v_position_1: vec3<f32>;
var<private> v_light_position_1: vec3<f32>;
var<private> v_view_position_1: vec3<f32>;
var<private> f_color: vec4<f32>;
@group(0) @binding(0) 
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) 
var s_diffuse: sampler;
@group(0) @binding(2) 
var t_normal: texture_2d<f32>;
@group(0) @binding(3) 
var s_normal: sampler;
@group(2) @binding(0) 
var<uniform> global: Light;

fn main_1() {
    var object_color: vec4<f32>;
    var object_normal: vec4<f32>;
    var ambient_strength: f32 = 0.1f;
    var ambient_color: vec3<f32>;
    var normal: vec3<f32>;
    var light_dir: vec3<f32>;
    var diffuse_strength: f32;
    var diffuse_color: vec3<f32>;
    var view_dir: vec3<f32>;
    var half_dir: vec3<f32>;
    var specular_strength: f32;
    var specular_color: vec3<f32>;
    var result: vec3<f32>;

    let _e14 = v_tex_coords_1;
    let _e15 = textureSample(t_diffuse, s_diffuse, _e14);
    object_color = _e15;
    let _e18 = v_tex_coords_1;
    let _e19 = textureSample(t_normal, s_normal, _e18);
    object_normal = _e19;
    let _e23 = global.light_color;
    let _e24 = ambient_strength;
    ambient_color = (_e23 * _e24);
    let _e27 = object_normal;
    let _e34 = object_normal;
    normal = normalize(((_e34.xyz * 2f) - vec3(1f)));
    let _e43 = v_light_position_1;
    let _e44 = v_position_1;
    let _e46 = v_light_position_1;
    let _e47 = v_position_1;
    light_dir = normalize((_e46 - _e47));
    let _e53 = normal;
    let _e54 = light_dir;
    let _e59 = normal;
    let _e60 = light_dir;
    diffuse_strength = max(dot(_e59, _e60), 0f);
    let _e65 = global.light_color;
    let _e66 = diffuse_strength;
    diffuse_color = (_e65 * _e66);
    let _e69 = v_view_position_1;
    let _e70 = v_position_1;
    let _e72 = v_view_position_1;
    let _e73 = v_position_1;
    view_dir = normalize((_e72 - _e73));
    let _e77 = view_dir;
    let _e78 = light_dir;
    let _e80 = view_dir;
    let _e81 = light_dir;
    half_dir = normalize((_e80 + _e81));
    let _e87 = normal;
    let _e88 = half_dir;
    let _e93 = normal;
    let _e94 = half_dir;
    let _e101 = normal;
    let _e102 = half_dir;
    let _e107 = normal;
    let _e108 = half_dir;
    specular_strength = pow(max(dot(_e107, _e108), 0f), 32f);
    let _e116 = specular_strength;
    let _e117 = global.light_color;
    specular_color = (_e116 * _e117);
    let _e120 = ambient_color;
    let _e121 = diffuse_color;
    let _e123 = specular_color;
    let _e125 = object_color;
    result = (((_e120 + _e121) + _e123) * _e125.xyz);
    let _e129 = result;
    let _e130 = object_color;
    f_color = vec4<f32>(_e129.x, _e129.y, _e129.z, _e130.w);
    return;
}

@fragment 
fn main(@location(0) v_tex_coords: vec2<f32>, @location(1) v_position: vec3<f32>, @location(2) v_light_position: vec3<f32>, @location(3) v_view_position: vec3<f32>) -> FragmentOutput {
    v_tex_coords_1 = v_tex_coords;
    v_position_1 = v_position;
    v_light_position_1 = v_light_position;
    v_view_position_1 = v_view_position;
    main_1();
    let _e31 = f_color;
    return FragmentOutput(_e31);
}
