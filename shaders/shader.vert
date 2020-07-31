#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;
layout(location=2) in vec3 a_normal;

layout(location=0) out vec3 v_color;
layout(location=1) out vec3 v_normal;

layout(set=0, binding=0)
uniform Uniforms {
    vec3 u_view_position;
    mat4 u_view_proj;
};

layout(set=1, binding=0)
uniform Block {
    vec3 b_position;
};

void main() {
    v_color = a_color;
    v_normal = a_normal;
    vec3 pos = a_position + b_position;
    gl_Position = u_view_proj * vec4(pos, 1.0);
}