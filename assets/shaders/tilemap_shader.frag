#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

# ifdef COLORMATERIAL_TEXTURE
layout(set = 1, binding = 0) uniform texture2D ColorMaterial_texture;
layout(set = 1, binding = 1) uniform sampler ColorMaterial_texture_sampler;
# endif

void main() {
# ifdef COLORMATERIAL_TEXTURE
    o_Target = texture(sampler2D(ColorMaterial_texture, ColorMaterial_texture_sampler), v_Uv);
# endif
}