#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec2 v_Pos;

layout(location = 0) out vec4 o_Target;

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
    float prog = v_Uv.x * max_value;

    vec4 color = vec4(0.0);

    vec2 border = size / 2.0 - border_thickness;

    if (abs(v_Pos).x > border.x || abs(v_Pos).y > border.y) {
        color = border_color;
    } else if (prog > current_value) {
        color = background_color;
    } else {
        if (v_Uv.y < 0.4) { 
            color = color_b;
        } else {
            color = color_a;
        }
    }

    o_Target = color;
}