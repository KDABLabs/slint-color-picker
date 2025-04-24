#version 440

layout(location = 0) in vec4 qt_Vertex;
layout(location = 1) in vec2 qt_MultiTexCoord0;

layout(location = 0) out vec2 uv;

layout(std140, binding = 0) uniform MatrixBlock {
    mat4 qt_Matrix;
} transforms;

void main()
{
    gl_Position = transforms.qt_Matrix * qt_Vertex;
    uv = qt_MultiTexCoord0;
}

// #version 440

// layout(location = 0) in vec3 inPosition;
// layout(location = 1) in vec2 inTexCoord;

// layout(location = 0) out vec2 coord;

// void main()
// {
//     gl_Position = vec4(inPosition, 1.0);
//     coord = inTexCoord;
// }
