#version 460

#extension GL_EXT_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_scalar_block_layout : enable
#extension GL_GOOGLE_include_directive : enable

#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require
#extension GL_EXT_buffer_reference2 : require

#include "common.glsl"
#include "host.glsl"

hitAttributeEXT vec2 attribs;

layout(location = 0) rayPayloadInEXT Payload prd;
layout(location = 1) rayPayloadEXT bool isShadowed;

layout(buffer_reference, scalar) buffer Vertices {Vertex v[]; }; // Positions of an object
layout(buffer_reference, scalar) buffer Indices {ivec3 i[]; }; // Triangle indices
layout(buffer_reference, scalar) buffer Materials {Material m[]; }; // Array of all materials on an object
layout(buffer_reference, scalar) buffer MatIndices {int i[]; }; // Material ID for each triangle
layout(set = 0, binding = 0) uniform accelerationStructureEXT topLevelAS;
layout(set = 0, binding = 3, scalar) buffer ObjDesc_ { ObjDesc i[]; } objDesc;
layout(set = 0, binding = 4) uniform sampler2D textureSamplers[];

void main()
{
    // Object data
    ObjDesc    objResource = objDesc.i[gl_InstanceCustomIndexEXT];
    MatIndices matIndices  = MatIndices(objResource.materialIndexAddress);
    Materials  materials   = Materials(objResource.materialAddress);
    Indices    indices     = Indices(objResource.indexAddress);
    Vertices   vertices    = Vertices(objResource.vertexAddress);

    // Indices of the triangle
    ivec3 ind = indices.i[gl_PrimitiveID];

    // Vertex of the triangle
    Vertex v0 = vertices.v[ind.x];
    Vertex v1 = vertices.v[ind.y];
    Vertex v2 = vertices.v[ind.z];

    const vec3 barycentrics = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);

    // Computing the coordinates of the hit position
    const vec3 pos      = v0.position * barycentrics.x + v1.position * barycentrics.y + v2.position * barycentrics.z;
    const vec3 worldPos = vec3(gl_ObjectToWorldEXT * vec4(pos, 1.0));  // Transforming the position to world space

    // Computing the normal at hit position
    const vec3 nrm      = v0.normal * barycentrics.x + v1.normal * barycentrics.y + v2.normal * barycentrics.z;
    const vec3 worldNrm = normalize(vec3(nrm * gl_WorldToObjectEXT));  // Transforming the normal to world space

    // Vector toward the light
    vec3  L;
    float lightIntensity = 3.0;
    float lightDistance  = 100000.0;
    // Point light
    L = normalize(vec3(100.0, 50.0, 0.0));

    // Material of the object
    int               matIdx = matIndices.i[gl_PrimitiveID];
    Material          mat    = materials.m[matIdx];


    // Diffuse
    vec3 diffuse = vec3(1.0, 1.0, 1.0);//computeDiffuse(mat, L, worldNrm);
    if(mat.textureId >= 0)
    {
        uint txtId    = mat.textureId + objDesc.i[gl_InstanceCustomIndexEXT].textureOffset;
        vec2 texCoord = v0.texCoord0 * barycentrics.x + v1.texCoord0 * barycentrics.y + v2.texCoord0 * barycentrics.z;
        diffuse *= texture(textureSamplers[nonuniformEXT(txtId)], texCoord).xyz;
    }

    vec3  specular    = vec3(0);
    float attenuation = 1;

    // Tracing shadow ray only if the light is visible from the surface
    if(dot(worldNrm, L) > 0)
    {
        float tMin   = 0.001;
        float tMax   = lightDistance;
        vec3  origin = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
        vec3  rayDir = L;
        uint  flags  = gl_RayFlagsTerminateOnFirstHitEXT | gl_RayFlagsOpaqueEXT | gl_RayFlagsSkipClosestHitShaderEXT;
        isShadowed   = true;
        traceRayEXT(topLevelAS,  // acceleration structure
                    flags,       // rayFlags
                    0xFF,        // cullMask
                    0,           // sbtRecordOffset
                    0,           // sbtRecordStride
                    1,           // missIndex
                    origin,      // ray origin
                    tMin,        // ray min range
                    rayDir,      // ray direction
                    tMax,        // ray max range
                    1            // payload (location = 1)
        );

        if(isShadowed)
        {
            attenuation = 0.3;
        }
        else
        {
            // Specular
            specular = vec3(0);//computeSpecular(mat, gl_WorldRayDirectionEXT, L, worldNrm);
        }
    }

    prd.hitValue = vec3(lightIntensity * attenuation * (diffuse + specular));
}