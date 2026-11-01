
sampler2D s0 : register(s0);
float4 c0 : register(c0); // xy: one texel of the source along the blur's axis
float4 c1 : register(c1); // x: the scale

float4 main(float2 uv : TEXCOORD0) : COLOR
{
    float2 d = c0.xy;
    float4 c = tex2D(s0, uv) * 0.2013;
    c += (tex2D(s0, uv + d * 1.3366) + tex2D(s0, uv - d * 1.3366)) * 0.2185;
    c += (tex2D(s0, uv + d * 3.4295) + tex2D(s0, uv - d * 3.4295)) * 0.0821;
    c += (tex2D(s0, uv + d * 5.4264) + tex2D(s0, uv - d * 5.4264)) * 0.0461;
    c += (tex2D(s0, uv + d * 7.4359) + tex2D(s0, uv - d * 7.4359)) * 0.0262;
    c += (tex2D(s0, uv + d * 9.4436) + tex2D(s0, uv - d * 9.4436)) * 0.0162;
    c += (tex2D(s0, uv + d * 11.4401) + tex2D(s0, uv - d * 11.4401)) * 0.0102;
    c.rgb *= c1.x;
    return c;
}
