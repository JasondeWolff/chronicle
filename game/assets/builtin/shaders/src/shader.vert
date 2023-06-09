#version 450

#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec4 inTangent;
layout(location = 3) in vec2 inTexCoord0;
layout(location = 4) in vec2 inTexCoord1;
layout(location = 5) in vec4 inColor;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

out gl_PerVertex {
    vec4 gl_Position;
};

// layout(set = 0, binding = 0) uniform UniformBufferObject {
//     mat4 model;
//     mat4 view;
//     mat4 proj;
// } ubo;

// void main() {
//     gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition.xyz, 1.0);
//     fragColor = inNormal * 0.5 + 0.5;
//     fragTexCoord = inTexCoord0;
// }

layout(push_constant) uniform PushConstants {
    mat4 mvp;
} pc;

void main() {
    gl_Position = pc.mvp * vec4(inPosition.xyz, 1.0);
    fragColor = inNormal * 0.5 + 0.5;
    fragTexCoord = inTexCoord0;
}