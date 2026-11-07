//////////////////////////////////////////
//
// NOTE: This is *not* a valid shader file
//
///////////////////////////////////////////
Shader "Mobile/Skybox" {
Properties {
 _FrontTex ("Front (+Z)", 2D) = "white" { }
 _BackTex ("Back (-Z)", 2D) = "white" { }
 _LeftTex ("Left (+X)", 2D) = "white" { }
 _RightTex ("Right (-X)", 2D) = "white" { }
 _UpTex ("Up (+Y)", 2D) = "white" { }
 _DownTex ("Down (-Y)", 2D) = "white" { }
}
SubShader { 
 Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
 Pass {
  Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
  ZWrite Off
  Cull Off
  GpuProgramID 20721
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _FrontTex_ST;
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
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _FrontTex_ST.xy) + _FrontTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _FrontTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_FrontTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
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
 Pass {
  Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
  ZWrite Off
  Cull Off
  GpuProgramID 107871
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _BackTex_ST;
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
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _BackTex_ST.xy) + _BackTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _BackTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_BackTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
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
 Pass {
  Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
  ZWrite Off
  Cull Off
  GpuProgramID 190997
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _LeftTex_ST;
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
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _LeftTex_ST.xy) + _LeftTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _LeftTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_LeftTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
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
 Pass {
  Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
  ZWrite Off
  Cull Off
  GpuProgramID 229975
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _RightTex_ST;
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
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _RightTex_ST.xy) + _RightTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _RightTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_RightTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
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
 Pass {
  Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
  ZWrite Off
  Cull Off
  GpuProgramID 295102
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _UpTex_ST;
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
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _UpTex_ST.xy) + _UpTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _UpTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_UpTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
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
 Pass {
  Tags { "QUEUE"="Background" "RenderType"="Background" "PreviewType"="Skybox" }
  ZWrite Off
  Cull Off
  GpuProgramID 386277
Program "vp" {
SubProgram "gles " {
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _DownTex_ST;
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
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _DownTex_ST.xy) + _DownTex_ST.zw);
  gl_Position = (glstate_matrix_mvp * tmpvar_3);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _DownTex;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  tmpvar_1 = texture2D (_DownTex, xlv_TEXCOORD0);
  gl_FragData[0] = tmpvar_1;
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
}