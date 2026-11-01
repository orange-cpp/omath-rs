
sampler2D s0 : register(s0);
sampler2D s1 : register(s1);
float4 c0 : register(c0);

float glow_at(float2 p)
{
    return tex2Dlod(s0, float4(p, 0, 0)).r;
}

float body_at(float2 p)
{
    return tex2Dlod(s1, float4(p, 0, 0)).r;
}

// the blurred silhouettes over the frame in the group's colour, premultiplied, and none of it on the models
float4 main(float2 uv : TEXCOORD0) : COLOR
{
    float g = glow_at(uv) * (1.0 - body_at(uv));
    return float4(c0.rgb * g, g);
}
