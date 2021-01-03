#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform ColorMaterial_color {
    vec4 Color;
};

# ifdef COLORMATERIAL_TEXTURE
layout(set = 1, binding = 1) uniform texture2D ColorMaterial_texture;
layout(set = 1, binding = 2) uniform sampler ColorMaterial_texture_sampler;
# endif

void main() {
# ifdef COLORMATERIAL_TEXTURE
    vec4 color = texture(sampler2D(ColorMaterial_texture, ColorMaterial_texture_sampler), v_Uv);

    if (color.a < 0.1) {
        discard;
    }

    o_Target = color;
# endif
}