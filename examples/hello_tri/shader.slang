struct PC
{
    float2 resolution;
    float time;
};

struct VSInput
{
    float3 pos;
    float3 normal;
    float2 uv;
};

struct VSOutput
{
    float4 pos : SV_POSITION;
    float3 normal;
    float2 uv;
};

[[vk::push_constant]]
PC pc;

static const float3 light_dir = float3(0.0, 1.0, -0.3);
static const float depth_far = 10000;
static const float depth_near = 0.01;

[shader("vertex")]
VSOutput main(uint vertexID: SV_VertexID, VSInput input)
{
    VSOutput output;

    float angle = pc.time / 1000;
    float3 iHat = float3(cos(angle), 0, sin(angle));
    float3 jHat = float3(0, 1, 0);
    float3 kHat = cross(iHat, jHat);

    float3 vertWorld = 0;
    vertWorld += iHat * input.pos.x;
    vertWorld += jHat * input.pos.y;
    vertWorld += kHat * input.pos.z;

    vertWorld.z += 10;

    float3 vertView = vertWorld;
    float depth = (vertView.z - depth_near) / (depth_far - depth_near);
    float4 vertClip = float4(vertView.xy, depth, 1);
    vertClip.x *= pc.resolution.y / pc.resolution.x;

    output.pos = vertClip - float4(0.0, 1.0, 0.0, 0.0);
    // output.pos = vertClip;
    output.normal = input.normal;
    output.uv = input.uv;

    return output;
}

[shader("fragment")]
float4 main(VSOutput input)
{
    float2 pixelCoord = input.pos.xy;

    float2 screenUV = pixelCoord / pc.resolution;

    float3 normal = normalize(input.normal);
    float3 col = normal * 0.5 + 0.5;

    col = float3(input.uv, 0.0);
    // col = pow(col, 2.2);
    return float4(col, 1.0);
}
