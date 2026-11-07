"""Decode stripped MonoBehaviour / ScriptableObject data using class layouts from Il2CppDumper's dump.cs.

Unity 2018.3 serialization rules implemented here (Unity 5.3 differences are noted where they apply):
- serialized fields: (public or [SerializeField]) and not static/const/readonly/[NonSerialized]
- inherited fields first (base classes up to MonoBehaviour/ScriptableObject)
- primitives, string, enums (by underlying type), UnityEngine.Object refs (PPtr: i32 fileID + i64 pathID)
- arrays T[] / List<T> (i32 count + elements), custom [Serializable] non-generic classes/structs inline
- built-in structs (Vector*, Quaternion, Color, Rect, Bounds, LayerMask, AnimationCurve, Gradient, ...)
- 4-byte alignment after primitives smaller than 4 bytes, strings and arrays
- custom class nesting depth limit 7

A decode is only trusted when it consumes the object's bytes exactly (`complete` = True).
"""
import glob
import os
import re
import struct

from dotnet_meta import NOT_SERIALIZED, read_types

PRIMITIVES = {
    'bool': ('<?', 1), 'byte': ('<B', 1), 'sbyte': ('<b', 1), 'short': ('<h', 2), 'ushort': ('<H', 2),
    'char': ('<H', 2), 'int': ('<i', 4), 'uint': ('<I', 4), 'long': ('<q', 8), 'ulong': ('<Q', 8),
    'float': ('<f', 4), 'double': ('<d', 8),
}
BUILTIN_STRUCTS = {
    'Vector2': ['x', 'y'], 'Vector3': ['x', 'y', 'z'], 'Vector4': ['x', 'y', 'z', 'w'],
    'Quaternion': ['x', 'y', 'z', 'w'], 'Color': ['r', 'g', 'b', 'a'],
    'Rect': ['x', 'y', 'width', 'height'],
}
INT_STRUCTS = {
    'Vector2Int': ['x', 'y'], 'Vector3Int': ['x', 'y', 'z'], 'RectInt': ['x', 'y', 'width', 'height'],
    'RectOffset': ['left', 'right', 'top', 'bottom'],
}
UNITY_OBJECT_ROOTS = {'UnityEngine.Object'}
SCRIPT_BASES = {'UnityEngine.MonoBehaviour', 'UnityEngine.ScriptableObject', 'UnityEngine.Behaviour',
                'UnityEngine.Component', 'UnityEngine.Object'}
MAX_DEPTH = 7

TYPE_RE = re.compile(r'^(?:(?:public|internal|private|protected)\s+)*((?:(?:sealed|abstract|static|unsafe|readonly)\s+)*)'
                     r'(class|struct|enum|interface)\s+(.+?)(?:\s*:\s*(.+?))?\s*// TypeDefIndex: (\d+)\s*$')
FIELD_RE = re.compile(r'^\t((?:(?:public|private|protected|internal|static|readonly|const|volatile|new|fixed)\s+)+)'
                      r'(.+?)\s+([@\w]+)(?:\s*=.*?)?;')
ATTR_RE = re.compile(r'\[([\w.]+)')


class TypeInfo:
    def __init__(self, namespace, name, kind, modifiers, bases, index, attrs):
        self.namespace = namespace
        self.name = name
        self.kind = kind
        self.modifiers = modifiers
        self.bases = bases
        self.index = index
        self.attrs = attrs
        self.fields = []
        self.underlying = None

    @property
    def full(self):
        return f'{self.namespace}.{self.name}' if self.namespace else self.name


def parse_dump(path):
    types = []
    namespace = ''
    pending_attrs = []
    current = None
    field_attrs = []
    with open(path, encoding='utf-8', errors='replace') as f:
        for line in f:
            line = line.rstrip('\n')
            if line.startswith('// Namespace:'):
                namespace = line[len('// Namespace:'):].strip()
                pending_attrs = []
                continue
            if line.startswith('['):
                pending_attrs += ATTR_RE.findall(line)
                continue
            m = TYPE_RE.match(line)
            if m:
                modifiers, kind, name, bases, index = m.groups()
                current = TypeInfo(namespace, name.strip(), kind, modifiers.split(),
                                   [b.strip() for b in bases.split(',')] if bases else [], int(index),
                                   set(pending_attrs))
                types.append(current)
                pending_attrs = []
                field_attrs = []
                continue
            if current is None:
                continue
            if line.startswith('\t['):
                field_attrs += ATTR_RE.findall(line)
                continue
            if line.startswith('\t// Properties') or line.startswith('\t// Methods'):
                current_fields_done = True
                field_attrs = []
                continue
            fm = FIELD_RE.match(line)
            if fm:
                mods, ftype, fname = fm.groups()
                mods = mods.split()
                if fname == 'value__' and current.kind == 'enum':
                    current.underlying = ftype
                else:
                    current.fields.append({'name': fname, 'type': ftype.strip(), 'mods': mods,
                                           'attrs': set(field_attrs)})
                field_attrs = []
                continue
            if line.startswith('}'):
                current = None
            elif not line.startswith('\t//') and line.strip():
                field_attrs = []
    return types


def load_not_serialized(dummy_dll_dir):
    """(simple type name, field-name tuple) -> set of NotSerialized field names, from DummyDll flags."""
    table = {}
    for dll in sorted(glob.glob(os.path.join(dummy_dll_dir, '*.dll'))):
        for t in read_types(dll):
            names = tuple(n for n, _ in t['fields'] if n != 'value__')
            flagged = {n for n, f in t['fields'] if f & NOT_SERIALIZED}
            table.setdefault((t['name'], names), set()).update(flagged)
    return table


class Decoder:
    def __init__(self, dump_path, dummy_dll_dir, unity_version=(2018, 3)):
        # Keyframe weights (weightedMode, inWeight, outWeight) were added in Unity 2018.1.
        self.weighted_keyframes = unity_version >= (2018, 1)
        self.types = parse_dump(dump_path)
        not_serialized = load_not_serialized(dummy_dll_dir)
        for t in self.types:
            simple = t.name.split('.')[-1].split('<')[0]
            key = (simple, tuple(fd['name'] for fd in t.fields))
            for fd in t.fields:
                if fd['name'] in not_serialized.get(key, ()):
                    fd['attrs'].add('NonSerialized')
        self.by_full = {}
        self.by_short = {}
        for t in self.types:
            self.by_full.setdefault(t.full, t)
            self.by_short.setdefault(t.name, []).append(t)
        self.layout_cache = {}
        self.by_short_generic = {}
        for t in self.types:
            if '<' in t.name:
                self.by_short_generic.setdefault(t.name.split('<', 1)[0], []).append(t)

    # --- type resolution ------------------------------------------------------------------------
    def lookup(self, name, context=None):
        name = name.strip()
        if '.' in name and name in self.by_full:
            return self.by_full[name]
        candidates = list(self.by_short.get(name, []))
        if context is not None:
            # nested type referenced from inside its outer class: "Inner" -> "Outer.Inner"
            outer = context.name
            while True:
                nested = self.by_short.get(f'{outer}.{name}')
                if nested:
                    candidates = nested + candidates
                    break
                if '.' not in outer:
                    break
                outer = outer.rsplit('.', 1)[0]
        if not candidates and '.' in name:
            ns, short = name.rsplit('.', 1)
            candidates = [t for t in self.by_short.get(short, []) if t.namespace == ns]
        if not candidates:
            return None
        if context is not None:
            same_ns = [t for t in candidates if t.namespace == context.namespace]
            if same_ns:
                return same_ns[0]
        candidates.sort(key=lambda t: (0 if t.namespace.startswith('UnityEngine') else 1, -t.index))
        return candidates[0]

    def lookup_generic(self, name, context):
        base_name = name.split('<', 1)[0].strip()
        arity = name.count(',') + 1
        for t in self.by_short_generic.get(base_name, []):
            if t.name.count(',') + 1 == arity:
                return t
        return None

    def base_of(self, t):
        for b in t.bases:
            bt = self.lookup_generic(b, t) if '<' in b else self.lookup(b, t)
            if bt is not None and bt.kind == 'class':
                return bt
        return None

    def is_unity_object(self, t):
        seen = 0
        while t is not None and seen < 64:
            if t.full in UNITY_OBJECT_ROOTS:
                return True
            t = self.base_of(t)
            seen += 1
        return False

    # --- layout ---------------------------------------------------------------------------------
    def serialized_fields(self, t):
        if t.full in self.layout_cache:
            return self.layout_cache[t.full]
        chain = []
        cur = t
        while cur is not None and cur.full not in SCRIPT_BASES:
            chain.append(cur)
            cur = self.base_of(cur)
        fields = []
        for cls in reversed(chain):
            for fd in cls.fields:
                mods = fd['mods']
                if 'static' in mods or 'const' in mods or 'readonly' in mods:
                    continue
                if 'NonSerialized' in fd['attrs']:
                    continue
                if 'public' not in mods and 'SerializeField' not in fd['attrs']:
                    continue
                fields.append((fd['name'], fd['type'], cls))
        self.layout_cache[t.full] = fields
        return fields

    # --- reading --------------------------------------------------------------------------------
    def decode(self, script_full_name, raw, offset, resolve_ref):
        t = self.by_full.get(script_full_name) or self.lookup(script_full_name)
        if t is None:
            return {'complete': False, 'error': f'type {script_full_name} not found in dump'}
        self.raw = raw
        self.pos = offset
        self.resolve_ref = resolve_ref
        try:
            fields = self.read_class(t, 0)
        except Exception as e:
            return {'complete': False, 'error': f'{type(e).__name__}: {e}', 'consumed': self.pos,
                    'size': len(raw)}
        complete = self.pos == len(raw)
        result = {'complete': complete, 'fields': fields}
        if not complete:
            result.update(consumed=self.pos, size=len(raw))
        return result

    def align(self):
        self.pos = (self.pos + 3) & ~3

    def unpack(self, fmt, size):
        if self.pos + size > len(self.raw):
            raise ValueError(f'read past end at {self.pos}')
        value = struct.unpack_from(fmt, self.raw, self.pos)[0]
        self.pos += size
        return value

    def read_string(self):
        n = self.unpack('<i', 4)
        if n < 0 or self.pos + n > len(self.raw):
            raise ValueError(f'bad string length {n}')
        value = self.raw[self.pos:self.pos + n].decode('utf-8', 'replace')
        self.pos += n
        self.align()
        return value

    def read_class(self, t, depth):
        out = {}
        for name, ftype, owner in self.serialized_fields(t):
            value = self.read_value(ftype, owner, depth)
            if value is not SKIP:
                out[name] = value
        return out

    def read_value(self, ftype, context, depth, in_array=False):
        ftype = ftype.strip()
        if ftype.endswith('[]'):
            if in_array:
                return SKIP  # nested arrays are not serialized
            return self.read_array(ftype[:-2], context, depth)
        m = re.fullmatch(r'List<(.+)>', ftype)
        if m:
            if in_array:
                return SKIP
            return self.read_array(m.group(1), context, depth)
        if '<' in ftype:
            return SKIP  # other generics are not serialized in 2018.3
        if ftype in PRIMITIVES:
            fmt, size = PRIMITIVES[ftype]
            value = self.unpack(fmt, size)
            if size < 4 and not in_array:
                self.align()
            return value
        if ftype == 'string':
            return self.read_string()
        short = ftype.split('.')[-1] if ftype.startswith('UnityEngine.') else ftype
        if short in BUILTIN_STRUCTS:
            return {k: self.unpack('<f', 4) for k in BUILTIN_STRUCTS[short]}
        if short in INT_STRUCTS:
            return {k: self.unpack('<i', 4) for k in INT_STRUCTS[short]}
        if short == 'Color32':
            return {'rgba': self.unpack('<I', 4)}
        if short == 'LayerMask':
            return {'m_Bits': self.unpack('<I', 4)}
        if short == 'Bounds':
            return {'center': {k: self.unpack('<f', 4) for k in 'xyz'},
                    'extents': {k: self.unpack('<f', 4) for k in 'xyz'}}
        if short == 'Matrix4x4':
            return [self.unpack('<f', 4) for _ in range(16)]
        if short == 'AnimationCurve':
            return self.read_animation_curve()
        if short == 'Gradient':
            return self.read_gradient()
        if short == 'GUIStyle':
            return self.read_gui_style()
        t = self.lookup(ftype, context)
        if t is None:
            return SKIP
        if t.kind == 'enum':
            underlying = t.underlying or 'int'
            fmt, size = PRIMITIVES.get(underlying, ('<i', 4))
            value = self.unpack(fmt, size)
            if size < 4 and not in_array:
                self.align()
            return value
        if t.kind == 'interface':
            return SKIP
        if self.is_unity_object(t):
            file_id = self.unpack('<i', 4)
            path_id = self.unpack('<q', 8)
            return self.resolve_ref(file_id, path_id)
        if 'Serializable' in t.attrs and t.kind in ('class', 'struct') and 'abstract' not in t.modifiers:
            if depth >= MAX_DEPTH and t.kind == 'class':
                return SKIP
            return self.read_class(t, depth + 1)
        return SKIP

    def is_serializable(self, ftype, context, depth):
        ftype = ftype.strip()
        if ftype.endswith('[]') or ftype.startswith('List<'):
            return False  # nested arrays are not serialized
        if '<' in ftype:
            return False
        if ftype in PRIMITIVES or ftype == 'string':
            return True
        short = ftype.split('.')[-1] if ftype.startswith('UnityEngine.') else ftype
        if short in BUILTIN_STRUCTS or short in INT_STRUCTS or short in (
                'Color32', 'LayerMask', 'Bounds', 'Matrix4x4', 'AnimationCurve', 'Gradient', 'GUIStyle'):
            return True
        t = self.lookup(ftype, context)
        if t is None or t.kind == 'interface':
            return False
        if t.kind == 'enum' or self.is_unity_object(t):
            return True
        if 'Serializable' in t.attrs and t.kind in ('class', 'struct') and 'abstract' not in t.modifiers:
            return not (depth >= MAX_DEPTH and t.kind == 'class')
        return False

    def read_array(self, elem, context, depth):
        if not self.is_serializable(elem, context, depth):
            return SKIP
        n = self.unpack('<i', 4)
        if n < 0 or n > 10_000_000:
            raise ValueError(f'bad array length {n}')
        items = []
        for _ in range(n):
            value = self.read_value(elem, context, depth, in_array=True)
            if value is SKIP:
                raise ValueError(f'array element type {elem} not serializable but count {n}')
            items.append(value)
        self.align()
        return items

    def read_animation_curve(self):
        n = self.unpack('<i', 4)
        keys = []
        for _ in range(n):
            key = {
                'time': self.unpack('<f', 4), 'value': self.unpack('<f', 4),
                'inSlope': self.unpack('<f', 4), 'outSlope': self.unpack('<f', 4),
            }
            if self.weighted_keyframes:
                key.update(weightedMode=self.unpack('<i', 4), inWeight=self.unpack('<f', 4),
                           outWeight=self.unpack('<f', 4))
            keys.append(key)
        return {'keys': keys, 'preInfinity': self.unpack('<i', 4), 'postInfinity': self.unpack('<i', 4),
                'rotationOrder': self.unpack('<i', 4)}

    def read_gradient(self):
        colors = [{k: self.unpack('<f', 4) for k in 'rgba'} for _ in range(8)]
        ctimes = [self.unpack('<H', 2) for _ in range(8)]
        atimes = [self.unpack('<H', 2) for _ in range(8)]
        mode = self.unpack('<i', 4)
        num_color = self.unpack('<B', 1)
        num_alpha = self.unpack('<B', 1)
        self.align()
        return {'colorKeys': colors, 'colorTimes': ctimes, 'alphaTimes': atimes, 'mode': mode,
                'numColorKeys': num_color, 'numAlphaKeys': num_alpha}

    def read_gui_style_state(self):
        # 2018.3: no m_ScaledBackgrounds
        bg = self.resolve_ref(self.unpack('<i', 4), self.unpack('<q', 8))
        return {'background': bg, 'textColor': {k: self.unpack('<f', 4) for k in 'rgba'}}

    def read_gui_style(self):
        style = {'name': self.read_string()}
        for state in ('normal', 'hover', 'active', 'focused', 'onNormal', 'onHover', 'onActive', 'onFocused'):
            style[state] = self.read_gui_style_state()
        for rect in ('border', 'margin', 'padding', 'overflow'):
            style[rect] = {k: self.unpack('<i', 4) for k in ('left', 'right', 'top', 'bottom')}
        style['font'] = self.resolve_ref(self.unpack('<i', 4), self.unpack('<q', 8))
        style['fontSize'] = self.unpack('<i', 4)
        style['fontStyle'] = self.unpack('<i', 4)
        style['alignment'] = self.unpack('<i', 4)
        # consecutive native bools share one alignment
        style['wordWrap'] = self.unpack('<?', 1)
        style['richText'] = self.unpack('<?', 1)
        self.align()
        style['textClipping'] = self.unpack('<i', 4)
        style['imagePosition'] = self.unpack('<i', 4)
        style['contentOffset'] = {k: self.unpack('<f', 4) for k in 'xy'}
        style['fixedWidth'] = self.unpack('<f', 4)
        style['fixedHeight'] = self.unpack('<f', 4)
        style['stretchWidth'] = self.unpack('<?', 1)
        style['stretchHeight'] = self.unpack('<?', 1)
        self.align()
        return style


class _Skip:
    def __repr__(self):
        return 'SKIP'


SKIP = _Skip()
