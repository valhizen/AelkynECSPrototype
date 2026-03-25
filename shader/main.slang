// Per-object push constants.
struct PushConstants {
    float4x4 model;
    float    tex_blend;
    float    time;
    float2   _pad;
};
[[vk::push_constant]]
PushConstants push;

struct VertToFrag {
    float4 position : SV_Position;
    float3 color    : COLOR;
};

[shader("vertex")]
VertToFrag vertMain(
    float3 inPos    : POSITION,
    float3 inNormal : NORMAL,
    float3 inColor  : COLOR,
    float2 inUV     : TEXCOORD
) {
    VertToFrag o;
    o.position = mul(push.model, float4(inPos, 1.0));
    o.color    = inColor;
    return o;
}

[shader("fragment")]
float4 fragMain(VertToFrag input) : SV_Target {
    return float4(input.color, 1.0);
}
