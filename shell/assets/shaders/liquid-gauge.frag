#version 440
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec2 surfaceSize;
    vec4 liquidColor;
    float fillHeight;
    float phase;
    float available;
};
void main() {
    vec2 uv = qt_TexCoord0;
    vec2 p = (uv - 0.5) * 2.0;
    float r = length(p);
    float aa = max(fwidth(r), 0.008);
    float mask = 1.0 - smoothstep(0.96 - aa, 0.96 + aa, r);
    if (mask <= 0.0) { fragColor = vec4(0.0); return; }
    vec3 pigment = liquidColor.rgb / max(liquidColor.a, 0.001);
    float innerRadius = 0.86;
    float vessel = 1.0 - smoothstep(innerRadius - aa, innerRadius + aa, r);
    vec2 v = p / (innerRadius * 2.0) + 0.5;
    float amplitude = 0.020 * smoothstep(0.0, 0.07, fillHeight) * (1.0 - smoothstep(0.93, 1.0, fillHeight));
    float top = 1.0 - fillHeight;
    float back = top + sin(v.x * 7.6 - phase * 1.2 + 1.1) * amplitude
        + cos(v.x * 13.0 + phase * 0.6) * amplitude * 0.35;
    float front = top + sin(v.x * 8.8 + phase * 1.6) * amplitude
        + sin(v.x * 17.0 - phase * 0.7) * amplitude * 0.25;
    float wet = step(0.0001, fillHeight) * available;
    float backFill = smoothstep(back - 0.009, back + 0.009, v.y) * wet * vessel;
    float frontFill = smoothstep(front - 0.008, front + 0.008, v.y) * wet * vessel;

    // A shaded glass vessel remains visible even when the metric is unknown.
    vec3 color = mix(vec3(0.021, 0.041, 0.075), vec3(0.045, 0.078, 0.125), 1.0 - uv.y);
    color += pigment * pow(max(1.0 - r, 0.0), 3.0) * 0.025;
    color = mix(color, pigment * 0.37, backFill * 0.7);
    float waterDepth = clamp((v.y - front) / max(fillHeight, 0.06), 0.0, 1.0);
    vec3 water = pigment * mix(0.76, 0.30, waterDepth);
    water += pigment * exp(-pow((v.x - 0.26) * 7.0, 2.0)) * 0.14;
    color = mix(color, water, frontFill * 0.88);
    float meniscus = exp(-pow((v.y - front) * 150.0, 2.0)) * wet * vessel;
    color += mix(pigment, vec3(0.91, 1.0, 1.0), 0.35) * meniscus * 0.6;

    // Curved specular edges, a top reflection and a restrained lower caustic.
    vec2 normal = p / max(r, 0.001);
    float rim = exp(-pow((r - 0.92) * 45.0, 2.0));
    float upper = pow(max(dot(normal, normalize(vec2(-0.65, -0.8))), 0.0), 8.0);
    float lower = pow(max(dot(normal, normalize(vec2(0.45, 0.9))), 0.0), 6.0);
    color += rim * (vec3(0.12, 0.20, 0.30) + upper * vec3(0.65, 0.81, 0.92));
    color += pigment * rim * lower * (0.2 + frontFill * 0.18);
    color += vec3(0.18, 0.25, 0.33) * exp(-pow((p.y + 0.65 + p.x * 0.18) * 16.0, 2.0)) * (1.0 - smoothstep(0.45, 0.82, r)) * 0.22;
    float caustic = exp(-pow((r - 0.81) * 35.0, 2.0)) * max(p.y, 0.0) * frontFill;
    color += pigment * caustic * 0.20;
    fragColor = vec4(color * mask, mask) * qt_Opacity;
}
