#version 440

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 fragColor;

layout(binding = 1) uniform sampler2D source1; // blue
layout(binding = 2) uniform sampler2D source2; // green
layout(binding = 3) uniform sampler2D source3; // red

// Target color to apply
void main()
{
    // fragColor = vec4(1.0, 0.0, 0.0, 1.0);

    // Sample  textures
    vec4 source1V4 = texture(source1, uv);
    vec4 source2V4 = texture(source2, uv);
    vec4 source3V4 = texture(source3, uv);

    // Combine RGBA from textures
    fragColor = vec4(source1V4.r + source2V4.r + source3V4.r,
                     source1V4.g + source2V4.g + source3V4.g,
                     source1V4.b + source2V4.b + source3V4.b,
                     source1V4.a + source2V4.a + source3V4.a );

    // fragColor = source3V4;
}
