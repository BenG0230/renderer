struct VSInput
{
    float2 Pos;
    float3 Color;
};

struct VSOutput
{
    float4 Pos : SV_POSITION;
    float3 Color;
};

[shader("vertex")]
VSOutput main(uint vertexID: SV_VertexID, VSInput input)
{
    VSOutput output;

    output.Pos = float4(input.Pos * 3, 0.0, 1.0);
    output.Color = input.Color;

    return output;
}

[shader("fragment")]
float4 main(VSOutput input)
{

    float2 pixelCoord = input.Pos.xy;

    float2 screenUV = pixelCoord / float2(800, 600);

    float3 col = float3(screenUV, 0.0);
    return float4(col, 1.0);
}
