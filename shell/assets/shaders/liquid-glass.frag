#version 440
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec2 surfaceSize;
    float cornerRadius;
    float refraction;
    vec4 tintColor;
    vec4 upperTint;
    vec4 accentA;
    vec4 accentB;
    vec2 lightPosition;
    float hoverAmount;
    float hasBackdrop;
};
layout(binding = 1) uniform sampler2D backdropTexture;

float roundedBox(vec2 p, vec2 halfSize, float radius) {
    vec2 q = abs(p) - halfSize + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}
vec3 sampleSoft(vec2 uv) {
    vec2 d = vec2(5.5) / surfaceSize;
    vec3 c = texture(backdropTexture, clamp(uv, 0.001, 0.999)).rgb * 0.36;
    c += texture(backdropTexture, clamp(uv + vec2(d.x, d.y), 0.001, 0.999)).rgb * 0.16;
    c += texture(backdropTexture, clamp(uv + vec2(-d.x, d.y), 0.001, 0.999)).rgb * 0.16;
    c += texture(backdropTexture, clamp(uv + vec2(d.x, -d.y), 0.001, 0.999)).rgb * 0.16;
    c += texture(backdropTexture, clamp(uv - d, 0.001, 0.999)).rgb * 0.16;
    return c;
}
void main() {
    vec2 uv = qt_TexCoord0;
    vec2 size = max(surfaceSize, vec2(1.0));
    vec2 p = (uv - 0.5) * size;
    float radius = min(cornerRadius, min(size.x, size.y) * 0.5);
    float sd = roundedBox(p, size * 0.5, radius);
    float aa = max(fwidth(sd), 0.65);
    float mask = 1.0 - smoothstep(-aa, aa, sd);
    if (mask <= 0.0) { fragColor = vec4(0.0); return; }

    // Surface normal bends around the corners of the lens. The displacement
    // is strongest at the bevel, leaving text-bearing central regions calm.
    vec2 q = abs(p) - size * 0.5 + radius;
    vec2 n = q.x > q.y ? vec2(sign(p.x), 0.0) : vec2(0.0, sign(p.y));
    if (q.x > 0.0 && q.y > 0.0) n = normalize(q) * sign(p);
    float depth = max(-sd, 0.0);
    float bevel = 1.0 - smoothstep(0.0, 19.0, depth);
    float lens = bevel * bevel;
    vec2 refracted = uv - n * refraction * lens / size;
    vec3 backdrop = vec3(0.025, 0.045, 0.085);
    if (hasBackdrop > 0.5) {
        backdrop = sampleSoft(refracted);
        // Restrained dispersion: coloured light at the edge, not RGB fringes
        // through the middle of the panel or its foreground controls.
        vec2 split = n * lens * 0.85 / size;
        vec3 dispersion = vec3(texture(backdropTexture, clamp(refracted + split, 0.001, 0.999)).r,
            backdrop.g, texture(backdropTexture, clamp(refracted - split, 0.001, 0.999)).b);
        backdrop = mix(backdrop, dispersion, 0.24 * lens);
    }
    vec4 tint = mix(upperTint, tintColor, smoothstep(0.0, 0.7, uv.y));
    vec3 pigment = tint.rgb / max(tint.a, 0.001);
    float density = clamp(tint.a * 0.89 - lens * 0.25, 0.40, 0.92);
    vec3 color = mix(backdrop * 0.82, pigment, density);

    float topLight = pow(max(dot(n, normalize(vec2(-0.55, -0.85))), 0.0), 2.0);
    float underLight = pow(max(dot(n, normalize(vec2(0.65, 0.75))), 0.0), 2.0);
    float rim = 1.0 - smoothstep(0.8, 2.2, depth);
    float innerRim = exp(-pow((depth - 4.0) * 0.52, 2.0));
    vec3 edgeColor = mix(accentA.rgb, accentB.rgb, smoothstep(0.0, 1.0, uv.x * 0.45 + uv.y * 0.55));
    color += edgeColor * lens * 0.12;
    color += vec3(0.80, 0.90, 1.0) * rim * (0.22 + 0.48 * topLight);
    color += edgeColor * rim * underLight * 0.38;
    color += vec3(0.62, 0.77, 0.96) * innerRim * topLight * 0.12;

    float sheen = exp(-pow((uv.y + uv.x * 0.17 - 0.035) * 7.0, 2.0));
    color += vec3(0.35, 0.48, 0.70) * sheen * 0.06;
    vec2 pointerDelta = (uv - lightPosition) * vec2(size.x / max(size.y, 1.0), 1.0);
    color += vec3(0.40, 0.65, 0.85) * exp(-dot(pointerDelta, pointerDelta) * 10.0) * hoverAmount * 0.055;
    float alpha = mask * (hasBackdrop > 0.5 ? 1.0 : tint.a);
    fragColor = vec4(color * alpha, alpha) * qt_Opacity;
}
