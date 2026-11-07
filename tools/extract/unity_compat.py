"""Differences between the Unity versions of the two builds (2018.3 Build-7, 5.3 CrazyGames)."""
import re

SHADER_NAME_RE = re.compile(r'^\s*Shader\s+"([^"]*)"')


def shader_name(shader):
    """ShaderLab name of a UnityPy Shader. 5.5+ keeps it in m_ParsedForm; 5.3/5.4 only in the
    first line of the m_Script source (`Shader "Particles/Additive" {`)."""
    parsed = getattr(shader, 'm_ParsedForm', None)
    if parsed is not None:
        return parsed.m_Name
    script = shader.m_Script
    if not isinstance(script, str):
        script = bytes(script).decode('utf-8', 'replace')
    match = SHADER_NAME_RE.match(script)
    if match is None:
        raise ValueError(f'no Shader "name" line in {shader.m_Name!r}')
    return match.group(1)


def unity_version(env):
    """(major, minor) of the build's serialized files, e.g. (5, 3) for "5.3.5f1". The files of a
    build share it (the built-in resources may differ in the patch: "5.3.4p1")."""
    versions = {tuple(int(part) for part in f.unity_version.split('.')[:2])
                for f in env.files.values() if hasattr(f, 'unity_version')}
    if len(versions) != 1:
        raise ValueError(f'expected one Unity version, found {sorted(versions)}')
    return versions.pop()


def export_shader(shader):
    """ShaderLab text with the compiled programs, as UnityPy's Shader.export(). For 5.3/5.4 shaders
    (m_SubProgramBlob) UnityPy 1.25.4 reads the blob big-endian; it is little-endian (sub-program
    version 201509030 = 0x0C02C8A6 at each entry), so it is parsed here with the right byte order."""
    if not getattr(shader, 'm_SubProgramBlob', None):
        return shader.export()
    from UnityPy.export.ShaderConverter import HEADER, ShaderProgram
    from UnityPy.helpers import CompressionHelper
    from UnityPy.streams import EndianBinaryReader

    blob = CompressionHelper.decompress_lz4(bytes(shader.m_SubProgramBlob), shader.decompressedSize)
    program = ShaderProgram(EndianBinaryReader(blob, endian='<'), shader.object_reader.version)
    script = shader.m_Script
    if not isinstance(script, str):
        script = bytes(script).decode('utf-8')
    return HEADER + program.Export(script)
