
struct VsOut
{
    float4 pos : POSITION;
    float2 uv : TEXCOORD0;
};
VsOut main(float4 pos : POSITION, float2 uv : TEXCOORD0)
{
    VsOut o;
    o.pos = pos;
    o.uv = uv;
    return o;
}
