//////////////////////////////////////////
//
// NOTE: This is *not* a valid shader file
//
///////////////////////////////////////////
Shader "Particles/Additive (Soft)" {
Properties {
 _MainTex ("Particle Texture", 2D) = "white" { }
 _InvFade ("Soft Particles Factor", Range(0.01,3)) = 1
}
SubShader { 
 Tags { "QUEUE"="Transparent" "IGNOREPROJECTOR"="true" "RenderType"="Transparent" }
 Pass {
  Tags { "QUEUE"="Transparent" "IGNOREPROJECTOR"="true" "RenderType"="Transparent" }
  ZWrite Off
  Cull Off
  Blend One OneMinusSrcColor
  ColorMask RGB
  GpuProgramID 4861
Program "vp" {
SubProgram "gles " {
Keywords { "SOFTPARTICLES_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesColor;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 _MainTex_ST;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  gl_Position = (glstate_matrix_mvp * _glesVertex);
  xlv_COLOR = _glesColor;
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
}


#endif
#ifdef FRAGMENT
uniform sampler2D _MainTex;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
void main ()
{
  lowp vec4 tmpvar_1;
  mediump vec4 col_2;
  lowp vec4 tmpvar_3;
  tmpvar_3 = (xlv_COLOR * texture2D (_MainTex, xlv_TEXCOORD0));
  col_2 = tmpvar_3;
  col_2.xyz = (col_2.xyz * col_2.w);
  tmpvar_1 = col_2;
  gl_FragData[0] = tmpvar_1;
}


#endif
"
}
SubProgram "gles " {
Keywords { "SOFTPARTICLES_ON" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesColor;
attribute vec4 _glesMultiTexCoord0;
uniform highp vec4 _ProjectionParams;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 glstate_matrix_modelview0;
uniform highp vec4 _MainTex_ST;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
varying highp vec4 xlv_TEXCOORD2;
void main ()
{
  highp vec4 tmpvar_1;
  highp vec4 tmpvar_2;
  tmpvar_1 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 o_3;
  highp vec4 tmpvar_4;
  tmpvar_4 = (tmpvar_1 * 0.5);
  highp vec2 tmpvar_5;
  tmpvar_5.x = tmpvar_4.x;
  tmpvar_5.y = (tmpvar_4.y * _ProjectionParams.x);
  o_3.xy = (tmpvar_5 + tmpvar_4.w);
  o_3.zw = tmpvar_1.zw;
  tmpvar_2.xyw = o_3.xyw;
  tmpvar_2.z = -((glstate_matrix_modelview0 * _glesVertex).z);
  gl_Position = tmpvar_1;
  xlv_COLOR = _glesColor;
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  xlv_TEXCOORD2 = tmpvar_2;
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _ZBufferParams;
uniform sampler2D _MainTex;
uniform highp sampler2D _CameraDepthTexture;
uniform highp float _InvFade;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
varying highp vec4 xlv_TEXCOORD2;
void main ()
{
  lowp vec4 tmpvar_1;
  lowp vec4 tmpvar_2;
  tmpvar_2.xyz = xlv_COLOR.xyz;
  mediump vec4 col_3;
  highp float tmpvar_4;
  tmpvar_4 = clamp ((_InvFade * (
    (1.0/(((_ZBufferParams.z * texture2DProj (_CameraDepthTexture, xlv_TEXCOORD2).x) + _ZBufferParams.w)))
   - xlv_TEXCOORD2.z)), 0.0, 1.0);
  tmpvar_2.w = (xlv_COLOR.w * tmpvar_4);
  lowp vec4 tmpvar_5;
  tmpvar_5 = (tmpvar_2 * texture2D (_MainTex, xlv_TEXCOORD0));
  col_3 = tmpvar_5;
  col_3.xyz = (col_3.xyz * col_3.w);
  tmpvar_1 = col_3;
  gl_FragData[0] = tmpvar_1;
}


#endif
"
}
SubProgram "gles " {
Keywords { "FOG_EXP" "SOFTPARTICLES_OFF" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesColor;
attribute vec4 _glesMultiTexCoord0;
uniform highp mat4 glstate_matrix_mvp;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
varying highp float xlv_TEXCOORD1;
void main ()
{
  highp vec4 tmpvar_1;
  tmpvar_1 = (glstate_matrix_mvp * _glesVertex);
  gl_Position = tmpvar_1;
  xlv_COLOR = _glesColor;
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  xlv_TEXCOORD1 = exp2(-((unity_FogParams.y * tmpvar_1.z)));
}


#endif
#ifdef FRAGMENT
uniform sampler2D _MainTex;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
varying highp float xlv_TEXCOORD1;
void main ()
{
  lowp vec4 tmpvar_1;
  mediump vec4 col_2;
  lowp vec4 tmpvar_3;
  tmpvar_3 = (xlv_COLOR * texture2D (_MainTex, xlv_TEXCOORD0));
  col_2 = tmpvar_3;
  col_2.xyz = (col_2.xyz * col_2.w);
  highp float tmpvar_4;
  tmpvar_4 = clamp (xlv_TEXCOORD1, 0.0, 1.0);
  col_2.xyz = (col_2.xyz * vec3(tmpvar_4));
  tmpvar_1 = col_2;
  gl_FragData[0] = tmpvar_1;
}


#endif
"
}
SubProgram "gles " {
Keywords { "FOG_EXP" "SOFTPARTICLES_ON" }
"#version 100

#ifdef VERTEX
attribute vec4 _glesVertex;
attribute vec4 _glesColor;
attribute vec4 _glesMultiTexCoord0;
uniform highp vec4 _ProjectionParams;
uniform highp mat4 glstate_matrix_mvp;
uniform highp mat4 glstate_matrix_modelview0;
uniform highp vec4 unity_FogParams;
uniform highp vec4 _MainTex_ST;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
varying highp float xlv_TEXCOORD1;
varying highp vec4 xlv_TEXCOORD2;
void main ()
{
  highp vec4 tmpvar_1;
  highp vec4 tmpvar_2;
  tmpvar_1 = (glstate_matrix_mvp * _glesVertex);
  highp vec4 o_3;
  highp vec4 tmpvar_4;
  tmpvar_4 = (tmpvar_1 * 0.5);
  highp vec2 tmpvar_5;
  tmpvar_5.x = tmpvar_4.x;
  tmpvar_5.y = (tmpvar_4.y * _ProjectionParams.x);
  o_3.xy = (tmpvar_5 + tmpvar_4.w);
  o_3.zw = tmpvar_1.zw;
  tmpvar_2.xyw = o_3.xyw;
  tmpvar_2.z = -((glstate_matrix_modelview0 * _glesVertex).z);
  gl_Position = tmpvar_1;
  xlv_COLOR = _glesColor;
  xlv_TEXCOORD0 = ((_glesMultiTexCoord0.xy * _MainTex_ST.xy) + _MainTex_ST.zw);
  xlv_TEXCOORD1 = exp2(-((unity_FogParams.y * tmpvar_1.z)));
  xlv_TEXCOORD2 = tmpvar_2;
}


#endif
#ifdef FRAGMENT
uniform highp vec4 _ZBufferParams;
uniform sampler2D _MainTex;
uniform highp sampler2D _CameraDepthTexture;
uniform highp float _InvFade;
varying lowp vec4 xlv_COLOR;
varying highp vec2 xlv_TEXCOORD0;
varying highp float xlv_TEXCOORD1;
varying highp vec4 xlv_TEXCOORD2;
void main ()
{
  lowp vec4 tmpvar_1;
  lowp vec4 tmpvar_2;
  tmpvar_2.xyz = xlv_COLOR.xyz;
  mediump vec4 col_3;
  highp float tmpvar_4;
  tmpvar_4 = clamp ((_InvFade * (
    (1.0/(((_ZBufferParams.z * texture2DProj (_CameraDepthTexture, xlv_TEXCOORD2).x) + _ZBufferParams.w)))
   - xlv_TEXCOORD2.z)), 0.0, 1.0);
  tmpvar_2.w = (xlv_COLOR.w * tmpvar_4);
  lowp vec4 tmpvar_5;
  tmpvar_5 = (tmpvar_2 * texture2D (_MainTex, xlv_TEXCOORD0));
  col_3 = tmpvar_5;
  col_3.xyz = (col_3.xyz * col_3.w);
  highp float tmpvar_6;
  tmpvar_6 = clamp (xlv_TEXCOORD1, 0.0, 1.0);
  col_3.xyz = (col_3.xyz * vec3(tmpvar_6));
  tmpvar_1 = col_3;
  gl_FragData[0] = tmpvar_1;
}


#endif
"
}
}
Program "fp" {
SubProgram "gles " {
Keywords { "SOFTPARTICLES_OFF" }
""
}
SubProgram "gles " {
Keywords { "SOFTPARTICLES_ON" }
""
}
SubProgram "gles " {
Keywords { "FOG_EXP" "SOFTPARTICLES_OFF" }
""
}
SubProgram "gles " {
Keywords { "FOG_EXP" "SOFTPARTICLES_ON" }
""
}
}
 }
}
}