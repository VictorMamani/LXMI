# Reparación de mods 3DMigoto/XXMI

`repair_mod.py` crea una **copia reparada** de un mod; nunca modifica el
original. La copia se importa en LXMI como mod nuevo.

| Problema | Aviso de ZZMI en pantalla | Reparación |
|---|---|---|
| Sección con nombre propio repetida en un mismo `.ini` | `Duplicate section found` | Se comentan las copias posteriores. 3DMigoto ya las ignoraba, así que no cambia nada en el juego |
| `[Resource…]` con archivo inexistente y sin uso | `Failed to load custom … resource` | Se comenta la sección |
| Imagen inexistente que sí se usa (p. ej. menús) | Igual | Imagen sustituta de 4×4: los bordes, una línea clara, y el resto, transparente |
| Buffer de geometría inexistente | Igual, y el objeto desaparece | Se desactiva ese `.ini` entero con el prefijo `DISABLED_` |
| `run =` o `ref` hacia una sección que no existe (con `--core`) | `Unrecognised entry` | Se comenta esa línea; si era la geometría de una pieza (`vb`/`ib`), se desactiva la pieza entera y el juego dibuja la original |
| `endif` sobrante o `if` sin cerrar | `endif missing if` / `If missing endif` | Se comenta el `endif` sobrante o se cierra el `if` |
| Texturas enormes (8K) | Ninguno, pero agotan la memoria | `--max-texture 4096` reduce BC1/BC3/BC7 al tamaño indicado y las recodifica en el mismo formato |

Las secciones globales `[Constants]` y `[Present]` repetidas en un mod son
válidas, porque 3DMigoto las fusiona, y no se tocan. Las líneas cambiadas
empiezan por `; LXMI repair:`.

## Uso

```bash
python3 -m venv ~/.local/share/lxmi/tools/texvenv
~/.local/share/lxmi/tools/texvenv/bin/pip install "etcpak==0.9.13" "Pillow==11.0.0"
~/.local/share/lxmi/tools/texvenv/bin/python SDLC/05_Construccion/aplicacion/tools/mod-repair/repair_mod.py \
    "/ruta/al/mod" ~/LXMI-mods-reparados/MiMod_reparado --core ~/.local/share/lxmi/launches/zenless-zone-zero/<último>/ZZMI \
    --max-texture 4096 --report informe.json
```

Después: **Mods → Importar mod** con la carpeta reparada; en **Perfiles**
cambia el mod original por el reparado y vuelve a aplicar el perfil.

## Límites

- La reducción de texturas pierde detalle. En la prueba con BC7 de 8K a 4K,
  el error medio del ida y vuelta fue inferior a 0,5/255 por canal.
- Las texturas con mipmaps o en otros formatos se dejan igual y figuran en el
  informe.
- Un sustituto no recupera el diseño original del autor: solo evita el error.
