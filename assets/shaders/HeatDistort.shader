//////////////////////////////////////////
//
// NOTE: This is *not* a valid shader file
//
///////////////////////////////////////////
Shader "HeatDistort" {
Properties {
 _BumpAmt ("Distortion", Range(0,128)) = 10
 _MainTex ("Tint Color (RGB)", 2D) = "white" { }
 _BumpMap ("Normalmap", 2D) = "bump" { }
}
SubShader { 
 Tags { "QUEUE"="Transparent+100" "RenderType"="Opaque" }
 GrabPass {
  Name "BASE"
  Tags { "LIGHTMODE"="Always" }
 }
 Pass {
  Name "BASE"
  Tags { "LIGHTMODE"="Always" "QUEUE"="Transparent+100" "RenderType"="Opaque" }
  GpuProgramID 29515
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
varying highp vec4 xlv_TEXCOORD0;
varying highp vec2 xlv_TEXCOORD1;
varying highp vec2 xlv_TEXCOORD2;
void main ()
{
  highp vec2 tmpvar_1;
  tmpvar_1 = _glesMultiTexCoord0.xy;
  highp vec4 tmpvar_2;
  highp vec4 tmpvar_3;
  tmpvar_2 = (glstate_matrix_mvp * _glesVertex);
  tmpvar_3.xy = ((tmpvar_2.xy + tmpvar_2.w) * 0.5);
  tmpvar_3.zw = tmpvar_2.zw;
  highp vec4 tmpvar_4;
  tmpvar_4.zw = vec2(0.0, 0.0);
  tmpvar_4.xy = tmpvar_1;
  highp vec4 tmpvar_5;
  tmpvar_5.zw = vec2(0.0, 0.0);
  tmpvar_5.xy = tmpvar_1;
  gl_Position = tmpvar_2;
  xlv_TEXCOORD0 = tmpvar_3;
  xlv_TEXCOORD1 = (mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0) * tmpvar_4).xy;
  xlv_TEXCOORD2 = (mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0) * tmpvar_5).xy;
}


#endif
#ifdef FRAGMENT
uniform sampler2D _GrabTexture;
uniform highp vec4 _GrabTexture_TexelSize;
uniform sampler2D _BumpMap;
uniform sampler2D _MainTex;
uniform highp float _BumpAmt;
varying highp vec4 xlv_TEXCOORD0;
varying highp vec2 xlv_TEXCOORD1;
varying highp vec2 xlv_TEXCOORD2;
void main ()
{
  highp vec4 tmpvar_1;
  tmpvar_1.zw = xlv_TEXCOORD0.zw;
  mediump vec4 tint_2;
  mediump vec4 col_3;
  mediump vec2 bump_4;
  lowp vec3 normal_5;
  normal_5.xy = ((texture2D (_BumpMap, xlv_TEXCOORD1).wy * 2.0) - 1.0);
  normal_5.z = sqrt((1.0 - clamp (
    dot (normal_5.xy, normal_5.xy)
  , 0.0, 1.0)));
  lowp vec2 tmpvar_6;
  tmpvar_6 = normal_5.xy;
  bump_4 = tmpvar_6;
  tmpvar_1.xy = (((bump_4 * _BumpAmt) * (_GrabTexture_TexelSize.xy * xlv_TEXCOORD0.z)) + xlv_TEXCOORD0.xy);
  highp vec4 tmpvar_7;
  tmpvar_7.w = 0.0;
  tmpvar_7.xyz = tmpvar_1.xyw;
  lowp vec4 tmpvar_8;
  tmpvar_8 = texture2DProj (_GrabTexture, tmpvar_7);
  col_3 = tmpvar_8;
  lowp vec4 tmpvar_9;
  tmpvar_9 = texture2D (_MainTex, xlv_TEXCOORD2);
  tint_2 = tmpvar_9;
  gl_FragData[0] = (col_3 * tint_2);
}


#endif
"
}
}
Program "fp" {
SubProgram "gles " {
""
}
}
 }
}
SubShader { 
 Tags { "QUEUE"="Transparent+100" "RenderType"="Opaque" }
 Pass {
  Name "BASE"
  Tags { "QUEUE"="Transparent+100" "RenderType"="Opaque" }
  Blend DstColor Zero
  GpuProgramID 80141
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _MainTex_ST;
varying lowp vec4 xlv_COLOR0;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  mediump vec4 tmpvar_2;
  tmpvar_2 = clamp (vec4(0.0, 0.0, 0.0, 1.1), 0.0, 1.0);
  tmpvar_1 = tmpvar_2;
  highp vec4 tmpvar_3;
  tmpvar_3.w = 1.0;
  tmpvar_3.xyz = _glesVertex.xyz;
  xlv_COLOR0 = tmpvar_1;
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _MainTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_MainTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
}


#endif
"
}
SubProgram "gles " {
Keywords { "FOG_EXP" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 glstate_matrix_modelview0;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
varying lowp vec4 xlv_COLOR0;
varying highp vec2 xlv_TEXCOORD0;
varying lowp float xlv_TEXCOORD1;
void main ()
{
  highp vec3 tmpvar_1;
  tmpvar_1 = _glesVertex.xyz;
  highp vec4 tmpvar_2;
  tmpvar_2.w = 1.0;
  tmpvar_2.xyz = tmpvar_1;
  highp vec3 tmpvar_3;
  tmpvar_3 = (glstate_matrix_modelview0 * tmpvar_2).xyz;
  lowp vec4 tmpvar_4;
  mediump vec4 tmpvar_5;
  tmpvar_5 = clamp (vec4(0.0, 0.0, 0.0, 1.1), 0.0, 1.0);
  tmpvar_4 = tmpvar_5;
  lowp float tmpvar_6;
  highp float tmpvar_7;
  tmpvar_7 = clamp (exp2(-(
    (unity_FogParams.y * sqrt(dot (tmpvar_3, tmpvar_3)))
  )), 0.0, 1.0);
  tmpvar_6 = tmpvar_7;
  highp vec4 tmpvar_8;
  tmpvar_8.w = 1.0;
  tmpvar_8.xyz = tmpvar_1;
  xlv_COLOR0 = tmpvar_4;
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  xlv_TEXCOORD1 = tmpvar_6;
  gl_Position = (glstate_matrix_mvp * tmpvar_8);
}


#endif
#ifdef FRAGMENT
uniform lowp vec4 unity_FogColor;
uniform sampler2D _MainTex;
varying highp vec2 xlv_TEXCOORD0;
varying lowp float xlv_TEXCOORD1;
void main ()
{
  lowp vec4 col_1;
  lowp vec4 tmpvar_2;
  tmpvar_2 = texture2D (_MainTex, xlv_TEXCOORD0);
  col_1.w = tmpvar_2.w;
  col_1.xyz = mix (unity_FogColor.xyz, tmpvar_2.xyz, vec3(xlv_TEXCOORD1));
  gl_FragData[0] = col_1;
}


#endif
"
}
}
Program "fp" {
SubProgram "gles " {
""
}
SubProgram "gles " {
Keywords { "FOG_EXP" }
""
}
}
 }
}
}