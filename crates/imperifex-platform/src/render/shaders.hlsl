// One instance is one rectangle: a plain colour, or a glyph whose shape is in
// the atlas. Drawn as a strip of four corners, with no vertex buffer for them.

cbuffer Frame : register(b0) {
    float2 target_size; // in pixels
    float2 unused;
};

Texture2D<float> atlas : register(t0);

struct Instance {
    float4 area : AREA;     // left, top, width, height, in pixels
    float4 color : COLOR;
    float4 source : SOURCE; // the same in the atlas, for a glyph
    float glyph : GLYPH;
    float radius : RADIUS;  // of the corners of a rectangle, in pixels
};

struct Pixel {
    float4 position : SV_Position;
    float2 texel : TEXCOORD0;
    float4 color : COLOR;
    nointerpolation float glyph : GLYPH;
    float2 inside : TEXCOORD1; // from the rectangle's middle, in pixels
    nointerpolation float2 half_size : TEXCOORD2;
    nointerpolation float radius : RADIUS;
};

Pixel vs_main(Instance instance, uint id : SV_VertexID) {
    float2 corner = float2(id & 1, id >> 1);
    float2 at = instance.area.xy + corner * instance.area.zw;
    Pixel pixel;
    pixel.position = float4(at.x / target_size.x * 2.0 - 1.0,
                            1.0 - at.y / target_size.y * 2.0,
                            0.0, 1.0);
    pixel.texel = instance.source.xy + corner * instance.source.zw;
    pixel.color = instance.color;
    pixel.glyph = instance.glyph;
    pixel.half_size = instance.area.zw * 0.5;
    pixel.inside = (corner - 0.5) * instance.area.zw;
    pixel.radius = instance.radius;
    return pixel;
}

float4 ps_main(Pixel pixel) : SV_Target {
    // Glyphs fall on whole pixels, so the atlas is read without filtering.
    float coverage = pixel.glyph > 0.5 ? atlas.Load(int3(pixel.texel, 0)) : 1.0;
    if (pixel.radius > 0.0) {
        // How far outside the rounded edge the pixel's middle is; the pixels it
        // crosses are covered in part.
        float2 past = abs(pixel.inside) - (pixel.half_size - pixel.radius);
        float outside = length(max(past, 0.0)) - pixel.radius;
        coverage *= saturate(0.5 - outside);
    }
    return float4(pixel.color.rgb, pixel.color.a * coverage);
}
