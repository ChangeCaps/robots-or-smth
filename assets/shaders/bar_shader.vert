#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec2 v_Pos;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

layout(set = 1, binding = 1) uniform Bar {
    vec2 size;
    vec4 border_color;
    vec4 background_color;
    vec4 color_a;
    vec4 color_b;
    float border_thickness;
    float max_value;
    float current_value;
};


void main() {
    vec3 position = vec3(size, 1.0) * Vertex_Position;
    gl_Position = ViewProj * Model * vec4(position, 1.0);
    v_Uv = Vertex_Uv;
    v_Pos = position.xy;
}