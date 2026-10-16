#!/usr/bin/env python3
"""Writes a repaired copy of a 3DMigoto/XXMI mod folder; the source is never modified.

Repairs, following upstream 3DMigoto behaviour (IniHandler.cpp, CommandList.cpp):
  * Namespaced sections repeated in one INI: 3DMigoto keeps the first and warns
    ("Duplicate section"); later copies are commented out, so nothing in game changes.
  * [Resource*] whose file is missing ("Failed to load custom ... resource"):
    unreferenced sections are commented out; referenced images get a placeholder.
    An INI that needs a missing buffer is disabled as a whole (DISABLED_ prefix,
    which 3DMigoto skips): otherwise it hides the original mesh and draws nothing.
  * --core DIR: lines whose `run =`/`ref` target exists nowhere ("Unrecognised entry")
    are commented out, and unbalanced if/endif in command lists are closed.
  * --max-texture N: BC1/BC3/BC7 DDS textures larger than N are resampled to fit N
    and re-encoded in the same DXGI format.

Requires Pillow and etcpak (see README.md). Usage:
  repair_mod.py SOURCE DEST [--max-texture 4096] [--report report.json]
"""
import argparse
import json
import re
import shutil
import struct
import sys
from pathlib import Path

NAMESPACED = (
    "textureoverride", "shaderoverride", "shaderregex", "resource", "commandlist",
    "customshader", "builtincustomshader", "builtincommandlist", "key", "preset",
)
IMAGE_EXT = {".png", ".dds", ".jpg", ".jpeg", ".bmp", ".tga"}
MARK = "; LXMI repair: "
# DXGI formats -> (etcpak codec, bytes per 4x4 block)
DX10_FORMATS = {71: ("bc1", 8), 72: ("bc1", 8), 77: ("bc3", 16), 78: ("bc3", 16),
                98: ("bc7", 16), 99: ("bc7", 16)}
FOURCC_FORMATS = {b"DXT1": ("bc1", 8), b"DXT5": ("bc3", 16)}


def disabled(path: Path, root: Path) -> bool:
    return any(p.upper().startswith("DISABLED") for p in path.relative_to(root).parts)


def find_ci(base: Path, relative: str):
    """Case-insensitive lookup, as Wine resolves paths."""
    current = base
    for part in [p for p in relative.replace("\\", "/").split("/") if p not in ("", ".")]:
        if part == "..":
            current = current.parent
            continue
        try:
            names = {c.name.lower(): c for c in current.iterdir()}
        except OSError:
            return None
        current = names.get(part.lower())
        if current is None:
            return None
    return current


def sections(lines):
    """[(name, start, end)] with end exclusive; the header line is `start`."""
    found, current = [], None
    for i, line in enumerate(lines):
        text = line.split(";", 1)[0].strip()
        if text.startswith("[") and "]" in text:
            if current:
                found.append((current[0], current[1], i))
            current = (text[1:text.index("]")].strip(), i)
    if current:
        found.append((current[0], current[1], len(lines)))
    return found


def comment_out(lines, start, end, why):
    lines[start] = f"{MARK}{why}\n{MARK}{lines[start]}"
    for i in range(start + 1, end):
        if lines[i].strip():
            lines[i] = MARK + lines[i]


def placeholder(path: Path, name: str):
    from PIL import Image
    path.parent.mkdir(parents=True, exist_ok=True)
    # Borders stay visible as thin light lines; any other image becomes transparent.
    colour = (235, 235, 240, 220) if "border" in name.lower() else (0, 0, 0, 0)
    image = Image.new("RGBA", (4, 4), colour)
    if path.suffix.lower() == ".dds":
        image.save(path.with_suffix(".png"))
        return path.with_suffix(".png").name, "png sustituto (el INI se actualizó)"
    image.save(path)
    return path.name, "imagen sustituta"


def repair_inis(root: Path, report):
    inis = [p for p in root.rglob("*") if p.is_file() and p.suffix.lower() == ".ini"
            and p.name.lower() != "desktop.ini" and not disabled(p, root)]
    texts = {p: p.read_text(encoding="utf-8", errors="surrogateescape") for p in inis}
    corpus = "\n".join(t for t in texts.values())
    for ini, text in texts.items():
        lines = text.splitlines(keepends=True)
        if lines and not lines[-1].endswith("\n"):
            lines[-1] += "\n"
        changed, broken, seen = False, False, set()
        for name, start, end in sections(lines):
            key = name.lower()
            if key.startswith(NAMESPACED) and key in seen:
                comment_out(lines, start, end, "sección duplicada; 3DMigoto ya ignoraba esta copia")
                report["duplicates"].append({"ini": str(ini.relative_to(root)), "section": name})
                changed = True
                continue
            seen.add(key)
            if not key.startswith("resource"):
                continue
            for i in range(start + 1, end):
                m = re.match(r"\s*filename\s*=\s*(.+?)\s*$", lines[i].split(";", 1)[0], re.I)
                if not m:
                    continue
                target = m.group(1).strip('"')
                if find_ci(ini.parent, target) is not None:
                    break
                uses = len(re.findall(rf"(?<![\w\\]){re.escape(name)}(?![\w])", corpus, re.I))
                item = {"ini": str(ini.relative_to(root)), "section": name, "file": target}
                if uses <= 1:
                    comment_out(lines, start, end, "recurso sin archivo y sin uso")
                    report["unused_missing"].append(item)
                elif Path(target.replace("\\", "/")).suffix.lower() in IMAGE_EXT:
                    dest = ini.parent / target.replace("\\", "/")
                    new_name, kind = placeholder(dest, name)
                    if new_name != dest.name:
                        lines[i] = lines[i].replace(Path(target).name, new_name)
                    report["placeholders"].append({**item, "created": kind})
                else:
                    report["unrepairable"].append(item)
                    broken = True
                changed = True
                break
        if changed:
            ini.write_text("".join(lines), encoding="utf-8", errors="surrogateescape")
        if broken:
            disabled_ini = ini.with_name("DISABLED_" + ini.name)
            ini.rename(disabled_ini)
            report["disabled_inis"].append(str(disabled_ini.relative_to(root)))


REF = re.compile(r"(?i)(?:\brun\s*=\s*|\bref\s+|^[\w\-\\$.]+\s*=\s*)((?:Resource|CommandList|CustomShader)[^\s,;]*)")


def section_names(path: Path):
    names = set()
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        text = line.split(";", 1)[0].strip()
        if text.startswith("[") and "]" in text:
            names.add(text[1:text.index("]")].strip().lower())
    return names


def namespace_of(lines, ini: Path):
    for line in lines:
        text = line.split(";", 1)[0].strip()
        if text.startswith("["):
            break
        if re.match(r"(?i)namespace\s*=", text):
            return text.split("=", 1)[1].strip().lower()
    return str(ini)


def repair_references(root: Path, core: Path, report):
    """Unresolved `run =`/`ref` targets and unbalanced if/endif make 3DMigoto print
    "Unrecognised entry" / "endif missing if"; such lines are commented out or closed."""
    core_names = set()
    for path in core.rglob("*.ini"):
        if "mods" not in (p.lower() for p in path.relative_to(core).parts):
            core_names |= section_names(path)
    inis = [p for p in root.rglob("*") if p.is_file() and p.suffix.lower() == ".ini"
            and p.name.lower() != "desktop.ini" and not disabled(p, root)]
    texts = {p: p.read_text(encoding="utf-8", errors="surrogateescape").splitlines(keepends=True)
             for p in inis}
    defined = {}
    for ini, lines in texts.items():
        defined.setdefault(namespace_of(lines, ini), set()).update(section_names(ini))
    for ini, lines in texts.items():
        if lines and not lines[-1].endswith("\n"):
            lines[-1] += "\n"
        known = defined[namespace_of(lines, ini)] | core_names
        changed, geometry_stems = False, set()
        for name, start, end in reversed(sections(lines)):
            if name.lower().startswith("shaderregex"):
                continue  # HLSL bodies, not command lists
            depth, last = 0, start
            for i in range(start + 1, end):
                text = lines[i].split(";", 1)[0].strip()
                if not text:
                    continue
                last = i
                if re.match(r"(?i)filename\s*=", text):
                    continue
                missing = [r for r in REF.findall(text)
                           if "\\" not in r and "/" not in r and r.lower() not in known
                           and not r.lower().endswith((".hlsl", ".dds", ".png", ".buf", ".ib"))]
                if missing:
                    if name.lower().startswith("textureoverride") and re.match(r"(?i)(vb\d+|ib)\s*=", text):
                        stem = re.sub(r"(Blend|Tex[Cc]oord|Position|VertexLimitRaise|IB|[A-Z])$", "", name)
                        if len(stem) > len("TextureOverride") + 2:
                            geometry_stems.add(stem.lower())
                    lines[i] = f"{MARK}usa {missing[0]}, que no existe\n{MARK}{lines[i]}"
                    report["undefined_references"].append(
                        {"ini": str(ini.relative_to(root)), "section": name, "target": missing[0]})
                    changed = True
                    continue
                if re.match(r"(?i)((pre|post)\s+)?if\b", text):
                    depth += 1
                elif re.match(r"(?i)endif\b", text):
                    if depth == 0:
                        lines[i] = f"{MARK}endif sin if\n{MARK}{lines[i]}"
                        report["if_balance"].append({"ini": str(ini.relative_to(root)), "section": name,
                                                     "fix": "endif sobrante comentado"})
                        changed = True
                    else:
                        depth -= 1
            if depth > 0:
                lines.insert(last + 1, f"{MARK}if sin endif cerrado\n" + "endif\n" * depth)
                report["if_balance"].append({"ini": str(ini.relative_to(root)), "section": name,
                                             "fix": f"{depth} endif añadido"})
                changed = True
        # Without its geometry the part would still hide the original mesh and draw
        # with the mod's counts over game buffers: disable the whole part instead.
        for name, start, end in reversed(sections(lines)):
            if any(name.lower().startswith(stem) for stem in geometry_stems):
                comment_out(lines, start, end, "pieza sin geometría; se usa la original del juego")
                report["disabled_parts"].append({"ini": str(ini.relative_to(root)), "section": name})
        if changed:
            ini.write_text("".join(lines), encoding="utf-8", errors="surrogateescape")


def dds_info(raw: bytes):
    if raw[:4] != b"DDS " or len(raw) < 128:
        return None
    height, width = struct.unpack_from("<II", raw, 12)
    mips = struct.unpack_from("<I", raw, 28)[0]
    fourcc = raw[84:88]
    if fourcc == b"DX10":
        dxgi = struct.unpack_from("<I", raw, 128)[0]
        return width, height, mips, DX10_FORMATS.get(dxgi), 148, f"DXGI {dxgi}"
    return width, height, mips, FOURCC_FORMATS.get(fourcc), 128, fourcc.decode("ascii", "replace")


def shrink_textures(root: Path, limit: int, report):
    import etcpak
    from PIL import Image
    for path in sorted(p for p in root.rglob("*") if p.is_file() and p.suffix.lower() == ".dds"):
        raw = path.read_bytes()
        info = dds_info(raw)
        if not info or max(info[0], info[1]) <= limit:
            continue
        width, height, mips, codec, offset, label = info
        entry = {"file": str(path.relative_to(root)), "from": [width, height], "format": label}
        if codec is None or mips > 1:
            report["textures_skipped"].append({**entry, "reason": "formato o mipmaps no soportados"})
            continue
        name, block = codec
        image = Image.frombytes("RGBA", (width, height),
                                getattr(etcpak, f"decompress_{name}")(raw[offset:], width, height))
        scale = limit / max(width, height)
        size = (max(4, round(width * scale / 4) * 4), max(4, round(height * scale / 4) * 4))
        image = image.resize(size, Image.LANCZOS)
        rgba = image.tobytes()
        if name == "bc7":
            data = etcpak.compress_bc7(rgba, size[0], size[1], etcpak.BC7CompressBlockParams())
        else:
            data = getattr(etcpak, f"compress_{name}")(rgba, size[0], size[1])
        header = bytearray(raw[:offset])
        struct.pack_into("<II", header, 12, size[1], size[0])
        struct.pack_into("<I", header, 20, (size[0] // 4) * (size[1] // 4) * block)
        path.write_bytes(bytes(header) + data)
        report["textures"].append({**entry, "to": list(size),
                                   "bytes_before": len(raw), "bytes_after": len(header) + len(data)})
        print(f"  {entry['file']}: {width}x{height} -> {size[0]}x{size[1]}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("source", type=Path)
    parser.add_argument("dest", type=Path)
    parser.add_argument("--max-texture", type=int, default=0)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--core", type=Path,
                        help="Carpeta del importador (p. ej. ZZMI) para resolver referencias globales")
    args = parser.parse_args()
    source, dest = args.source.resolve(), args.dest.resolve()
    if not source.is_dir():
        sys.exit(f"No existe la carpeta {source}")
    if dest.exists() or dest.is_relative_to(source) or source.is_relative_to(dest):
        sys.exit("El destino debe ser una carpeta nueva fuera del mod original.")
    shutil.copytree(source, dest, symlinks=True)
    report = {"source": str(source), "dest": str(dest), "duplicates": [], "unused_missing": [],
              "placeholders": [], "unrepairable": [], "disabled_inis": [],
              "undefined_references": [], "if_balance": [], "disabled_parts": [], "textures": [], "textures_skipped": []}
    repair_inis(dest, report)
    if args.core:
        repair_references(dest, args.core.resolve(), report)
    else:
        print("Sin --core: no se revisan referencias a secciones inexistentes.")
    if args.max_texture:
        shrink_textures(dest, args.max_texture, report)
    before = sum(t["bytes_before"] for t in report["textures"])
    after = sum(t["bytes_after"] for t in report["textures"])
    print(f"Duplicadas comentadas: {len(report['duplicates'])} · recursos sin uso comentados: "
          f"{len(report['unused_missing'])} · sustitutos: {len(report['placeholders'])} · "
          f"INI desactivados: {len(report['disabled_inis'])} · referencias rotas: "
          f"{len(report['undefined_references'])} · if/endif: {len(report['if_balance'])} · "
          f"piezas desactivadas: {len(report['disabled_parts'])} · "
          f"texturas reducidas: "
          f"{len(report['textures'])} ({before / 2**20:.0f} MB -> {after / 2**20:.0f} MB)")
    if args.report:
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
