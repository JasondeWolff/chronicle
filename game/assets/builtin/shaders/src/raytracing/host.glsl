struct GlobalUniforms
{
    mat4 viewProj;
    mat4 viewInverse;
    mat4 projInverse;
};

struct ObjDesc
{
    int textureOffset;

    uint64_t  vertexAddress;
    uint64_t  indexAddress;
    uint64_t  materialAddress;
    uint64_t  materialIndexAddress;
};

struct Vertex
{
    vec3 position;
    vec3 normal;
    vec4 tangent;
    vec2 texCoord0;
    vec2 texCoord1;
    vec4 color;
};

struct Material
{
    vec3 diffuse;

    int textureId;
};