#version 100
precision mediump float;

varying vec2 uv;
varying vec4 color;
uniform sampler2D Texture;
uniform vec2 u_resolution;
uniform vec2 u_position;

// User uniforms
uniform float u_radius; // blur radius in pixels (default: 8.0)

void main() {
    vec4 base = texture2D(Texture, uv);

    // Convert radius from pixels to UV space
    vec2 texel = vec2(1.0) / u_resolution;
    float step_uv = u_radius * texel.x / 2.0;

    vec4 sum = vec4(0.0);
    float total_weight = 0.0;

    // Fixed 5x5 Gaussian kernel
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            vec2 offset = vec2(float(x), float(y)) * step_uv;
            float dist = length(vec2(float(x), float(y)));
            float weight = exp(-dist * dist / 8.0);
            vec4 s = texture2D(Texture, uv + offset);
            sum += s * weight;
            total_weight += weight;
        }
    }

    vec4 blurred = sum / total_weight;

    // Frosted glass: blend blurred background with a subtle white overlay
    float glass_alpha = 0.25;
    vec4 glass = vec4(blurred.rgb * 0.85 + vec3(0.15), 1.0) * glass_alpha;

    // Preserve original content alpha on top of the glass
    vec3 result = mix(glass.rgb, base.rgb, base.a);
    float alpha = max(glass.a, base.a);

    gl_FragColor = vec4(result, alpha);
}
