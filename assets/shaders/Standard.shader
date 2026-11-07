//////////////////////////////////////////
//
// NOTE: This is *not* a valid shader file
//
///////////////////////////////////////////
Shader "Standard" {
Properties {
 _Color ("Color", Color) = (1,1,1,1)
 _MainTex ("Albedo", 2D) = "white" { }
 _Cutoff ("Alpha Cutoff", Range(0,1)) = 0.5
 _Glossiness ("Smoothness", Range(0,1)) = 0.5
[Gamma]  _Metallic ("Metallic", Range(0,1)) = 0
 _MetallicGlossMap ("Metallic", 2D) = "white" { }
 _BumpScale ("Scale", Float) = 1
 _BumpMap ("Normal Map", 2D) = "bump" { }
 _Parallax ("Height Scale", Range(0.005,0.08)) = 0.02
 _ParallaxMap ("Height Map", 2D) = "black" { }
 _OcclusionStrength ("Strength", Range(0,1)) = 1
 _OcclusionMap ("Occlusion", 2D) = "white" { }
 _EmissionColor ("Color", Color) = (0,0,0,1)
 _EmissionMap ("Emission", 2D) = "white" { }
 _DetailMask ("Detail Mask", 2D) = "white" { }
 _DetailAlbedoMap ("Detail Albedo x2", 2D) = "grey" { }
 _DetailNormalMapScale ("Scale", Float) = 1
 _DetailNormalMap ("Normal Map", 2D) = "bump" { }
[Enum(UV0,0,UV1,1)]  _UVSec ("UV Set for secondary textures", Float) = 0
[HideInInspector]  _Mode ("__mode", Float) = 0
[HideInInspector]  _SrcBlend ("__src", Float) = 1
[HideInInspector]  _DstBlend ("__dst", Float) = 0
[HideInInspector]  _ZWrite ("__zw", Float) = 1
}
SubShader { 
 LOD 300
 Tags { "RenderType"="Opaque" "PerformanceChecks"="False" }
 Pass {
  Name "FORWARD"
  Tags { "LIGHTMODE"="ForwardBase" "SHADOWSUPPORT"="true" "RenderType"="Opaque" "PerformanceChecks"="False" }
  ZWrite [_ZWrite]
  Blend [_SrcBlend] [_DstBlend]
  GpuProgramID 39223
Program "vp" {
}
Program "fp" {
}
 }
 Pass {
  Name "FORWARD_DELTA"
  Tags { "LIGHTMODE"="ForwardAdd" "SHADOWSUPPORT"="true" "RenderType"="Opaque" "PerformanceChecks"="False" }
  ZWrite Off
  Blend [_SrcBlend] One
  GpuProgramID 129312
Program "vp" {
}
Program "fp" {
}
 }
 Pass {
  Name "SHADOWCASTER"
  Tags { "LIGHTMODE"="SHADOWCASTER" "SHADOWSUPPORT"="true" "RenderType"="Opaque" "PerformanceChecks"="False" }
  GpuProgramID 147192
Program "vp" {
}
Program "fp" {
}
 }
 Pass {
  Name "DEFERRED"
  Tags { "LIGHTMODE"="Deferred" "RenderType"="Opaque" "PerformanceChecks"="False" }
  GpuProgramID 219902
Program "vp" {
}
Program "fp" {
}
 }
}
SubShader { 
 LOD 150
 Tags { "RenderType"="Opaque" "PerformanceChecks"="False" }
 Pass {
  Name "FORWARD"
  Tags { "LIGHTMODE"="ForwardBase" "SHADOWSUPPORT"="true" "RenderType"="Opaque" "PerformanceChecks"="False" }
  ZWrite [_ZWrite]
  Blend [_SrcBlend] [_DstBlend]
  GpuProgramID 358221
Program "vp" {
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  highp vec3 tmpvar_8;
  highp vec4 tmpvar_9;
  tmpvar_9 = (_Object2World * _glesVertex);
  tmpvar_8 = tmpvar_9.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_10;
  texcoord_10.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_11;
  if ((_UVSec == 0.0)) {
    tmpvar_11 = tmpvar_2;
  } else {
    tmpvar_11 = tmpvar_3;
  };
  texcoord_10.zw = ((tmpvar_11 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_12;
  highp vec3 tmpvar_13;
  tmpvar_13 = normalize((tmpvar_9.xyz - _WorldSpaceCameraPos));
  tmpvar_12 = tmpvar_13;
  highp vec3 norm_14;
  norm_14 = tmpvar_1;
  highp vec4 v_15;
  v_15.x = _World2Object[0].x;
  v_15.y = _World2Object[1].x;
  v_15.z = _World2Object[2].x;
  v_15.w = _World2Object[3].x;
  highp vec4 v_16;
  v_16.x = _World2Object[0].y;
  v_16.y = _World2Object[1].y;
  v_16.z = _World2Object[2].y;
  v_16.w = _World2Object[3].y;
  highp vec4 v_17;
  v_17.x = _World2Object[0].z;
  v_17.y = _World2Object[1].z;
  v_17.z = _World2Object[2].z;
  v_17.w = _World2Object[3].z;
  highp vec3 tmpvar_18;
  tmpvar_18 = normalize(((
    (v_15.xyz * norm_14.x)
   + 
    (v_16.xyz * norm_14.y)
  ) + (v_17.xyz * norm_14.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_18;
  mediump vec3 normalWorld_19;
  normalWorld_19 = tmpvar_18;
  mediump vec4 ambientOrLightmapUV_20;
  ambientOrLightmapUV_20.w = 0.0;
  mediump vec4 tmpvar_21;
  tmpvar_21.w = 1.0;
  tmpvar_21.xyz = normalWorld_19;
  mediump vec3 res_22;
  mediump vec3 x_23;
  x_23.x = dot (unity_SHAr, tmpvar_21);
  x_23.y = dot (unity_SHAg, tmpvar_21);
  x_23.z = dot (unity_SHAb, tmpvar_21);
  mediump vec3 x1_24;
  mediump vec4 tmpvar_25;
  tmpvar_25 = (normalWorld_19.xyzz * normalWorld_19.yzzx);
  x1_24.x = dot (unity_SHBr, tmpvar_25);
  x1_24.y = dot (unity_SHBg, tmpvar_25);
  x1_24.z = dot (unity_SHBb, tmpvar_25);
  res_22 = (x_23 + (x1_24 + (unity_SHC.xyz * 
    ((normalWorld_19.x * normalWorld_19.x) - (normalWorld_19.y * normalWorld_19.y))
  )));
  res_22 = max (((1.055 * 
    pow (max (res_22, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_20.xyz = max (vec3(0.0, 0.0, 0.0), res_22);
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_10;
  xlv_TEXCOORD1 = tmpvar_12;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_20;
  xlv_TEXCOORD8 = tmpvar_8;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump vec3 i_posWorld_2;
  i_posWorld_2 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_3;
  tmpvar_3 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_4;
  tmpvar_4 = (_Color.xyz * tmpvar_3.xyz);
  mediump vec3 tmpvar_5;
  mediump vec3 tmpvar_6;
  tmpvar_6 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_4, vec3(_Metallic));
  mediump float tmpvar_7;
  tmpvar_7 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_5 = (tmpvar_4 * tmpvar_7);
  mediump vec3 tmpvar_8;
  tmpvar_8 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_9;
  tmpvar_9 = _LightColor0.xyz;
  mediump float tmpvar_10;
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_10 = tmpvar_11.y;
  highp vec3 tmpvar_12;
  highp vec4 tmpvar_13;
  highp vec4 tmpvar_14;
  tmpvar_12 = i_posWorld_2;
  tmpvar_13 = unity_SpecCube0_HDR;
  tmpvar_14 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_15;
  mediump vec3 tmpvar_16;
  tmpvar_15 = (xlv_TEXCOORD5.xyz * tmpvar_10);
  tmpvar_16 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_17;
  mediump vec3 tmpvar_18;
  tmpvar_17 = (1.0 - _Glossiness);
  tmpvar_18 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_8, xlv_TEXCOORD1)
   * tmpvar_8)));
  mediump vec3 originalReflUVW_19;
  mediump vec3 specular_20;
  originalReflUVW_19 = tmpvar_18;
  mediump vec3 worldRefl_21;
  worldRefl_21 = tmpvar_18;
  highp vec3 worldPos_22;
  worldPos_22 = tmpvar_12;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_23;
    mediump vec3 rbmax_24;
    mediump vec3 tmpvar_25;
    tmpvar_25 = normalize(tmpvar_18);
    highp vec3 tmpvar_26;
    tmpvar_26 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_12) / tmpvar_25);
    rbmax_24 = tmpvar_26;
    highp vec3 tmpvar_27;
    tmpvar_27 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_12) / tmpvar_25);
    rbmin_23 = tmpvar_27;
    bvec3 tmpvar_28;
    tmpvar_28 = greaterThan (tmpvar_25, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_29;
    if (tmpvar_28.x) {
      tmpvar_29 = rbmax_24.x;
    } else {
      tmpvar_29 = rbmin_23.x;
    };
    mediump float tmpvar_30;
    if (tmpvar_28.y) {
      tmpvar_30 = rbmax_24.y;
    } else {
      tmpvar_30 = rbmin_23.y;
    };
    mediump float tmpvar_31;
    if (tmpvar_28.z) {
      tmpvar_31 = rbmax_24.z;
    } else {
      tmpvar_31 = rbmin_23.z;
    };
    worldPos_22 = (tmpvar_12 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_21 = (worldPos_22 + (tmpvar_25 * min (
      min (tmpvar_29, tmpvar_30)
    , tmpvar_31)));
  };
  tmpvar_18 = worldRefl_21;
  mediump vec4 hdr_32;
  hdr_32 = tmpvar_13;
  mediump vec4 tmpvar_33;
  tmpvar_33.xyz = worldRefl_21;
  tmpvar_33.w = ((tmpvar_17 * (1.7 - 
    (0.7 * tmpvar_17)
  )) * 6.0);
  lowp vec4 tmpvar_34;
  tmpvar_34 = textureCube (unity_SpecCube0, worldRefl_21, tmpvar_33.w);
  mediump vec4 tmpvar_35;
  tmpvar_35 = tmpvar_34;
  mediump vec3 tmpvar_36;
  tmpvar_36 = ((tmpvar_35.w * hdr_32.x) * tmpvar_35.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_37;
    worldRefl_37 = originalReflUVW_19;
    highp vec3 worldPos_38;
    worldPos_38 = tmpvar_12;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_39;
      mediump vec3 rbmax_40;
      mediump vec3 tmpvar_41;
      tmpvar_41 = normalize(originalReflUVW_19);
      highp vec3 tmpvar_42;
      tmpvar_42 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_12) / tmpvar_41);
      rbmax_40 = tmpvar_42;
      highp vec3 tmpvar_43;
      tmpvar_43 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_12) / tmpvar_41);
      rbmin_39 = tmpvar_43;
      bvec3 tmpvar_44;
      tmpvar_44 = greaterThan (tmpvar_41, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_45;
      if (tmpvar_44.x) {
        tmpvar_45 = rbmax_40.x;
      } else {
        tmpvar_45 = rbmin_39.x;
      };
      mediump float tmpvar_46;
      if (tmpvar_44.y) {
        tmpvar_46 = rbmax_40.y;
      } else {
        tmpvar_46 = rbmin_39.y;
      };
      mediump float tmpvar_47;
      if (tmpvar_44.z) {
        tmpvar_47 = rbmax_40.z;
      } else {
        tmpvar_47 = rbmin_39.z;
      };
      worldPos_38 = (tmpvar_12 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_37 = (worldPos_38 + (tmpvar_41 * min (
        min (tmpvar_45, tmpvar_46)
      , tmpvar_47)));
    };
    tmpvar_18 = worldRefl_37;
    mediump vec4 hdr_48;
    hdr_48 = tmpvar_14;
    mediump vec4 tmpvar_49;
    tmpvar_49.xyz = worldRefl_37;
    tmpvar_49.w = ((tmpvar_17 * (1.7 - 
      (0.7 * tmpvar_17)
    )) * 6.0);
    lowp vec4 tmpvar_50;
    tmpvar_50 = textureCube (unity_SpecCube1, worldRefl_37, tmpvar_49.w);
    mediump vec4 tmpvar_51;
    tmpvar_51 = tmpvar_50;
    mediump vec3 tmpvar_52;
    tmpvar_52 = ((tmpvar_51.w * hdr_48.x) * tmpvar_51.xyz);
    highp vec3 tmpvar_53;
    tmpvar_53 = mix (tmpvar_52, tmpvar_36, unity_SpecCube0_BoxMin.www);
    specular_20 = tmpvar_53;
  } else {
    specular_20 = tmpvar_36;
  };
  tmpvar_16 = (specular_20 * tmpvar_10);
  mediump vec3 viewDir_54;
  viewDir_54 = -(xlv_TEXCOORD1);
  mediump float specularTerm_55;
  mediump float tmpvar_56;
  tmpvar_56 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_57;
  mediump vec3 inVec_58;
  inVec_58 = (_WorldSpaceLightPos0.xyz + viewDir_54);
  tmpvar_57 = (inVec_58 * inversesqrt(max (0.001, 
    dot (inVec_58, inVec_58)
  )));
  mediump float tmpvar_59;
  tmpvar_59 = dot (tmpvar_8, viewDir_54);
  mediump vec3 tmpvar_60;
  if ((tmpvar_59 < 0.0)) {
    tmpvar_60 = (tmpvar_8 + (viewDir_54 * (
      -(tmpvar_59)
     + 1e-05)));
  } else {
    tmpvar_60 = tmpvar_8;
  };
  mediump float tmpvar_61;
  tmpvar_61 = clamp (dot (tmpvar_60, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_62;
  tmpvar_62 = clamp (dot (tmpvar_60, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_63;
  tmpvar_63 = clamp (dot (tmpvar_60, viewDir_54), 0.0, 1.0);
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_66;
  tmpvar_66 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_67;
  tmpvar_67 = (tmpvar_66 * tmpvar_66);
  mediump float tmpvar_68;
  tmpvar_68 = (((tmpvar_62 * tmpvar_62) * (tmpvar_67 - 1.0)) + 1.0);
  mediump float x_69;
  x_69 = (1.0 - tmpvar_61);
  mediump float x_70;
  x_70 = (1.0 - tmpvar_63);
  mediump float tmpvar_71;
  tmpvar_71 = (0.5 + ((2.0 * tmpvar_64) * (tmpvar_64 * tmpvar_56)));
  specularTerm_55 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_61 * ((tmpvar_63 * (1.0 - tmpvar_65)) + tmpvar_65))
     + 
      (tmpvar_63 * ((tmpvar_61 * (1.0 - tmpvar_65)) + tmpvar_65))
    ) + 1e-05)) * (tmpvar_67 / ((
      (3.141593 * tmpvar_68)
     * tmpvar_68) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_72;
  tmpvar_72 = max (0.0, (specularTerm_55 * tmpvar_61));
  specularTerm_55 = tmpvar_72;
  mediump float x_73;
  x_73 = (1.0 - tmpvar_64);
  mediump float x_74;
  x_74 = (1.0 - tmpvar_63);
  mediump vec3 tmpvar_75;
  tmpvar_75 = (((tmpvar_5 * 
    (tmpvar_15 + (tmpvar_9 * ((
      (1.0 + ((tmpvar_71 - 1.0) * ((x_69 * x_69) * (
        (x_69 * x_69)
       * x_69))))
     * 
      (1.0 + ((tmpvar_71 - 1.0) * ((x_70 * x_70) * (
        (x_70 * x_70)
       * x_70))))
    ) * tmpvar_61)))
  ) + (
    (tmpvar_72 * tmpvar_9)
   * 
    (tmpvar_6 + ((1.0 - tmpvar_6) * ((x_73 * x_73) * (
      (x_73 * x_73)
     * x_73))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_56) * (tmpvar_56 * tmpvar_56)))
   * tmpvar_16) * mix (tmpvar_6, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_7)), 0.0, 1.0)
  ), vec3(
    ((x_74 * x_74) * ((x_74 * x_74) * x_74))
  ))));
  mediump vec4 tmpvar_76;
  tmpvar_76.w = 1.0;
  tmpvar_76.xyz = tmpvar_75;
  c_1.w = tmpvar_76.w;
  c_1.xyz = tmpvar_75;
  c_1.xyz = c_1.xyz;
  mediump vec4 xlat_varoutput_77;
  xlat_varoutput_77.xyz = c_1.xyz;
  xlat_varoutput_77.w = 1.0;
  gl_FragData[0] = xlat_varoutput_77;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  highp vec3 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_9 = tmpvar_10.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_19;
  highp vec4 o_20;
  highp vec4 tmpvar_21;
  tmpvar_21 = (tmpvar_4 * 0.5);
  highp vec2 tmpvar_22;
  tmpvar_22.x = tmpvar_21.x;
  tmpvar_22.y = (tmpvar_21.y * _ProjectionParams.x);
  o_20.xy = (tmpvar_22 + tmpvar_21.w);
  o_20.zw = tmpvar_4.zw;
  tmpvar_8 = o_20;
  mediump vec3 normalWorld_23;
  normalWorld_23 = tmpvar_19;
  mediump vec4 ambientOrLightmapUV_24;
  ambientOrLightmapUV_24.w = 0.0;
  mediump vec4 tmpvar_25;
  tmpvar_25.w = 1.0;
  tmpvar_25.xyz = normalWorld_23;
  mediump vec3 res_26;
  mediump vec3 x_27;
  x_27.x = dot (unity_SHAr, tmpvar_25);
  x_27.y = dot (unity_SHAg, tmpvar_25);
  x_27.z = dot (unity_SHAb, tmpvar_25);
  mediump vec3 x1_28;
  mediump vec4 tmpvar_29;
  tmpvar_29 = (normalWorld_23.xyzz * normalWorld_23.yzzx);
  x1_28.x = dot (unity_SHBr, tmpvar_29);
  x1_28.y = dot (unity_SHBg, tmpvar_29);
  x1_28.z = dot (unity_SHBb, tmpvar_29);
  res_26 = (x_27 + (x1_28 + (unity_SHC.xyz * 
    ((normalWorld_23.x * normalWorld_23.x) - (normalWorld_23.y * normalWorld_23.y))
  )));
  res_26 = max (((1.055 * 
    pow (max (res_26, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_24.xyz = max (vec3(0.0, 0.0, 0.0), res_26);
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_24;
  xlv_TEXCOORD6 = tmpvar_8;
  xlv_TEXCOORD8 = tmpvar_9;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
uniform sampler2D _ShadowMapTexture;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump float atten_2;
  mediump vec3 i_posWorld_3;
  i_posWorld_3 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_4;
  tmpvar_4 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_5;
  tmpvar_5 = (_Color.xyz * tmpvar_4.xyz);
  mediump vec3 tmpvar_6;
  mediump vec3 tmpvar_7;
  tmpvar_7 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_5, vec3(_Metallic));
  mediump float tmpvar_8;
  tmpvar_8 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_6 = (tmpvar_5 * tmpvar_8);
  mediump vec3 tmpvar_9;
  tmpvar_9 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  lowp float tmpvar_11;
  tmpvar_11 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD6).x;
  atten_2 = tmpvar_11;
  mediump float tmpvar_12;
  lowp vec4 tmpvar_13;
  tmpvar_13 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_12 = tmpvar_13.y;
  highp vec3 tmpvar_14;
  highp vec4 tmpvar_15;
  highp vec4 tmpvar_16;
  tmpvar_14 = i_posWorld_3;
  tmpvar_15 = unity_SpecCube0_HDR;
  tmpvar_16 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_17;
  mediump vec3 tmpvar_18;
  mediump vec3 tmpvar_19;
  tmpvar_17 = (tmpvar_10 * atten_2);
  tmpvar_18 = (xlv_TEXCOORD5.xyz * tmpvar_12);
  tmpvar_19 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_20;
  mediump vec3 tmpvar_21;
  tmpvar_20 = (1.0 - _Glossiness);
  tmpvar_21 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_9, xlv_TEXCOORD1)
   * tmpvar_9)));
  mediump vec3 originalReflUVW_22;
  mediump vec3 specular_23;
  originalReflUVW_22 = tmpvar_21;
  mediump vec3 worldRefl_24;
  worldRefl_24 = tmpvar_21;
  highp vec3 worldPos_25;
  worldPos_25 = tmpvar_14;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_26;
    mediump vec3 rbmax_27;
    mediump vec3 tmpvar_28;
    tmpvar_28 = normalize(tmpvar_21);
    highp vec3 tmpvar_29;
    tmpvar_29 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_14) / tmpvar_28);
    rbmax_27 = tmpvar_29;
    highp vec3 tmpvar_30;
    tmpvar_30 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_14) / tmpvar_28);
    rbmin_26 = tmpvar_30;
    bvec3 tmpvar_31;
    tmpvar_31 = greaterThan (tmpvar_28, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_32;
    if (tmpvar_31.x) {
      tmpvar_32 = rbmax_27.x;
    } else {
      tmpvar_32 = rbmin_26.x;
    };
    mediump float tmpvar_33;
    if (tmpvar_31.y) {
      tmpvar_33 = rbmax_27.y;
    } else {
      tmpvar_33 = rbmin_26.y;
    };
    mediump float tmpvar_34;
    if (tmpvar_31.z) {
      tmpvar_34 = rbmax_27.z;
    } else {
      tmpvar_34 = rbmin_26.z;
    };
    worldPos_25 = (tmpvar_14 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_24 = (worldPos_25 + (tmpvar_28 * min (
      min (tmpvar_32, tmpvar_33)
    , tmpvar_34)));
  };
  tmpvar_21 = worldRefl_24;
  mediump vec4 hdr_35;
  hdr_35 = tmpvar_15;
  mediump vec4 tmpvar_36;
  tmpvar_36.xyz = worldRefl_24;
  tmpvar_36.w = ((tmpvar_20 * (1.7 - 
    (0.7 * tmpvar_20)
  )) * 6.0);
  lowp vec4 tmpvar_37;
  tmpvar_37 = textureCube (unity_SpecCube0, worldRefl_24, tmpvar_36.w);
  mediump vec4 tmpvar_38;
  tmpvar_38 = tmpvar_37;
  mediump vec3 tmpvar_39;
  tmpvar_39 = ((tmpvar_38.w * hdr_35.x) * tmpvar_38.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_40;
    worldRefl_40 = originalReflUVW_22;
    highp vec3 worldPos_41;
    worldPos_41 = tmpvar_14;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_42;
      mediump vec3 rbmax_43;
      mediump vec3 tmpvar_44;
      tmpvar_44 = normalize(originalReflUVW_22);
      highp vec3 tmpvar_45;
      tmpvar_45 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_14) / tmpvar_44);
      rbmax_43 = tmpvar_45;
      highp vec3 tmpvar_46;
      tmpvar_46 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_14) / tmpvar_44);
      rbmin_42 = tmpvar_46;
      bvec3 tmpvar_47;
      tmpvar_47 = greaterThan (tmpvar_44, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_48;
      if (tmpvar_47.x) {
        tmpvar_48 = rbmax_43.x;
      } else {
        tmpvar_48 = rbmin_42.x;
      };
      mediump float tmpvar_49;
      if (tmpvar_47.y) {
        tmpvar_49 = rbmax_43.y;
      } else {
        tmpvar_49 = rbmin_42.y;
      };
      mediump float tmpvar_50;
      if (tmpvar_47.z) {
        tmpvar_50 = rbmax_43.z;
      } else {
        tmpvar_50 = rbmin_42.z;
      };
      worldPos_41 = (tmpvar_14 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_40 = (worldPos_41 + (tmpvar_44 * min (
        min (tmpvar_48, tmpvar_49)
      , tmpvar_50)));
    };
    tmpvar_21 = worldRefl_40;
    mediump vec4 hdr_51;
    hdr_51 = tmpvar_16;
    mediump vec4 tmpvar_52;
    tmpvar_52.xyz = worldRefl_40;
    tmpvar_52.w = ((tmpvar_20 * (1.7 - 
      (0.7 * tmpvar_20)
    )) * 6.0);
    lowp vec4 tmpvar_53;
    tmpvar_53 = textureCube (unity_SpecCube1, worldRefl_40, tmpvar_52.w);
    mediump vec4 tmpvar_54;
    tmpvar_54 = tmpvar_53;
    mediump vec3 tmpvar_55;
    tmpvar_55 = ((tmpvar_54.w * hdr_51.x) * tmpvar_54.xyz);
    highp vec3 tmpvar_56;
    tmpvar_56 = mix (tmpvar_55, tmpvar_39, unity_SpecCube0_BoxMin.www);
    specular_23 = tmpvar_56;
  } else {
    specular_23 = tmpvar_39;
  };
  tmpvar_19 = (specular_23 * tmpvar_12);
  mediump vec3 viewDir_57;
  viewDir_57 = -(xlv_TEXCOORD1);
  mediump float specularTerm_58;
  mediump float tmpvar_59;
  tmpvar_59 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_60;
  mediump vec3 inVec_61;
  inVec_61 = (_WorldSpaceLightPos0.xyz + viewDir_57);
  tmpvar_60 = (inVec_61 * inversesqrt(max (0.001, 
    dot (inVec_61, inVec_61)
  )));
  mediump float tmpvar_62;
  tmpvar_62 = dot (tmpvar_9, viewDir_57);
  mediump vec3 tmpvar_63;
  if ((tmpvar_62 < 0.0)) {
    tmpvar_63 = (tmpvar_9 + (viewDir_57 * (
      -(tmpvar_62)
     + 1e-05)));
  } else {
    tmpvar_63 = tmpvar_9;
  };
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (tmpvar_63, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = clamp (dot (tmpvar_63, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_66;
  tmpvar_66 = clamp (dot (tmpvar_63, viewDir_57), 0.0, 1.0);
  mediump float tmpvar_67;
  tmpvar_67 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_68;
  tmpvar_68 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_69;
  tmpvar_69 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_70;
  tmpvar_70 = (tmpvar_69 * tmpvar_69);
  mediump float tmpvar_71;
  tmpvar_71 = (((tmpvar_65 * tmpvar_65) * (tmpvar_70 - 1.0)) + 1.0);
  mediump float x_72;
  x_72 = (1.0 - tmpvar_64);
  mediump float x_73;
  x_73 = (1.0 - tmpvar_66);
  mediump float tmpvar_74;
  tmpvar_74 = (0.5 + ((2.0 * tmpvar_67) * (tmpvar_67 * tmpvar_59)));
  specularTerm_58 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_64 * ((tmpvar_66 * (1.0 - tmpvar_68)) + tmpvar_68))
     + 
      (tmpvar_66 * ((tmpvar_64 * (1.0 - tmpvar_68)) + tmpvar_68))
    ) + 1e-05)) * (tmpvar_70 / ((
      (3.141593 * tmpvar_71)
     * tmpvar_71) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_75;
  tmpvar_75 = max (0.0, (specularTerm_58 * tmpvar_64));
  specularTerm_58 = tmpvar_75;
  mediump float x_76;
  x_76 = (1.0 - tmpvar_67);
  mediump float x_77;
  x_77 = (1.0 - tmpvar_66);
  mediump vec3 tmpvar_78;
  tmpvar_78 = (((tmpvar_6 * 
    (tmpvar_18 + (tmpvar_17 * ((
      (1.0 + ((tmpvar_74 - 1.0) * ((x_72 * x_72) * (
        (x_72 * x_72)
       * x_72))))
     * 
      (1.0 + ((tmpvar_74 - 1.0) * ((x_73 * x_73) * (
        (x_73 * x_73)
       * x_73))))
    ) * tmpvar_64)))
  ) + (
    (tmpvar_75 * tmpvar_17)
   * 
    (tmpvar_7 + ((1.0 - tmpvar_7) * ((x_76 * x_76) * (
      (x_76 * x_76)
     * x_76))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_59) * (tmpvar_59 * tmpvar_59)))
   * tmpvar_19) * mix (tmpvar_7, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_8)), 0.0, 1.0)
  ), vec3(
    ((x_77 * x_77) * ((x_77 * x_77) * x_77))
  ))));
  mediump vec4 tmpvar_79;
  tmpvar_79.w = 1.0;
  tmpvar_79.xyz = tmpvar_78;
  c_1.w = tmpvar_79.w;
  c_1.xyz = tmpvar_78;
  c_1.xyz = c_1.xyz;
  mediump vec4 xlat_varoutput_80;
  xlat_varoutput_80.xyz = c_1.xyz;
  xlat_varoutput_80.w = 1.0;
  gl_FragData[0] = xlat_varoutput_80;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "VERTEXLIGHT_ON" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 unity_4LightPosX0;
uniform highp vec4 unity_4LightPosY0;
uniform highp vec4 unity_4LightPosZ0;
uniform mediump vec4 unity_4LightAtten0;
uniform mediump vec4 unity_LightColor[8];
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  highp vec3 tmpvar_8;
  highp vec4 tmpvar_9;
  tmpvar_9 = (_Object2World * _glesVertex);
  tmpvar_8 = tmpvar_9.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_10;
  texcoord_10.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_11;
  if ((_UVSec == 0.0)) {
    tmpvar_11 = tmpvar_2;
  } else {
    tmpvar_11 = tmpvar_3;
  };
  texcoord_10.zw = ((tmpvar_11 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_12;
  highp vec3 tmpvar_13;
  tmpvar_13 = normalize((tmpvar_9.xyz - _WorldSpaceCameraPos));
  tmpvar_12 = tmpvar_13;
  highp vec3 norm_14;
  norm_14 = tmpvar_1;
  highp vec4 v_15;
  v_15.x = _World2Object[0].x;
  v_15.y = _World2Object[1].x;
  v_15.z = _World2Object[2].x;
  v_15.w = _World2Object[3].x;
  highp vec4 v_16;
  v_16.x = _World2Object[0].y;
  v_16.y = _World2Object[1].y;
  v_16.z = _World2Object[2].y;
  v_16.w = _World2Object[3].y;
  highp vec4 v_17;
  v_17.x = _World2Object[0].z;
  v_17.y = _World2Object[1].z;
  v_17.z = _World2Object[2].z;
  v_17.w = _World2Object[3].z;
  highp vec3 tmpvar_18;
  tmpvar_18 = normalize(((
    (v_15.xyz * norm_14.x)
   + 
    (v_16.xyz * norm_14.y)
  ) + (v_17.xyz * norm_14.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_18;
  mediump vec3 normalWorld_19;
  normalWorld_19 = tmpvar_18;
  mediump vec4 ambientOrLightmapUV_20;
  ambientOrLightmapUV_20.w = 0.0;
  highp vec3 lightColor0_21;
  lightColor0_21 = unity_LightColor[0].xyz;
  highp vec3 lightColor1_22;
  lightColor1_22 = unity_LightColor[1].xyz;
  highp vec3 lightColor2_23;
  lightColor2_23 = unity_LightColor[2].xyz;
  highp vec3 lightColor3_24;
  lightColor3_24 = unity_LightColor[3].xyz;
  highp vec4 lightAttenSq_25;
  lightAttenSq_25 = unity_4LightAtten0;
  highp vec3 normal_26;
  normal_26 = normalWorld_19;
  highp vec3 col_27;
  highp vec4 ndotl_28;
  highp vec4 lengthSq_29;
  highp vec4 tmpvar_30;
  tmpvar_30 = (unity_4LightPosX0 - tmpvar_9.x);
  highp vec4 tmpvar_31;
  tmpvar_31 = (unity_4LightPosY0 - tmpvar_9.y);
  highp vec4 tmpvar_32;
  tmpvar_32 = (unity_4LightPosZ0 - tmpvar_9.z);
  lengthSq_29 = (tmpvar_30 * tmpvar_30);
  lengthSq_29 = (lengthSq_29 + (tmpvar_31 * tmpvar_31));
  lengthSq_29 = (lengthSq_29 + (tmpvar_32 * tmpvar_32));
  ndotl_28 = (tmpvar_30 * normal_26.x);
  ndotl_28 = (ndotl_28 + (tmpvar_31 * normal_26.y));
  ndotl_28 = (ndotl_28 + (tmpvar_32 * normal_26.z));
  highp vec4 tmpvar_33;
  tmpvar_33 = max (vec4(0.0, 0.0, 0.0, 0.0), (ndotl_28 * inversesqrt(lengthSq_29)));
  ndotl_28 = tmpvar_33;
  highp vec4 tmpvar_34;
  tmpvar_34 = (tmpvar_33 * (1.0/((1.0 + 
    (lengthSq_29 * lightAttenSq_25)
  ))));
  col_27 = (lightColor0_21 * tmpvar_34.x);
  col_27 = (col_27 + (lightColor1_22 * tmpvar_34.y));
  col_27 = (col_27 + (lightColor2_23 * tmpvar_34.z));
  col_27 = (col_27 + (lightColor3_24 * tmpvar_34.w));
  ambientOrLightmapUV_20.xyz = col_27;
  mediump vec4 tmpvar_35;
  tmpvar_35.w = 1.0;
  tmpvar_35.xyz = normalWorld_19;
  mediump vec3 res_36;
  mediump vec3 x_37;
  x_37.x = dot (unity_SHAr, tmpvar_35);
  x_37.y = dot (unity_SHAg, tmpvar_35);
  x_37.z = dot (unity_SHAb, tmpvar_35);
  mediump vec3 x1_38;
  mediump vec4 tmpvar_39;
  tmpvar_39 = (normalWorld_19.xyzz * normalWorld_19.yzzx);
  x1_38.x = dot (unity_SHBr, tmpvar_39);
  x1_38.y = dot (unity_SHBg, tmpvar_39);
  x1_38.z = dot (unity_SHBb, tmpvar_39);
  res_36 = (x_37 + (x1_38 + (unity_SHC.xyz * 
    ((normalWorld_19.x * normalWorld_19.x) - (normalWorld_19.y * normalWorld_19.y))
  )));
  res_36 = max (((1.055 * 
    pow (max (res_36, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_20.xyz = (ambientOrLightmapUV_20.xyz + max (vec3(0.0, 0.0, 0.0), res_36));
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_10;
  xlv_TEXCOORD1 = tmpvar_12;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_20;
  xlv_TEXCOORD8 = tmpvar_8;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump vec3 i_posWorld_2;
  i_posWorld_2 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_3;
  tmpvar_3 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_4;
  tmpvar_4 = (_Color.xyz * tmpvar_3.xyz);
  mediump vec3 tmpvar_5;
  mediump vec3 tmpvar_6;
  tmpvar_6 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_4, vec3(_Metallic));
  mediump float tmpvar_7;
  tmpvar_7 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_5 = (tmpvar_4 * tmpvar_7);
  mediump vec3 tmpvar_8;
  tmpvar_8 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_9;
  tmpvar_9 = _LightColor0.xyz;
  mediump float tmpvar_10;
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_10 = tmpvar_11.y;
  highp vec3 tmpvar_12;
  highp vec4 tmpvar_13;
  highp vec4 tmpvar_14;
  tmpvar_12 = i_posWorld_2;
  tmpvar_13 = unity_SpecCube0_HDR;
  tmpvar_14 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_15;
  mediump vec3 tmpvar_16;
  tmpvar_15 = (xlv_TEXCOORD5.xyz * tmpvar_10);
  tmpvar_16 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_17;
  mediump vec3 tmpvar_18;
  tmpvar_17 = (1.0 - _Glossiness);
  tmpvar_18 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_8, xlv_TEXCOORD1)
   * tmpvar_8)));
  mediump vec3 originalReflUVW_19;
  mediump vec3 specular_20;
  originalReflUVW_19 = tmpvar_18;
  mediump vec3 worldRefl_21;
  worldRefl_21 = tmpvar_18;
  highp vec3 worldPos_22;
  worldPos_22 = tmpvar_12;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_23;
    mediump vec3 rbmax_24;
    mediump vec3 tmpvar_25;
    tmpvar_25 = normalize(tmpvar_18);
    highp vec3 tmpvar_26;
    tmpvar_26 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_12) / tmpvar_25);
    rbmax_24 = tmpvar_26;
    highp vec3 tmpvar_27;
    tmpvar_27 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_12) / tmpvar_25);
    rbmin_23 = tmpvar_27;
    bvec3 tmpvar_28;
    tmpvar_28 = greaterThan (tmpvar_25, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_29;
    if (tmpvar_28.x) {
      tmpvar_29 = rbmax_24.x;
    } else {
      tmpvar_29 = rbmin_23.x;
    };
    mediump float tmpvar_30;
    if (tmpvar_28.y) {
      tmpvar_30 = rbmax_24.y;
    } else {
      tmpvar_30 = rbmin_23.y;
    };
    mediump float tmpvar_31;
    if (tmpvar_28.z) {
      tmpvar_31 = rbmax_24.z;
    } else {
      tmpvar_31 = rbmin_23.z;
    };
    worldPos_22 = (tmpvar_12 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_21 = (worldPos_22 + (tmpvar_25 * min (
      min (tmpvar_29, tmpvar_30)
    , tmpvar_31)));
  };
  tmpvar_18 = worldRefl_21;
  mediump vec4 hdr_32;
  hdr_32 = tmpvar_13;
  mediump vec4 tmpvar_33;
  tmpvar_33.xyz = worldRefl_21;
  tmpvar_33.w = ((tmpvar_17 * (1.7 - 
    (0.7 * tmpvar_17)
  )) * 6.0);
  lowp vec4 tmpvar_34;
  tmpvar_34 = textureCube (unity_SpecCube0, worldRefl_21, tmpvar_33.w);
  mediump vec4 tmpvar_35;
  tmpvar_35 = tmpvar_34;
  mediump vec3 tmpvar_36;
  tmpvar_36 = ((tmpvar_35.w * hdr_32.x) * tmpvar_35.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_37;
    worldRefl_37 = originalReflUVW_19;
    highp vec3 worldPos_38;
    worldPos_38 = tmpvar_12;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_39;
      mediump vec3 rbmax_40;
      mediump vec3 tmpvar_41;
      tmpvar_41 = normalize(originalReflUVW_19);
      highp vec3 tmpvar_42;
      tmpvar_42 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_12) / tmpvar_41);
      rbmax_40 = tmpvar_42;
      highp vec3 tmpvar_43;
      tmpvar_43 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_12) / tmpvar_41);
      rbmin_39 = tmpvar_43;
      bvec3 tmpvar_44;
      tmpvar_44 = greaterThan (tmpvar_41, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_45;
      if (tmpvar_44.x) {
        tmpvar_45 = rbmax_40.x;
      } else {
        tmpvar_45 = rbmin_39.x;
      };
      mediump float tmpvar_46;
      if (tmpvar_44.y) {
        tmpvar_46 = rbmax_40.y;
      } else {
        tmpvar_46 = rbmin_39.y;
      };
      mediump float tmpvar_47;
      if (tmpvar_44.z) {
        tmpvar_47 = rbmax_40.z;
      } else {
        tmpvar_47 = rbmin_39.z;
      };
      worldPos_38 = (tmpvar_12 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_37 = (worldPos_38 + (tmpvar_41 * min (
        min (tmpvar_45, tmpvar_46)
      , tmpvar_47)));
    };
    tmpvar_18 = worldRefl_37;
    mediump vec4 hdr_48;
    hdr_48 = tmpvar_14;
    mediump vec4 tmpvar_49;
    tmpvar_49.xyz = worldRefl_37;
    tmpvar_49.w = ((tmpvar_17 * (1.7 - 
      (0.7 * tmpvar_17)
    )) * 6.0);
    lowp vec4 tmpvar_50;
    tmpvar_50 = textureCube (unity_SpecCube1, worldRefl_37, tmpvar_49.w);
    mediump vec4 tmpvar_51;
    tmpvar_51 = tmpvar_50;
    mediump vec3 tmpvar_52;
    tmpvar_52 = ((tmpvar_51.w * hdr_48.x) * tmpvar_51.xyz);
    highp vec3 tmpvar_53;
    tmpvar_53 = mix (tmpvar_52, tmpvar_36, unity_SpecCube0_BoxMin.www);
    specular_20 = tmpvar_53;
  } else {
    specular_20 = tmpvar_36;
  };
  tmpvar_16 = (specular_20 * tmpvar_10);
  mediump vec3 viewDir_54;
  viewDir_54 = -(xlv_TEXCOORD1);
  mediump float specularTerm_55;
  mediump float tmpvar_56;
  tmpvar_56 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_57;
  mediump vec3 inVec_58;
  inVec_58 = (_WorldSpaceLightPos0.xyz + viewDir_54);
  tmpvar_57 = (inVec_58 * inversesqrt(max (0.001, 
    dot (inVec_58, inVec_58)
  )));
  mediump float tmpvar_59;
  tmpvar_59 = dot (tmpvar_8, viewDir_54);
  mediump vec3 tmpvar_60;
  if ((tmpvar_59 < 0.0)) {
    tmpvar_60 = (tmpvar_8 + (viewDir_54 * (
      -(tmpvar_59)
     + 1e-05)));
  } else {
    tmpvar_60 = tmpvar_8;
  };
  mediump float tmpvar_61;
  tmpvar_61 = clamp (dot (tmpvar_60, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_62;
  tmpvar_62 = clamp (dot (tmpvar_60, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_63;
  tmpvar_63 = clamp (dot (tmpvar_60, viewDir_54), 0.0, 1.0);
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_66;
  tmpvar_66 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_67;
  tmpvar_67 = (tmpvar_66 * tmpvar_66);
  mediump float tmpvar_68;
  tmpvar_68 = (((tmpvar_62 * tmpvar_62) * (tmpvar_67 - 1.0)) + 1.0);
  mediump float x_69;
  x_69 = (1.0 - tmpvar_61);
  mediump float x_70;
  x_70 = (1.0 - tmpvar_63);
  mediump float tmpvar_71;
  tmpvar_71 = (0.5 + ((2.0 * tmpvar_64) * (tmpvar_64 * tmpvar_56)));
  specularTerm_55 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_61 * ((tmpvar_63 * (1.0 - tmpvar_65)) + tmpvar_65))
     + 
      (tmpvar_63 * ((tmpvar_61 * (1.0 - tmpvar_65)) + tmpvar_65))
    ) + 1e-05)) * (tmpvar_67 / ((
      (3.141593 * tmpvar_68)
     * tmpvar_68) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_72;
  tmpvar_72 = max (0.0, (specularTerm_55 * tmpvar_61));
  specularTerm_55 = tmpvar_72;
  mediump float x_73;
  x_73 = (1.0 - tmpvar_64);
  mediump float x_74;
  x_74 = (1.0 - tmpvar_63);
  mediump vec3 tmpvar_75;
  tmpvar_75 = (((tmpvar_5 * 
    (tmpvar_15 + (tmpvar_9 * ((
      (1.0 + ((tmpvar_71 - 1.0) * ((x_69 * x_69) * (
        (x_69 * x_69)
       * x_69))))
     * 
      (1.0 + ((tmpvar_71 - 1.0) * ((x_70 * x_70) * (
        (x_70 * x_70)
       * x_70))))
    ) * tmpvar_61)))
  ) + (
    (tmpvar_72 * tmpvar_9)
   * 
    (tmpvar_6 + ((1.0 - tmpvar_6) * ((x_73 * x_73) * (
      (x_73 * x_73)
     * x_73))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_56) * (tmpvar_56 * tmpvar_56)))
   * tmpvar_16) * mix (tmpvar_6, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_7)), 0.0, 1.0)
  ), vec3(
    ((x_74 * x_74) * ((x_74 * x_74) * x_74))
  ))));
  mediump vec4 tmpvar_76;
  tmpvar_76.w = 1.0;
  tmpvar_76.xyz = tmpvar_75;
  c_1.w = tmpvar_76.w;
  c_1.xyz = tmpvar_75;
  c_1.xyz = c_1.xyz;
  mediump vec4 xlat_varoutput_77;
  xlat_varoutput_77.xyz = c_1.xyz;
  xlat_varoutput_77.w = 1.0;
  gl_FragData[0] = xlat_varoutput_77;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "VERTEXLIGHT_ON" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform highp vec4 unity_4LightPosX0;
uniform highp vec4 unity_4LightPosY0;
uniform highp vec4 unity_4LightPosZ0;
uniform mediump vec4 unity_4LightAtten0;
uniform mediump vec4 unity_LightColor[8];
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  highp vec3 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_9 = tmpvar_10.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_19;
  highp vec4 o_20;
  highp vec4 tmpvar_21;
  tmpvar_21 = (tmpvar_4 * 0.5);
  highp vec2 tmpvar_22;
  tmpvar_22.x = tmpvar_21.x;
  tmpvar_22.y = (tmpvar_21.y * _ProjectionParams.x);
  o_20.xy = (tmpvar_22 + tmpvar_21.w);
  o_20.zw = tmpvar_4.zw;
  tmpvar_8 = o_20;
  mediump vec3 normalWorld_23;
  normalWorld_23 = tmpvar_19;
  mediump vec4 ambientOrLightmapUV_24;
  ambientOrLightmapUV_24.w = 0.0;
  highp vec3 lightColor0_25;
  lightColor0_25 = unity_LightColor[0].xyz;
  highp vec3 lightColor1_26;
  lightColor1_26 = unity_LightColor[1].xyz;
  highp vec3 lightColor2_27;
  lightColor2_27 = unity_LightColor[2].xyz;
  highp vec3 lightColor3_28;
  lightColor3_28 = unity_LightColor[3].xyz;
  highp vec4 lightAttenSq_29;
  lightAttenSq_29 = unity_4LightAtten0;
  highp vec3 normal_30;
  normal_30 = normalWorld_23;
  highp vec3 col_31;
  highp vec4 ndotl_32;
  highp vec4 lengthSq_33;
  highp vec4 tmpvar_34;
  tmpvar_34 = (unity_4LightPosX0 - tmpvar_10.x);
  highp vec4 tmpvar_35;
  tmpvar_35 = (unity_4LightPosY0 - tmpvar_10.y);
  highp vec4 tmpvar_36;
  tmpvar_36 = (unity_4LightPosZ0 - tmpvar_10.z);
  lengthSq_33 = (tmpvar_34 * tmpvar_34);
  lengthSq_33 = (lengthSq_33 + (tmpvar_35 * tmpvar_35));
  lengthSq_33 = (lengthSq_33 + (tmpvar_36 * tmpvar_36));
  ndotl_32 = (tmpvar_34 * normal_30.x);
  ndotl_32 = (ndotl_32 + (tmpvar_35 * normal_30.y));
  ndotl_32 = (ndotl_32 + (tmpvar_36 * normal_30.z));
  highp vec4 tmpvar_37;
  tmpvar_37 = max (vec4(0.0, 0.0, 0.0, 0.0), (ndotl_32 * inversesqrt(lengthSq_33)));
  ndotl_32 = tmpvar_37;
  highp vec4 tmpvar_38;
  tmpvar_38 = (tmpvar_37 * (1.0/((1.0 + 
    (lengthSq_33 * lightAttenSq_29)
  ))));
  col_31 = (lightColor0_25 * tmpvar_38.x);
  col_31 = (col_31 + (lightColor1_26 * tmpvar_38.y));
  col_31 = (col_31 + (lightColor2_27 * tmpvar_38.z));
  col_31 = (col_31 + (lightColor3_28 * tmpvar_38.w));
  ambientOrLightmapUV_24.xyz = col_31;
  mediump vec4 tmpvar_39;
  tmpvar_39.w = 1.0;
  tmpvar_39.xyz = normalWorld_23;
  mediump vec3 res_40;
  mediump vec3 x_41;
  x_41.x = dot (unity_SHAr, tmpvar_39);
  x_41.y = dot (unity_SHAg, tmpvar_39);
  x_41.z = dot (unity_SHAb, tmpvar_39);
  mediump vec3 x1_42;
  mediump vec4 tmpvar_43;
  tmpvar_43 = (normalWorld_23.xyzz * normalWorld_23.yzzx);
  x1_42.x = dot (unity_SHBr, tmpvar_43);
  x1_42.y = dot (unity_SHBg, tmpvar_43);
  x1_42.z = dot (unity_SHBb, tmpvar_43);
  res_40 = (x_41 + (x1_42 + (unity_SHC.xyz * 
    ((normalWorld_23.x * normalWorld_23.x) - (normalWorld_23.y * normalWorld_23.y))
  )));
  res_40 = max (((1.055 * 
    pow (max (res_40, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_24.xyz = (ambientOrLightmapUV_24.xyz + max (vec3(0.0, 0.0, 0.0), res_40));
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_24;
  xlv_TEXCOORD6 = tmpvar_8;
  xlv_TEXCOORD8 = tmpvar_9;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
uniform sampler2D _ShadowMapTexture;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump float atten_2;
  mediump vec3 i_posWorld_3;
  i_posWorld_3 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_4;
  tmpvar_4 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_5;
  tmpvar_5 = (_Color.xyz * tmpvar_4.xyz);
  mediump vec3 tmpvar_6;
  mediump vec3 tmpvar_7;
  tmpvar_7 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_5, vec3(_Metallic));
  mediump float tmpvar_8;
  tmpvar_8 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_6 = (tmpvar_5 * tmpvar_8);
  mediump vec3 tmpvar_9;
  tmpvar_9 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  lowp float tmpvar_11;
  tmpvar_11 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD6).x;
  atten_2 = tmpvar_11;
  mediump float tmpvar_12;
  lowp vec4 tmpvar_13;
  tmpvar_13 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_12 = tmpvar_13.y;
  highp vec3 tmpvar_14;
  highp vec4 tmpvar_15;
  highp vec4 tmpvar_16;
  tmpvar_14 = i_posWorld_3;
  tmpvar_15 = unity_SpecCube0_HDR;
  tmpvar_16 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_17;
  mediump vec3 tmpvar_18;
  mediump vec3 tmpvar_19;
  tmpvar_17 = (tmpvar_10 * atten_2);
  tmpvar_18 = (xlv_TEXCOORD5.xyz * tmpvar_12);
  tmpvar_19 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_20;
  mediump vec3 tmpvar_21;
  tmpvar_20 = (1.0 - _Glossiness);
  tmpvar_21 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_9, xlv_TEXCOORD1)
   * tmpvar_9)));
  mediump vec3 originalReflUVW_22;
  mediump vec3 specular_23;
  originalReflUVW_22 = tmpvar_21;
  mediump vec3 worldRefl_24;
  worldRefl_24 = tmpvar_21;
  highp vec3 worldPos_25;
  worldPos_25 = tmpvar_14;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_26;
    mediump vec3 rbmax_27;
    mediump vec3 tmpvar_28;
    tmpvar_28 = normalize(tmpvar_21);
    highp vec3 tmpvar_29;
    tmpvar_29 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_14) / tmpvar_28);
    rbmax_27 = tmpvar_29;
    highp vec3 tmpvar_30;
    tmpvar_30 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_14) / tmpvar_28);
    rbmin_26 = tmpvar_30;
    bvec3 tmpvar_31;
    tmpvar_31 = greaterThan (tmpvar_28, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_32;
    if (tmpvar_31.x) {
      tmpvar_32 = rbmax_27.x;
    } else {
      tmpvar_32 = rbmin_26.x;
    };
    mediump float tmpvar_33;
    if (tmpvar_31.y) {
      tmpvar_33 = rbmax_27.y;
    } else {
      tmpvar_33 = rbmin_26.y;
    };
    mediump float tmpvar_34;
    if (tmpvar_31.z) {
      tmpvar_34 = rbmax_27.z;
    } else {
      tmpvar_34 = rbmin_26.z;
    };
    worldPos_25 = (tmpvar_14 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_24 = (worldPos_25 + (tmpvar_28 * min (
      min (tmpvar_32, tmpvar_33)
    , tmpvar_34)));
  };
  tmpvar_21 = worldRefl_24;
  mediump vec4 hdr_35;
  hdr_35 = tmpvar_15;
  mediump vec4 tmpvar_36;
  tmpvar_36.xyz = worldRefl_24;
  tmpvar_36.w = ((tmpvar_20 * (1.7 - 
    (0.7 * tmpvar_20)
  )) * 6.0);
  lowp vec4 tmpvar_37;
  tmpvar_37 = textureCube (unity_SpecCube0, worldRefl_24, tmpvar_36.w);
  mediump vec4 tmpvar_38;
  tmpvar_38 = tmpvar_37;
  mediump vec3 tmpvar_39;
  tmpvar_39 = ((tmpvar_38.w * hdr_35.x) * tmpvar_38.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_40;
    worldRefl_40 = originalReflUVW_22;
    highp vec3 worldPos_41;
    worldPos_41 = tmpvar_14;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_42;
      mediump vec3 rbmax_43;
      mediump vec3 tmpvar_44;
      tmpvar_44 = normalize(originalReflUVW_22);
      highp vec3 tmpvar_45;
      tmpvar_45 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_14) / tmpvar_44);
      rbmax_43 = tmpvar_45;
      highp vec3 tmpvar_46;
      tmpvar_46 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_14) / tmpvar_44);
      rbmin_42 = tmpvar_46;
      bvec3 tmpvar_47;
      tmpvar_47 = greaterThan (tmpvar_44, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_48;
      if (tmpvar_47.x) {
        tmpvar_48 = rbmax_43.x;
      } else {
        tmpvar_48 = rbmin_42.x;
      };
      mediump float tmpvar_49;
      if (tmpvar_47.y) {
        tmpvar_49 = rbmax_43.y;
      } else {
        tmpvar_49 = rbmin_42.y;
      };
      mediump float tmpvar_50;
      if (tmpvar_47.z) {
        tmpvar_50 = rbmax_43.z;
      } else {
        tmpvar_50 = rbmin_42.z;
      };
      worldPos_41 = (tmpvar_14 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_40 = (worldPos_41 + (tmpvar_44 * min (
        min (tmpvar_48, tmpvar_49)
      , tmpvar_50)));
    };
    tmpvar_21 = worldRefl_40;
    mediump vec4 hdr_51;
    hdr_51 = tmpvar_16;
    mediump vec4 tmpvar_52;
    tmpvar_52.xyz = worldRefl_40;
    tmpvar_52.w = ((tmpvar_20 * (1.7 - 
      (0.7 * tmpvar_20)
    )) * 6.0);
    lowp vec4 tmpvar_53;
    tmpvar_53 = textureCube (unity_SpecCube1, worldRefl_40, tmpvar_52.w);
    mediump vec4 tmpvar_54;
    tmpvar_54 = tmpvar_53;
    mediump vec3 tmpvar_55;
    tmpvar_55 = ((tmpvar_54.w * hdr_51.x) * tmpvar_54.xyz);
    highp vec3 tmpvar_56;
    tmpvar_56 = mix (tmpvar_55, tmpvar_39, unity_SpecCube0_BoxMin.www);
    specular_23 = tmpvar_56;
  } else {
    specular_23 = tmpvar_39;
  };
  tmpvar_19 = (specular_23 * tmpvar_12);
  mediump vec3 viewDir_57;
  viewDir_57 = -(xlv_TEXCOORD1);
  mediump float specularTerm_58;
  mediump float tmpvar_59;
  tmpvar_59 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_60;
  mediump vec3 inVec_61;
  inVec_61 = (_WorldSpaceLightPos0.xyz + viewDir_57);
  tmpvar_60 = (inVec_61 * inversesqrt(max (0.001, 
    dot (inVec_61, inVec_61)
  )));
  mediump float tmpvar_62;
  tmpvar_62 = dot (tmpvar_9, viewDir_57);
  mediump vec3 tmpvar_63;
  if ((tmpvar_62 < 0.0)) {
    tmpvar_63 = (tmpvar_9 + (viewDir_57 * (
      -(tmpvar_62)
     + 1e-05)));
  } else {
    tmpvar_63 = tmpvar_9;
  };
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (tmpvar_63, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = clamp (dot (tmpvar_63, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_66;
  tmpvar_66 = clamp (dot (tmpvar_63, viewDir_57), 0.0, 1.0);
  mediump float tmpvar_67;
  tmpvar_67 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_68;
  tmpvar_68 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_69;
  tmpvar_69 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_70;
  tmpvar_70 = (tmpvar_69 * tmpvar_69);
  mediump float tmpvar_71;
  tmpvar_71 = (((tmpvar_65 * tmpvar_65) * (tmpvar_70 - 1.0)) + 1.0);
  mediump float x_72;
  x_72 = (1.0 - tmpvar_64);
  mediump float x_73;
  x_73 = (1.0 - tmpvar_66);
  mediump float tmpvar_74;
  tmpvar_74 = (0.5 + ((2.0 * tmpvar_67) * (tmpvar_67 * tmpvar_59)));
  specularTerm_58 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_64 * ((tmpvar_66 * (1.0 - tmpvar_68)) + tmpvar_68))
     + 
      (tmpvar_66 * ((tmpvar_64 * (1.0 - tmpvar_68)) + tmpvar_68))
    ) + 1e-05)) * (tmpvar_70 / ((
      (3.141593 * tmpvar_71)
     * tmpvar_71) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_75;
  tmpvar_75 = max (0.0, (specularTerm_58 * tmpvar_64));
  specularTerm_58 = tmpvar_75;
  mediump float x_76;
  x_76 = (1.0 - tmpvar_67);
  mediump float x_77;
  x_77 = (1.0 - tmpvar_66);
  mediump vec3 tmpvar_78;
  tmpvar_78 = (((tmpvar_6 * 
    (tmpvar_18 + (tmpvar_17 * ((
      (1.0 + ((tmpvar_74 - 1.0) * ((x_72 * x_72) * (
        (x_72 * x_72)
       * x_72))))
     * 
      (1.0 + ((tmpvar_74 - 1.0) * ((x_73 * x_73) * (
        (x_73 * x_73)
       * x_73))))
    ) * tmpvar_64)))
  ) + (
    (tmpvar_75 * tmpvar_17)
   * 
    (tmpvar_7 + ((1.0 - tmpvar_7) * ((x_76 * x_76) * (
      (x_76 * x_76)
     * x_76))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_59) * (tmpvar_59 * tmpvar_59)))
   * tmpvar_19) * mix (tmpvar_7, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_8)), 0.0, 1.0)
  ), vec3(
    ((x_77 * x_77) * ((x_77 * x_77) * x_77))
  ))));
  mediump vec4 tmpvar_79;
  tmpvar_79.w = 1.0;
  tmpvar_79.xyz = tmpvar_78;
  c_1.w = tmpvar_79.w;
  c_1.xyz = tmpvar_78;
  c_1.xyz = c_1.xyz;
  mediump vec4 xlat_varoutput_80;
  xlat_varoutput_80.xyz = c_1.xyz;
  xlat_varoutput_80.w = 1.0;
  gl_FragData[0] = xlat_varoutput_80;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  highp vec3 tmpvar_8;
  highp vec4 tmpvar_9;
  tmpvar_9 = (_Object2World * _glesVertex);
  tmpvar_8 = tmpvar_9.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_10;
  texcoord_10.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_11;
  if ((_UVSec == 0.0)) {
    tmpvar_11 = tmpvar_2;
  } else {
    tmpvar_11 = tmpvar_3;
  };
  texcoord_10.zw = ((tmpvar_11 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_12;
  highp vec3 tmpvar_13;
  tmpvar_13 = normalize((tmpvar_9.xyz - _WorldSpaceCameraPos));
  tmpvar_12 = tmpvar_13;
  highp vec3 norm_14;
  norm_14 = tmpvar_1;
  highp vec4 v_15;
  v_15.x = _World2Object[0].x;
  v_15.y = _World2Object[1].x;
  v_15.z = _World2Object[2].x;
  v_15.w = _World2Object[3].x;
  highp vec4 v_16;
  v_16.x = _World2Object[0].y;
  v_16.y = _World2Object[1].y;
  v_16.z = _World2Object[2].y;
  v_16.w = _World2Object[3].y;
  highp vec4 v_17;
  v_17.x = _World2Object[0].z;
  v_17.y = _World2Object[1].z;
  v_17.z = _World2Object[2].z;
  v_17.w = _World2Object[3].z;
  highp vec3 tmpvar_18;
  tmpvar_18 = normalize(((
    (v_15.xyz * norm_14.x)
   + 
    (v_16.xyz * norm_14.y)
  ) + (v_17.xyz * norm_14.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_18;
  mediump vec3 normalWorld_19;
  normalWorld_19 = tmpvar_18;
  mediump vec4 ambientOrLightmapUV_20;
  ambientOrLightmapUV_20.w = 0.0;
  mediump vec4 tmpvar_21;
  tmpvar_21.w = 1.0;
  tmpvar_21.xyz = normalWorld_19;
  mediump vec3 res_22;
  mediump vec3 x_23;
  x_23.x = dot (unity_SHAr, tmpvar_21);
  x_23.y = dot (unity_SHAg, tmpvar_21);
  x_23.z = dot (unity_SHAb, tmpvar_21);
  mediump vec3 x1_24;
  mediump vec4 tmpvar_25;
  tmpvar_25 = (normalWorld_19.xyzz * normalWorld_19.yzzx);
  x1_24.x = dot (unity_SHBr, tmpvar_25);
  x1_24.y = dot (unity_SHBg, tmpvar_25);
  x1_24.z = dot (unity_SHBb, tmpvar_25);
  res_22 = (x_23 + (x1_24 + (unity_SHC.xyz * 
    ((normalWorld_19.x * normalWorld_19.x) - (normalWorld_19.y * normalWorld_19.y))
  )));
  res_22 = max (((1.055 * 
    pow (max (res_22, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_20.xyz = max (vec3(0.0, 0.0, 0.0), res_22);
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_10;
  xlv_TEXCOORD1 = tmpvar_12;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_20;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_4.z)));
  xlv_TEXCOORD8 = tmpvar_8;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp vec4 unity_FogColor;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump vec3 i_posWorld_2;
  i_posWorld_2 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_3;
  tmpvar_3 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_4;
  tmpvar_4 = (_Color.xyz * tmpvar_3.xyz);
  mediump vec3 tmpvar_5;
  mediump vec3 tmpvar_6;
  tmpvar_6 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_4, vec3(_Metallic));
  mediump float tmpvar_7;
  tmpvar_7 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_5 = (tmpvar_4 * tmpvar_7);
  mediump vec3 tmpvar_8;
  tmpvar_8 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_9;
  tmpvar_9 = _LightColor0.xyz;
  mediump float tmpvar_10;
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_10 = tmpvar_11.y;
  highp vec3 tmpvar_12;
  highp vec4 tmpvar_13;
  highp vec4 tmpvar_14;
  tmpvar_12 = i_posWorld_2;
  tmpvar_13 = unity_SpecCube0_HDR;
  tmpvar_14 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_15;
  mediump vec3 tmpvar_16;
  tmpvar_15 = (xlv_TEXCOORD5.xyz * tmpvar_10);
  tmpvar_16 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_17;
  mediump vec3 tmpvar_18;
  tmpvar_17 = (1.0 - _Glossiness);
  tmpvar_18 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_8, xlv_TEXCOORD1)
   * tmpvar_8)));
  mediump vec3 originalReflUVW_19;
  mediump vec3 specular_20;
  originalReflUVW_19 = tmpvar_18;
  mediump vec3 worldRefl_21;
  worldRefl_21 = tmpvar_18;
  highp vec3 worldPos_22;
  worldPos_22 = tmpvar_12;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_23;
    mediump vec3 rbmax_24;
    mediump vec3 tmpvar_25;
    tmpvar_25 = normalize(tmpvar_18);
    highp vec3 tmpvar_26;
    tmpvar_26 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_12) / tmpvar_25);
    rbmax_24 = tmpvar_26;
    highp vec3 tmpvar_27;
    tmpvar_27 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_12) / tmpvar_25);
    rbmin_23 = tmpvar_27;
    bvec3 tmpvar_28;
    tmpvar_28 = greaterThan (tmpvar_25, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_29;
    if (tmpvar_28.x) {
      tmpvar_29 = rbmax_24.x;
    } else {
      tmpvar_29 = rbmin_23.x;
    };
    mediump float tmpvar_30;
    if (tmpvar_28.y) {
      tmpvar_30 = rbmax_24.y;
    } else {
      tmpvar_30 = rbmin_23.y;
    };
    mediump float tmpvar_31;
    if (tmpvar_28.z) {
      tmpvar_31 = rbmax_24.z;
    } else {
      tmpvar_31 = rbmin_23.z;
    };
    worldPos_22 = (tmpvar_12 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_21 = (worldPos_22 + (tmpvar_25 * min (
      min (tmpvar_29, tmpvar_30)
    , tmpvar_31)));
  };
  tmpvar_18 = worldRefl_21;
  mediump vec4 hdr_32;
  hdr_32 = tmpvar_13;
  mediump vec4 tmpvar_33;
  tmpvar_33.xyz = worldRefl_21;
  tmpvar_33.w = ((tmpvar_17 * (1.7 - 
    (0.7 * tmpvar_17)
  )) * 6.0);
  lowp vec4 tmpvar_34;
  tmpvar_34 = textureCube (unity_SpecCube0, worldRefl_21, tmpvar_33.w);
  mediump vec4 tmpvar_35;
  tmpvar_35 = tmpvar_34;
  mediump vec3 tmpvar_36;
  tmpvar_36 = ((tmpvar_35.w * hdr_32.x) * tmpvar_35.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_37;
    worldRefl_37 = originalReflUVW_19;
    highp vec3 worldPos_38;
    worldPos_38 = tmpvar_12;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_39;
      mediump vec3 rbmax_40;
      mediump vec3 tmpvar_41;
      tmpvar_41 = normalize(originalReflUVW_19);
      highp vec3 tmpvar_42;
      tmpvar_42 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_12) / tmpvar_41);
      rbmax_40 = tmpvar_42;
      highp vec3 tmpvar_43;
      tmpvar_43 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_12) / tmpvar_41);
      rbmin_39 = tmpvar_43;
      bvec3 tmpvar_44;
      tmpvar_44 = greaterThan (tmpvar_41, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_45;
      if (tmpvar_44.x) {
        tmpvar_45 = rbmax_40.x;
      } else {
        tmpvar_45 = rbmin_39.x;
      };
      mediump float tmpvar_46;
      if (tmpvar_44.y) {
        tmpvar_46 = rbmax_40.y;
      } else {
        tmpvar_46 = rbmin_39.y;
      };
      mediump float tmpvar_47;
      if (tmpvar_44.z) {
        tmpvar_47 = rbmax_40.z;
      } else {
        tmpvar_47 = rbmin_39.z;
      };
      worldPos_38 = (tmpvar_12 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_37 = (worldPos_38 + (tmpvar_41 * min (
        min (tmpvar_45, tmpvar_46)
      , tmpvar_47)));
    };
    tmpvar_18 = worldRefl_37;
    mediump vec4 hdr_48;
    hdr_48 = tmpvar_14;
    mediump vec4 tmpvar_49;
    tmpvar_49.xyz = worldRefl_37;
    tmpvar_49.w = ((tmpvar_17 * (1.7 - 
      (0.7 * tmpvar_17)
    )) * 6.0);
    lowp vec4 tmpvar_50;
    tmpvar_50 = textureCube (unity_SpecCube1, worldRefl_37, tmpvar_49.w);
    mediump vec4 tmpvar_51;
    tmpvar_51 = tmpvar_50;
    mediump vec3 tmpvar_52;
    tmpvar_52 = ((tmpvar_51.w * hdr_48.x) * tmpvar_51.xyz);
    highp vec3 tmpvar_53;
    tmpvar_53 = mix (tmpvar_52, tmpvar_36, unity_SpecCube0_BoxMin.www);
    specular_20 = tmpvar_53;
  } else {
    specular_20 = tmpvar_36;
  };
  tmpvar_16 = (specular_20 * tmpvar_10);
  mediump vec3 viewDir_54;
  viewDir_54 = -(xlv_TEXCOORD1);
  mediump float specularTerm_55;
  mediump float tmpvar_56;
  tmpvar_56 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_57;
  mediump vec3 inVec_58;
  inVec_58 = (_WorldSpaceLightPos0.xyz + viewDir_54);
  tmpvar_57 = (inVec_58 * inversesqrt(max (0.001, 
    dot (inVec_58, inVec_58)
  )));
  mediump float tmpvar_59;
  tmpvar_59 = dot (tmpvar_8, viewDir_54);
  mediump vec3 tmpvar_60;
  if ((tmpvar_59 < 0.0)) {
    tmpvar_60 = (tmpvar_8 + (viewDir_54 * (
      -(tmpvar_59)
     + 1e-05)));
  } else {
    tmpvar_60 = tmpvar_8;
  };
  mediump float tmpvar_61;
  tmpvar_61 = clamp (dot (tmpvar_60, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_62;
  tmpvar_62 = clamp (dot (tmpvar_60, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_63;
  tmpvar_63 = clamp (dot (tmpvar_60, viewDir_54), 0.0, 1.0);
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_66;
  tmpvar_66 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_67;
  tmpvar_67 = (tmpvar_66 * tmpvar_66);
  mediump float tmpvar_68;
  tmpvar_68 = (((tmpvar_62 * tmpvar_62) * (tmpvar_67 - 1.0)) + 1.0);
  mediump float x_69;
  x_69 = (1.0 - tmpvar_61);
  mediump float x_70;
  x_70 = (1.0 - tmpvar_63);
  mediump float tmpvar_71;
  tmpvar_71 = (0.5 + ((2.0 * tmpvar_64) * (tmpvar_64 * tmpvar_56)));
  specularTerm_55 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_61 * ((tmpvar_63 * (1.0 - tmpvar_65)) + tmpvar_65))
     + 
      (tmpvar_63 * ((tmpvar_61 * (1.0 - tmpvar_65)) + tmpvar_65))
    ) + 1e-05)) * (tmpvar_67 / ((
      (3.141593 * tmpvar_68)
     * tmpvar_68) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_72;
  tmpvar_72 = max (0.0, (specularTerm_55 * tmpvar_61));
  specularTerm_55 = tmpvar_72;
  mediump float x_73;
  x_73 = (1.0 - tmpvar_64);
  mediump float x_74;
  x_74 = (1.0 - tmpvar_63);
  mediump vec3 tmpvar_75;
  tmpvar_75 = (((tmpvar_5 * 
    (tmpvar_15 + (tmpvar_9 * ((
      (1.0 + ((tmpvar_71 - 1.0) * ((x_69 * x_69) * (
        (x_69 * x_69)
       * x_69))))
     * 
      (1.0 + ((tmpvar_71 - 1.0) * ((x_70 * x_70) * (
        (x_70 * x_70)
       * x_70))))
    ) * tmpvar_61)))
  ) + (
    (tmpvar_72 * tmpvar_9)
   * 
    (tmpvar_6 + ((1.0 - tmpvar_6) * ((x_73 * x_73) * (
      (x_73 * x_73)
     * x_73))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_56) * (tmpvar_56 * tmpvar_56)))
   * tmpvar_16) * mix (tmpvar_6, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_7)), 0.0, 1.0)
  ), vec3(
    ((x_74 * x_74) * ((x_74 * x_74) * x_74))
  ))));
  mediump vec4 tmpvar_76;
  tmpvar_76.w = 1.0;
  tmpvar_76.xyz = tmpvar_75;
  c_1.w = tmpvar_76.w;
  c_1.xyz = tmpvar_75;
  c_1.xyz = c_1.xyz;
  highp float tmpvar_77;
  tmpvar_77 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = mix (unity_FogColor.xyz, c_1.xyz, vec3(tmpvar_77));
  mediump vec4 xlat_varoutput_78;
  xlat_varoutput_78.xyz = c_1.xyz;
  xlat_varoutput_78.w = 1.0;
  gl_FragData[0] = xlat_varoutput_78;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  highp vec3 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_9 = tmpvar_10.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_19;
  highp vec4 o_20;
  highp vec4 tmpvar_21;
  tmpvar_21 = (tmpvar_4 * 0.5);
  highp vec2 tmpvar_22;
  tmpvar_22.x = tmpvar_21.x;
  tmpvar_22.y = (tmpvar_21.y * _ProjectionParams.x);
  o_20.xy = (tmpvar_22 + tmpvar_21.w);
  o_20.zw = tmpvar_4.zw;
  tmpvar_8 = o_20;
  mediump vec3 normalWorld_23;
  normalWorld_23 = tmpvar_19;
  mediump vec4 ambientOrLightmapUV_24;
  ambientOrLightmapUV_24.w = 0.0;
  mediump vec4 tmpvar_25;
  tmpvar_25.w = 1.0;
  tmpvar_25.xyz = normalWorld_23;
  mediump vec3 res_26;
  mediump vec3 x_27;
  x_27.x = dot (unity_SHAr, tmpvar_25);
  x_27.y = dot (unity_SHAg, tmpvar_25);
  x_27.z = dot (unity_SHAb, tmpvar_25);
  mediump vec3 x1_28;
  mediump vec4 tmpvar_29;
  tmpvar_29 = (normalWorld_23.xyzz * normalWorld_23.yzzx);
  x1_28.x = dot (unity_SHBr, tmpvar_29);
  x1_28.y = dot (unity_SHBg, tmpvar_29);
  x1_28.z = dot (unity_SHBb, tmpvar_29);
  res_26 = (x_27 + (x1_28 + (unity_SHC.xyz * 
    ((normalWorld_23.x * normalWorld_23.x) - (normalWorld_23.y * normalWorld_23.y))
  )));
  res_26 = max (((1.055 * 
    pow (max (res_26, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_24.xyz = max (vec3(0.0, 0.0, 0.0), res_26);
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_24;
  xlv_TEXCOORD6 = tmpvar_8;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_4.z)));
  xlv_TEXCOORD8 = tmpvar_9;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp vec4 unity_FogColor;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
uniform sampler2D _ShadowMapTexture;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump float atten_2;
  mediump vec3 i_posWorld_3;
  i_posWorld_3 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_4;
  tmpvar_4 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_5;
  tmpvar_5 = (_Color.xyz * tmpvar_4.xyz);
  mediump vec3 tmpvar_6;
  mediump vec3 tmpvar_7;
  tmpvar_7 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_5, vec3(_Metallic));
  mediump float tmpvar_8;
  tmpvar_8 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_6 = (tmpvar_5 * tmpvar_8);
  mediump vec3 tmpvar_9;
  tmpvar_9 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  lowp float tmpvar_11;
  tmpvar_11 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD6).x;
  atten_2 = tmpvar_11;
  mediump float tmpvar_12;
  lowp vec4 tmpvar_13;
  tmpvar_13 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_12 = tmpvar_13.y;
  highp vec3 tmpvar_14;
  highp vec4 tmpvar_15;
  highp vec4 tmpvar_16;
  tmpvar_14 = i_posWorld_3;
  tmpvar_15 = unity_SpecCube0_HDR;
  tmpvar_16 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_17;
  mediump vec3 tmpvar_18;
  mediump vec3 tmpvar_19;
  tmpvar_17 = (tmpvar_10 * atten_2);
  tmpvar_18 = (xlv_TEXCOORD5.xyz * tmpvar_12);
  tmpvar_19 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_20;
  mediump vec3 tmpvar_21;
  tmpvar_20 = (1.0 - _Glossiness);
  tmpvar_21 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_9, xlv_TEXCOORD1)
   * tmpvar_9)));
  mediump vec3 originalReflUVW_22;
  mediump vec3 specular_23;
  originalReflUVW_22 = tmpvar_21;
  mediump vec3 worldRefl_24;
  worldRefl_24 = tmpvar_21;
  highp vec3 worldPos_25;
  worldPos_25 = tmpvar_14;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_26;
    mediump vec3 rbmax_27;
    mediump vec3 tmpvar_28;
    tmpvar_28 = normalize(tmpvar_21);
    highp vec3 tmpvar_29;
    tmpvar_29 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_14) / tmpvar_28);
    rbmax_27 = tmpvar_29;
    highp vec3 tmpvar_30;
    tmpvar_30 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_14) / tmpvar_28);
    rbmin_26 = tmpvar_30;
    bvec3 tmpvar_31;
    tmpvar_31 = greaterThan (tmpvar_28, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_32;
    if (tmpvar_31.x) {
      tmpvar_32 = rbmax_27.x;
    } else {
      tmpvar_32 = rbmin_26.x;
    };
    mediump float tmpvar_33;
    if (tmpvar_31.y) {
      tmpvar_33 = rbmax_27.y;
    } else {
      tmpvar_33 = rbmin_26.y;
    };
    mediump float tmpvar_34;
    if (tmpvar_31.z) {
      tmpvar_34 = rbmax_27.z;
    } else {
      tmpvar_34 = rbmin_26.z;
    };
    worldPos_25 = (tmpvar_14 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_24 = (worldPos_25 + (tmpvar_28 * min (
      min (tmpvar_32, tmpvar_33)
    , tmpvar_34)));
  };
  tmpvar_21 = worldRefl_24;
  mediump vec4 hdr_35;
  hdr_35 = tmpvar_15;
  mediump vec4 tmpvar_36;
  tmpvar_36.xyz = worldRefl_24;
  tmpvar_36.w = ((tmpvar_20 * (1.7 - 
    (0.7 * tmpvar_20)
  )) * 6.0);
  lowp vec4 tmpvar_37;
  tmpvar_37 = textureCube (unity_SpecCube0, worldRefl_24, tmpvar_36.w);
  mediump vec4 tmpvar_38;
  tmpvar_38 = tmpvar_37;
  mediump vec3 tmpvar_39;
  tmpvar_39 = ((tmpvar_38.w * hdr_35.x) * tmpvar_38.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_40;
    worldRefl_40 = originalReflUVW_22;
    highp vec3 worldPos_41;
    worldPos_41 = tmpvar_14;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_42;
      mediump vec3 rbmax_43;
      mediump vec3 tmpvar_44;
      tmpvar_44 = normalize(originalReflUVW_22);
      highp vec3 tmpvar_45;
      tmpvar_45 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_14) / tmpvar_44);
      rbmax_43 = tmpvar_45;
      highp vec3 tmpvar_46;
      tmpvar_46 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_14) / tmpvar_44);
      rbmin_42 = tmpvar_46;
      bvec3 tmpvar_47;
      tmpvar_47 = greaterThan (tmpvar_44, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_48;
      if (tmpvar_47.x) {
        tmpvar_48 = rbmax_43.x;
      } else {
        tmpvar_48 = rbmin_42.x;
      };
      mediump float tmpvar_49;
      if (tmpvar_47.y) {
        tmpvar_49 = rbmax_43.y;
      } else {
        tmpvar_49 = rbmin_42.y;
      };
      mediump float tmpvar_50;
      if (tmpvar_47.z) {
        tmpvar_50 = rbmax_43.z;
      } else {
        tmpvar_50 = rbmin_42.z;
      };
      worldPos_41 = (tmpvar_14 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_40 = (worldPos_41 + (tmpvar_44 * min (
        min (tmpvar_48, tmpvar_49)
      , tmpvar_50)));
    };
    tmpvar_21 = worldRefl_40;
    mediump vec4 hdr_51;
    hdr_51 = tmpvar_16;
    mediump vec4 tmpvar_52;
    tmpvar_52.xyz = worldRefl_40;
    tmpvar_52.w = ((tmpvar_20 * (1.7 - 
      (0.7 * tmpvar_20)
    )) * 6.0);
    lowp vec4 tmpvar_53;
    tmpvar_53 = textureCube (unity_SpecCube1, worldRefl_40, tmpvar_52.w);
    mediump vec4 tmpvar_54;
    tmpvar_54 = tmpvar_53;
    mediump vec3 tmpvar_55;
    tmpvar_55 = ((tmpvar_54.w * hdr_51.x) * tmpvar_54.xyz);
    highp vec3 tmpvar_56;
    tmpvar_56 = mix (tmpvar_55, tmpvar_39, unity_SpecCube0_BoxMin.www);
    specular_23 = tmpvar_56;
  } else {
    specular_23 = tmpvar_39;
  };
  tmpvar_19 = (specular_23 * tmpvar_12);
  mediump vec3 viewDir_57;
  viewDir_57 = -(xlv_TEXCOORD1);
  mediump float specularTerm_58;
  mediump float tmpvar_59;
  tmpvar_59 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_60;
  mediump vec3 inVec_61;
  inVec_61 = (_WorldSpaceLightPos0.xyz + viewDir_57);
  tmpvar_60 = (inVec_61 * inversesqrt(max (0.001, 
    dot (inVec_61, inVec_61)
  )));
  mediump float tmpvar_62;
  tmpvar_62 = dot (tmpvar_9, viewDir_57);
  mediump vec3 tmpvar_63;
  if ((tmpvar_62 < 0.0)) {
    tmpvar_63 = (tmpvar_9 + (viewDir_57 * (
      -(tmpvar_62)
     + 1e-05)));
  } else {
    tmpvar_63 = tmpvar_9;
  };
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (tmpvar_63, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = clamp (dot (tmpvar_63, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_66;
  tmpvar_66 = clamp (dot (tmpvar_63, viewDir_57), 0.0, 1.0);
  mediump float tmpvar_67;
  tmpvar_67 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_68;
  tmpvar_68 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_69;
  tmpvar_69 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_70;
  tmpvar_70 = (tmpvar_69 * tmpvar_69);
  mediump float tmpvar_71;
  tmpvar_71 = (((tmpvar_65 * tmpvar_65) * (tmpvar_70 - 1.0)) + 1.0);
  mediump float x_72;
  x_72 = (1.0 - tmpvar_64);
  mediump float x_73;
  x_73 = (1.0 - tmpvar_66);
  mediump float tmpvar_74;
  tmpvar_74 = (0.5 + ((2.0 * tmpvar_67) * (tmpvar_67 * tmpvar_59)));
  specularTerm_58 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_64 * ((tmpvar_66 * (1.0 - tmpvar_68)) + tmpvar_68))
     + 
      (tmpvar_66 * ((tmpvar_64 * (1.0 - tmpvar_68)) + tmpvar_68))
    ) + 1e-05)) * (tmpvar_70 / ((
      (3.141593 * tmpvar_71)
     * tmpvar_71) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_75;
  tmpvar_75 = max (0.0, (specularTerm_58 * tmpvar_64));
  specularTerm_58 = tmpvar_75;
  mediump float x_76;
  x_76 = (1.0 - tmpvar_67);
  mediump float x_77;
  x_77 = (1.0 - tmpvar_66);
  mediump vec3 tmpvar_78;
  tmpvar_78 = (((tmpvar_6 * 
    (tmpvar_18 + (tmpvar_17 * ((
      (1.0 + ((tmpvar_74 - 1.0) * ((x_72 * x_72) * (
        (x_72 * x_72)
       * x_72))))
     * 
      (1.0 + ((tmpvar_74 - 1.0) * ((x_73 * x_73) * (
        (x_73 * x_73)
       * x_73))))
    ) * tmpvar_64)))
  ) + (
    (tmpvar_75 * tmpvar_17)
   * 
    (tmpvar_7 + ((1.0 - tmpvar_7) * ((x_76 * x_76) * (
      (x_76 * x_76)
     * x_76))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_59) * (tmpvar_59 * tmpvar_59)))
   * tmpvar_19) * mix (tmpvar_7, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_8)), 0.0, 1.0)
  ), vec3(
    ((x_77 * x_77) * ((x_77 * x_77) * x_77))
  ))));
  mediump vec4 tmpvar_79;
  tmpvar_79.w = 1.0;
  tmpvar_79.xyz = tmpvar_78;
  c_1.w = tmpvar_79.w;
  c_1.xyz = tmpvar_78;
  c_1.xyz = c_1.xyz;
  highp float tmpvar_80;
  tmpvar_80 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = mix (unity_FogColor.xyz, c_1.xyz, vec3(tmpvar_80));
  mediump vec4 xlat_varoutput_81;
  xlat_varoutput_81.xyz = c_1.xyz;
  xlat_varoutput_81.w = 1.0;
  gl_FragData[0] = xlat_varoutput_81;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "FOG_EXP" "VERTEXLIGHT_ON" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 unity_4LightPosX0;
uniform highp vec4 unity_4LightPosY0;
uniform highp vec4 unity_4LightPosZ0;
uniform mediump vec4 unity_4LightAtten0;
uniform mediump vec4 unity_LightColor[8];
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  highp vec3 tmpvar_8;
  highp vec4 tmpvar_9;
  tmpvar_9 = (_Object2World * _glesVertex);
  tmpvar_8 = tmpvar_9.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_10;
  texcoord_10.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_11;
  if ((_UVSec == 0.0)) {
    tmpvar_11 = tmpvar_2;
  } else {
    tmpvar_11 = tmpvar_3;
  };
  texcoord_10.zw = ((tmpvar_11 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_12;
  highp vec3 tmpvar_13;
  tmpvar_13 = normalize((tmpvar_9.xyz - _WorldSpaceCameraPos));
  tmpvar_12 = tmpvar_13;
  highp vec3 norm_14;
  norm_14 = tmpvar_1;
  highp vec4 v_15;
  v_15.x = _World2Object[0].x;
  v_15.y = _World2Object[1].x;
  v_15.z = _World2Object[2].x;
  v_15.w = _World2Object[3].x;
  highp vec4 v_16;
  v_16.x = _World2Object[0].y;
  v_16.y = _World2Object[1].y;
  v_16.z = _World2Object[2].y;
  v_16.w = _World2Object[3].y;
  highp vec4 v_17;
  v_17.x = _World2Object[0].z;
  v_17.y = _World2Object[1].z;
  v_17.z = _World2Object[2].z;
  v_17.w = _World2Object[3].z;
  highp vec3 tmpvar_18;
  tmpvar_18 = normalize(((
    (v_15.xyz * norm_14.x)
   + 
    (v_16.xyz * norm_14.y)
  ) + (v_17.xyz * norm_14.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_18;
  mediump vec3 normalWorld_19;
  normalWorld_19 = tmpvar_18;
  mediump vec4 ambientOrLightmapUV_20;
  ambientOrLightmapUV_20.w = 0.0;
  highp vec3 lightColor0_21;
  lightColor0_21 = unity_LightColor[0].xyz;
  highp vec3 lightColor1_22;
  lightColor1_22 = unity_LightColor[1].xyz;
  highp vec3 lightColor2_23;
  lightColor2_23 = unity_LightColor[2].xyz;
  highp vec3 lightColor3_24;
  lightColor3_24 = unity_LightColor[3].xyz;
  highp vec4 lightAttenSq_25;
  lightAttenSq_25 = unity_4LightAtten0;
  highp vec3 normal_26;
  normal_26 = normalWorld_19;
  highp vec3 col_27;
  highp vec4 ndotl_28;
  highp vec4 lengthSq_29;
  highp vec4 tmpvar_30;
  tmpvar_30 = (unity_4LightPosX0 - tmpvar_9.x);
  highp vec4 tmpvar_31;
  tmpvar_31 = (unity_4LightPosY0 - tmpvar_9.y);
  highp vec4 tmpvar_32;
  tmpvar_32 = (unity_4LightPosZ0 - tmpvar_9.z);
  lengthSq_29 = (tmpvar_30 * tmpvar_30);
  lengthSq_29 = (lengthSq_29 + (tmpvar_31 * tmpvar_31));
  lengthSq_29 = (lengthSq_29 + (tmpvar_32 * tmpvar_32));
  ndotl_28 = (tmpvar_30 * normal_26.x);
  ndotl_28 = (ndotl_28 + (tmpvar_31 * normal_26.y));
  ndotl_28 = (ndotl_28 + (tmpvar_32 * normal_26.z));
  highp vec4 tmpvar_33;
  tmpvar_33 = max (vec4(0.0, 0.0, 0.0, 0.0), (ndotl_28 * inversesqrt(lengthSq_29)));
  ndotl_28 = tmpvar_33;
  highp vec4 tmpvar_34;
  tmpvar_34 = (tmpvar_33 * (1.0/((1.0 + 
    (lengthSq_29 * lightAttenSq_25)
  ))));
  col_27 = (lightColor0_21 * tmpvar_34.x);
  col_27 = (col_27 + (lightColor1_22 * tmpvar_34.y));
  col_27 = (col_27 + (lightColor2_23 * tmpvar_34.z));
  col_27 = (col_27 + (lightColor3_24 * tmpvar_34.w));
  ambientOrLightmapUV_20.xyz = col_27;
  mediump vec4 tmpvar_35;
  tmpvar_35.w = 1.0;
  tmpvar_35.xyz = normalWorld_19;
  mediump vec3 res_36;
  mediump vec3 x_37;
  x_37.x = dot (unity_SHAr, tmpvar_35);
  x_37.y = dot (unity_SHAg, tmpvar_35);
  x_37.z = dot (unity_SHAb, tmpvar_35);
  mediump vec3 x1_38;
  mediump vec4 tmpvar_39;
  tmpvar_39 = (normalWorld_19.xyzz * normalWorld_19.yzzx);
  x1_38.x = dot (unity_SHBr, tmpvar_39);
  x1_38.y = dot (unity_SHBg, tmpvar_39);
  x1_38.z = dot (unity_SHBb, tmpvar_39);
  res_36 = (x_37 + (x1_38 + (unity_SHC.xyz * 
    ((normalWorld_19.x * normalWorld_19.x) - (normalWorld_19.y * normalWorld_19.y))
  )));
  res_36 = max (((1.055 * 
    pow (max (res_36, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_20.xyz = (ambientOrLightmapUV_20.xyz + max (vec3(0.0, 0.0, 0.0), res_36));
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_10;
  xlv_TEXCOORD1 = tmpvar_12;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_20;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_4.z)));
  xlv_TEXCOORD8 = tmpvar_8;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp vec4 unity_FogColor;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump vec3 i_posWorld_2;
  i_posWorld_2 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_3;
  tmpvar_3 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_4;
  tmpvar_4 = (_Color.xyz * tmpvar_3.xyz);
  mediump vec3 tmpvar_5;
  mediump vec3 tmpvar_6;
  tmpvar_6 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_4, vec3(_Metallic));
  mediump float tmpvar_7;
  tmpvar_7 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_5 = (tmpvar_4 * tmpvar_7);
  mediump vec3 tmpvar_8;
  tmpvar_8 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_9;
  tmpvar_9 = _LightColor0.xyz;
  mediump float tmpvar_10;
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_10 = tmpvar_11.y;
  highp vec3 tmpvar_12;
  highp vec4 tmpvar_13;
  highp vec4 tmpvar_14;
  tmpvar_12 = i_posWorld_2;
  tmpvar_13 = unity_SpecCube0_HDR;
  tmpvar_14 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_15;
  mediump vec3 tmpvar_16;
  tmpvar_15 = (xlv_TEXCOORD5.xyz * tmpvar_10);
  tmpvar_16 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_17;
  mediump vec3 tmpvar_18;
  tmpvar_17 = (1.0 - _Glossiness);
  tmpvar_18 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_8, xlv_TEXCOORD1)
   * tmpvar_8)));
  mediump vec3 originalReflUVW_19;
  mediump vec3 specular_20;
  originalReflUVW_19 = tmpvar_18;
  mediump vec3 worldRefl_21;
  worldRefl_21 = tmpvar_18;
  highp vec3 worldPos_22;
  worldPos_22 = tmpvar_12;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_23;
    mediump vec3 rbmax_24;
    mediump vec3 tmpvar_25;
    tmpvar_25 = normalize(tmpvar_18);
    highp vec3 tmpvar_26;
    tmpvar_26 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_12) / tmpvar_25);
    rbmax_24 = tmpvar_26;
    highp vec3 tmpvar_27;
    tmpvar_27 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_12) / tmpvar_25);
    rbmin_23 = tmpvar_27;
    bvec3 tmpvar_28;
    tmpvar_28 = greaterThan (tmpvar_25, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_29;
    if (tmpvar_28.x) {
      tmpvar_29 = rbmax_24.x;
    } else {
      tmpvar_29 = rbmin_23.x;
    };
    mediump float tmpvar_30;
    if (tmpvar_28.y) {
      tmpvar_30 = rbmax_24.y;
    } else {
      tmpvar_30 = rbmin_23.y;
    };
    mediump float tmpvar_31;
    if (tmpvar_28.z) {
      tmpvar_31 = rbmax_24.z;
    } else {
      tmpvar_31 = rbmin_23.z;
    };
    worldPos_22 = (tmpvar_12 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_21 = (worldPos_22 + (tmpvar_25 * min (
      min (tmpvar_29, tmpvar_30)
    , tmpvar_31)));
  };
  tmpvar_18 = worldRefl_21;
  mediump vec4 hdr_32;
  hdr_32 = tmpvar_13;
  mediump vec4 tmpvar_33;
  tmpvar_33.xyz = worldRefl_21;
  tmpvar_33.w = ((tmpvar_17 * (1.7 - 
    (0.7 * tmpvar_17)
  )) * 6.0);
  lowp vec4 tmpvar_34;
  tmpvar_34 = textureCube (unity_SpecCube0, worldRefl_21, tmpvar_33.w);
  mediump vec4 tmpvar_35;
  tmpvar_35 = tmpvar_34;
  mediump vec3 tmpvar_36;
  tmpvar_36 = ((tmpvar_35.w * hdr_32.x) * tmpvar_35.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_37;
    worldRefl_37 = originalReflUVW_19;
    highp vec3 worldPos_38;
    worldPos_38 = tmpvar_12;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_39;
      mediump vec3 rbmax_40;
      mediump vec3 tmpvar_41;
      tmpvar_41 = normalize(originalReflUVW_19);
      highp vec3 tmpvar_42;
      tmpvar_42 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_12) / tmpvar_41);
      rbmax_40 = tmpvar_42;
      highp vec3 tmpvar_43;
      tmpvar_43 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_12) / tmpvar_41);
      rbmin_39 = tmpvar_43;
      bvec3 tmpvar_44;
      tmpvar_44 = greaterThan (tmpvar_41, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_45;
      if (tmpvar_44.x) {
        tmpvar_45 = rbmax_40.x;
      } else {
        tmpvar_45 = rbmin_39.x;
      };
      mediump float tmpvar_46;
      if (tmpvar_44.y) {
        tmpvar_46 = rbmax_40.y;
      } else {
        tmpvar_46 = rbmin_39.y;
      };
      mediump float tmpvar_47;
      if (tmpvar_44.z) {
        tmpvar_47 = rbmax_40.z;
      } else {
        tmpvar_47 = rbmin_39.z;
      };
      worldPos_38 = (tmpvar_12 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_37 = (worldPos_38 + (tmpvar_41 * min (
        min (tmpvar_45, tmpvar_46)
      , tmpvar_47)));
    };
    tmpvar_18 = worldRefl_37;
    mediump vec4 hdr_48;
    hdr_48 = tmpvar_14;
    mediump vec4 tmpvar_49;
    tmpvar_49.xyz = worldRefl_37;
    tmpvar_49.w = ((tmpvar_17 * (1.7 - 
      (0.7 * tmpvar_17)
    )) * 6.0);
    lowp vec4 tmpvar_50;
    tmpvar_50 = textureCube (unity_SpecCube1, worldRefl_37, tmpvar_49.w);
    mediump vec4 tmpvar_51;
    tmpvar_51 = tmpvar_50;
    mediump vec3 tmpvar_52;
    tmpvar_52 = ((tmpvar_51.w * hdr_48.x) * tmpvar_51.xyz);
    highp vec3 tmpvar_53;
    tmpvar_53 = mix (tmpvar_52, tmpvar_36, unity_SpecCube0_BoxMin.www);
    specular_20 = tmpvar_53;
  } else {
    specular_20 = tmpvar_36;
  };
  tmpvar_16 = (specular_20 * tmpvar_10);
  mediump vec3 viewDir_54;
  viewDir_54 = -(xlv_TEXCOORD1);
  mediump float specularTerm_55;
  mediump float tmpvar_56;
  tmpvar_56 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_57;
  mediump vec3 inVec_58;
  inVec_58 = (_WorldSpaceLightPos0.xyz + viewDir_54);
  tmpvar_57 = (inVec_58 * inversesqrt(max (0.001, 
    dot (inVec_58, inVec_58)
  )));
  mediump float tmpvar_59;
  tmpvar_59 = dot (tmpvar_8, viewDir_54);
  mediump vec3 tmpvar_60;
  if ((tmpvar_59 < 0.0)) {
    tmpvar_60 = (tmpvar_8 + (viewDir_54 * (
      -(tmpvar_59)
     + 1e-05)));
  } else {
    tmpvar_60 = tmpvar_8;
  };
  mediump float tmpvar_61;
  tmpvar_61 = clamp (dot (tmpvar_60, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_62;
  tmpvar_62 = clamp (dot (tmpvar_60, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_63;
  tmpvar_63 = clamp (dot (tmpvar_60, viewDir_54), 0.0, 1.0);
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_57), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_66;
  tmpvar_66 = (tmpvar_56 * tmpvar_56);
  mediump float tmpvar_67;
  tmpvar_67 = (tmpvar_66 * tmpvar_66);
  mediump float tmpvar_68;
  tmpvar_68 = (((tmpvar_62 * tmpvar_62) * (tmpvar_67 - 1.0)) + 1.0);
  mediump float x_69;
  x_69 = (1.0 - tmpvar_61);
  mediump float x_70;
  x_70 = (1.0 - tmpvar_63);
  mediump float tmpvar_71;
  tmpvar_71 = (0.5 + ((2.0 * tmpvar_64) * (tmpvar_64 * tmpvar_56)));
  specularTerm_55 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_61 * ((tmpvar_63 * (1.0 - tmpvar_65)) + tmpvar_65))
     + 
      (tmpvar_63 * ((tmpvar_61 * (1.0 - tmpvar_65)) + tmpvar_65))
    ) + 1e-05)) * (tmpvar_67 / ((
      (3.141593 * tmpvar_68)
     * tmpvar_68) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_72;
  tmpvar_72 = max (0.0, (specularTerm_55 * tmpvar_61));
  specularTerm_55 = tmpvar_72;
  mediump float x_73;
  x_73 = (1.0 - tmpvar_64);
  mediump float x_74;
  x_74 = (1.0 - tmpvar_63);
  mediump vec3 tmpvar_75;
  tmpvar_75 = (((tmpvar_5 * 
    (tmpvar_15 + (tmpvar_9 * ((
      (1.0 + ((tmpvar_71 - 1.0) * ((x_69 * x_69) * (
        (x_69 * x_69)
       * x_69))))
     * 
      (1.0 + ((tmpvar_71 - 1.0) * ((x_70 * x_70) * (
        (x_70 * x_70)
       * x_70))))
    ) * tmpvar_61)))
  ) + (
    (tmpvar_72 * tmpvar_9)
   * 
    (tmpvar_6 + ((1.0 - tmpvar_6) * ((x_73 * x_73) * (
      (x_73 * x_73)
     * x_73))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_56) * (tmpvar_56 * tmpvar_56)))
   * tmpvar_16) * mix (tmpvar_6, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_7)), 0.0, 1.0)
  ), vec3(
    ((x_74 * x_74) * ((x_74 * x_74) * x_74))
  ))));
  mediump vec4 tmpvar_76;
  tmpvar_76.w = 1.0;
  tmpvar_76.xyz = tmpvar_75;
  c_1.w = tmpvar_76.w;
  c_1.xyz = tmpvar_75;
  c_1.xyz = c_1.xyz;
  highp float tmpvar_77;
  tmpvar_77 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = mix (unity_FogColor.xyz, c_1.xyz, vec3(tmpvar_77));
  mediump vec4 xlat_varoutput_78;
  xlat_varoutput_78.xyz = c_1.xyz;
  xlat_varoutput_78.w = 1.0;
  gl_FragData[0] = xlat_varoutput_78;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "FOG_EXP" "VERTEXLIGHT_ON" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform highp vec4 unity_4LightPosX0;
uniform highp vec4 unity_4LightPosY0;
uniform highp vec4 unity_4LightPosZ0;
uniform mediump vec4 unity_4LightAtten0;
uniform mediump vec4 unity_LightColor[8];
uniform mediump vec4 unity_SHAr;
uniform mediump vec4 unity_SHAg;
uniform mediump vec4 unity_SHAb;
uniform mediump vec4 unity_SHBr;
uniform mediump vec4 unity_SHBg;
uniform mediump vec4 unity_SHBb;
uniform mediump vec4 unity_SHC;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  highp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  mediump vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  highp vec3 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_9 = tmpvar_10.xyz;
  tmpvar_4 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  tmpvar_5.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_6.xyz = vec3(0.0, 0.0, 0.0);
  tmpvar_7.xyz = tmpvar_19;
  highp vec4 o_20;
  highp vec4 tmpvar_21;
  tmpvar_21 = (tmpvar_4 * 0.5);
  highp vec2 tmpvar_22;
  tmpvar_22.x = tmpvar_21.x;
  tmpvar_22.y = (tmpvar_21.y * _ProjectionParams.x);
  o_20.xy = (tmpvar_22 + tmpvar_21.w);
  o_20.zw = tmpvar_4.zw;
  tmpvar_8 = o_20;
  mediump vec3 normalWorld_23;
  normalWorld_23 = tmpvar_19;
  mediump vec4 ambientOrLightmapUV_24;
  ambientOrLightmapUV_24.w = 0.0;
  highp vec3 lightColor0_25;
  lightColor0_25 = unity_LightColor[0].xyz;
  highp vec3 lightColor1_26;
  lightColor1_26 = unity_LightColor[1].xyz;
  highp vec3 lightColor2_27;
  lightColor2_27 = unity_LightColor[2].xyz;
  highp vec3 lightColor3_28;
  lightColor3_28 = unity_LightColor[3].xyz;
  highp vec4 lightAttenSq_29;
  lightAttenSq_29 = unity_4LightAtten0;
  highp vec3 normal_30;
  normal_30 = normalWorld_23;
  highp vec3 col_31;
  highp vec4 ndotl_32;
  highp vec4 lengthSq_33;
  highp vec4 tmpvar_34;
  tmpvar_34 = (unity_4LightPosX0 - tmpvar_10.x);
  highp vec4 tmpvar_35;
  tmpvar_35 = (unity_4LightPosY0 - tmpvar_10.y);
  highp vec4 tmpvar_36;
  tmpvar_36 = (unity_4LightPosZ0 - tmpvar_10.z);
  lengthSq_33 = (tmpvar_34 * tmpvar_34);
  lengthSq_33 = (lengthSq_33 + (tmpvar_35 * tmpvar_35));
  lengthSq_33 = (lengthSq_33 + (tmpvar_36 * tmpvar_36));
  ndotl_32 = (tmpvar_34 * normal_30.x);
  ndotl_32 = (ndotl_32 + (tmpvar_35 * normal_30.y));
  ndotl_32 = (ndotl_32 + (tmpvar_36 * normal_30.z));
  highp vec4 tmpvar_37;
  tmpvar_37 = max (vec4(0.0, 0.0, 0.0, 0.0), (ndotl_32 * inversesqrt(lengthSq_33)));
  ndotl_32 = tmpvar_37;
  highp vec4 tmpvar_38;
  tmpvar_38 = (tmpvar_37 * (1.0/((1.0 + 
    (lengthSq_33 * lightAttenSq_29)
  ))));
  col_31 = (lightColor0_25 * tmpvar_38.x);
  col_31 = (col_31 + (lightColor1_26 * tmpvar_38.y));
  col_31 = (col_31 + (lightColor2_27 * tmpvar_38.z));
  col_31 = (col_31 + (lightColor3_28 * tmpvar_38.w));
  ambientOrLightmapUV_24.xyz = col_31;
  mediump vec4 tmpvar_39;
  tmpvar_39.w = 1.0;
  tmpvar_39.xyz = normalWorld_23;
  mediump vec3 res_40;
  mediump vec3 x_41;
  x_41.x = dot (unity_SHAr, tmpvar_39);
  x_41.y = dot (unity_SHAg, tmpvar_39);
  x_41.z = dot (unity_SHAb, tmpvar_39);
  mediump vec3 x1_42;
  mediump vec4 tmpvar_43;
  tmpvar_43 = (normalWorld_23.xyzz * normalWorld_23.yzzx);
  x1_42.x = dot (unity_SHBr, tmpvar_43);
  x1_42.y = dot (unity_SHBg, tmpvar_43);
  x1_42.z = dot (unity_SHBb, tmpvar_43);
  res_40 = (x_41 + (x1_42 + (unity_SHC.xyz * 
    ((normalWorld_23.x * normalWorld_23.x) - (normalWorld_23.y * normalWorld_23.y))
  )));
  res_40 = max (((1.055 * 
    pow (max (res_40, vec3(0.0, 0.0, 0.0)), vec3(0.4166667, 0.4166667, 0.4166667))
  ) - 0.055), vec3(0.0, 0.0, 0.0));
  ambientOrLightmapUV_24.xyz = (ambientOrLightmapUV_24.xyz + max (vec3(0.0, 0.0, 0.0), res_40));
  gl_Position = tmpvar_4;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_5;
  xlv_TEXCOORD2_1 = tmpvar_6;
  xlv_TEXCOORD2_2 = tmpvar_7;
  xlv_TEXCOORD5 = ambientOrLightmapUV_24;
  xlv_TEXCOORD6 = tmpvar_8;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_4.z)));
  xlv_TEXCOORD8 = tmpvar_9;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _WorldSpaceLightPos0;
uniform lowp vec4 unity_FogColor;
uniform lowp samplerCube unity_SpecCube0;
uniform lowp samplerCube unity_SpecCube1;
uniform highp vec4 unity_SpecCube0_BoxMax;
uniform highp vec4 unity_SpecCube0_BoxMin;
uniform highp vec4 unity_SpecCube0_ProbePosition;
uniform mediump vec4 unity_SpecCube0_HDR;
uniform highp vec4 unity_SpecCube1_BoxMax;
uniform highp vec4 unity_SpecCube1_BoxMin;
uniform highp vec4 unity_SpecCube1_ProbePosition;
uniform mediump vec4 unity_SpecCube1_HDR;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _OcclusionMap;
uniform sampler2D _ShadowMapTexture;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
varying highp vec3 xlv_TEXCOORD8;
void main ()
{
  mediump vec4 c_1;
  mediump float atten_2;
  mediump vec3 i_posWorld_3;
  i_posWorld_3 = xlv_TEXCOORD8;
  lowp vec4 tmpvar_4;
  tmpvar_4 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_5;
  tmpvar_5 = (_Color.xyz * tmpvar_4.xyz);
  mediump vec3 tmpvar_6;
  mediump vec3 tmpvar_7;
  tmpvar_7 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_5, vec3(_Metallic));
  mediump float tmpvar_8;
  tmpvar_8 = (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w));
  tmpvar_6 = (tmpvar_5 * tmpvar_8);
  mediump vec3 tmpvar_9;
  tmpvar_9 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  lowp float tmpvar_11;
  tmpvar_11 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD6).x;
  atten_2 = tmpvar_11;
  mediump float tmpvar_12;
  lowp vec4 tmpvar_13;
  tmpvar_13 = texture2D (_OcclusionMap, xlv_TEXCOORD0.xy);
  tmpvar_12 = tmpvar_13.y;
  highp vec3 tmpvar_14;
  highp vec4 tmpvar_15;
  highp vec4 tmpvar_16;
  tmpvar_14 = i_posWorld_3;
  tmpvar_15 = unity_SpecCube0_HDR;
  tmpvar_16 = unity_SpecCube1_HDR;
  mediump vec3 tmpvar_17;
  mediump vec3 tmpvar_18;
  mediump vec3 tmpvar_19;
  tmpvar_17 = (tmpvar_10 * atten_2);
  tmpvar_18 = (xlv_TEXCOORD5.xyz * tmpvar_12);
  tmpvar_19 = vec3(0.0, 0.0, 0.0);
  mediump float tmpvar_20;
  mediump vec3 tmpvar_21;
  tmpvar_20 = (1.0 - _Glossiness);
  tmpvar_21 = (xlv_TEXCOORD1 - (2.0 * (
    dot (tmpvar_9, xlv_TEXCOORD1)
   * tmpvar_9)));
  mediump vec3 originalReflUVW_22;
  mediump vec3 specular_23;
  originalReflUVW_22 = tmpvar_21;
  mediump vec3 worldRefl_24;
  worldRefl_24 = tmpvar_21;
  highp vec3 worldPos_25;
  worldPos_25 = tmpvar_14;
  if ((unity_SpecCube0_ProbePosition.w > 0.0)) {
    mediump vec3 rbmin_26;
    mediump vec3 rbmax_27;
    mediump vec3 tmpvar_28;
    tmpvar_28 = normalize(tmpvar_21);
    highp vec3 tmpvar_29;
    tmpvar_29 = ((unity_SpecCube0_BoxMax.xyz - tmpvar_14) / tmpvar_28);
    rbmax_27 = tmpvar_29;
    highp vec3 tmpvar_30;
    tmpvar_30 = ((unity_SpecCube0_BoxMin.xyz - tmpvar_14) / tmpvar_28);
    rbmin_26 = tmpvar_30;
    bvec3 tmpvar_31;
    tmpvar_31 = greaterThan (tmpvar_28, vec3(0.0, 0.0, 0.0));
    mediump float tmpvar_32;
    if (tmpvar_31.x) {
      tmpvar_32 = rbmax_27.x;
    } else {
      tmpvar_32 = rbmin_26.x;
    };
    mediump float tmpvar_33;
    if (tmpvar_31.y) {
      tmpvar_33 = rbmax_27.y;
    } else {
      tmpvar_33 = rbmin_26.y;
    };
    mediump float tmpvar_34;
    if (tmpvar_31.z) {
      tmpvar_34 = rbmax_27.z;
    } else {
      tmpvar_34 = rbmin_26.z;
    };
    worldPos_25 = (tmpvar_14 - unity_SpecCube0_ProbePosition.xyz);
    worldRefl_24 = (worldPos_25 + (tmpvar_28 * min (
      min (tmpvar_32, tmpvar_33)
    , tmpvar_34)));
  };
  tmpvar_21 = worldRefl_24;
  mediump vec4 hdr_35;
  hdr_35 = tmpvar_15;
  mediump vec4 tmpvar_36;
  tmpvar_36.xyz = worldRefl_24;
  tmpvar_36.w = ((tmpvar_20 * (1.7 - 
    (0.7 * tmpvar_20)
  )) * 6.0);
  lowp vec4 tmpvar_37;
  tmpvar_37 = textureCube (unity_SpecCube0, worldRefl_24, tmpvar_36.w);
  mediump vec4 tmpvar_38;
  tmpvar_38 = tmpvar_37;
  mediump vec3 tmpvar_39;
  tmpvar_39 = ((tmpvar_38.w * hdr_35.x) * tmpvar_38.xyz);
  if ((unity_SpecCube0_BoxMin.w < 0.99999)) {
    mediump vec3 worldRefl_40;
    worldRefl_40 = originalReflUVW_22;
    highp vec3 worldPos_41;
    worldPos_41 = tmpvar_14;
    if ((unity_SpecCube1_ProbePosition.w > 0.0)) {
      mediump vec3 rbmin_42;
      mediump vec3 rbmax_43;
      mediump vec3 tmpvar_44;
      tmpvar_44 = normalize(originalReflUVW_22);
      highp vec3 tmpvar_45;
      tmpvar_45 = ((unity_SpecCube1_BoxMax.xyz - tmpvar_14) / tmpvar_44);
      rbmax_43 = tmpvar_45;
      highp vec3 tmpvar_46;
      tmpvar_46 = ((unity_SpecCube1_BoxMin.xyz - tmpvar_14) / tmpvar_44);
      rbmin_42 = tmpvar_46;
      bvec3 tmpvar_47;
      tmpvar_47 = greaterThan (tmpvar_44, vec3(0.0, 0.0, 0.0));
      mediump float tmpvar_48;
      if (tmpvar_47.x) {
        tmpvar_48 = rbmax_43.x;
      } else {
        tmpvar_48 = rbmin_42.x;
      };
      mediump float tmpvar_49;
      if (tmpvar_47.y) {
        tmpvar_49 = rbmax_43.y;
      } else {
        tmpvar_49 = rbmin_42.y;
      };
      mediump float tmpvar_50;
      if (tmpvar_47.z) {
        tmpvar_50 = rbmax_43.z;
      } else {
        tmpvar_50 = rbmin_42.z;
      };
      worldPos_41 = (tmpvar_14 - unity_SpecCube1_ProbePosition.xyz);
      worldRefl_40 = (worldPos_41 + (tmpvar_44 * min (
        min (tmpvar_48, tmpvar_49)
      , tmpvar_50)));
    };
    tmpvar_21 = worldRefl_40;
    mediump vec4 hdr_51;
    hdr_51 = tmpvar_16;
    mediump vec4 tmpvar_52;
    tmpvar_52.xyz = worldRefl_40;
    tmpvar_52.w = ((tmpvar_20 * (1.7 - 
      (0.7 * tmpvar_20)
    )) * 6.0);
    lowp vec4 tmpvar_53;
    tmpvar_53 = textureCube (unity_SpecCube1, worldRefl_40, tmpvar_52.w);
    mediump vec4 tmpvar_54;
    tmpvar_54 = tmpvar_53;
    mediump vec3 tmpvar_55;
    tmpvar_55 = ((tmpvar_54.w * hdr_51.x) * tmpvar_54.xyz);
    highp vec3 tmpvar_56;
    tmpvar_56 = mix (tmpvar_55, tmpvar_39, unity_SpecCube0_BoxMin.www);
    specular_23 = tmpvar_56;
  } else {
    specular_23 = tmpvar_39;
  };
  tmpvar_19 = (specular_23 * tmpvar_12);
  mediump vec3 viewDir_57;
  viewDir_57 = -(xlv_TEXCOORD1);
  mediump float specularTerm_58;
  mediump float tmpvar_59;
  tmpvar_59 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_60;
  mediump vec3 inVec_61;
  inVec_61 = (_WorldSpaceLightPos0.xyz + viewDir_57);
  tmpvar_60 = (inVec_61 * inversesqrt(max (0.001, 
    dot (inVec_61, inVec_61)
  )));
  mediump float tmpvar_62;
  tmpvar_62 = dot (tmpvar_9, viewDir_57);
  mediump vec3 tmpvar_63;
  if ((tmpvar_62 < 0.0)) {
    tmpvar_63 = (tmpvar_9 + (viewDir_57 * (
      -(tmpvar_62)
     + 1e-05)));
  } else {
    tmpvar_63 = tmpvar_9;
  };
  mediump float tmpvar_64;
  tmpvar_64 = clamp (dot (tmpvar_63, _WorldSpaceLightPos0.xyz), 0.0, 1.0);
  mediump float tmpvar_65;
  tmpvar_65 = clamp (dot (tmpvar_63, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_66;
  tmpvar_66 = clamp (dot (tmpvar_63, viewDir_57), 0.0, 1.0);
  mediump float tmpvar_67;
  tmpvar_67 = clamp (dot (_WorldSpaceLightPos0.xyz, tmpvar_60), 0.0, 1.0);
  mediump float tmpvar_68;
  tmpvar_68 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_69;
  tmpvar_69 = (tmpvar_59 * tmpvar_59);
  mediump float tmpvar_70;
  tmpvar_70 = (tmpvar_69 * tmpvar_69);
  mediump float tmpvar_71;
  tmpvar_71 = (((tmpvar_65 * tmpvar_65) * (tmpvar_70 - 1.0)) + 1.0);
  mediump float x_72;
  x_72 = (1.0 - tmpvar_64);
  mediump float x_73;
  x_73 = (1.0 - tmpvar_66);
  mediump float tmpvar_74;
  tmpvar_74 = (0.5 + ((2.0 * tmpvar_67) * (tmpvar_67 * tmpvar_59)));
  specularTerm_58 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_64 * ((tmpvar_66 * (1.0 - tmpvar_68)) + tmpvar_68))
     + 
      (tmpvar_66 * ((tmpvar_64 * (1.0 - tmpvar_68)) + tmpvar_68))
    ) + 1e-05)) * (tmpvar_70 / ((
      (3.141593 * tmpvar_71)
     * tmpvar_71) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_75;
  tmpvar_75 = max (0.0, (specularTerm_58 * tmpvar_64));
  specularTerm_58 = tmpvar_75;
  mediump float x_76;
  x_76 = (1.0 - tmpvar_67);
  mediump float x_77;
  x_77 = (1.0 - tmpvar_66);
  mediump vec3 tmpvar_78;
  tmpvar_78 = (((tmpvar_6 * 
    (tmpvar_18 + (tmpvar_17 * ((
      (1.0 + ((tmpvar_74 - 1.0) * ((x_72 * x_72) * (
        (x_72 * x_72)
       * x_72))))
     * 
      (1.0 + ((tmpvar_74 - 1.0) * ((x_73 * x_73) * (
        (x_73 * x_73)
       * x_73))))
    ) * tmpvar_64)))
  ) + (
    (tmpvar_75 * tmpvar_17)
   * 
    (tmpvar_7 + ((1.0 - tmpvar_7) * ((x_76 * x_76) * (
      (x_76 * x_76)
     * x_76))))
  )) + ((
    (1.0 - ((0.28 * tmpvar_59) * (tmpvar_59 * tmpvar_59)))
   * tmpvar_19) * mix (tmpvar_7, vec3(
    clamp ((_Glossiness + (1.0 - tmpvar_8)), 0.0, 1.0)
  ), vec3(
    ((x_77 * x_77) * ((x_77 * x_77) * x_77))
  ))));
  mediump vec4 tmpvar_79;
  tmpvar_79.w = 1.0;
  tmpvar_79.xyz = tmpvar_78;
  c_1.w = tmpvar_79.w;
  c_1.xyz = tmpvar_78;
  c_1.xyz = c_1.xyz;
  highp float tmpvar_80;
  tmpvar_80 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = mix (unity_FogColor.xyz, c_1.xyz, vec3(tmpvar_80));
  mediump vec4 xlat_varoutput_81;
  xlat_varoutput_81.xyz = c_1.xyz;
  xlat_varoutput_81.w = 1.0;
  gl_FragData[0] = xlat_varoutput_81;
}


#endif
"
}
}
Program "fp" {
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "LIGHTMAP_OFF" "DIRLIGHTMAP_OFF" "DYNAMICLIGHTMAP_OFF" "FOG_EXP" }
""
}
}
 }
 Pass {
  Name "FORWARD_DELTA"
  Tags { "LIGHTMODE"="ForwardAdd" "SHADOWSUPPORT"="true" "RenderType"="Opaque" "PerformanceChecks"="False" }
  ZWrite Off
  Blend [_SrcBlend] One
  GpuProgramID 443686
Program "vp" {
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xyz;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_7;
  tmpvar_7 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2D (_LightTexture0, vec2(tmpvar_7));
  mediump float atten_9;
  atten_9 = tmpvar_8.w;
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  tmpvar_10 = (tmpvar_10 * atten_9);
  mediump vec3 viewDir_11;
  viewDir_11 = -(xlv_TEXCOORD1);
  mediump float specularTerm_12;
  mediump float tmpvar_13;
  tmpvar_13 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_14;
  mediump vec3 inVec_15;
  inVec_15 = (tmpvar_6 + viewDir_11);
  tmpvar_14 = (inVec_15 * inversesqrt(max (0.001, 
    dot (inVec_15, inVec_15)
  )));
  mediump float tmpvar_16;
  tmpvar_16 = dot (tmpvar_5, viewDir_11);
  mediump vec3 tmpvar_17;
  if ((tmpvar_16 < 0.0)) {
    tmpvar_17 = (tmpvar_5 + (viewDir_11 * (
      -(tmpvar_16)
     + 1e-05)));
  } else {
    tmpvar_17 = tmpvar_5;
  };
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_17, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_17, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_17, viewDir_11), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_6, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_23 * tmpvar_23);
  mediump float tmpvar_25;
  tmpvar_25 = (((tmpvar_19 * tmpvar_19) * (tmpvar_24 - 1.0)) + 1.0);
  mediump float x_26;
  x_26 = (1.0 - tmpvar_18);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_20);
  mediump float tmpvar_28;
  tmpvar_28 = (0.5 + ((2.0 * tmpvar_21) * (tmpvar_21 * tmpvar_13)));
  specularTerm_12 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_18 * ((tmpvar_20 * (1.0 - tmpvar_22)) + tmpvar_22))
     + 
      (tmpvar_20 * ((tmpvar_18 * (1.0 - tmpvar_22)) + tmpvar_22))
    ) + 1e-05)) * (tmpvar_24 / ((
      (3.141593 * tmpvar_25)
     * tmpvar_25) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_29;
  tmpvar_29 = max (0.0, (specularTerm_12 * tmpvar_18));
  specularTerm_12 = tmpvar_29;
  mediump float x_30;
  x_30 = (1.0 - tmpvar_21);
  mediump vec4 tmpvar_31;
  tmpvar_31.w = 1.0;
  tmpvar_31.xyz = ((tmpvar_3 * (tmpvar_10 * 
    (((1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_26 * x_26) * ((x_26 * x_26) * x_26))
    )) * (1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    ))) * tmpvar_18)
  )) + ((tmpvar_29 * tmpvar_10) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_30 * x_30) * ((x_30 * x_30) * x_30)))
  )));
  mediump vec4 xlat_varoutput_32;
  xlat_varoutput_32.xyz = tmpvar_31.xyz;
  xlat_varoutput_32.w = 1.0;
  gl_FragData[0] = xlat_varoutput_32;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  highp vec3 dir_20;
  dir_20 = tmpvar_4.xyz;
  highp mat3 tmpvar_21;
  tmpvar_21[0] = _Object2World[0].xyz;
  tmpvar_21[1] = _Object2World[1].xyz;
  tmpvar_21[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_22;
  tmpvar_22.xyz = normalize((tmpvar_21 * dir_20));
  tmpvar_22.w = tmpvar_4.w;
  mediump vec3 normal_23;
  normal_23 = tmpvar_19;
  mediump vec3 tangent_24;
  tangent_24 = tmpvar_22.xyz;
  mediump float tangentSign_25;
  tangentSign_25 = tmpvar_22.w;
  mediump float xlat_varsign_26;
  highp float tmpvar_27;
  tmpvar_27 = (tangentSign_25 * unity_WorldTransformParams.w);
  xlat_varsign_26 = tmpvar_27;
  mediump vec3 tmpvar_28;
  tmpvar_28 = (((normal_23.yzx * tangent_24.zxy) - (normal_23.zxy * tangent_24.yzx)) * xlat_varsign_26);
  mediump mat3 tmpvar_29;
  tmpvar_29[0].x = tangent_24.x;
  tmpvar_29[0].y = tmpvar_28.x;
  tmpvar_29[0].z = normal_23.x;
  tmpvar_29[1].x = tangent_24.y;
  tmpvar_29[1].y = tmpvar_28.y;
  tmpvar_29[1].z = normal_23.y;
  tmpvar_29[2].x = tangent_24.z;
  tmpvar_29[2].y = tmpvar_28.z;
  tmpvar_29[2].z = normal_23.z;
  tangentToWorld_5 = tmpvar_29;
  highp vec3 v_30;
  v_30.x = tangentToWorld_5[0].x;
  v_30.y = tangentToWorld_5[1].x;
  v_30.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].y;
  v_31.y = tangentToWorld_5[1].y;
  v_31.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].z;
  v_32.y = tangentToWorld_5[1].z;
  v_32.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_32;
  highp vec3 tmpvar_33;
  tmpvar_33 = (_WorldSpaceLightPos0.xyz - (tmpvar_10.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_33.x;
  tmpvar_8.w = tmpvar_33.y;
  tmpvar_9.w = tmpvar_33.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  mediump vec3 tmpvar_7;
  tmpvar_7 = _LightColor0.xyz;
  mediump vec3 viewDir_8;
  viewDir_8 = -(xlv_TEXCOORD1);
  mediump float specularTerm_9;
  mediump float tmpvar_10;
  tmpvar_10 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_11;
  mediump vec3 inVec_12;
  inVec_12 = (tmpvar_6 + viewDir_8);
  tmpvar_11 = (inVec_12 * inversesqrt(max (0.001, 
    dot (inVec_12, inVec_12)
  )));
  mediump float tmpvar_13;
  tmpvar_13 = dot (tmpvar_5, viewDir_8);
  mediump vec3 tmpvar_14;
  if ((tmpvar_13 < 0.0)) {
    tmpvar_14 = (tmpvar_5 + (viewDir_8 * (
      -(tmpvar_13)
     + 1e-05)));
  } else {
    tmpvar_14 = tmpvar_5;
  };
  mediump float tmpvar_15;
  tmpvar_15 = clamp (dot (tmpvar_14, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_16;
  tmpvar_16 = clamp (dot (tmpvar_14, tmpvar_11), 0.0, 1.0);
  mediump float tmpvar_17;
  tmpvar_17 = clamp (dot (tmpvar_14, viewDir_8), 0.0, 1.0);
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_6, tmpvar_11), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = (tmpvar_10 * tmpvar_10);
  mediump float tmpvar_20;
  tmpvar_20 = (tmpvar_10 * tmpvar_10);
  mediump float tmpvar_21;
  tmpvar_21 = (tmpvar_20 * tmpvar_20);
  mediump float tmpvar_22;
  tmpvar_22 = (((tmpvar_16 * tmpvar_16) * (tmpvar_21 - 1.0)) + 1.0);
  mediump float x_23;
  x_23 = (1.0 - tmpvar_15);
  mediump float x_24;
  x_24 = (1.0 - tmpvar_17);
  mediump float tmpvar_25;
  tmpvar_25 = (0.5 + ((2.0 * tmpvar_18) * (tmpvar_18 * tmpvar_10)));
  specularTerm_9 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_15 * ((tmpvar_17 * (1.0 - tmpvar_19)) + tmpvar_19))
     + 
      (tmpvar_17 * ((tmpvar_15 * (1.0 - tmpvar_19)) + tmpvar_19))
    ) + 1e-05)) * (tmpvar_21 / ((
      (3.141593 * tmpvar_22)
     * tmpvar_22) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_26;
  tmpvar_26 = max (0.0, (specularTerm_9 * tmpvar_15));
  specularTerm_9 = tmpvar_26;
  mediump float x_27;
  x_27 = (1.0 - tmpvar_18);
  mediump vec4 tmpvar_28;
  tmpvar_28.w = 1.0;
  tmpvar_28.xyz = ((tmpvar_3 * (tmpvar_7 * 
    (((1.0 + (
      (tmpvar_25 - 1.0)
     * 
      ((x_23 * x_23) * ((x_23 * x_23) * x_23))
    )) * (1.0 + (
      (tmpvar_25 - 1.0)
     * 
      ((x_24 * x_24) * ((x_24 * x_24) * x_24))
    ))) * tmpvar_15)
  )) + ((tmpvar_26 * tmpvar_7) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_27 * x_27) * ((x_27 * x_27) * x_27)))
  )));
  mediump vec4 xlat_varoutput_29;
  xlat_varoutput_29.xyz = tmpvar_28.xyz;
  xlat_varoutput_29.w = 1.0;
  gl_FragData[0] = xlat_varoutput_29;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  mediump vec4 tmpvar_11;
  highp vec4 tmpvar_12;
  tmpvar_12 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_13;
  texcoord_13.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_14;
  if ((_UVSec == 0.0)) {
    tmpvar_14 = tmpvar_2;
  } else {
    tmpvar_14 = tmpvar_3;
  };
  texcoord_13.zw = ((tmpvar_14 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_15;
  highp vec3 tmpvar_16;
  tmpvar_16 = normalize((tmpvar_12.xyz - _WorldSpaceCameraPos));
  tmpvar_15 = tmpvar_16;
  highp vec3 norm_17;
  norm_17 = tmpvar_1;
  highp vec4 v_18;
  v_18.x = _World2Object[0].x;
  v_18.y = _World2Object[1].x;
  v_18.z = _World2Object[2].x;
  v_18.w = _World2Object[3].x;
  highp vec4 v_19;
  v_19.x = _World2Object[0].y;
  v_19.y = _World2Object[1].y;
  v_19.z = _World2Object[2].y;
  v_19.w = _World2Object[3].y;
  highp vec4 v_20;
  v_20.x = _World2Object[0].z;
  v_20.y = _World2Object[1].z;
  v_20.z = _World2Object[2].z;
  v_20.w = _World2Object[3].z;
  highp vec3 tmpvar_21;
  tmpvar_21 = normalize(((
    (v_18.xyz * norm_17.x)
   + 
    (v_19.xyz * norm_17.y)
  ) + (v_20.xyz * norm_17.z)));
  highp vec3 dir_22;
  dir_22 = tmpvar_4.xyz;
  highp mat3 tmpvar_23;
  tmpvar_23[0] = _Object2World[0].xyz;
  tmpvar_23[1] = _Object2World[1].xyz;
  tmpvar_23[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_24;
  tmpvar_24.xyz = normalize((tmpvar_23 * dir_22));
  tmpvar_24.w = tmpvar_4.w;
  mediump vec3 normal_25;
  normal_25 = tmpvar_21;
  mediump vec3 tangent_26;
  tangent_26 = tmpvar_24.xyz;
  mediump float tangentSign_27;
  tangentSign_27 = tmpvar_24.w;
  mediump float xlat_varsign_28;
  highp float tmpvar_29;
  tmpvar_29 = (tangentSign_27 * unity_WorldTransformParams.w);
  xlat_varsign_28 = tmpvar_29;
  mediump vec3 tmpvar_30;
  tmpvar_30 = (((normal_25.yzx * tangent_26.zxy) - (normal_25.zxy * tangent_26.yzx)) * xlat_varsign_28);
  mediump mat3 tmpvar_31;
  tmpvar_31[0].x = tangent_26.x;
  tmpvar_31[0].y = tmpvar_30.x;
  tmpvar_31[0].z = normal_25.x;
  tmpvar_31[1].x = tangent_26.y;
  tmpvar_31[1].y = tmpvar_30.y;
  tmpvar_31[1].z = normal_25.y;
  tmpvar_31[2].x = tangent_26.z;
  tmpvar_31[2].y = tmpvar_30.z;
  tmpvar_31[2].z = normal_25.z;
  tangentToWorld_6 = tmpvar_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].x;
  v_32.y = tangentToWorld_6[1].x;
  v_32.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].y;
  v_33.y = tangentToWorld_6[1].y;
  v_33.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_33;
  highp vec3 v_34;
  v_34.x = tangentToWorld_6[0].z;
  v_34.y = tangentToWorld_6[1].z;
  v_34.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_34;
  tmpvar_11 = (_LightMatrix0 * (_Object2World * _glesVertex));
  mediump vec3 tmpvar_35;
  highp vec3 tmpvar_36;
  tmpvar_36 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_12.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_35 = tmpvar_36;
  lightDir_5 = tmpvar_35;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_13;
  xlv_TEXCOORD1 = tmpvar_15;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = tmpvar_11;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_7;
  mediump vec2 P_8;
  P_8 = ((xlv_TEXCOORD5.xy / xlv_TEXCOORD5.w) + 0.5);
  tmpvar_7 = texture2D (_LightTexture0, P_8);
  highp vec3 LightCoord_9;
  LightCoord_9 = xlv_TEXCOORD5.xyz;
  highp float tmpvar_10;
  tmpvar_10 = dot (LightCoord_9, LightCoord_9);
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_LightTextureB0, vec2(tmpvar_10));
  mediump vec3 tmpvar_12;
  tmpvar_12 = _LightColor0.xyz;
  tmpvar_12 = (tmpvar_12 * ((
    float((xlv_TEXCOORD5.z > 0.0))
   * tmpvar_7.w) * tmpvar_11.w));
  mediump vec3 viewDir_13;
  viewDir_13 = -(xlv_TEXCOORD1);
  mediump float specularTerm_14;
  mediump float tmpvar_15;
  tmpvar_15 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_16;
  mediump vec3 inVec_17;
  inVec_17 = (tmpvar_6 + viewDir_13);
  tmpvar_16 = (inVec_17 * inversesqrt(max (0.001, 
    dot (inVec_17, inVec_17)
  )));
  mediump float tmpvar_18;
  tmpvar_18 = dot (tmpvar_5, viewDir_13);
  mediump vec3 tmpvar_19;
  if ((tmpvar_18 < 0.0)) {
    tmpvar_19 = (tmpvar_5 + (viewDir_13 * (
      -(tmpvar_18)
     + 1e-05)));
  } else {
    tmpvar_19 = tmpvar_5;
  };
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_19, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_19, tmpvar_16), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_19, viewDir_13), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_6, tmpvar_16), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_15 * tmpvar_15);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_15 * tmpvar_15);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_25 * tmpvar_25);
  mediump float tmpvar_27;
  tmpvar_27 = (((tmpvar_21 * tmpvar_21) * (tmpvar_26 - 1.0)) + 1.0);
  mediump float x_28;
  x_28 = (1.0 - tmpvar_20);
  mediump float x_29;
  x_29 = (1.0 - tmpvar_22);
  mediump float tmpvar_30;
  tmpvar_30 = (0.5 + ((2.0 * tmpvar_23) * (tmpvar_23 * tmpvar_15)));
  specularTerm_14 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_20 * ((tmpvar_22 * (1.0 - tmpvar_24)) + tmpvar_24))
     + 
      (tmpvar_22 * ((tmpvar_20 * (1.0 - tmpvar_24)) + tmpvar_24))
    ) + 1e-05)) * (tmpvar_26 / ((
      (3.141593 * tmpvar_27)
     * tmpvar_27) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_31;
  tmpvar_31 = max (0.0, (specularTerm_14 * tmpvar_20));
  specularTerm_14 = tmpvar_31;
  mediump float x_32;
  x_32 = (1.0 - tmpvar_23);
  mediump vec4 tmpvar_33;
  tmpvar_33.w = 1.0;
  tmpvar_33.xyz = ((tmpvar_3 * (tmpvar_12 * 
    (((1.0 + (
      (tmpvar_30 - 1.0)
     * 
      ((x_28 * x_28) * ((x_28 * x_28) * x_28))
    )) * (1.0 + (
      (tmpvar_30 - 1.0)
     * 
      ((x_29 * x_29) * ((x_29 * x_29) * x_29))
    ))) * tmpvar_20)
  )) + ((tmpvar_31 * tmpvar_12) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_32 * x_32) * ((x_32 * x_32) * x_32)))
  )));
  mediump vec4 xlat_varoutput_34;
  xlat_varoutput_34.xyz = tmpvar_33.xyz;
  xlat_varoutput_34.w = 1.0;
  gl_FragData[0] = xlat_varoutput_34;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xyz;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform lowp samplerCube _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_7;
  tmpvar_7 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2D (_LightTextureB0, vec2(tmpvar_7));
  lowp vec4 tmpvar_9;
  tmpvar_9 = textureCube (_LightTexture0, xlv_TEXCOORD5);
  mediump float atten_10;
  atten_10 = (tmpvar_8.w * tmpvar_9.w);
  mediump vec3 tmpvar_11;
  tmpvar_11 = _LightColor0.xyz;
  tmpvar_11 = (tmpvar_11 * atten_10);
  mediump vec3 viewDir_12;
  viewDir_12 = -(xlv_TEXCOORD1);
  mediump float specularTerm_13;
  mediump float tmpvar_14;
  tmpvar_14 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_15;
  mediump vec3 inVec_16;
  inVec_16 = (tmpvar_6 + viewDir_12);
  tmpvar_15 = (inVec_16 * inversesqrt(max (0.001, 
    dot (inVec_16, inVec_16)
  )));
  mediump float tmpvar_17;
  tmpvar_17 = dot (tmpvar_5, viewDir_12);
  mediump vec3 tmpvar_18;
  if ((tmpvar_17 < 0.0)) {
    tmpvar_18 = (tmpvar_5 + (viewDir_12 * (
      -(tmpvar_17)
     + 1e-05)));
  } else {
    tmpvar_18 = tmpvar_5;
  };
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_18, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_18, tmpvar_15), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_18, viewDir_12), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_6, tmpvar_15), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_14 * tmpvar_14);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_14 * tmpvar_14);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_24 * tmpvar_24);
  mediump float tmpvar_26;
  tmpvar_26 = (((tmpvar_20 * tmpvar_20) * (tmpvar_25 - 1.0)) + 1.0);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_19);
  mediump float x_28;
  x_28 = (1.0 - tmpvar_21);
  mediump float tmpvar_29;
  tmpvar_29 = (0.5 + ((2.0 * tmpvar_22) * (tmpvar_22 * tmpvar_14)));
  specularTerm_13 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_19 * ((tmpvar_21 * (1.0 - tmpvar_23)) + tmpvar_23))
     + 
      (tmpvar_21 * ((tmpvar_19 * (1.0 - tmpvar_23)) + tmpvar_23))
    ) + 1e-05)) * (tmpvar_25 / ((
      (3.141593 * tmpvar_26)
     * tmpvar_26) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_30;
  tmpvar_30 = max (0.0, (specularTerm_13 * tmpvar_19));
  specularTerm_13 = tmpvar_30;
  mediump float x_31;
  x_31 = (1.0 - tmpvar_22);
  mediump vec4 tmpvar_32;
  tmpvar_32.w = 1.0;
  tmpvar_32.xyz = ((tmpvar_3 * (tmpvar_11 * 
    (((1.0 + (
      (tmpvar_29 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    )) * (1.0 + (
      (tmpvar_29 - 1.0)
     * 
      ((x_28 * x_28) * ((x_28 * x_28) * x_28))
    ))) * tmpvar_19)
  )) + ((tmpvar_30 * tmpvar_11) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_31 * x_31) * ((x_31 * x_31) * x_31)))
  )));
  mediump vec4 xlat_varoutput_33;
  xlat_varoutput_33.xyz = tmpvar_32.xyz;
  xlat_varoutput_33.w = 1.0;
  gl_FragData[0] = xlat_varoutput_33;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  highp vec3 dir_20;
  dir_20 = tmpvar_4.xyz;
  highp mat3 tmpvar_21;
  tmpvar_21[0] = _Object2World[0].xyz;
  tmpvar_21[1] = _Object2World[1].xyz;
  tmpvar_21[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_22;
  tmpvar_22.xyz = normalize((tmpvar_21 * dir_20));
  tmpvar_22.w = tmpvar_4.w;
  mediump vec3 normal_23;
  normal_23 = tmpvar_19;
  mediump vec3 tangent_24;
  tangent_24 = tmpvar_22.xyz;
  mediump float tangentSign_25;
  tangentSign_25 = tmpvar_22.w;
  mediump float xlat_varsign_26;
  highp float tmpvar_27;
  tmpvar_27 = (tangentSign_25 * unity_WorldTransformParams.w);
  xlat_varsign_26 = tmpvar_27;
  mediump vec3 tmpvar_28;
  tmpvar_28 = (((normal_23.yzx * tangent_24.zxy) - (normal_23.zxy * tangent_24.yzx)) * xlat_varsign_26);
  mediump mat3 tmpvar_29;
  tmpvar_29[0].x = tangent_24.x;
  tmpvar_29[0].y = tmpvar_28.x;
  tmpvar_29[0].z = normal_23.x;
  tmpvar_29[1].x = tangent_24.y;
  tmpvar_29[1].y = tmpvar_28.y;
  tmpvar_29[1].z = normal_23.y;
  tmpvar_29[2].x = tangent_24.z;
  tmpvar_29[2].y = tmpvar_28.z;
  tmpvar_29[2].z = normal_23.z;
  tangentToWorld_5 = tmpvar_29;
  highp vec3 v_30;
  v_30.x = tangentToWorld_5[0].x;
  v_30.y = tangentToWorld_5[1].x;
  v_30.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].y;
  v_31.y = tangentToWorld_5[1].y;
  v_31.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].z;
  v_32.y = tangentToWorld_5[1].z;
  v_32.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_32;
  highp vec3 tmpvar_33;
  tmpvar_33 = (_WorldSpaceLightPos0.xyz - (tmpvar_10.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_33.x;
  tmpvar_8.w = tmpvar_33.y;
  tmpvar_9.w = tmpvar_33.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xy;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_7;
  tmpvar_7 = texture2D (_LightTexture0, xlv_TEXCOORD5);
  mediump float atten_8;
  atten_8 = tmpvar_7.w;
  mediump vec3 tmpvar_9;
  tmpvar_9 = _LightColor0.xyz;
  tmpvar_9 = (tmpvar_9 * atten_8);
  mediump vec3 viewDir_10;
  viewDir_10 = -(xlv_TEXCOORD1);
  mediump float specularTerm_11;
  mediump float tmpvar_12;
  tmpvar_12 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_13;
  mediump vec3 inVec_14;
  inVec_14 = (tmpvar_6 + viewDir_10);
  tmpvar_13 = (inVec_14 * inversesqrt(max (0.001, 
    dot (inVec_14, inVec_14)
  )));
  mediump float tmpvar_15;
  tmpvar_15 = dot (tmpvar_5, viewDir_10);
  mediump vec3 tmpvar_16;
  if ((tmpvar_15 < 0.0)) {
    tmpvar_16 = (tmpvar_5 + (viewDir_10 * (
      -(tmpvar_15)
     + 1e-05)));
  } else {
    tmpvar_16 = tmpvar_5;
  };
  mediump float tmpvar_17;
  tmpvar_17 = clamp (dot (tmpvar_16, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_16, tmpvar_13), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_16, viewDir_10), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_6, tmpvar_13), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = (tmpvar_12 * tmpvar_12);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_12 * tmpvar_12);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_22 * tmpvar_22);
  mediump float tmpvar_24;
  tmpvar_24 = (((tmpvar_18 * tmpvar_18) * (tmpvar_23 - 1.0)) + 1.0);
  mediump float x_25;
  x_25 = (1.0 - tmpvar_17);
  mediump float x_26;
  x_26 = (1.0 - tmpvar_19);
  mediump float tmpvar_27;
  tmpvar_27 = (0.5 + ((2.0 * tmpvar_20) * (tmpvar_20 * tmpvar_12)));
  specularTerm_11 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_17 * ((tmpvar_19 * (1.0 - tmpvar_21)) + tmpvar_21))
     + 
      (tmpvar_19 * ((tmpvar_17 * (1.0 - tmpvar_21)) + tmpvar_21))
    ) + 1e-05)) * (tmpvar_23 / ((
      (3.141593 * tmpvar_24)
     * tmpvar_24) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_28;
  tmpvar_28 = max (0.0, (specularTerm_11 * tmpvar_17));
  specularTerm_11 = tmpvar_28;
  mediump float x_29;
  x_29 = (1.0 - tmpvar_20);
  mediump vec4 tmpvar_30;
  tmpvar_30.w = 1.0;
  tmpvar_30.xyz = ((tmpvar_3 * (tmpvar_9 * 
    (((1.0 + (
      (tmpvar_27 - 1.0)
     * 
      ((x_25 * x_25) * ((x_25 * x_25) * x_25))
    )) * (1.0 + (
      (tmpvar_27 - 1.0)
     * 
      ((x_26 * x_26) * ((x_26 * x_26) * x_26))
    ))) * tmpvar_17)
  )) + ((tmpvar_28 * tmpvar_9) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_29 * x_29) * ((x_29 * x_29) * x_29)))
  )));
  mediump vec4 xlat_varoutput_31;
  xlat_varoutput_31.xyz = tmpvar_30.xyz;
  xlat_varoutput_31.w = 1.0;
  gl_FragData[0] = xlat_varoutput_31;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "SHADOWS_NONATIVE" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 unity_World2Shadow[4];
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  mediump vec4 tmpvar_11;
  mediump vec4 tmpvar_12;
  highp vec4 tmpvar_13;
  tmpvar_13 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_14;
  texcoord_14.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_15;
  if ((_UVSec == 0.0)) {
    tmpvar_15 = tmpvar_2;
  } else {
    tmpvar_15 = tmpvar_3;
  };
  texcoord_14.zw = ((tmpvar_15 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_16;
  highp vec3 tmpvar_17;
  tmpvar_17 = normalize((tmpvar_13.xyz - _WorldSpaceCameraPos));
  tmpvar_16 = tmpvar_17;
  highp vec3 norm_18;
  norm_18 = tmpvar_1;
  highp vec4 v_19;
  v_19.x = _World2Object[0].x;
  v_19.y = _World2Object[1].x;
  v_19.z = _World2Object[2].x;
  v_19.w = _World2Object[3].x;
  highp vec4 v_20;
  v_20.x = _World2Object[0].y;
  v_20.y = _World2Object[1].y;
  v_20.z = _World2Object[2].y;
  v_20.w = _World2Object[3].y;
  highp vec4 v_21;
  v_21.x = _World2Object[0].z;
  v_21.y = _World2Object[1].z;
  v_21.z = _World2Object[2].z;
  v_21.w = _World2Object[3].z;
  highp vec3 tmpvar_22;
  tmpvar_22 = normalize(((
    (v_19.xyz * norm_18.x)
   + 
    (v_20.xyz * norm_18.y)
  ) + (v_21.xyz * norm_18.z)));
  highp vec3 dir_23;
  dir_23 = tmpvar_4.xyz;
  highp mat3 tmpvar_24;
  tmpvar_24[0] = _Object2World[0].xyz;
  tmpvar_24[1] = _Object2World[1].xyz;
  tmpvar_24[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_25;
  tmpvar_25.xyz = normalize((tmpvar_24 * dir_23));
  tmpvar_25.w = tmpvar_4.w;
  mediump vec3 normal_26;
  normal_26 = tmpvar_22;
  mediump vec3 tangent_27;
  tangent_27 = tmpvar_25.xyz;
  mediump float tangentSign_28;
  tangentSign_28 = tmpvar_25.w;
  mediump float xlat_varsign_29;
  highp float tmpvar_30;
  tmpvar_30 = (tangentSign_28 * unity_WorldTransformParams.w);
  xlat_varsign_29 = tmpvar_30;
  mediump vec3 tmpvar_31;
  tmpvar_31 = (((normal_26.yzx * tangent_27.zxy) - (normal_26.zxy * tangent_27.yzx)) * xlat_varsign_29);
  mediump mat3 tmpvar_32;
  tmpvar_32[0].x = tangent_27.x;
  tmpvar_32[0].y = tmpvar_31.x;
  tmpvar_32[0].z = normal_26.x;
  tmpvar_32[1].x = tangent_27.y;
  tmpvar_32[1].y = tmpvar_31.y;
  tmpvar_32[1].z = normal_26.y;
  tmpvar_32[2].x = tangent_27.z;
  tmpvar_32[2].y = tmpvar_31.z;
  tmpvar_32[2].z = normal_26.z;
  tangentToWorld_6 = tmpvar_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].x;
  v_33.y = tangentToWorld_6[1].x;
  v_33.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_33;
  highp vec3 v_34;
  v_34.x = tangentToWorld_6[0].y;
  v_34.y = tangentToWorld_6[1].y;
  v_34.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_34;
  highp vec3 v_35;
  v_35.x = tangentToWorld_6[0].z;
  v_35.y = tangentToWorld_6[1].z;
  v_35.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_35;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  tmpvar_11 = (_LightMatrix0 * tmpvar_36);
  tmpvar_12 = (unity_World2Shadow[0] * tmpvar_36);
  mediump vec3 tmpvar_37;
  highp vec3 tmpvar_38;
  tmpvar_38 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_13.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_37 = tmpvar_38;
  lightDir_5 = tmpvar_37;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_14;
  xlv_TEXCOORD1 = tmpvar_16;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = tmpvar_11;
  xlv_TEXCOORD6 = tmpvar_12;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform highp sampler2D _ShadowMapTexture;
uniform sampler2D _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_7;
  mediump vec2 P_8;
  P_8 = ((xlv_TEXCOORD5.xy / xlv_TEXCOORD5.w) + 0.5);
  tmpvar_7 = texture2D (_LightTexture0, P_8);
  highp vec3 LightCoord_9;
  LightCoord_9 = xlv_TEXCOORD5.xyz;
  highp float tmpvar_10;
  tmpvar_10 = dot (LightCoord_9, LightCoord_9);
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_LightTextureB0, vec2(tmpvar_10));
  lowp float tmpvar_12;
  highp vec4 shadowCoord_13;
  shadowCoord_13 = xlv_TEXCOORD6;
  highp vec4 tmpvar_14;
  tmpvar_14 = texture2DProj (_ShadowMapTexture, shadowCoord_13);
  mediump float tmpvar_15;
  if ((tmpvar_14.x < (shadowCoord_13.z / shadowCoord_13.w))) {
    tmpvar_15 = _LightShadowData.x;
  } else {
    tmpvar_15 = 1.0;
  };
  tmpvar_12 = tmpvar_15;
  mediump vec3 tmpvar_16;
  tmpvar_16 = _LightColor0.xyz;
  tmpvar_16 = (tmpvar_16 * ((
    (float((xlv_TEXCOORD5.z > 0.0)) * tmpvar_7.w)
   * tmpvar_11.w) * tmpvar_12));
  mediump vec3 viewDir_17;
  viewDir_17 = -(xlv_TEXCOORD1);
  mediump float specularTerm_18;
  mediump float tmpvar_19;
  tmpvar_19 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_20;
  mediump vec3 inVec_21;
  inVec_21 = (tmpvar_6 + viewDir_17);
  tmpvar_20 = (inVec_21 * inversesqrt(max (0.001, 
    dot (inVec_21, inVec_21)
  )));
  mediump float tmpvar_22;
  tmpvar_22 = dot (tmpvar_5, viewDir_17);
  mediump vec3 tmpvar_23;
  if ((tmpvar_22 < 0.0)) {
    tmpvar_23 = (tmpvar_5 + (viewDir_17 * (
      -(tmpvar_22)
     + 1e-05)));
  } else {
    tmpvar_23 = tmpvar_5;
  };
  mediump float tmpvar_24;
  tmpvar_24 = clamp (dot (tmpvar_23, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_25;
  tmpvar_25 = clamp (dot (tmpvar_23, tmpvar_20), 0.0, 1.0);
  mediump float tmpvar_26;
  tmpvar_26 = clamp (dot (tmpvar_23, viewDir_17), 0.0, 1.0);
  mediump float tmpvar_27;
  tmpvar_27 = clamp (dot (tmpvar_6, tmpvar_20), 0.0, 1.0);
  mediump float tmpvar_28;
  tmpvar_28 = (tmpvar_19 * tmpvar_19);
  mediump float tmpvar_29;
  tmpvar_29 = (tmpvar_19 * tmpvar_19);
  mediump float tmpvar_30;
  tmpvar_30 = (tmpvar_29 * tmpvar_29);
  mediump float tmpvar_31;
  tmpvar_31 = (((tmpvar_25 * tmpvar_25) * (tmpvar_30 - 1.0)) + 1.0);
  mediump float x_32;
  x_32 = (1.0 - tmpvar_24);
  mediump float x_33;
  x_33 = (1.0 - tmpvar_26);
  mediump float tmpvar_34;
  tmpvar_34 = (0.5 + ((2.0 * tmpvar_27) * (tmpvar_27 * tmpvar_19)));
  specularTerm_18 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_24 * ((tmpvar_26 * (1.0 - tmpvar_28)) + tmpvar_28))
     + 
      (tmpvar_26 * ((tmpvar_24 * (1.0 - tmpvar_28)) + tmpvar_28))
    ) + 1e-05)) * (tmpvar_30 / ((
      (3.141593 * tmpvar_31)
     * tmpvar_31) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_35;
  tmpvar_35 = max (0.0, (specularTerm_18 * tmpvar_24));
  specularTerm_18 = tmpvar_35;
  mediump float x_36;
  x_36 = (1.0 - tmpvar_27);
  mediump vec4 tmpvar_37;
  tmpvar_37.w = 1.0;
  tmpvar_37.xyz = ((tmpvar_3 * (tmpvar_16 * 
    (((1.0 + (
      (tmpvar_34 - 1.0)
     * 
      ((x_32 * x_32) * ((x_32 * x_32) * x_32))
    )) * (1.0 + (
      (tmpvar_34 - 1.0)
     * 
      ((x_33 * x_33) * ((x_33 * x_33) * x_33))
    ))) * tmpvar_24)
  )) + ((tmpvar_35 * tmpvar_16) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_36 * x_36) * ((x_36 * x_36) * x_36)))
  )));
  mediump vec4 xlat_varoutput_38;
  xlat_varoutput_38.xyz = tmpvar_37.xyz;
  xlat_varoutput_38.w = 1.0;
  gl_FragData[0] = xlat_varoutput_38;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "SHADOWS_NATIVE" }
"#version 100

#ifdef VERTEX
#extension GL_EXT_shadow_samplers : enable
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 unity_World2Shadow[4];
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  mediump vec4 tmpvar_11;
  mediump vec4 tmpvar_12;
  highp vec4 tmpvar_13;
  tmpvar_13 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_14;
  texcoord_14.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_15;
  if ((_UVSec == 0.0)) {
    tmpvar_15 = tmpvar_2;
  } else {
    tmpvar_15 = tmpvar_3;
  };
  texcoord_14.zw = ((tmpvar_15 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_16;
  highp vec3 tmpvar_17;
  tmpvar_17 = normalize((tmpvar_13.xyz - _WorldSpaceCameraPos));
  tmpvar_16 = tmpvar_17;
  highp vec3 norm_18;
  norm_18 = tmpvar_1;
  highp vec4 v_19;
  v_19.x = _World2Object[0].x;
  v_19.y = _World2Object[1].x;
  v_19.z = _World2Object[2].x;
  v_19.w = _World2Object[3].x;
  highp vec4 v_20;
  v_20.x = _World2Object[0].y;
  v_20.y = _World2Object[1].y;
  v_20.z = _World2Object[2].y;
  v_20.w = _World2Object[3].y;
  highp vec4 v_21;
  v_21.x = _World2Object[0].z;
  v_21.y = _World2Object[1].z;
  v_21.z = _World2Object[2].z;
  v_21.w = _World2Object[3].z;
  highp vec3 tmpvar_22;
  tmpvar_22 = normalize(((
    (v_19.xyz * norm_18.x)
   + 
    (v_20.xyz * norm_18.y)
  ) + (v_21.xyz * norm_18.z)));
  highp vec3 dir_23;
  dir_23 = tmpvar_4.xyz;
  highp mat3 tmpvar_24;
  tmpvar_24[0] = _Object2World[0].xyz;
  tmpvar_24[1] = _Object2World[1].xyz;
  tmpvar_24[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_25;
  tmpvar_25.xyz = normalize((tmpvar_24 * dir_23));
  tmpvar_25.w = tmpvar_4.w;
  mediump vec3 normal_26;
  normal_26 = tmpvar_22;
  mediump vec3 tangent_27;
  tangent_27 = tmpvar_25.xyz;
  mediump float tangentSign_28;
  tangentSign_28 = tmpvar_25.w;
  mediump float xlat_varsign_29;
  highp float tmpvar_30;
  tmpvar_30 = (tangentSign_28 * unity_WorldTransformParams.w);
  xlat_varsign_29 = tmpvar_30;
  mediump vec3 tmpvar_31;
  tmpvar_31 = (((normal_26.yzx * tangent_27.zxy) - (normal_26.zxy * tangent_27.yzx)) * xlat_varsign_29);
  mediump mat3 tmpvar_32;
  tmpvar_32[0].x = tangent_27.x;
  tmpvar_32[0].y = tmpvar_31.x;
  tmpvar_32[0].z = normal_26.x;
  tmpvar_32[1].x = tangent_27.y;
  tmpvar_32[1].y = tmpvar_31.y;
  tmpvar_32[1].z = normal_26.y;
  tmpvar_32[2].x = tangent_27.z;
  tmpvar_32[2].y = tmpvar_31.z;
  tmpvar_32[2].z = normal_26.z;
  tangentToWorld_6 = tmpvar_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].x;
  v_33.y = tangentToWorld_6[1].x;
  v_33.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_33;
  highp vec3 v_34;
  v_34.x = tangentToWorld_6[0].y;
  v_34.y = tangentToWorld_6[1].y;
  v_34.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_34;
  highp vec3 v_35;
  v_35.x = tangentToWorld_6[0].z;
  v_35.y = tangentToWorld_6[1].z;
  v_35.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_35;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  tmpvar_11 = (_LightMatrix0 * tmpvar_36);
  tmpvar_12 = (unity_World2Shadow[0] * tmpvar_36);
  mediump vec3 tmpvar_37;
  highp vec3 tmpvar_38;
  tmpvar_38 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_13.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_37 = tmpvar_38;
  lightDir_5 = tmpvar_37;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_14;
  xlv_TEXCOORD1 = tmpvar_16;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = tmpvar_11;
  xlv_TEXCOORD6 = tmpvar_12;
}


#endif
#ifdef FRAGMENT
#extension GL_EXT_shadow_samplers : enable
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform lowp sampler2DShadow _ShadowMapTexture;
uniform sampler2D _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_7;
  mediump vec2 P_8;
  P_8 = ((xlv_TEXCOORD5.xy / xlv_TEXCOORD5.w) + 0.5);
  tmpvar_7 = texture2D (_LightTexture0, P_8);
  mediump float tmpvar_9;
  tmpvar_9 = dot (xlv_TEXCOORD5.xyz, xlv_TEXCOORD5.xyz);
  lowp vec4 tmpvar_10;
  tmpvar_10 = texture2D (_LightTextureB0, vec2(tmpvar_9));
  lowp float tmpvar_11;
  highp vec4 shadowCoord_12;
  shadowCoord_12 = xlv_TEXCOORD6;
  mediump float shadow_13;
  lowp float tmpvar_14;
  tmpvar_14 = shadow2DProjEXT (_ShadowMapTexture, shadowCoord_12);
  mediump float tmpvar_15;
  tmpvar_15 = tmpvar_14;
  shadow_13 = (_LightShadowData.x + (tmpvar_15 * (1.0 - _LightShadowData.x)));
  tmpvar_11 = shadow_13;
  mediump vec3 tmpvar_16;
  tmpvar_16 = _LightColor0.xyz;
  tmpvar_16 = (tmpvar_16 * ((
    (float((xlv_TEXCOORD5.z > 0.0)) * tmpvar_7.w)
   * tmpvar_10.w) * tmpvar_11));
  mediump vec3 viewDir_17;
  viewDir_17 = -(xlv_TEXCOORD1);
  mediump float specularTerm_18;
  mediump float tmpvar_19;
  tmpvar_19 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_20;
  mediump vec3 inVec_21;
  inVec_21 = (tmpvar_6 + viewDir_17);
  tmpvar_20 = (inVec_21 * inversesqrt(max (0.001, 
    dot (inVec_21, inVec_21)
  )));
  mediump float tmpvar_22;
  tmpvar_22 = dot (tmpvar_5, viewDir_17);
  mediump vec3 tmpvar_23;
  if ((tmpvar_22 < 0.0)) {
    tmpvar_23 = (tmpvar_5 + (viewDir_17 * (
      -(tmpvar_22)
     + 1e-05)));
  } else {
    tmpvar_23 = tmpvar_5;
  };
  mediump float tmpvar_24;
  tmpvar_24 = clamp (dot (tmpvar_23, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_25;
  tmpvar_25 = clamp (dot (tmpvar_23, tmpvar_20), 0.0, 1.0);
  mediump float tmpvar_26;
  tmpvar_26 = clamp (dot (tmpvar_23, viewDir_17), 0.0, 1.0);
  mediump float tmpvar_27;
  tmpvar_27 = clamp (dot (tmpvar_6, tmpvar_20), 0.0, 1.0);
  mediump float tmpvar_28;
  tmpvar_28 = (tmpvar_19 * tmpvar_19);
  mediump float tmpvar_29;
  tmpvar_29 = (tmpvar_19 * tmpvar_19);
  mediump float tmpvar_30;
  tmpvar_30 = (tmpvar_29 * tmpvar_29);
  mediump float tmpvar_31;
  tmpvar_31 = (((tmpvar_25 * tmpvar_25) * (tmpvar_30 - 1.0)) + 1.0);
  mediump float x_32;
  x_32 = (1.0 - tmpvar_24);
  mediump float x_33;
  x_33 = (1.0 - tmpvar_26);
  mediump float tmpvar_34;
  tmpvar_34 = (0.5 + ((2.0 * tmpvar_27) * (tmpvar_27 * tmpvar_19)));
  specularTerm_18 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_24 * ((tmpvar_26 * (1.0 - tmpvar_28)) + tmpvar_28))
     + 
      (tmpvar_26 * ((tmpvar_24 * (1.0 - tmpvar_28)) + tmpvar_28))
    ) + 1e-05)) * (tmpvar_30 / ((
      (3.141593 * tmpvar_31)
     * tmpvar_31) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_35;
  tmpvar_35 = max (0.0, (specularTerm_18 * tmpvar_24));
  specularTerm_18 = tmpvar_35;
  mediump float x_36;
  x_36 = (1.0 - tmpvar_27);
  mediump vec4 tmpvar_37;
  tmpvar_37.w = 1.0;
  tmpvar_37.xyz = ((tmpvar_3 * (tmpvar_16 * 
    (((1.0 + (
      (tmpvar_34 - 1.0)
     * 
      ((x_32 * x_32) * ((x_32 * x_32) * x_32))
    )) * (1.0 + (
      (tmpvar_34 - 1.0)
     * 
      ((x_33 * x_33) * ((x_33 * x_33) * x_33))
    ))) * tmpvar_24)
  )) + ((tmpvar_35 * tmpvar_16) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_36 * x_36) * ((x_36 * x_36) * x_36)))
  )));
  mediump vec4 xlat_varoutput_38;
  xlat_varoutput_38.xyz = tmpvar_37.xyz;
  xlat_varoutput_38.w = 1.0;
  gl_FragData[0] = xlat_varoutput_38;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_5 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].x;
  v_31.y = tangentToWorld_5[1].x;
  v_31.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].y;
  v_32.y = tangentToWorld_5[1].y;
  v_32.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_5[0].z;
  v_33.y = tangentToWorld_5[1].z;
  v_33.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_33;
  highp vec4 o_34;
  highp vec4 tmpvar_35;
  tmpvar_35 = (tmpvar_6 * 0.5);
  highp vec2 tmpvar_36;
  tmpvar_36.x = tmpvar_35.x;
  tmpvar_36.y = (tmpvar_35.y * _ProjectionParams.x);
  o_34.xy = (tmpvar_36 + tmpvar_35.w);
  o_34.zw = tmpvar_6.zw;
  tmpvar_10 = o_34;
  highp vec3 tmpvar_37;
  tmpvar_37 = (_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_37.x;
  tmpvar_8.w = tmpvar_37.y;
  tmpvar_9.w = tmpvar_37.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD5 = tmpvar_10;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _ShadowMapTexture;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  lowp float tmpvar_7;
  tmpvar_7 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD5).x;
  mediump float atten_8;
  atten_8 = tmpvar_7;
  mediump vec3 tmpvar_9;
  tmpvar_9 = _LightColor0.xyz;
  tmpvar_9 = (tmpvar_9 * atten_8);
  mediump vec3 viewDir_10;
  viewDir_10 = -(xlv_TEXCOORD1);
  mediump float specularTerm_11;
  mediump float tmpvar_12;
  tmpvar_12 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_13;
  mediump vec3 inVec_14;
  inVec_14 = (tmpvar_6 + viewDir_10);
  tmpvar_13 = (inVec_14 * inversesqrt(max (0.001, 
    dot (inVec_14, inVec_14)
  )));
  mediump float tmpvar_15;
  tmpvar_15 = dot (tmpvar_5, viewDir_10);
  mediump vec3 tmpvar_16;
  if ((tmpvar_15 < 0.0)) {
    tmpvar_16 = (tmpvar_5 + (viewDir_10 * (
      -(tmpvar_15)
     + 1e-05)));
  } else {
    tmpvar_16 = tmpvar_5;
  };
  mediump float tmpvar_17;
  tmpvar_17 = clamp (dot (tmpvar_16, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_16, tmpvar_13), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_16, viewDir_10), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_6, tmpvar_13), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = (tmpvar_12 * tmpvar_12);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_12 * tmpvar_12);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_22 * tmpvar_22);
  mediump float tmpvar_24;
  tmpvar_24 = (((tmpvar_18 * tmpvar_18) * (tmpvar_23 - 1.0)) + 1.0);
  mediump float x_25;
  x_25 = (1.0 - tmpvar_17);
  mediump float x_26;
  x_26 = (1.0 - tmpvar_19);
  mediump float tmpvar_27;
  tmpvar_27 = (0.5 + ((2.0 * tmpvar_20) * (tmpvar_20 * tmpvar_12)));
  specularTerm_11 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_17 * ((tmpvar_19 * (1.0 - tmpvar_21)) + tmpvar_21))
     + 
      (tmpvar_19 * ((tmpvar_17 * (1.0 - tmpvar_21)) + tmpvar_21))
    ) + 1e-05)) * (tmpvar_23 / ((
      (3.141593 * tmpvar_24)
     * tmpvar_24) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_28;
  tmpvar_28 = max (0.0, (specularTerm_11 * tmpvar_17));
  specularTerm_11 = tmpvar_28;
  mediump float x_29;
  x_29 = (1.0 - tmpvar_20);
  mediump vec4 tmpvar_30;
  tmpvar_30.w = 1.0;
  tmpvar_30.xyz = ((tmpvar_3 * (tmpvar_9 * 
    (((1.0 + (
      (tmpvar_27 - 1.0)
     * 
      ((x_25 * x_25) * ((x_25 * x_25) * x_25))
    )) * (1.0 + (
      (tmpvar_27 - 1.0)
     * 
      ((x_26 * x_26) * ((x_26 * x_26) * x_26))
    ))) * tmpvar_17)
  )) + ((tmpvar_28 * tmpvar_9) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_29 * x_29) * ((x_29 * x_29) * x_29)))
  )));
  mediump vec4 xlat_varoutput_31;
  xlat_varoutput_31.xyz = tmpvar_30.xyz;
  xlat_varoutput_31.w = 1.0;
  gl_FragData[0] = xlat_varoutput_31;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_SCREEN" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_5 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].x;
  v_31.y = tangentToWorld_5[1].x;
  v_31.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].y;
  v_32.y = tangentToWorld_5[1].y;
  v_32.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_5[0].z;
  v_33.y = tangentToWorld_5[1].z;
  v_33.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_33;
  highp vec4 o_34;
  highp vec4 tmpvar_35;
  tmpvar_35 = (tmpvar_6 * 0.5);
  highp vec2 tmpvar_36;
  tmpvar_36.x = tmpvar_35.x;
  tmpvar_36.y = (tmpvar_35.y * _ProjectionParams.x);
  o_34.xy = (tmpvar_36 + tmpvar_35.w);
  o_34.zw = tmpvar_6.zw;
  tmpvar_10 = o_34;
  highp vec3 tmpvar_37;
  tmpvar_37 = (_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_37.x;
  tmpvar_8.w = tmpvar_37.y;
  tmpvar_9.w = tmpvar_37.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xy;
  xlv_TEXCOORD6 = tmpvar_10;
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _ShadowMapTexture;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_7;
  tmpvar_7 = texture2D (_LightTexture0, xlv_TEXCOORD5);
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD6);
  mediump float atten_9;
  atten_9 = (tmpvar_7.w * tmpvar_8.x);
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  tmpvar_10 = (tmpvar_10 * atten_9);
  mediump vec3 viewDir_11;
  viewDir_11 = -(xlv_TEXCOORD1);
  mediump float specularTerm_12;
  mediump float tmpvar_13;
  tmpvar_13 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_14;
  mediump vec3 inVec_15;
  inVec_15 = (tmpvar_6 + viewDir_11);
  tmpvar_14 = (inVec_15 * inversesqrt(max (0.001, 
    dot (inVec_15, inVec_15)
  )));
  mediump float tmpvar_16;
  tmpvar_16 = dot (tmpvar_5, viewDir_11);
  mediump vec3 tmpvar_17;
  if ((tmpvar_16 < 0.0)) {
    tmpvar_17 = (tmpvar_5 + (viewDir_11 * (
      -(tmpvar_16)
     + 1e-05)));
  } else {
    tmpvar_17 = tmpvar_5;
  };
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_17, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_17, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_17, viewDir_11), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_6, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_23 * tmpvar_23);
  mediump float tmpvar_25;
  tmpvar_25 = (((tmpvar_19 * tmpvar_19) * (tmpvar_24 - 1.0)) + 1.0);
  mediump float x_26;
  x_26 = (1.0 - tmpvar_18);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_20);
  mediump float tmpvar_28;
  tmpvar_28 = (0.5 + ((2.0 * tmpvar_21) * (tmpvar_21 * tmpvar_13)));
  specularTerm_12 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_18 * ((tmpvar_20 * (1.0 - tmpvar_22)) + tmpvar_22))
     + 
      (tmpvar_20 * ((tmpvar_18 * (1.0 - tmpvar_22)) + tmpvar_22))
    ) + 1e-05)) * (tmpvar_24 / ((
      (3.141593 * tmpvar_25)
     * tmpvar_25) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_29;
  tmpvar_29 = max (0.0, (specularTerm_12 * tmpvar_18));
  specularTerm_12 = tmpvar_29;
  mediump float x_30;
  x_30 = (1.0 - tmpvar_21);
  mediump vec4 tmpvar_31;
  tmpvar_31.w = 1.0;
  tmpvar_31.xyz = ((tmpvar_3 * (tmpvar_10 * 
    (((1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_26 * x_26) * ((x_26 * x_26) * x_26))
    )) * (1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    ))) * tmpvar_18)
  )) + ((tmpvar_29 * tmpvar_10) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_30 * x_30) * ((x_30 * x_30) * x_30)))
  )));
  mediump vec4 xlat_varoutput_32;
  xlat_varoutput_32.xyz = tmpvar_31.xyz;
  xlat_varoutput_32.w = 1.0;
  gl_FragData[0] = xlat_varoutput_32;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_CUBE" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp vec4 _LightPositionRange;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  xlv_TEXCOORD5 = (_LightMatrix0 * tmpvar_36).xyz;
  xlv_TEXCOORD6 = (tmpvar_36.xyz - _LightPositionRange.xyz);
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _LightPositionRange;
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform highp samplerCube _ShadowMapTexture;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_7;
  tmpvar_7 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2D (_LightTexture0, vec2(tmpvar_7));
  highp float mydist_9;
  mydist_9 = ((sqrt(
    dot (xlv_TEXCOORD6, xlv_TEXCOORD6)
  ) * _LightPositionRange.w) * 0.97);
  highp float tmpvar_10;
  tmpvar_10 = dot (textureCube (_ShadowMapTexture, xlv_TEXCOORD6), vec4(1.0, 0.003921569, 1.53787e-05, 6.030863e-08));
  mediump float tmpvar_11;
  if ((tmpvar_10 < mydist_9)) {
    tmpvar_11 = _LightShadowData.x;
  } else {
    tmpvar_11 = 1.0;
  };
  mediump vec3 tmpvar_12;
  tmpvar_12 = _LightColor0.xyz;
  tmpvar_12 = (tmpvar_12 * (tmpvar_8.w * tmpvar_11));
  mediump vec3 viewDir_13;
  viewDir_13 = -(xlv_TEXCOORD1);
  mediump float specularTerm_14;
  mediump float tmpvar_15;
  tmpvar_15 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_16;
  mediump vec3 inVec_17;
  inVec_17 = (tmpvar_6 + viewDir_13);
  tmpvar_16 = (inVec_17 * inversesqrt(max (0.001, 
    dot (inVec_17, inVec_17)
  )));
  mediump float tmpvar_18;
  tmpvar_18 = dot (tmpvar_5, viewDir_13);
  mediump vec3 tmpvar_19;
  if ((tmpvar_18 < 0.0)) {
    tmpvar_19 = (tmpvar_5 + (viewDir_13 * (
      -(tmpvar_18)
     + 1e-05)));
  } else {
    tmpvar_19 = tmpvar_5;
  };
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_19, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_19, tmpvar_16), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_19, viewDir_13), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_6, tmpvar_16), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_15 * tmpvar_15);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_15 * tmpvar_15);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_25 * tmpvar_25);
  mediump float tmpvar_27;
  tmpvar_27 = (((tmpvar_21 * tmpvar_21) * (tmpvar_26 - 1.0)) + 1.0);
  mediump float x_28;
  x_28 = (1.0 - tmpvar_20);
  mediump float x_29;
  x_29 = (1.0 - tmpvar_22);
  mediump float tmpvar_30;
  tmpvar_30 = (0.5 + ((2.0 * tmpvar_23) * (tmpvar_23 * tmpvar_15)));
  specularTerm_14 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_20 * ((tmpvar_22 * (1.0 - tmpvar_24)) + tmpvar_24))
     + 
      (tmpvar_22 * ((tmpvar_20 * (1.0 - tmpvar_24)) + tmpvar_24))
    ) + 1e-05)) * (tmpvar_26 / ((
      (3.141593 * tmpvar_27)
     * tmpvar_27) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_31;
  tmpvar_31 = max (0.0, (specularTerm_14 * tmpvar_20));
  specularTerm_14 = tmpvar_31;
  mediump float x_32;
  x_32 = (1.0 - tmpvar_23);
  mediump vec4 tmpvar_33;
  tmpvar_33.w = 1.0;
  tmpvar_33.xyz = ((tmpvar_3 * (tmpvar_12 * 
    (((1.0 + (
      (tmpvar_30 - 1.0)
     * 
      ((x_28 * x_28) * ((x_28 * x_28) * x_28))
    )) * (1.0 + (
      (tmpvar_30 - 1.0)
     * 
      ((x_29 * x_29) * ((x_29 * x_29) * x_29))
    ))) * tmpvar_20)
  )) + ((tmpvar_31 * tmpvar_12) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_32 * x_32) * ((x_32 * x_32) * x_32)))
  )));
  mediump vec4 xlat_varoutput_34;
  xlat_varoutput_34.xyz = tmpvar_33.xyz;
  xlat_varoutput_34.w = 1.0;
  gl_FragData[0] = xlat_varoutput_34;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_CUBE" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp vec4 _LightPositionRange;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  xlv_TEXCOORD5 = (_LightMatrix0 * tmpvar_36).xyz;
  xlv_TEXCOORD6 = (tmpvar_36.xyz - _LightPositionRange.xyz);
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _LightPositionRange;
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform highp samplerCube _ShadowMapTexture;
uniform lowp samplerCube _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_2;
  tmpvar_2 = (_Color.xyz * tmpvar_1.xyz);
  mediump vec3 tmpvar_3;
  mediump vec3 tmpvar_4;
  tmpvar_4 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_2, vec3(_Metallic));
  tmpvar_3 = (tmpvar_2 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_5;
  tmpvar_5 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_6;
  tmpvar_6.x = xlv_TEXCOORD2.w;
  tmpvar_6.y = xlv_TEXCOORD2_1.w;
  tmpvar_6.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_7;
  tmpvar_7 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2D (_LightTextureB0, vec2(tmpvar_7));
  lowp vec4 tmpvar_9;
  tmpvar_9 = textureCube (_LightTexture0, xlv_TEXCOORD5);
  highp float mydist_10;
  mydist_10 = ((sqrt(
    dot (xlv_TEXCOORD6, xlv_TEXCOORD6)
  ) * _LightPositionRange.w) * 0.97);
  highp float tmpvar_11;
  tmpvar_11 = dot (textureCube (_ShadowMapTexture, xlv_TEXCOORD6), vec4(1.0, 0.003921569, 1.53787e-05, 6.030863e-08));
  mediump float tmpvar_12;
  if ((tmpvar_11 < mydist_10)) {
    tmpvar_12 = _LightShadowData.x;
  } else {
    tmpvar_12 = 1.0;
  };
  mediump vec3 tmpvar_13;
  tmpvar_13 = _LightColor0.xyz;
  tmpvar_13 = (tmpvar_13 * ((tmpvar_8.w * tmpvar_9.w) * tmpvar_12));
  mediump vec3 viewDir_14;
  viewDir_14 = -(xlv_TEXCOORD1);
  mediump float specularTerm_15;
  mediump float tmpvar_16;
  tmpvar_16 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_17;
  mediump vec3 inVec_18;
  inVec_18 = (tmpvar_6 + viewDir_14);
  tmpvar_17 = (inVec_18 * inversesqrt(max (0.001, 
    dot (inVec_18, inVec_18)
  )));
  mediump float tmpvar_19;
  tmpvar_19 = dot (tmpvar_5, viewDir_14);
  mediump vec3 tmpvar_20;
  if ((tmpvar_19 < 0.0)) {
    tmpvar_20 = (tmpvar_5 + (viewDir_14 * (
      -(tmpvar_19)
     + 1e-05)));
  } else {
    tmpvar_20 = tmpvar_5;
  };
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_20, tmpvar_6), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_20, tmpvar_17), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_20, viewDir_14), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = clamp (dot (tmpvar_6, tmpvar_17), 0.0, 1.0);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_16 * tmpvar_16);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_16 * tmpvar_16);
  mediump float tmpvar_27;
  tmpvar_27 = (tmpvar_26 * tmpvar_26);
  mediump float tmpvar_28;
  tmpvar_28 = (((tmpvar_22 * tmpvar_22) * (tmpvar_27 - 1.0)) + 1.0);
  mediump float x_29;
  x_29 = (1.0 - tmpvar_21);
  mediump float x_30;
  x_30 = (1.0 - tmpvar_23);
  mediump float tmpvar_31;
  tmpvar_31 = (0.5 + ((2.0 * tmpvar_24) * (tmpvar_24 * tmpvar_16)));
  specularTerm_15 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_21 * ((tmpvar_23 * (1.0 - tmpvar_25)) + tmpvar_25))
     + 
      (tmpvar_23 * ((tmpvar_21 * (1.0 - tmpvar_25)) + tmpvar_25))
    ) + 1e-05)) * (tmpvar_27 / ((
      (3.141593 * tmpvar_28)
     * tmpvar_28) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_32;
  tmpvar_32 = max (0.0, (specularTerm_15 * tmpvar_21));
  specularTerm_15 = tmpvar_32;
  mediump float x_33;
  x_33 = (1.0 - tmpvar_24);
  mediump vec4 tmpvar_34;
  tmpvar_34.w = 1.0;
  tmpvar_34.xyz = ((tmpvar_3 * (tmpvar_13 * 
    (((1.0 + (
      (tmpvar_31 - 1.0)
     * 
      ((x_29 * x_29) * ((x_29 * x_29) * x_29))
    )) * (1.0 + (
      (tmpvar_31 - 1.0)
     * 
      ((x_30 * x_30) * ((x_30 * x_30) * x_30))
    ))) * tmpvar_21)
  )) + ((tmpvar_32 * tmpvar_13) * (tmpvar_4 + 
    ((1.0 - tmpvar_4) * ((x_33 * x_33) * ((x_33 * x_33) * x_33)))
  )));
  mediump vec4 xlat_varoutput_35;
  xlat_varoutput_35.xyz = tmpvar_34.xyz;
  xlat_varoutput_35.w = 1.0;
  gl_FragData[0] = xlat_varoutput_35;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xyz;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_8;
  tmpvar_8 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_9;
  tmpvar_9 = texture2D (_LightTexture0, vec2(tmpvar_8));
  mediump float atten_10;
  atten_10 = tmpvar_9.w;
  mediump vec3 tmpvar_11;
  tmpvar_11 = _LightColor0.xyz;
  tmpvar_11 = (tmpvar_11 * atten_10);
  mediump vec3 viewDir_12;
  viewDir_12 = -(xlv_TEXCOORD1);
  mediump float specularTerm_13;
  mediump float tmpvar_14;
  tmpvar_14 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_15;
  mediump vec3 inVec_16;
  inVec_16 = (tmpvar_7 + viewDir_12);
  tmpvar_15 = (inVec_16 * inversesqrt(max (0.001, 
    dot (inVec_16, inVec_16)
  )));
  mediump float tmpvar_17;
  tmpvar_17 = dot (tmpvar_6, viewDir_12);
  mediump vec3 tmpvar_18;
  if ((tmpvar_17 < 0.0)) {
    tmpvar_18 = (tmpvar_6 + (viewDir_12 * (
      -(tmpvar_17)
     + 1e-05)));
  } else {
    tmpvar_18 = tmpvar_6;
  };
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_18, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_18, tmpvar_15), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_18, viewDir_12), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_7, tmpvar_15), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_14 * tmpvar_14);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_14 * tmpvar_14);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_24 * tmpvar_24);
  mediump float tmpvar_26;
  tmpvar_26 = (((tmpvar_20 * tmpvar_20) * (tmpvar_25 - 1.0)) + 1.0);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_19);
  mediump float x_28;
  x_28 = (1.0 - tmpvar_21);
  mediump float tmpvar_29;
  tmpvar_29 = (0.5 + ((2.0 * tmpvar_22) * (tmpvar_22 * tmpvar_14)));
  specularTerm_13 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_19 * ((tmpvar_21 * (1.0 - tmpvar_23)) + tmpvar_23))
     + 
      (tmpvar_21 * ((tmpvar_19 * (1.0 - tmpvar_23)) + tmpvar_23))
    ) + 1e-05)) * (tmpvar_25 / ((
      (3.141593 * tmpvar_26)
     * tmpvar_26) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_30;
  tmpvar_30 = max (0.0, (specularTerm_13 * tmpvar_19));
  specularTerm_13 = tmpvar_30;
  mediump float x_31;
  x_31 = (1.0 - tmpvar_22);
  mediump vec4 tmpvar_32;
  tmpvar_32.w = 1.0;
  tmpvar_32.xyz = ((tmpvar_4 * (tmpvar_11 * 
    (((1.0 + (
      (tmpvar_29 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    )) * (1.0 + (
      (tmpvar_29 - 1.0)
     * 
      ((x_28 * x_28) * ((x_28 * x_28) * x_28))
    ))) * tmpvar_19)
  )) + ((tmpvar_30 * tmpvar_11) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_31 * x_31) * ((x_31 * x_31) * x_31)))
  )));
  c_1.w = tmpvar_32.w;
  highp float tmpvar_33;
  tmpvar_33 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_32.xyz * vec3(tmpvar_33));
  mediump vec4 xlat_varoutput_34;
  xlat_varoutput_34.xyz = c_1.xyz;
  xlat_varoutput_34.w = 1.0;
  gl_FragData[0] = xlat_varoutput_34;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  highp vec3 dir_20;
  dir_20 = tmpvar_4.xyz;
  highp mat3 tmpvar_21;
  tmpvar_21[0] = _Object2World[0].xyz;
  tmpvar_21[1] = _Object2World[1].xyz;
  tmpvar_21[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_22;
  tmpvar_22.xyz = normalize((tmpvar_21 * dir_20));
  tmpvar_22.w = tmpvar_4.w;
  mediump vec3 normal_23;
  normal_23 = tmpvar_19;
  mediump vec3 tangent_24;
  tangent_24 = tmpvar_22.xyz;
  mediump float tangentSign_25;
  tangentSign_25 = tmpvar_22.w;
  mediump float xlat_varsign_26;
  highp float tmpvar_27;
  tmpvar_27 = (tangentSign_25 * unity_WorldTransformParams.w);
  xlat_varsign_26 = tmpvar_27;
  mediump vec3 tmpvar_28;
  tmpvar_28 = (((normal_23.yzx * tangent_24.zxy) - (normal_23.zxy * tangent_24.yzx)) * xlat_varsign_26);
  mediump mat3 tmpvar_29;
  tmpvar_29[0].x = tangent_24.x;
  tmpvar_29[0].y = tmpvar_28.x;
  tmpvar_29[0].z = normal_23.x;
  tmpvar_29[1].x = tangent_24.y;
  tmpvar_29[1].y = tmpvar_28.y;
  tmpvar_29[1].z = normal_23.y;
  tmpvar_29[2].x = tangent_24.z;
  tmpvar_29[2].y = tmpvar_28.z;
  tmpvar_29[2].z = normal_23.z;
  tangentToWorld_5 = tmpvar_29;
  highp vec3 v_30;
  v_30.x = tangentToWorld_5[0].x;
  v_30.y = tangentToWorld_5[1].x;
  v_30.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].y;
  v_31.y = tangentToWorld_5[1].y;
  v_31.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].z;
  v_32.y = tangentToWorld_5[1].z;
  v_32.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_32;
  highp vec3 tmpvar_33;
  tmpvar_33 = (_WorldSpaceLightPos0.xyz - (tmpvar_10.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_33.x;
  tmpvar_8.w = tmpvar_33.y;
  tmpvar_9.w = tmpvar_33.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_6.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  mediump vec3 tmpvar_8;
  tmpvar_8 = _LightColor0.xyz;
  mediump vec3 viewDir_9;
  viewDir_9 = -(xlv_TEXCOORD1);
  mediump float specularTerm_10;
  mediump float tmpvar_11;
  tmpvar_11 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_12;
  mediump vec3 inVec_13;
  inVec_13 = (tmpvar_7 + viewDir_9);
  tmpvar_12 = (inVec_13 * inversesqrt(max (0.001, 
    dot (inVec_13, inVec_13)
  )));
  mediump float tmpvar_14;
  tmpvar_14 = dot (tmpvar_6, viewDir_9);
  mediump vec3 tmpvar_15;
  if ((tmpvar_14 < 0.0)) {
    tmpvar_15 = (tmpvar_6 + (viewDir_9 * (
      -(tmpvar_14)
     + 1e-05)));
  } else {
    tmpvar_15 = tmpvar_6;
  };
  mediump float tmpvar_16;
  tmpvar_16 = clamp (dot (tmpvar_15, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_17;
  tmpvar_17 = clamp (dot (tmpvar_15, tmpvar_12), 0.0, 1.0);
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_15, viewDir_9), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_7, tmpvar_12), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = (tmpvar_11 * tmpvar_11);
  mediump float tmpvar_21;
  tmpvar_21 = (tmpvar_11 * tmpvar_11);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_21 * tmpvar_21);
  mediump float tmpvar_23;
  tmpvar_23 = (((tmpvar_17 * tmpvar_17) * (tmpvar_22 - 1.0)) + 1.0);
  mediump float x_24;
  x_24 = (1.0 - tmpvar_16);
  mediump float x_25;
  x_25 = (1.0 - tmpvar_18);
  mediump float tmpvar_26;
  tmpvar_26 = (0.5 + ((2.0 * tmpvar_19) * (tmpvar_19 * tmpvar_11)));
  specularTerm_10 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_16 * ((tmpvar_18 * (1.0 - tmpvar_20)) + tmpvar_20))
     + 
      (tmpvar_18 * ((tmpvar_16 * (1.0 - tmpvar_20)) + tmpvar_20))
    ) + 1e-05)) * (tmpvar_22 / ((
      (3.141593 * tmpvar_23)
     * tmpvar_23) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_27;
  tmpvar_27 = max (0.0, (specularTerm_10 * tmpvar_16));
  specularTerm_10 = tmpvar_27;
  mediump float x_28;
  x_28 = (1.0 - tmpvar_19);
  mediump vec4 tmpvar_29;
  tmpvar_29.w = 1.0;
  tmpvar_29.xyz = ((tmpvar_4 * (tmpvar_8 * 
    (((1.0 + (
      (tmpvar_26 - 1.0)
     * 
      ((x_24 * x_24) * ((x_24 * x_24) * x_24))
    )) * (1.0 + (
      (tmpvar_26 - 1.0)
     * 
      ((x_25 * x_25) * ((x_25 * x_25) * x_25))
    ))) * tmpvar_16)
  )) + ((tmpvar_27 * tmpvar_8) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_28 * x_28) * ((x_28 * x_28) * x_28)))
  )));
  c_1.w = tmpvar_29.w;
  highp float tmpvar_30;
  tmpvar_30 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_29.xyz * vec3(tmpvar_30));
  mediump vec4 xlat_varoutput_31;
  xlat_varoutput_31.xyz = c_1.xyz;
  xlat_varoutput_31.w = 1.0;
  gl_FragData[0] = xlat_varoutput_31;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  mediump vec4 tmpvar_11;
  highp vec4 tmpvar_12;
  tmpvar_12 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_13;
  texcoord_13.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_14;
  if ((_UVSec == 0.0)) {
    tmpvar_14 = tmpvar_2;
  } else {
    tmpvar_14 = tmpvar_3;
  };
  texcoord_13.zw = ((tmpvar_14 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_15;
  highp vec3 tmpvar_16;
  tmpvar_16 = normalize((tmpvar_12.xyz - _WorldSpaceCameraPos));
  tmpvar_15 = tmpvar_16;
  highp vec3 norm_17;
  norm_17 = tmpvar_1;
  highp vec4 v_18;
  v_18.x = _World2Object[0].x;
  v_18.y = _World2Object[1].x;
  v_18.z = _World2Object[2].x;
  v_18.w = _World2Object[3].x;
  highp vec4 v_19;
  v_19.x = _World2Object[0].y;
  v_19.y = _World2Object[1].y;
  v_19.z = _World2Object[2].y;
  v_19.w = _World2Object[3].y;
  highp vec4 v_20;
  v_20.x = _World2Object[0].z;
  v_20.y = _World2Object[1].z;
  v_20.z = _World2Object[2].z;
  v_20.w = _World2Object[3].z;
  highp vec3 tmpvar_21;
  tmpvar_21 = normalize(((
    (v_18.xyz * norm_17.x)
   + 
    (v_19.xyz * norm_17.y)
  ) + (v_20.xyz * norm_17.z)));
  highp vec3 dir_22;
  dir_22 = tmpvar_4.xyz;
  highp mat3 tmpvar_23;
  tmpvar_23[0] = _Object2World[0].xyz;
  tmpvar_23[1] = _Object2World[1].xyz;
  tmpvar_23[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_24;
  tmpvar_24.xyz = normalize((tmpvar_23 * dir_22));
  tmpvar_24.w = tmpvar_4.w;
  mediump vec3 normal_25;
  normal_25 = tmpvar_21;
  mediump vec3 tangent_26;
  tangent_26 = tmpvar_24.xyz;
  mediump float tangentSign_27;
  tangentSign_27 = tmpvar_24.w;
  mediump float xlat_varsign_28;
  highp float tmpvar_29;
  tmpvar_29 = (tangentSign_27 * unity_WorldTransformParams.w);
  xlat_varsign_28 = tmpvar_29;
  mediump vec3 tmpvar_30;
  tmpvar_30 = (((normal_25.yzx * tangent_26.zxy) - (normal_25.zxy * tangent_26.yzx)) * xlat_varsign_28);
  mediump mat3 tmpvar_31;
  tmpvar_31[0].x = tangent_26.x;
  tmpvar_31[0].y = tmpvar_30.x;
  tmpvar_31[0].z = normal_25.x;
  tmpvar_31[1].x = tangent_26.y;
  tmpvar_31[1].y = tmpvar_30.y;
  tmpvar_31[1].z = normal_25.y;
  tmpvar_31[2].x = tangent_26.z;
  tmpvar_31[2].y = tmpvar_30.z;
  tmpvar_31[2].z = normal_25.z;
  tangentToWorld_6 = tmpvar_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].x;
  v_32.y = tangentToWorld_6[1].x;
  v_32.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].y;
  v_33.y = tangentToWorld_6[1].y;
  v_33.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_33;
  highp vec3 v_34;
  v_34.x = tangentToWorld_6[0].z;
  v_34.y = tangentToWorld_6[1].z;
  v_34.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_34;
  tmpvar_11 = (_LightMatrix0 * (_Object2World * _glesVertex));
  mediump vec3 tmpvar_35;
  highp vec3 tmpvar_36;
  tmpvar_36 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_12.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_35 = tmpvar_36;
  lightDir_5 = tmpvar_35;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_13;
  xlv_TEXCOORD1 = tmpvar_15;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = tmpvar_11;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_8;
  mediump vec2 P_9;
  P_9 = ((xlv_TEXCOORD5.xy / xlv_TEXCOORD5.w) + 0.5);
  tmpvar_8 = texture2D (_LightTexture0, P_9);
  highp vec3 LightCoord_10;
  LightCoord_10 = xlv_TEXCOORD5.xyz;
  highp float tmpvar_11;
  tmpvar_11 = dot (LightCoord_10, LightCoord_10);
  lowp vec4 tmpvar_12;
  tmpvar_12 = texture2D (_LightTextureB0, vec2(tmpvar_11));
  mediump vec3 tmpvar_13;
  tmpvar_13 = _LightColor0.xyz;
  tmpvar_13 = (tmpvar_13 * ((
    float((xlv_TEXCOORD5.z > 0.0))
   * tmpvar_8.w) * tmpvar_12.w));
  mediump vec3 viewDir_14;
  viewDir_14 = -(xlv_TEXCOORD1);
  mediump float specularTerm_15;
  mediump float tmpvar_16;
  tmpvar_16 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_17;
  mediump vec3 inVec_18;
  inVec_18 = (tmpvar_7 + viewDir_14);
  tmpvar_17 = (inVec_18 * inversesqrt(max (0.001, 
    dot (inVec_18, inVec_18)
  )));
  mediump float tmpvar_19;
  tmpvar_19 = dot (tmpvar_6, viewDir_14);
  mediump vec3 tmpvar_20;
  if ((tmpvar_19 < 0.0)) {
    tmpvar_20 = (tmpvar_6 + (viewDir_14 * (
      -(tmpvar_19)
     + 1e-05)));
  } else {
    tmpvar_20 = tmpvar_6;
  };
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_20, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_20, tmpvar_17), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_20, viewDir_14), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = clamp (dot (tmpvar_7, tmpvar_17), 0.0, 1.0);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_16 * tmpvar_16);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_16 * tmpvar_16);
  mediump float tmpvar_27;
  tmpvar_27 = (tmpvar_26 * tmpvar_26);
  mediump float tmpvar_28;
  tmpvar_28 = (((tmpvar_22 * tmpvar_22) * (tmpvar_27 - 1.0)) + 1.0);
  mediump float x_29;
  x_29 = (1.0 - tmpvar_21);
  mediump float x_30;
  x_30 = (1.0 - tmpvar_23);
  mediump float tmpvar_31;
  tmpvar_31 = (0.5 + ((2.0 * tmpvar_24) * (tmpvar_24 * tmpvar_16)));
  specularTerm_15 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_21 * ((tmpvar_23 * (1.0 - tmpvar_25)) + tmpvar_25))
     + 
      (tmpvar_23 * ((tmpvar_21 * (1.0 - tmpvar_25)) + tmpvar_25))
    ) + 1e-05)) * (tmpvar_27 / ((
      (3.141593 * tmpvar_28)
     * tmpvar_28) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_32;
  tmpvar_32 = max (0.0, (specularTerm_15 * tmpvar_21));
  specularTerm_15 = tmpvar_32;
  mediump float x_33;
  x_33 = (1.0 - tmpvar_24);
  mediump vec4 tmpvar_34;
  tmpvar_34.w = 1.0;
  tmpvar_34.xyz = ((tmpvar_4 * (tmpvar_13 * 
    (((1.0 + (
      (tmpvar_31 - 1.0)
     * 
      ((x_29 * x_29) * ((x_29 * x_29) * x_29))
    )) * (1.0 + (
      (tmpvar_31 - 1.0)
     * 
      ((x_30 * x_30) * ((x_30 * x_30) * x_30))
    ))) * tmpvar_21)
  )) + ((tmpvar_32 * tmpvar_13) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_33 * x_33) * ((x_33 * x_33) * x_33)))
  )));
  c_1.w = tmpvar_34.w;
  highp float tmpvar_35;
  tmpvar_35 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_34.xyz * vec3(tmpvar_35));
  mediump vec4 xlat_varoutput_36;
  xlat_varoutput_36.xyz = c_1.xyz;
  xlat_varoutput_36.w = 1.0;
  gl_FragData[0] = xlat_varoutput_36;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xyz;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform lowp samplerCube _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_8;
  tmpvar_8 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_9;
  tmpvar_9 = texture2D (_LightTextureB0, vec2(tmpvar_8));
  lowp vec4 tmpvar_10;
  tmpvar_10 = textureCube (_LightTexture0, xlv_TEXCOORD5);
  mediump float atten_11;
  atten_11 = (tmpvar_9.w * tmpvar_10.w);
  mediump vec3 tmpvar_12;
  tmpvar_12 = _LightColor0.xyz;
  tmpvar_12 = (tmpvar_12 * atten_11);
  mediump vec3 viewDir_13;
  viewDir_13 = -(xlv_TEXCOORD1);
  mediump float specularTerm_14;
  mediump float tmpvar_15;
  tmpvar_15 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_16;
  mediump vec3 inVec_17;
  inVec_17 = (tmpvar_7 + viewDir_13);
  tmpvar_16 = (inVec_17 * inversesqrt(max (0.001, 
    dot (inVec_17, inVec_17)
  )));
  mediump float tmpvar_18;
  tmpvar_18 = dot (tmpvar_6, viewDir_13);
  mediump vec3 tmpvar_19;
  if ((tmpvar_18 < 0.0)) {
    tmpvar_19 = (tmpvar_6 + (viewDir_13 * (
      -(tmpvar_18)
     + 1e-05)));
  } else {
    tmpvar_19 = tmpvar_6;
  };
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_19, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_19, tmpvar_16), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_19, viewDir_13), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_7, tmpvar_16), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_15 * tmpvar_15);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_15 * tmpvar_15);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_25 * tmpvar_25);
  mediump float tmpvar_27;
  tmpvar_27 = (((tmpvar_21 * tmpvar_21) * (tmpvar_26 - 1.0)) + 1.0);
  mediump float x_28;
  x_28 = (1.0 - tmpvar_20);
  mediump float x_29;
  x_29 = (1.0 - tmpvar_22);
  mediump float tmpvar_30;
  tmpvar_30 = (0.5 + ((2.0 * tmpvar_23) * (tmpvar_23 * tmpvar_15)));
  specularTerm_14 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_20 * ((tmpvar_22 * (1.0 - tmpvar_24)) + tmpvar_24))
     + 
      (tmpvar_22 * ((tmpvar_20 * (1.0 - tmpvar_24)) + tmpvar_24))
    ) + 1e-05)) * (tmpvar_26 / ((
      (3.141593 * tmpvar_27)
     * tmpvar_27) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_31;
  tmpvar_31 = max (0.0, (specularTerm_14 * tmpvar_20));
  specularTerm_14 = tmpvar_31;
  mediump float x_32;
  x_32 = (1.0 - tmpvar_23);
  mediump vec4 tmpvar_33;
  tmpvar_33.w = 1.0;
  tmpvar_33.xyz = ((tmpvar_4 * (tmpvar_12 * 
    (((1.0 + (
      (tmpvar_30 - 1.0)
     * 
      ((x_28 * x_28) * ((x_28 * x_28) * x_28))
    )) * (1.0 + (
      (tmpvar_30 - 1.0)
     * 
      ((x_29 * x_29) * ((x_29 * x_29) * x_29))
    ))) * tmpvar_20)
  )) + ((tmpvar_31 * tmpvar_12) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_32 * x_32) * ((x_32 * x_32) * x_32)))
  )));
  c_1.w = tmpvar_33.w;
  highp float tmpvar_34;
  tmpvar_34 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_33.xyz * vec3(tmpvar_34));
  mediump vec4 xlat_varoutput_35;
  xlat_varoutput_35.xyz = c_1.xyz;
  xlat_varoutput_35.w = 1.0;
  gl_FragData[0] = xlat_varoutput_35;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_OFF" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  highp vec4 tmpvar_10;
  tmpvar_10 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_11;
  texcoord_11.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_12;
  if ((_UVSec == 0.0)) {
    tmpvar_12 = tmpvar_2;
  } else {
    tmpvar_12 = tmpvar_3;
  };
  texcoord_11.zw = ((tmpvar_12 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_13;
  highp vec3 tmpvar_14;
  tmpvar_14 = normalize((tmpvar_10.xyz - _WorldSpaceCameraPos));
  tmpvar_13 = tmpvar_14;
  highp vec3 norm_15;
  norm_15 = tmpvar_1;
  highp vec4 v_16;
  v_16.x = _World2Object[0].x;
  v_16.y = _World2Object[1].x;
  v_16.z = _World2Object[2].x;
  v_16.w = _World2Object[3].x;
  highp vec4 v_17;
  v_17.x = _World2Object[0].y;
  v_17.y = _World2Object[1].y;
  v_17.z = _World2Object[2].y;
  v_17.w = _World2Object[3].y;
  highp vec4 v_18;
  v_18.x = _World2Object[0].z;
  v_18.y = _World2Object[1].z;
  v_18.z = _World2Object[2].z;
  v_18.w = _World2Object[3].z;
  highp vec3 tmpvar_19;
  tmpvar_19 = normalize(((
    (v_16.xyz * norm_15.x)
   + 
    (v_17.xyz * norm_15.y)
  ) + (v_18.xyz * norm_15.z)));
  highp vec3 dir_20;
  dir_20 = tmpvar_4.xyz;
  highp mat3 tmpvar_21;
  tmpvar_21[0] = _Object2World[0].xyz;
  tmpvar_21[1] = _Object2World[1].xyz;
  tmpvar_21[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_22;
  tmpvar_22.xyz = normalize((tmpvar_21 * dir_20));
  tmpvar_22.w = tmpvar_4.w;
  mediump vec3 normal_23;
  normal_23 = tmpvar_19;
  mediump vec3 tangent_24;
  tangent_24 = tmpvar_22.xyz;
  mediump float tangentSign_25;
  tangentSign_25 = tmpvar_22.w;
  mediump float xlat_varsign_26;
  highp float tmpvar_27;
  tmpvar_27 = (tangentSign_25 * unity_WorldTransformParams.w);
  xlat_varsign_26 = tmpvar_27;
  mediump vec3 tmpvar_28;
  tmpvar_28 = (((normal_23.yzx * tangent_24.zxy) - (normal_23.zxy * tangent_24.yzx)) * xlat_varsign_26);
  mediump mat3 tmpvar_29;
  tmpvar_29[0].x = tangent_24.x;
  tmpvar_29[0].y = tmpvar_28.x;
  tmpvar_29[0].z = normal_23.x;
  tmpvar_29[1].x = tangent_24.y;
  tmpvar_29[1].y = tmpvar_28.y;
  tmpvar_29[1].z = normal_23.y;
  tmpvar_29[2].x = tangent_24.z;
  tmpvar_29[2].y = tmpvar_28.z;
  tmpvar_29[2].z = normal_23.z;
  tangentToWorld_5 = tmpvar_29;
  highp vec3 v_30;
  v_30.x = tangentToWorld_5[0].x;
  v_30.y = tangentToWorld_5[1].x;
  v_30.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].y;
  v_31.y = tangentToWorld_5[1].y;
  v_31.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].z;
  v_32.y = tangentToWorld_5[1].z;
  v_32.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_32;
  highp vec3 tmpvar_33;
  tmpvar_33 = (_WorldSpaceLightPos0.xyz - (tmpvar_10.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_33.x;
  tmpvar_8.w = tmpvar_33.y;
  tmpvar_9.w = tmpvar_33.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_11;
  xlv_TEXCOORD1 = tmpvar_13;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xy;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_6.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2D (_LightTexture0, xlv_TEXCOORD5);
  mediump float atten_9;
  atten_9 = tmpvar_8.w;
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  tmpvar_10 = (tmpvar_10 * atten_9);
  mediump vec3 viewDir_11;
  viewDir_11 = -(xlv_TEXCOORD1);
  mediump float specularTerm_12;
  mediump float tmpvar_13;
  tmpvar_13 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_14;
  mediump vec3 inVec_15;
  inVec_15 = (tmpvar_7 + viewDir_11);
  tmpvar_14 = (inVec_15 * inversesqrt(max (0.001, 
    dot (inVec_15, inVec_15)
  )));
  mediump float tmpvar_16;
  tmpvar_16 = dot (tmpvar_6, viewDir_11);
  mediump vec3 tmpvar_17;
  if ((tmpvar_16 < 0.0)) {
    tmpvar_17 = (tmpvar_6 + (viewDir_11 * (
      -(tmpvar_16)
     + 1e-05)));
  } else {
    tmpvar_17 = tmpvar_6;
  };
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_17, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_17, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_17, viewDir_11), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_7, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_23 * tmpvar_23);
  mediump float tmpvar_25;
  tmpvar_25 = (((tmpvar_19 * tmpvar_19) * (tmpvar_24 - 1.0)) + 1.0);
  mediump float x_26;
  x_26 = (1.0 - tmpvar_18);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_20);
  mediump float tmpvar_28;
  tmpvar_28 = (0.5 + ((2.0 * tmpvar_21) * (tmpvar_21 * tmpvar_13)));
  specularTerm_12 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_18 * ((tmpvar_20 * (1.0 - tmpvar_22)) + tmpvar_22))
     + 
      (tmpvar_20 * ((tmpvar_18 * (1.0 - tmpvar_22)) + tmpvar_22))
    ) + 1e-05)) * (tmpvar_24 / ((
      (3.141593 * tmpvar_25)
     * tmpvar_25) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_29;
  tmpvar_29 = max (0.0, (specularTerm_12 * tmpvar_18));
  specularTerm_12 = tmpvar_29;
  mediump float x_30;
  x_30 = (1.0 - tmpvar_21);
  mediump vec4 tmpvar_31;
  tmpvar_31.w = 1.0;
  tmpvar_31.xyz = ((tmpvar_4 * (tmpvar_10 * 
    (((1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_26 * x_26) * ((x_26 * x_26) * x_26))
    )) * (1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    ))) * tmpvar_18)
  )) + ((tmpvar_29 * tmpvar_10) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_30 * x_30) * ((x_30 * x_30) * x_30)))
  )));
  c_1.w = tmpvar_31.w;
  highp float tmpvar_32;
  tmpvar_32 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_31.xyz * vec3(tmpvar_32));
  mediump vec4 xlat_varoutput_33;
  xlat_varoutput_33.xyz = c_1.xyz;
  xlat_varoutput_33.w = 1.0;
  gl_FragData[0] = xlat_varoutput_33;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "FOG_EXP" "SHADOWS_NONATIVE" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 unity_World2Shadow[4];
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  mediump vec4 tmpvar_11;
  mediump vec4 tmpvar_12;
  highp vec4 tmpvar_13;
  tmpvar_13 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_14;
  texcoord_14.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_15;
  if ((_UVSec == 0.0)) {
    tmpvar_15 = tmpvar_2;
  } else {
    tmpvar_15 = tmpvar_3;
  };
  texcoord_14.zw = ((tmpvar_15 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_16;
  highp vec3 tmpvar_17;
  tmpvar_17 = normalize((tmpvar_13.xyz - _WorldSpaceCameraPos));
  tmpvar_16 = tmpvar_17;
  highp vec3 norm_18;
  norm_18 = tmpvar_1;
  highp vec4 v_19;
  v_19.x = _World2Object[0].x;
  v_19.y = _World2Object[1].x;
  v_19.z = _World2Object[2].x;
  v_19.w = _World2Object[3].x;
  highp vec4 v_20;
  v_20.x = _World2Object[0].y;
  v_20.y = _World2Object[1].y;
  v_20.z = _World2Object[2].y;
  v_20.w = _World2Object[3].y;
  highp vec4 v_21;
  v_21.x = _World2Object[0].z;
  v_21.y = _World2Object[1].z;
  v_21.z = _World2Object[2].z;
  v_21.w = _World2Object[3].z;
  highp vec3 tmpvar_22;
  tmpvar_22 = normalize(((
    (v_19.xyz * norm_18.x)
   + 
    (v_20.xyz * norm_18.y)
  ) + (v_21.xyz * norm_18.z)));
  highp vec3 dir_23;
  dir_23 = tmpvar_4.xyz;
  highp mat3 tmpvar_24;
  tmpvar_24[0] = _Object2World[0].xyz;
  tmpvar_24[1] = _Object2World[1].xyz;
  tmpvar_24[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_25;
  tmpvar_25.xyz = normalize((tmpvar_24 * dir_23));
  tmpvar_25.w = tmpvar_4.w;
  mediump vec3 normal_26;
  normal_26 = tmpvar_22;
  mediump vec3 tangent_27;
  tangent_27 = tmpvar_25.xyz;
  mediump float tangentSign_28;
  tangentSign_28 = tmpvar_25.w;
  mediump float xlat_varsign_29;
  highp float tmpvar_30;
  tmpvar_30 = (tangentSign_28 * unity_WorldTransformParams.w);
  xlat_varsign_29 = tmpvar_30;
  mediump vec3 tmpvar_31;
  tmpvar_31 = (((normal_26.yzx * tangent_27.zxy) - (normal_26.zxy * tangent_27.yzx)) * xlat_varsign_29);
  mediump mat3 tmpvar_32;
  tmpvar_32[0].x = tangent_27.x;
  tmpvar_32[0].y = tmpvar_31.x;
  tmpvar_32[0].z = normal_26.x;
  tmpvar_32[1].x = tangent_27.y;
  tmpvar_32[1].y = tmpvar_31.y;
  tmpvar_32[1].z = normal_26.y;
  tmpvar_32[2].x = tangent_27.z;
  tmpvar_32[2].y = tmpvar_31.z;
  tmpvar_32[2].z = normal_26.z;
  tangentToWorld_6 = tmpvar_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].x;
  v_33.y = tangentToWorld_6[1].x;
  v_33.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_33;
  highp vec3 v_34;
  v_34.x = tangentToWorld_6[0].y;
  v_34.y = tangentToWorld_6[1].y;
  v_34.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_34;
  highp vec3 v_35;
  v_35.x = tangentToWorld_6[0].z;
  v_35.y = tangentToWorld_6[1].z;
  v_35.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_35;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  tmpvar_11 = (_LightMatrix0 * tmpvar_36);
  tmpvar_12 = (unity_World2Shadow[0] * tmpvar_36);
  mediump vec3 tmpvar_37;
  highp vec3 tmpvar_38;
  tmpvar_38 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_13.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_37 = tmpvar_38;
  lightDir_5 = tmpvar_37;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_14;
  xlv_TEXCOORD1 = tmpvar_16;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = tmpvar_11;
  xlv_TEXCOORD6 = tmpvar_12;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform highp sampler2D _ShadowMapTexture;
uniform sampler2D _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_8;
  mediump vec2 P_9;
  P_9 = ((xlv_TEXCOORD5.xy / xlv_TEXCOORD5.w) + 0.5);
  tmpvar_8 = texture2D (_LightTexture0, P_9);
  highp vec3 LightCoord_10;
  LightCoord_10 = xlv_TEXCOORD5.xyz;
  highp float tmpvar_11;
  tmpvar_11 = dot (LightCoord_10, LightCoord_10);
  lowp vec4 tmpvar_12;
  tmpvar_12 = texture2D (_LightTextureB0, vec2(tmpvar_11));
  lowp float tmpvar_13;
  highp vec4 shadowCoord_14;
  shadowCoord_14 = xlv_TEXCOORD6;
  highp vec4 tmpvar_15;
  tmpvar_15 = texture2DProj (_ShadowMapTexture, shadowCoord_14);
  mediump float tmpvar_16;
  if ((tmpvar_15.x < (shadowCoord_14.z / shadowCoord_14.w))) {
    tmpvar_16 = _LightShadowData.x;
  } else {
    tmpvar_16 = 1.0;
  };
  tmpvar_13 = tmpvar_16;
  mediump vec3 tmpvar_17;
  tmpvar_17 = _LightColor0.xyz;
  tmpvar_17 = (tmpvar_17 * ((
    (float((xlv_TEXCOORD5.z > 0.0)) * tmpvar_8.w)
   * tmpvar_12.w) * tmpvar_13));
  mediump vec3 viewDir_18;
  viewDir_18 = -(xlv_TEXCOORD1);
  mediump float specularTerm_19;
  mediump float tmpvar_20;
  tmpvar_20 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_21;
  mediump vec3 inVec_22;
  inVec_22 = (tmpvar_7 + viewDir_18);
  tmpvar_21 = (inVec_22 * inversesqrt(max (0.001, 
    dot (inVec_22, inVec_22)
  )));
  mediump float tmpvar_23;
  tmpvar_23 = dot (tmpvar_6, viewDir_18);
  mediump vec3 tmpvar_24;
  if ((tmpvar_23 < 0.0)) {
    tmpvar_24 = (tmpvar_6 + (viewDir_18 * (
      -(tmpvar_23)
     + 1e-05)));
  } else {
    tmpvar_24 = tmpvar_6;
  };
  mediump float tmpvar_25;
  tmpvar_25 = clamp (dot (tmpvar_24, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_26;
  tmpvar_26 = clamp (dot (tmpvar_24, tmpvar_21), 0.0, 1.0);
  mediump float tmpvar_27;
  tmpvar_27 = clamp (dot (tmpvar_24, viewDir_18), 0.0, 1.0);
  mediump float tmpvar_28;
  tmpvar_28 = clamp (dot (tmpvar_7, tmpvar_21), 0.0, 1.0);
  mediump float tmpvar_29;
  tmpvar_29 = (tmpvar_20 * tmpvar_20);
  mediump float tmpvar_30;
  tmpvar_30 = (tmpvar_20 * tmpvar_20);
  mediump float tmpvar_31;
  tmpvar_31 = (tmpvar_30 * tmpvar_30);
  mediump float tmpvar_32;
  tmpvar_32 = (((tmpvar_26 * tmpvar_26) * (tmpvar_31 - 1.0)) + 1.0);
  mediump float x_33;
  x_33 = (1.0 - tmpvar_25);
  mediump float x_34;
  x_34 = (1.0 - tmpvar_27);
  mediump float tmpvar_35;
  tmpvar_35 = (0.5 + ((2.0 * tmpvar_28) * (tmpvar_28 * tmpvar_20)));
  specularTerm_19 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_25 * ((tmpvar_27 * (1.0 - tmpvar_29)) + tmpvar_29))
     + 
      (tmpvar_27 * ((tmpvar_25 * (1.0 - tmpvar_29)) + tmpvar_29))
    ) + 1e-05)) * (tmpvar_31 / ((
      (3.141593 * tmpvar_32)
     * tmpvar_32) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_36;
  tmpvar_36 = max (0.0, (specularTerm_19 * tmpvar_25));
  specularTerm_19 = tmpvar_36;
  mediump float x_37;
  x_37 = (1.0 - tmpvar_28);
  mediump vec4 tmpvar_38;
  tmpvar_38.w = 1.0;
  tmpvar_38.xyz = ((tmpvar_4 * (tmpvar_17 * 
    (((1.0 + (
      (tmpvar_35 - 1.0)
     * 
      ((x_33 * x_33) * ((x_33 * x_33) * x_33))
    )) * (1.0 + (
      (tmpvar_35 - 1.0)
     * 
      ((x_34 * x_34) * ((x_34 * x_34) * x_34))
    ))) * tmpvar_25)
  )) + ((tmpvar_36 * tmpvar_17) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_37 * x_37) * ((x_37 * x_37) * x_37)))
  )));
  c_1.w = tmpvar_38.w;
  highp float tmpvar_39;
  tmpvar_39 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_38.xyz * vec3(tmpvar_39));
  mediump vec4 xlat_varoutput_40;
  xlat_varoutput_40.xyz = c_1.xyz;
  xlat_varoutput_40.w = 1.0;
  gl_FragData[0] = xlat_varoutput_40;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "SHADOWS_NATIVE" "FOG_EXP" }
"#version 100

#ifdef VERTEX
#extension GL_EXT_shadow_samplers : enable
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp mat4 unity_World2Shadow[4];
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  mediump vec4 tmpvar_11;
  mediump vec4 tmpvar_12;
  highp vec4 tmpvar_13;
  tmpvar_13 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_14;
  texcoord_14.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_15;
  if ((_UVSec == 0.0)) {
    tmpvar_15 = tmpvar_2;
  } else {
    tmpvar_15 = tmpvar_3;
  };
  texcoord_14.zw = ((tmpvar_15 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_16;
  highp vec3 tmpvar_17;
  tmpvar_17 = normalize((tmpvar_13.xyz - _WorldSpaceCameraPos));
  tmpvar_16 = tmpvar_17;
  highp vec3 norm_18;
  norm_18 = tmpvar_1;
  highp vec4 v_19;
  v_19.x = _World2Object[0].x;
  v_19.y = _World2Object[1].x;
  v_19.z = _World2Object[2].x;
  v_19.w = _World2Object[3].x;
  highp vec4 v_20;
  v_20.x = _World2Object[0].y;
  v_20.y = _World2Object[1].y;
  v_20.z = _World2Object[2].y;
  v_20.w = _World2Object[3].y;
  highp vec4 v_21;
  v_21.x = _World2Object[0].z;
  v_21.y = _World2Object[1].z;
  v_21.z = _World2Object[2].z;
  v_21.w = _World2Object[3].z;
  highp vec3 tmpvar_22;
  tmpvar_22 = normalize(((
    (v_19.xyz * norm_18.x)
   + 
    (v_20.xyz * norm_18.y)
  ) + (v_21.xyz * norm_18.z)));
  highp vec3 dir_23;
  dir_23 = tmpvar_4.xyz;
  highp mat3 tmpvar_24;
  tmpvar_24[0] = _Object2World[0].xyz;
  tmpvar_24[1] = _Object2World[1].xyz;
  tmpvar_24[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_25;
  tmpvar_25.xyz = normalize((tmpvar_24 * dir_23));
  tmpvar_25.w = tmpvar_4.w;
  mediump vec3 normal_26;
  normal_26 = tmpvar_22;
  mediump vec3 tangent_27;
  tangent_27 = tmpvar_25.xyz;
  mediump float tangentSign_28;
  tangentSign_28 = tmpvar_25.w;
  mediump float xlat_varsign_29;
  highp float tmpvar_30;
  tmpvar_30 = (tangentSign_28 * unity_WorldTransformParams.w);
  xlat_varsign_29 = tmpvar_30;
  mediump vec3 tmpvar_31;
  tmpvar_31 = (((normal_26.yzx * tangent_27.zxy) - (normal_26.zxy * tangent_27.yzx)) * xlat_varsign_29);
  mediump mat3 tmpvar_32;
  tmpvar_32[0].x = tangent_27.x;
  tmpvar_32[0].y = tmpvar_31.x;
  tmpvar_32[0].z = normal_26.x;
  tmpvar_32[1].x = tangent_27.y;
  tmpvar_32[1].y = tmpvar_31.y;
  tmpvar_32[1].z = normal_26.y;
  tmpvar_32[2].x = tangent_27.z;
  tmpvar_32[2].y = tmpvar_31.z;
  tmpvar_32[2].z = normal_26.z;
  tangentToWorld_6 = tmpvar_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].x;
  v_33.y = tangentToWorld_6[1].x;
  v_33.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_33;
  highp vec3 v_34;
  v_34.x = tangentToWorld_6[0].y;
  v_34.y = tangentToWorld_6[1].y;
  v_34.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_34;
  highp vec3 v_35;
  v_35.x = tangentToWorld_6[0].z;
  v_35.y = tangentToWorld_6[1].z;
  v_35.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_35;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  tmpvar_11 = (_LightMatrix0 * tmpvar_36);
  tmpvar_12 = (unity_World2Shadow[0] * tmpvar_36);
  mediump vec3 tmpvar_37;
  highp vec3 tmpvar_38;
  tmpvar_38 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_13.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_37 = tmpvar_38;
  lightDir_5 = tmpvar_37;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_14;
  xlv_TEXCOORD1 = tmpvar_16;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  xlv_TEXCOORD5 = tmpvar_11;
  xlv_TEXCOORD6 = tmpvar_12;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
#extension GL_EXT_shadow_samplers : enable
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform lowp sampler2DShadow _ShadowMapTexture;
uniform sampler2D _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_8;
  mediump vec2 P_9;
  P_9 = ((xlv_TEXCOORD5.xy / xlv_TEXCOORD5.w) + 0.5);
  tmpvar_8 = texture2D (_LightTexture0, P_9);
  mediump float tmpvar_10;
  tmpvar_10 = dot (xlv_TEXCOORD5.xyz, xlv_TEXCOORD5.xyz);
  lowp vec4 tmpvar_11;
  tmpvar_11 = texture2D (_LightTextureB0, vec2(tmpvar_10));
  lowp float tmpvar_12;
  highp vec4 shadowCoord_13;
  shadowCoord_13 = xlv_TEXCOORD6;
  mediump float shadow_14;
  lowp float tmpvar_15;
  tmpvar_15 = shadow2DProjEXT (_ShadowMapTexture, shadowCoord_13);
  mediump float tmpvar_16;
  tmpvar_16 = tmpvar_15;
  shadow_14 = (_LightShadowData.x + (tmpvar_16 * (1.0 - _LightShadowData.x)));
  tmpvar_12 = shadow_14;
  mediump vec3 tmpvar_17;
  tmpvar_17 = _LightColor0.xyz;
  tmpvar_17 = (tmpvar_17 * ((
    (float((xlv_TEXCOORD5.z > 0.0)) * tmpvar_8.w)
   * tmpvar_11.w) * tmpvar_12));
  mediump vec3 viewDir_18;
  viewDir_18 = -(xlv_TEXCOORD1);
  mediump float specularTerm_19;
  mediump float tmpvar_20;
  tmpvar_20 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_21;
  mediump vec3 inVec_22;
  inVec_22 = (tmpvar_7 + viewDir_18);
  tmpvar_21 = (inVec_22 * inversesqrt(max (0.001, 
    dot (inVec_22, inVec_22)
  )));
  mediump float tmpvar_23;
  tmpvar_23 = dot (tmpvar_6, viewDir_18);
  mediump vec3 tmpvar_24;
  if ((tmpvar_23 < 0.0)) {
    tmpvar_24 = (tmpvar_6 + (viewDir_18 * (
      -(tmpvar_23)
     + 1e-05)));
  } else {
    tmpvar_24 = tmpvar_6;
  };
  mediump float tmpvar_25;
  tmpvar_25 = clamp (dot (tmpvar_24, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_26;
  tmpvar_26 = clamp (dot (tmpvar_24, tmpvar_21), 0.0, 1.0);
  mediump float tmpvar_27;
  tmpvar_27 = clamp (dot (tmpvar_24, viewDir_18), 0.0, 1.0);
  mediump float tmpvar_28;
  tmpvar_28 = clamp (dot (tmpvar_7, tmpvar_21), 0.0, 1.0);
  mediump float tmpvar_29;
  tmpvar_29 = (tmpvar_20 * tmpvar_20);
  mediump float tmpvar_30;
  tmpvar_30 = (tmpvar_20 * tmpvar_20);
  mediump float tmpvar_31;
  tmpvar_31 = (tmpvar_30 * tmpvar_30);
  mediump float tmpvar_32;
  tmpvar_32 = (((tmpvar_26 * tmpvar_26) * (tmpvar_31 - 1.0)) + 1.0);
  mediump float x_33;
  x_33 = (1.0 - tmpvar_25);
  mediump float x_34;
  x_34 = (1.0 - tmpvar_27);
  mediump float tmpvar_35;
  tmpvar_35 = (0.5 + ((2.0 * tmpvar_28) * (tmpvar_28 * tmpvar_20)));
  specularTerm_19 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_25 * ((tmpvar_27 * (1.0 - tmpvar_29)) + tmpvar_29))
     + 
      (tmpvar_27 * ((tmpvar_25 * (1.0 - tmpvar_29)) + tmpvar_29))
    ) + 1e-05)) * (tmpvar_31 / ((
      (3.141593 * tmpvar_32)
     * tmpvar_32) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_36;
  tmpvar_36 = max (0.0, (specularTerm_19 * tmpvar_25));
  specularTerm_19 = tmpvar_36;
  mediump float x_37;
  x_37 = (1.0 - tmpvar_28);
  mediump vec4 tmpvar_38;
  tmpvar_38.w = 1.0;
  tmpvar_38.xyz = ((tmpvar_4 * (tmpvar_17 * 
    (((1.0 + (
      (tmpvar_35 - 1.0)
     * 
      ((x_33 * x_33) * ((x_33 * x_33) * x_33))
    )) * (1.0 + (
      (tmpvar_35 - 1.0)
     * 
      ((x_34 * x_34) * ((x_34 * x_34) * x_34))
    ))) * tmpvar_25)
  )) + ((tmpvar_36 * tmpvar_17) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_37 * x_37) * ((x_37 * x_37) * x_37)))
  )));
  c_1.w = tmpvar_38.w;
  highp float tmpvar_39;
  tmpvar_39 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_38.xyz * vec3(tmpvar_39));
  mediump vec4 xlat_varoutput_40;
  xlat_varoutput_40.xyz = c_1.xyz;
  xlat_varoutput_40.w = 1.0;
  gl_FragData[0] = xlat_varoutput_40;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_5 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].x;
  v_31.y = tangentToWorld_5[1].x;
  v_31.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].y;
  v_32.y = tangentToWorld_5[1].y;
  v_32.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_5[0].z;
  v_33.y = tangentToWorld_5[1].z;
  v_33.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_33;
  highp vec4 o_34;
  highp vec4 tmpvar_35;
  tmpvar_35 = (tmpvar_6 * 0.5);
  highp vec2 tmpvar_36;
  tmpvar_36.x = tmpvar_35.x;
  tmpvar_36.y = (tmpvar_35.y * _ProjectionParams.x);
  o_34.xy = (tmpvar_36 + tmpvar_35.w);
  o_34.zw = tmpvar_6.zw;
  tmpvar_10 = o_34;
  highp vec3 tmpvar_37;
  tmpvar_37 = (_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_37.x;
  tmpvar_8.w = tmpvar_37.y;
  tmpvar_9.w = tmpvar_37.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD5 = tmpvar_10;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_6.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _ShadowMapTexture;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying mediump vec4 xlv_TEXCOORD5;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  lowp float tmpvar_8;
  tmpvar_8 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD5).x;
  mediump float atten_9;
  atten_9 = tmpvar_8;
  mediump vec3 tmpvar_10;
  tmpvar_10 = _LightColor0.xyz;
  tmpvar_10 = (tmpvar_10 * atten_9);
  mediump vec3 viewDir_11;
  viewDir_11 = -(xlv_TEXCOORD1);
  mediump float specularTerm_12;
  mediump float tmpvar_13;
  tmpvar_13 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_14;
  mediump vec3 inVec_15;
  inVec_15 = (tmpvar_7 + viewDir_11);
  tmpvar_14 = (inVec_15 * inversesqrt(max (0.001, 
    dot (inVec_15, inVec_15)
  )));
  mediump float tmpvar_16;
  tmpvar_16 = dot (tmpvar_6, viewDir_11);
  mediump vec3 tmpvar_17;
  if ((tmpvar_16 < 0.0)) {
    tmpvar_17 = (tmpvar_6 + (viewDir_11 * (
      -(tmpvar_16)
     + 1e-05)));
  } else {
    tmpvar_17 = tmpvar_6;
  };
  mediump float tmpvar_18;
  tmpvar_18 = clamp (dot (tmpvar_17, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_17, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_17, viewDir_11), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_7, tmpvar_14), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_13 * tmpvar_13);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_23 * tmpvar_23);
  mediump float tmpvar_25;
  tmpvar_25 = (((tmpvar_19 * tmpvar_19) * (tmpvar_24 - 1.0)) + 1.0);
  mediump float x_26;
  x_26 = (1.0 - tmpvar_18);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_20);
  mediump float tmpvar_28;
  tmpvar_28 = (0.5 + ((2.0 * tmpvar_21) * (tmpvar_21 * tmpvar_13)));
  specularTerm_12 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_18 * ((tmpvar_20 * (1.0 - tmpvar_22)) + tmpvar_22))
     + 
      (tmpvar_20 * ((tmpvar_18 * (1.0 - tmpvar_22)) + tmpvar_22))
    ) + 1e-05)) * (tmpvar_24 / ((
      (3.141593 * tmpvar_25)
     * tmpvar_25) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_29;
  tmpvar_29 = max (0.0, (specularTerm_12 * tmpvar_18));
  specularTerm_12 = tmpvar_29;
  mediump float x_30;
  x_30 = (1.0 - tmpvar_21);
  mediump vec4 tmpvar_31;
  tmpvar_31.w = 1.0;
  tmpvar_31.xyz = ((tmpvar_4 * (tmpvar_10 * 
    (((1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_26 * x_26) * ((x_26 * x_26) * x_26))
    )) * (1.0 + (
      (tmpvar_28 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    ))) * tmpvar_18)
  )) + ((tmpvar_29 * tmpvar_10) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_30 * x_30) * ((x_30 * x_30) * x_30)))
  )));
  c_1.w = tmpvar_31.w;
  highp float tmpvar_32;
  tmpvar_32 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_31.xyz * vec3(tmpvar_32));
  mediump vec4 xlat_varoutput_33;
  xlat_varoutput_33.xyz = c_1.xyz;
  xlat_varoutput_33.w = 1.0;
  gl_FragData[0] = xlat_varoutput_33;
}


#endif
"
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_SCREEN" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _ProjectionParams;
uniform mediump vec4 _WorldSpaceLightPos0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp mat3 tangentToWorld_5;
  highp vec4 tmpvar_6;
  mediump vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_6 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_5 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_5[0].x;
  v_31.y = tangentToWorld_5[1].x;
  v_31.z = tangentToWorld_5[2].x;
  tmpvar_7.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_5[0].y;
  v_32.y = tangentToWorld_5[1].y;
  v_32.z = tangentToWorld_5[2].y;
  tmpvar_8.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_5[0].z;
  v_33.y = tangentToWorld_5[1].z;
  v_33.z = tangentToWorld_5[2].z;
  tmpvar_9.xyz = v_33;
  highp vec4 o_34;
  highp vec4 tmpvar_35;
  tmpvar_35 = (tmpvar_6 * 0.5);
  highp vec2 tmpvar_36;
  tmpvar_36.x = tmpvar_35.x;
  tmpvar_36.y = (tmpvar_35.y * _ProjectionParams.x);
  o_34.xy = (tmpvar_36 + tmpvar_35.w);
  o_34.zw = tmpvar_6.zw;
  tmpvar_10 = o_34;
  highp vec3 tmpvar_37;
  tmpvar_37 = (_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w));
  tmpvar_7.w = tmpvar_37.x;
  tmpvar_8.w = tmpvar_37.y;
  tmpvar_9.w = tmpvar_37.z;
  gl_Position = tmpvar_6;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_7;
  xlv_TEXCOORD2_1 = tmpvar_8;
  xlv_TEXCOORD2_2 = tmpvar_9;
  xlv_TEXCOORD5 = (_LightMatrix0 * (_Object2World * _glesVertex)).xy;
  xlv_TEXCOORD6 = tmpvar_10;
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_6.z)));
}


#endif
#ifdef FRAGMENT
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform sampler2D _ShadowMapTexture;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec2 xlv_TEXCOORD5;
varying mediump vec4 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2D (_LightTexture0, xlv_TEXCOORD5);
  lowp vec4 tmpvar_9;
  tmpvar_9 = texture2DProj (_ShadowMapTexture, xlv_TEXCOORD6);
  mediump float atten_10;
  atten_10 = (tmpvar_8.w * tmpvar_9.x);
  mediump vec3 tmpvar_11;
  tmpvar_11 = _LightColor0.xyz;
  tmpvar_11 = (tmpvar_11 * atten_10);
  mediump vec3 viewDir_12;
  viewDir_12 = -(xlv_TEXCOORD1);
  mediump float specularTerm_13;
  mediump float tmpvar_14;
  tmpvar_14 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_15;
  mediump vec3 inVec_16;
  inVec_16 = (tmpvar_7 + viewDir_12);
  tmpvar_15 = (inVec_16 * inversesqrt(max (0.001, 
    dot (inVec_16, inVec_16)
  )));
  mediump float tmpvar_17;
  tmpvar_17 = dot (tmpvar_6, viewDir_12);
  mediump vec3 tmpvar_18;
  if ((tmpvar_17 < 0.0)) {
    tmpvar_18 = (tmpvar_6 + (viewDir_12 * (
      -(tmpvar_17)
     + 1e-05)));
  } else {
    tmpvar_18 = tmpvar_6;
  };
  mediump float tmpvar_19;
  tmpvar_19 = clamp (dot (tmpvar_18, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_20;
  tmpvar_20 = clamp (dot (tmpvar_18, tmpvar_15), 0.0, 1.0);
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_18, viewDir_12), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_7, tmpvar_15), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = (tmpvar_14 * tmpvar_14);
  mediump float tmpvar_24;
  tmpvar_24 = (tmpvar_14 * tmpvar_14);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_24 * tmpvar_24);
  mediump float tmpvar_26;
  tmpvar_26 = (((tmpvar_20 * tmpvar_20) * (tmpvar_25 - 1.0)) + 1.0);
  mediump float x_27;
  x_27 = (1.0 - tmpvar_19);
  mediump float x_28;
  x_28 = (1.0 - tmpvar_21);
  mediump float tmpvar_29;
  tmpvar_29 = (0.5 + ((2.0 * tmpvar_22) * (tmpvar_22 * tmpvar_14)));
  specularTerm_13 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_19 * ((tmpvar_21 * (1.0 - tmpvar_23)) + tmpvar_23))
     + 
      (tmpvar_21 * ((tmpvar_19 * (1.0 - tmpvar_23)) + tmpvar_23))
    ) + 1e-05)) * (tmpvar_25 / ((
      (3.141593 * tmpvar_26)
     * tmpvar_26) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_30;
  tmpvar_30 = max (0.0, (specularTerm_13 * tmpvar_19));
  specularTerm_13 = tmpvar_30;
  mediump float x_31;
  x_31 = (1.0 - tmpvar_22);
  mediump vec4 tmpvar_32;
  tmpvar_32.w = 1.0;
  tmpvar_32.xyz = ((tmpvar_4 * (tmpvar_11 * 
    (((1.0 + (
      (tmpvar_29 - 1.0)
     * 
      ((x_27 * x_27) * ((x_27 * x_27) * x_27))
    )) * (1.0 + (
      (tmpvar_29 - 1.0)
     * 
      ((x_28 * x_28) * ((x_28 * x_28) * x_28))
    ))) * tmpvar_19)
  )) + ((tmpvar_30 * tmpvar_11) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_31 * x_31) * ((x_31 * x_31) * x_31)))
  )));
  c_1.w = tmpvar_32.w;
  highp float tmpvar_33;
  tmpvar_33 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_32.xyz * vec3(tmpvar_33));
  mediump vec4 xlat_varoutput_34;
  xlat_varoutput_34.xyz = c_1.xyz;
  xlat_varoutput_34.w = 1.0;
  gl_FragData[0] = xlat_varoutput_34;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_CUBE" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp vec4 _LightPositionRange;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  xlv_TEXCOORD5 = (_LightMatrix0 * tmpvar_36).xyz;
  xlv_TEXCOORD6 = (tmpvar_36.xyz - _LightPositionRange.xyz);
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _LightPositionRange;
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform highp samplerCube _ShadowMapTexture;
uniform sampler2D _LightTexture0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_8;
  tmpvar_8 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_9;
  tmpvar_9 = texture2D (_LightTexture0, vec2(tmpvar_8));
  highp float mydist_10;
  mydist_10 = ((sqrt(
    dot (xlv_TEXCOORD6, xlv_TEXCOORD6)
  ) * _LightPositionRange.w) * 0.97);
  highp float tmpvar_11;
  tmpvar_11 = dot (textureCube (_ShadowMapTexture, xlv_TEXCOORD6), vec4(1.0, 0.003921569, 1.53787e-05, 6.030863e-08));
  mediump float tmpvar_12;
  if ((tmpvar_11 < mydist_10)) {
    tmpvar_12 = _LightShadowData.x;
  } else {
    tmpvar_12 = 1.0;
  };
  mediump vec3 tmpvar_13;
  tmpvar_13 = _LightColor0.xyz;
  tmpvar_13 = (tmpvar_13 * (tmpvar_9.w * tmpvar_12));
  mediump vec3 viewDir_14;
  viewDir_14 = -(xlv_TEXCOORD1);
  mediump float specularTerm_15;
  mediump float tmpvar_16;
  tmpvar_16 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_17;
  mediump vec3 inVec_18;
  inVec_18 = (tmpvar_7 + viewDir_14);
  tmpvar_17 = (inVec_18 * inversesqrt(max (0.001, 
    dot (inVec_18, inVec_18)
  )));
  mediump float tmpvar_19;
  tmpvar_19 = dot (tmpvar_6, viewDir_14);
  mediump vec3 tmpvar_20;
  if ((tmpvar_19 < 0.0)) {
    tmpvar_20 = (tmpvar_6 + (viewDir_14 * (
      -(tmpvar_19)
     + 1e-05)));
  } else {
    tmpvar_20 = tmpvar_6;
  };
  mediump float tmpvar_21;
  tmpvar_21 = clamp (dot (tmpvar_20, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_20, tmpvar_17), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_20, viewDir_14), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = clamp (dot (tmpvar_7, tmpvar_17), 0.0, 1.0);
  mediump float tmpvar_25;
  tmpvar_25 = (tmpvar_16 * tmpvar_16);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_16 * tmpvar_16);
  mediump float tmpvar_27;
  tmpvar_27 = (tmpvar_26 * tmpvar_26);
  mediump float tmpvar_28;
  tmpvar_28 = (((tmpvar_22 * tmpvar_22) * (tmpvar_27 - 1.0)) + 1.0);
  mediump float x_29;
  x_29 = (1.0 - tmpvar_21);
  mediump float x_30;
  x_30 = (1.0 - tmpvar_23);
  mediump float tmpvar_31;
  tmpvar_31 = (0.5 + ((2.0 * tmpvar_24) * (tmpvar_24 * tmpvar_16)));
  specularTerm_15 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_21 * ((tmpvar_23 * (1.0 - tmpvar_25)) + tmpvar_25))
     + 
      (tmpvar_23 * ((tmpvar_21 * (1.0 - tmpvar_25)) + tmpvar_25))
    ) + 1e-05)) * (tmpvar_27 / ((
      (3.141593 * tmpvar_28)
     * tmpvar_28) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_32;
  tmpvar_32 = max (0.0, (specularTerm_15 * tmpvar_21));
  specularTerm_15 = tmpvar_32;
  mediump float x_33;
  x_33 = (1.0 - tmpvar_24);
  mediump vec4 tmpvar_34;
  tmpvar_34.w = 1.0;
  tmpvar_34.xyz = ((tmpvar_4 * (tmpvar_13 * 
    (((1.0 + (
      (tmpvar_31 - 1.0)
     * 
      ((x_29 * x_29) * ((x_29 * x_29) * x_29))
    )) * (1.0 + (
      (tmpvar_31 - 1.0)
     * 
      ((x_30 * x_30) * ((x_30 * x_30) * x_30))
    ))) * tmpvar_21)
  )) + ((tmpvar_32 * tmpvar_13) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_33 * x_33) * ((x_33 * x_33) * x_33)))
  )));
  c_1.w = tmpvar_34.w;
  highp float tmpvar_35;
  tmpvar_35 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_34.xyz * vec3(tmpvar_35));
  mediump vec4 xlat_varoutput_36;
  xlat_varoutput_36.xyz = c_1.xyz;
  xlat_varoutput_36.w = 1.0;
  gl_FragData[0] = xlat_varoutput_36;
}


#endif
"
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_CUBE" "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesTANGENT;
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
attribute vec4 _glesMultiTexCoord0;
attribute vec4 _glesMultiTexCoord1;
uniform highp vec3 _WorldSpaceCameraPos;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp vec4 _LightPositionRange;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp vec4 unity_WorldTransformParams;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
uniform highp vec4 _DetailAlbedoMap_ST;
uniform mediump float _UVSec;
uniform mediump mat4 _LightMatrix0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec3 tmpvar_1;
  tmpvar_1 = _glesNormal;
  highp vec2 tmpvar_2;
  tmpvar_2 = _glesMultiTexCoord0.xy;
  highp vec2 tmpvar_3;
  tmpvar_3 = _glesMultiTexCoord1.xy;
  mediump vec4 tmpvar_4;
  tmpvar_4 = _glesTANGENT;
  highp vec3 lightDir_5;
  highp mat3 tangentToWorld_6;
  highp vec4 tmpvar_7;
  mediump vec4 tmpvar_8;
  mediump vec4 tmpvar_9;
  mediump vec4 tmpvar_10;
  highp vec4 tmpvar_11;
  tmpvar_11 = (_Object2World * _glesVertex);
  tmpvar_7 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 texcoord_12;
  texcoord_12.xy = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  highp vec2 tmpvar_13;
  if ((_UVSec == 0.0)) {
    tmpvar_13 = tmpvar_2;
  } else {
    tmpvar_13 = tmpvar_3;
  };
  texcoord_12.zw = ((tmpvar_13 * _DetailAlbedoMap_ST.xy) + _DetailAlbedoMap_ST.zw);
  mediump vec3 tmpvar_14;
  highp vec3 tmpvar_15;
  tmpvar_15 = normalize((tmpvar_11.xyz - _WorldSpaceCameraPos));
  tmpvar_14 = tmpvar_15;
  highp vec3 norm_16;
  norm_16 = tmpvar_1;
  highp vec4 v_17;
  v_17.x = _World2Object[0].x;
  v_17.y = _World2Object[1].x;
  v_17.z = _World2Object[2].x;
  v_17.w = _World2Object[3].x;
  highp vec4 v_18;
  v_18.x = _World2Object[0].y;
  v_18.y = _World2Object[1].y;
  v_18.z = _World2Object[2].y;
  v_18.w = _World2Object[3].y;
  highp vec4 v_19;
  v_19.x = _World2Object[0].z;
  v_19.y = _World2Object[1].z;
  v_19.z = _World2Object[2].z;
  v_19.w = _World2Object[3].z;
  highp vec3 tmpvar_20;
  tmpvar_20 = normalize(((
    (v_17.xyz * norm_16.x)
   + 
    (v_18.xyz * norm_16.y)
  ) + (v_19.xyz * norm_16.z)));
  highp vec3 dir_21;
  dir_21 = tmpvar_4.xyz;
  highp mat3 tmpvar_22;
  tmpvar_22[0] = _Object2World[0].xyz;
  tmpvar_22[1] = _Object2World[1].xyz;
  tmpvar_22[2] = _Object2World[2].xyz;
  highp vec4 tmpvar_23;
  tmpvar_23.xyz = normalize((tmpvar_22 * dir_21));
  tmpvar_23.w = tmpvar_4.w;
  mediump vec3 normal_24;
  normal_24 = tmpvar_20;
  mediump vec3 tangent_25;
  tangent_25 = tmpvar_23.xyz;
  mediump float tangentSign_26;
  tangentSign_26 = tmpvar_23.w;
  mediump float xlat_varsign_27;
  highp float tmpvar_28;
  tmpvar_28 = (tangentSign_26 * unity_WorldTransformParams.w);
  xlat_varsign_27 = tmpvar_28;
  mediump vec3 tmpvar_29;
  tmpvar_29 = (((normal_24.yzx * tangent_25.zxy) - (normal_24.zxy * tangent_25.yzx)) * xlat_varsign_27);
  mediump mat3 tmpvar_30;
  tmpvar_30[0].x = tangent_25.x;
  tmpvar_30[0].y = tmpvar_29.x;
  tmpvar_30[0].z = normal_24.x;
  tmpvar_30[1].x = tangent_25.y;
  tmpvar_30[1].y = tmpvar_29.y;
  tmpvar_30[1].z = normal_24.y;
  tmpvar_30[2].x = tangent_25.z;
  tmpvar_30[2].y = tmpvar_29.z;
  tmpvar_30[2].z = normal_24.z;
  tangentToWorld_6 = tmpvar_30;
  highp vec3 v_31;
  v_31.x = tangentToWorld_6[0].x;
  v_31.y = tangentToWorld_6[1].x;
  v_31.z = tangentToWorld_6[2].x;
  tmpvar_8.xyz = v_31;
  highp vec3 v_32;
  v_32.x = tangentToWorld_6[0].y;
  v_32.y = tangentToWorld_6[1].y;
  v_32.z = tangentToWorld_6[2].y;
  tmpvar_9.xyz = v_32;
  highp vec3 v_33;
  v_33.x = tangentToWorld_6[0].z;
  v_33.y = tangentToWorld_6[1].z;
  v_33.z = tangentToWorld_6[2].z;
  tmpvar_10.xyz = v_33;
  mediump vec3 tmpvar_34;
  highp vec3 tmpvar_35;
  tmpvar_35 = normalize((_WorldSpaceLightPos0.xyz - (tmpvar_11.xyz * _WorldSpaceLightPos0.w)));
  tmpvar_34 = tmpvar_35;
  lightDir_5 = tmpvar_34;
  tmpvar_8.w = lightDir_5.x;
  tmpvar_9.w = lightDir_5.y;
  tmpvar_10.w = lightDir_5.z;
  gl_Position = tmpvar_7;
  xlv_TEXCOORD0 = texcoord_12;
  xlv_TEXCOORD1 = tmpvar_14;
  xlv_TEXCOORD2 = tmpvar_8;
  xlv_TEXCOORD2_1 = tmpvar_9;
  xlv_TEXCOORD2_2 = tmpvar_10;
  highp vec4 tmpvar_36;
  tmpvar_36 = (_Object2World * _glesVertex);
  xlv_TEXCOORD5 = (_LightMatrix0 * tmpvar_36).xyz;
  xlv_TEXCOORD6 = (tmpvar_36.xyz - _LightPositionRange.xyz);
  xlv_TEXCOORD7 = exp2(-((unity_FogParams.y * tmpvar_7.z)));
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _LightPositionRange;
uniform mediump vec4 _LightShadowData;
uniform mediump vec4 unity_ColorSpaceDielectricSpec;
uniform lowp vec4 _LightColor0;
uniform mediump vec4 _Color;
uniform sampler2D _MainTex;
uniform mediump float _Metallic;
uniform mediump float _Glossiness;
uniform highp samplerCube _ShadowMapTexture;
uniform lowp samplerCube _LightTexture0;
uniform sampler2D _LightTextureB0;
varying highp vec4 xlv_TEXCOORD0;
varying mediump vec3 xlv_TEXCOORD1;
varying mediump vec4 xlv_TEXCOORD2;
varying mediump vec4 xlv_TEXCOORD2_1;
varying mediump vec4 xlv_TEXCOORD2_2;
varying highp vec3 xlv_TEXCOORD5;
varying highp vec3 xlv_TEXCOORD6;
varying highp float xlv_TEXCOORD7;
void main ()
{
  mediump vec4 c_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0.xy);
  mediump vec3 tmpvar_3;
  tmpvar_3 = (_Color.xyz * tmpvar_2.xyz);
  mediump vec3 tmpvar_4;
  mediump vec3 tmpvar_5;
  tmpvar_5 = mix (unity_ColorSpaceDielectricSpec.xyz, tmpvar_3, vec3(_Metallic));
  tmpvar_4 = (tmpvar_3 * (unity_ColorSpaceDielectricSpec.w - (_Metallic * unity_ColorSpaceDielectricSpec.w)));
  mediump vec3 tmpvar_6;
  tmpvar_6 = normalize(xlv_TEXCOORD2_2.xyz);
  mediump vec3 tmpvar_7;
  tmpvar_7.x = xlv_TEXCOORD2.w;
  tmpvar_7.y = xlv_TEXCOORD2_1.w;
  tmpvar_7.z = xlv_TEXCOORD2_2.w;
  highp float tmpvar_8;
  tmpvar_8 = dot (xlv_TEXCOORD5, xlv_TEXCOORD5);
  lowp vec4 tmpvar_9;
  tmpvar_9 = texture2D (_LightTextureB0, vec2(tmpvar_8));
  lowp vec4 tmpvar_10;
  tmpvar_10 = textureCube (_LightTexture0, xlv_TEXCOORD5);
  highp float mydist_11;
  mydist_11 = ((sqrt(
    dot (xlv_TEXCOORD6, xlv_TEXCOORD6)
  ) * _LightPositionRange.w) * 0.97);
  highp float tmpvar_12;
  tmpvar_12 = dot (textureCube (_ShadowMapTexture, xlv_TEXCOORD6), vec4(1.0, 0.003921569, 1.53787e-05, 6.030863e-08));
  mediump float tmpvar_13;
  if ((tmpvar_12 < mydist_11)) {
    tmpvar_13 = _LightShadowData.x;
  } else {
    tmpvar_13 = 1.0;
  };
  mediump vec3 tmpvar_14;
  tmpvar_14 = _LightColor0.xyz;
  tmpvar_14 = (tmpvar_14 * ((tmpvar_9.w * tmpvar_10.w) * tmpvar_13));
  mediump vec3 viewDir_15;
  viewDir_15 = -(xlv_TEXCOORD1);
  mediump float specularTerm_16;
  mediump float tmpvar_17;
  tmpvar_17 = (1.0 - _Glossiness);
  mediump vec3 tmpvar_18;
  mediump vec3 inVec_19;
  inVec_19 = (tmpvar_7 + viewDir_15);
  tmpvar_18 = (inVec_19 * inversesqrt(max (0.001, 
    dot (inVec_19, inVec_19)
  )));
  mediump float tmpvar_20;
  tmpvar_20 = dot (tmpvar_6, viewDir_15);
  mediump vec3 tmpvar_21;
  if ((tmpvar_20 < 0.0)) {
    tmpvar_21 = (tmpvar_6 + (viewDir_15 * (
      -(tmpvar_20)
     + 1e-05)));
  } else {
    tmpvar_21 = tmpvar_6;
  };
  mediump float tmpvar_22;
  tmpvar_22 = clamp (dot (tmpvar_21, tmpvar_7), 0.0, 1.0);
  mediump float tmpvar_23;
  tmpvar_23 = clamp (dot (tmpvar_21, tmpvar_18), 0.0, 1.0);
  mediump float tmpvar_24;
  tmpvar_24 = clamp (dot (tmpvar_21, viewDir_15), 0.0, 1.0);
  mediump float tmpvar_25;
  tmpvar_25 = clamp (dot (tmpvar_7, tmpvar_18), 0.0, 1.0);
  mediump float tmpvar_26;
  tmpvar_26 = (tmpvar_17 * tmpvar_17);
  mediump float tmpvar_27;
  tmpvar_27 = (tmpvar_17 * tmpvar_17);
  mediump float tmpvar_28;
  tmpvar_28 = (tmpvar_27 * tmpvar_27);
  mediump float tmpvar_29;
  tmpvar_29 = (((tmpvar_23 * tmpvar_23) * (tmpvar_28 - 1.0)) + 1.0);
  mediump float x_30;
  x_30 = (1.0 - tmpvar_22);
  mediump float x_31;
  x_31 = (1.0 - tmpvar_24);
  mediump float tmpvar_32;
  tmpvar_32 = (0.5 + ((2.0 * tmpvar_25) * (tmpvar_25 * tmpvar_17)));
  specularTerm_16 = sqrt(max (0.0001, (
    ((2.0 / ((
      (tmpvar_22 * ((tmpvar_24 * (1.0 - tmpvar_26)) + tmpvar_26))
     + 
      (tmpvar_24 * ((tmpvar_22 * (1.0 - tmpvar_26)) + tmpvar_26))
    ) + 1e-05)) * (tmpvar_28 / ((
      (3.141593 * tmpvar_29)
     * tmpvar_29) + 1e-07)))
   * 0.7853982)));
  mediump float tmpvar_33;
  tmpvar_33 = max (0.0, (specularTerm_16 * tmpvar_22));
  specularTerm_16 = tmpvar_33;
  mediump float x_34;
  x_34 = (1.0 - tmpvar_25);
  mediump vec4 tmpvar_35;
  tmpvar_35.w = 1.0;
  tmpvar_35.xyz = ((tmpvar_4 * (tmpvar_14 * 
    (((1.0 + (
      (tmpvar_32 - 1.0)
     * 
      ((x_30 * x_30) * ((x_30 * x_30) * x_30))
    )) * (1.0 + (
      (tmpvar_32 - 1.0)
     * 
      ((x_31 * x_31) * ((x_31 * x_31) * x_31))
    ))) * tmpvar_22)
  )) + ((tmpvar_33 * tmpvar_14) * (tmpvar_5 + 
    ((1.0 - tmpvar_5) * ((x_34 * x_34) * ((x_34 * x_34) * x_34)))
  )));
  c_1.w = tmpvar_35.w;
  highp float tmpvar_36;
  tmpvar_36 = clamp (xlv_TEXCOORD7, 0.0, 1.0);
  c_1.xyz = (tmpvar_35.xyz * vec3(tmpvar_36));
  mediump vec4 xlat_varoutput_37;
  xlat_varoutput_37.xyz = c_1.xyz;
  xlat_varoutput_37.w = 1.0;
  gl_FragData[0] = xlat_varoutput_37;
}


#endif
"
}
}
Program "fp" {
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_OFF" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" }
""
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_OFF" }
""
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_OFF" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_OFF" }
""
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "SHADOWS_NONATIVE" }
""
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "SHADOWS_NATIVE" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_SCREEN" }
""
}
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_CUBE" }
""
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_CUBE" }
""
}
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_OFF" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_OFF" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_OFF" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_OFF" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_OFF" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "FOG_EXP" "SHADOWS_NONATIVE" }
""
}
SubProgram "gles " {
Keywords { "SPOT" "SHADOWS_DEPTH" "SHADOWS_NATIVE" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL" "SHADOWS_SCREEN" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "DIRECTIONAL_COOKIE" "SHADOWS_SCREEN" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "POINT" "SHADOWS_CUBE" "FOG_EXP" }
""
}
SubProgram "gles " {
Keywords { "POINT_COOKIE" "SHADOWS_CUBE" "FOG_EXP" }
""
}
}
 }
 Pass {
  Name "SHADOWCASTER"
  Tags { "LIGHTMODE"="SHADOWCASTER" "SHADOWSUPPORT"="true" "RenderType"="Opaque" "PerformanceChecks"="False" }
  GpuProgramID 488501
Program "vp" {
SubProgram "gles " {
Keywords { "SHADOWS_DEPTH" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec3 _glesNormal;
uniform highp vec4 _WorldSpaceLightPos0;
uniform highp vec4 unity_LightShadowBias;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
uniform highp mat4 _World2Object;
uniform highp mat4 unity_MatrixVP;
void main ()
{
  highp vec3 vertex_1;
  vertex_1 = _glesVertex.xyz;
  highp vec4 clipPos_2;
  if ((unity_LightShadowBias.z != 0.0)) {
    highp vec4 tmpvar_3;
    tmpvar_3.w = 1.0;
    tmpvar_3.xyz = vertex_1;
    highp vec3 tmpvar_4;
    tmpvar_4 = (_Object2World * tmpvar_3).xyz;
    highp vec4 v_5;
    v_5.x = _World2Object[0].x;
    v_5.y = _World2Object[1].x;
    v_5.z = _World2Object[2].x;
    v_5.w = _World2Object[3].x;
    highp vec4 v_6;
    v_6.x = _World2Object[0].y;
    v_6.y = _World2Object[1].y;
    v_6.z = _World2Object[2].y;
    v_6.w = _World2Object[3].y;
    highp vec4 v_7;
    v_7.x = _World2Object[0].z;
    v_7.y = _World2Object[1].z;
    v_7.z = _World2Object[2].z;
    v_7.w = _World2Object[3].z;
    highp vec3 tmpvar_8;
    tmpvar_8 = normalize(((
      (v_5.xyz * _glesNormal.x)
     + 
      (v_6.xyz * _glesNormal.y)
    ) + (v_7.xyz * _glesNormal.z)));
    highp float tmpvar_9;
    tmpvar_9 = dot (tmpvar_8, normalize((_WorldSpaceLightPos0.xyz - 
      (tmpvar_4 * _WorldSpaceLightPos0.w)
    )));
    highp vec4 tmpvar_10;
    tmpvar_10.w = 1.0;
    tmpvar_10.xyz = (tmpvar_4 - (tmpvar_8 * (unity_LightShadowBias.z * 
      sqrt((1.0 - (tmpvar_9 * tmpvar_9)))
    )));
    clipPos_2 = (unity_MatrixVP * tmpvar_10);
  } else {
    highp vec4 tmpvar_11;
    tmpvar_11.w = 1.0;
    tmpvar_11.xyz = vertex_1;
    clipPos_2 = (glstate_matrix_mvp * tmpvar_11);
  };
  highp vec4 clipPos_12;
  clipPos_12.xyw = clipPos_2.xyw;
  clipPos_12.z = (clipPos_2.z + clamp ((unity_LightShadowBias.x / clipPos_2.w), 0.0, 1.0));
  clipPos_12.z = mix (clipPos_12.z, max (clipPos_12.z, -(clipPos_2.w)), unity_LightShadowBias.y);
  gl_Position = clipPos_12;
}


#endif
#ifdef FRAGMENT
void main ()
{
  gl_FragData[0] = vec4(0.0, 0.0, 0.0, 0.0);
}


#endif
"
}
SubProgram "gles " {
Keywords { "SHADOWS_CUBE" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
uniform highp vec4 _LightPositionRange;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 _Object2World;
varying highp vec3 xlv_TEXCOORD0;
void main ()
{
  xlv_TEXCOORD0 = ((_Object2World * _glesVertex).xyz - _LightPositionRange.xyz);
  gl_Position = (glstate_matrix_mvp * _glesVertex);
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _LightPositionRange;
uniform highp vec4 unity_LightShadowBias;
varying highp vec3 xlv_TEXCOORD0;
void main ()
{
  mediump vec4 tmpvar_1;
  highp vec4 enc_2;
  highp vec4 tmpvar_3;
  tmpvar_3 = fract((vec4(1.0, 255.0, 65025.0, 1.658138e+07) * min (
    ((sqrt(dot (xlv_TEXCOORD0, xlv_TEXCOORD0)) + unity_LightShadowBias.x) * _LightPositionRange.w)
  , 0.999)));
  enc_2 = (tmpvar_3 - (tmpvar_3.yzww * 0.003921569));
  tmpvar_1 = enc_2;
  gl_FragData[0] = tmpvar_1;
}


#endif
"
}
}
Program "fp" {
SubProgram "gles " {
Keywords { "SHADOWS_DEPTH" }
""
}
SubProgram "gles " {
Keywords { "SHADOWS_CUBE" }
""
}
}
 }
}
Fallback "VertexLit"
}