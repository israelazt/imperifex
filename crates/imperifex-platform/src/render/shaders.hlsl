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
};

struct Pixel {
    float4 position : SV_Position;
    float2 texel : TEXCOORD0;
    float4 color : COLOR;
    nointerpolation float glyph : GLYPH;
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
    return pixel;
}

float4 ps_main(Pixel pixel) : SV_Target {
    // Glyphs fall on whole pixels, so the atlas is read without filtering.
    float coverage = pixel.glyph > 0.5 ? atlas.Load(int3(pixel.texel, 0)) : 1.0;
    return float4(pixel.color.rgb, pixel.color.a * coverage);
}
