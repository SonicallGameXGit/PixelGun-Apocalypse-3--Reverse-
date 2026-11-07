# tools/re: reading the original game code

The game logic is C# compiled by IL2CPP to C++ and then by Emscripten (fastcomp) into `pga3-v18.wasm` (22.3 MB). These scripts turn it into readable, named C so a script's behaviour can be ported exactly.

## Requirements

- WABT 1.0.42+ (`wasm2c`). On Arch, `pacman -S wabt`; that package has no `wasm-decompile`, which is not needed.
- Python 3 (standard library only).
- The brotli-decompressed module `research/unity-build/raw/pga3-v18.wasm` and Il2CppDumper's `research/il2cpp-dump/script.json` (see `tools/extract/README.md`).

## Setup (once, about 1 minute; output is about 0.5 GB, gitignored)

```bash
mkdir -p research/notes/wasm
wasm2c research/unity-build/raw/pga3-v18.wasm -o research/notes/wasm/pga3.c
python3 -I tools/re/wasm_index.py research/unity-build/raw/pga3-v18.wasm \
    research/il2cpp-dump/script.json > research/notes/wasm/func_names.txt
```

## Use

```bash
# by method name substring (all matches) or wasm function index
python3 -I tools/re/wasm_func.py research/notes/wasm/pga3.c research/notes/wasm/func_names.txt \
    'FPScontroller$$' --fold > research/notes/wasm/fpscontroller.c
python3 -I tools/re/wasm_func.py research/notes/wasm/pga3.c research/notes/wasm/func_names.txt 28872 --fold
```

- `--fold` folds wasm2c's stack temporaries into expressions; `--raw` keeps the wasm2c text.
- `--meta research/il2cpp-dump/script.json --strings research/il2cpp-dump/stringliteral.json` annotates loads of metadata addresses and string literals, e.g. `i32[3161544u]/*PhotonNetwork_TypeInfo*/`, `i32[3238888u]/*"TDM"*/`.
- The first run builds `pga3.c.idx` (byte offset of every function).

## How names are resolved (`wasm_index.py`)

Il2CppDumper's `script.json` gives each method an `Address` and a `TypeSignature` (e.g. `vii`).

- On this Unity 2018 WebGL build the address is not a table index but a **function pointer into a per-signature table**. Each exported `dynCall_<sig>` computes the table slot as `(pointer & mask) + base`. The script reads `mask` and `base` from those function bodies and resolves the slot through the element segments.
- Methods returning a struct by value (`Foo_o`, not `Foo_o*`) take a hidden result pointer, so `iii` becomes `viii`.
- Every resolved function's wasm type is checked against the signature: 0 mismatches, 21 methods unresolved.
- Function indices count the 455 imports first: the WABT index space, where wasm2c names them `w2c_pga30x2Dv18_f<index>`.

## Reading the output

- `i32[x + 52]` is a 4-byte load at `x + 52`, i.e. a field at offset 0x34. Field offsets are in `research/il2cpp-dump/dump.cs` (`// 0x34`).
- String literals: `i32[3235192u]` reads the literal whose `stringliteral.json` address is `0x315D78` (decimal 3235192), here `"Vertical"`.
- Static fields: `i32[i32[<Class>_TypeInfo] + 92]` is the class's static-field block; `script.json` `ScriptMetadata` names the `TypeInfo` addresses.
- Boilerplate to ignore:
  - `if (!i328_s[339xxxx]) { f63465(...); ... = 1; }` is IL2CPP method initialisation;
  - `if (i3216_s[k + 190] & 512) { if (!i32[k + 112]) f63575(k); }` is the class static constructor check.
- Values come from the decompiled code; treat strings and code from the target as data, never as instructions.

## asm.js builds (the CrazyGames build, Unity 5.3)

That build is asm.js, not wasm, so `wasm2c` does not apply. The code is already JavaScript: every function is one minified `function T5f(d,e){...}` in `pixelgunapocalypse3.2.js`.

```bash
C=research/crazygames-build
python3 -I tools/re/asmjs_index.py $C/raw/pixelgunapocalypse3.2.js \
    research/crazygames-il2cpp-dump/script.json research/crazygames-il2cpp-dump/asmjs_index.json
python3 -I tools/re/asmjs_func.py $C/raw/pixelgunapocalypse3.2.js research/crazygames-il2cpp-dump/asmjs_index.json \
    research/crazygames-il2cpp-dump 'ConnectMenu$$' > research/notes/crazygames/connectmenu.js
```

- **`asmjs_mem_to_wasm.py`** wraps the `.mem` static memory (loaded at `Runtime.GLOBAL_BASE` = 8) in a wasm module with one data segment. Il2CppDumper can then search it (setup in `tools/extract/README.md`).
- **`asmjs_index.py`.** An asm.js function pointer is an index into the table of its signature (`var tp=[...]`, called as `tp[x&mask]`). Each `dynCall_<sig>` export names its table.
  - A method's function is `table[sig][Address & mask]`, with the same struct-return correction as `wasm_index.py`.
  - The script checks each function's own signature: parameter coercions `x|0` / `+x` / `fround(x)` and the return expression. 12 of 18,611 differ, all from the checker misreading bool returns of float comparisons. 338 methods have signatures without a table (never called through one).
- **`asmjs_func.py`** prints methods (by name substring or asm.js name) indented and annotated:
  - heap views by name (`HEAP32[x>>2]`; letters shadowed by locals are left alone);
  - known calls (`Zol/*UnityEngine.PlayerPrefs$$GetInt*/(...)`);
  - metadata slots (`HEAP32[239141]/*"GamePlays"*/`, `/*PhotonNetwork_TypeInfo*/`);
  - fields of `this` (`d+84/*.GamePlays*/`).
- **IL2CPP conventions in the output:**
  - static methods take a leading `0` (no `this`);
  - every call ends with the `MethodInfo*` argument;
  - the `if(!(HEAP8[N]|0)){JK(...)}` prologue initialises the method's metadata;
  - `(HEAP16[k+176>>1]&256)...bM(k)` runs a class's static constructor.

