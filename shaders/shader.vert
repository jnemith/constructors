#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;
layout(location=2) in vec3 a_normal;

layout(location=0) out vec3 v_color;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec3 v_position;

layout(set=0, binding=0)
uniform Uniforms {
    vec3 u_view_position;
    mat4 u_view_proj;
};

void main() {
    v_color = a_color;
    v_normal = a_normal;
    v_position = a_position;
    gl_Position = u_view_proj * vec4(a_position, 1.0);
}