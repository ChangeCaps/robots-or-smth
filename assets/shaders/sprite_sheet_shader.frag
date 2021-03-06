#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 2) uniform texture2D TextureAtlas_texture;
layout(set = 1, binding = 3) uniform sampler TextureAtlas_texture_sampler;

void main() {
    vec4 color = texture(sampler2D(TextureAtlas_texture, TextureAtlas_texture_sampler), v_Uv);

    if (color.a < 0.1) {
        discard;
    }

    o_Target = color;
}