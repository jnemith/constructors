#version 450

layout(location=0) in vec3 v_color;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_position;

layout(location=0) out vec4 f_color;

void main() {
    vec3 lightColor = vec3(1.0, 1.0, 1.0);

    vec3 norm = normalize(v_normal);
    vec3 sunPosition = vec3(v_position.x, 255.0, v_position.z);
    vec3 lightDir = normalize(sunPosition - v_position);

    float ambientStrength = 0.3;
    vec3 ambient = ambientStrength * lightColor;

    float diffuseStrength = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diffuseStrength * lightColor;

    vec3 result = (ambient + diffuse) * v_color;
    f_color = vec4(result, 1.0);
}