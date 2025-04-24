#version 440 core

layout(location = 0) in vec2 uv; // Input texture coordinate from vertex shader
layout(location = 0) out vec4 fragColor; // Output color

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix; // Transformation matrix (unused in fragment shader)
    float qt_Opacity; // Opacity
    vec2 sv; // Saturation and Value adjustments
};

layout(binding = 1) uniform sampler2D source; // Input texture
// Convert RGB to HSV
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// Convert HSV to RGB
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    // Sample the texture (including alpha)
    vec4 texColor = texture(source, uv);

    // Convert RGB to HSV
    vec3 hsvColor = rgb2hsv(texColor.rgb);

    // Adjust Saturation and Value using the uniform buffer
    hsvColor.y *= sv.x; // Modify Saturation (sv.x)
    hsvColor.z *= sv.y; // Modify Value (sv.y)

    // Clamp values to valid range
    hsvColor.y = clamp(hsvColor.y, 0.0, 1.0);
    hsvColor.z = clamp(hsvColor.z, 0.0, 1.0);

    // Convert HSV back to RGB
    vec3 newRgbColor = hsv2rgb(hsvColor);

    // Output the final color with the original alpha and qt_Opacity
    fragColor = vec4(newRgbColor, texColor.a) * qt_Opacity;
}
