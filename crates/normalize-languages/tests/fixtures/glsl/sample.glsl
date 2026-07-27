#version 450 core

#include "common.glsl"
#include "lighting.glsl"
#include <pbr_utils.glsl>

// Uniforms
uniform mat4 u_Model;
uniform mat4 u_View;
uniform mat4 u_Projection;
uniform vec3 u_LightPos;
uniform vec3 u_CameraPos;
uniform sampler2D u_AlbedoMap;
uniform sampler2D u_NormalMap;

// Input/output
in vec3 v_Position;
in vec3 v_Normal;
in vec2 v_TexCoord;
in vec4 v_Color;

out vec4 fragColor;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

vec3 calculateDiffuse(vec3 normal, vec3 lightDir, vec3 lightColor) {
    float diff = max(dot(normal, lightDir), 0.0);
    return diff * lightColor;
}

vec3 calculateSpecular(vec3 normal, vec3 lightDir, vec3 viewDir, float shininess) {
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    return spec * vec3(1.0);
}

vec3 applyFog(vec3 color, float depth) {
    float fogFactor = exp(-depth * 0.01);
    return mix(vec3(0.5, 0.6, 0.7), color, fogFactor);
}

// Classify a scalar via if/else if/else
float classify(float n) {
    if (n < 0.0) {
        return -1.0;
    } else if (n > 0.0) {
        return 1.0;
    } else {
        return 0.0;
    }
}

// Select a tone-mapping mode via switch
vec3 toneMap(int mode, vec3 color) {
    switch (mode) {
        case 0:
            return color;
        case 1:
            return color * 1.2;
        default:
            return vec3(0.0);
    }
}

// Sum weighted samples with break/continue
float weightedSum(int count) {
    float total = 0.0;
    for (int i = 0; i < count; i++) {
        if (i == 5) {
            continue;
        }
        if (i == 20) {
            break;
        }
        total += float(i);
    }
    return total;
}

// Accumulate via while and do-while
float accumulate(int limit) {
    int i = 0;
    float total = 0.0;
    while (i < limit) {
        total += float(i);
        i++;
    }

    int j = 0;
    do {
        total += 0.5;
        j++;
    } while (j < limit);

    return total;
}

void main() {
    vec3 norm = normalize(v_Normal);
    vec3 lightDir = normalize(u_LightPos - v_Position);
    vec3 viewDir = normalize(u_CameraPos - v_Position);

    vec4 albedo = texture(u_AlbedoMap, v_TexCoord);
    vec3 diffuse = calculateDiffuse(norm, lightDir, vec3(1.0));
    vec3 specular = calculateSpecular(norm, lightDir, viewDir, 32.0);

    vec3 result = (vec3(0.1) + diffuse + specular) * albedo.rgb;
    float depth = length(u_CameraPos - v_Position);
    result = applyFog(result, depth);

    // Discard fully transparent fragments
    if (albedo.a < 0.01) {
        discard;
    }

    fragColor = vec4(result, albedo.a);
}
