#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec2 v_Pos;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform Bar_size {
    vec2 size;
};

layout(set = 1, binding = 1) uniform Bar_max_value {
    float max_value;
};

layout(set = 1, binding = 2) uniform Bar_current_value {
    float current_value;
};

layout(set = 1, binding = 3) uniform Bar_border_thickness {
    float border_thickness;
};

layout(set = 1, binding = 4) uniform Bar_background_color {
    vec4 background_color;
};

layout(set = 1, binding = 5) uniform Bar_color_a {
    vec4 color_a;
};

layout(set = 1, binding = 6) uniform Bar_color_b {
    vec4 color_b;
};

void main() {
    float prog = v_Uv.x * max_value;

    vec4 color = vec4(0.0);

    vec2 border = size / 2.0 - border_thickness;

    if (prog > current_value || abs(v_Pos).x > border.x || abs(v_Pos).y > border.y) {
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