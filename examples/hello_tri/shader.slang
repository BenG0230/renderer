struct PC
{
    float2 resolution;
};

struct VSInput
{
    float3 Pos;
    float3 Normal;
};

struct VSOutput
{
    float4 Pos : SV_POSITION;
    float3 Normal;
};

[[vk::push_constant]]
PC pc;

static const float3 light_dir = float3(0.0, 1.0, -0.3);

[shader("vertex")]
VSOutput main(uint vertexID: SV_VertexID, VSInput input)
{
    VSOutput output;

    output.Pos = float4(input.Pos.xy - float2(0.0, 1.0), 0.0, 1.0);
    output.Pos.x *= pc.resolution.y / pc.resolution.x;
    output.Normal = input.Normal;

    return output;
}

[shader("fragment")]
float4 main(VSOutput input)
{
    float2 pixelCoord = input.Pos.xy;

    float2 screenUV = pixelCoord / pc.resolution;

    float3 normal = normalize(input.Normal);
    float3 col = normal * 0.5 + 0.5;
    return float4(col, 1.0);
}
