---
name: LXMI
description: Workspace de escritorio oscuro derivado del prototipo LXMI autorizado.
colors:
  deep: "#090b0c"
  background: "#0e1011"
  surface: "#151819"
  elevated: "#1b1f20"
  soft-surface: "#202526"
  text: "#f0eee9"
  secondary: "#aaa9a4"
  muted: "#858c86"
  border: "#292d2e"
  accent: "#86b8a9"
  accent-quiet: "#6d9182"
  warning: "#c4a96f"
  danger: "#bc7c72"
  action-light: "#e5e4dc"
  action-ink: "#171b19"
  action-light-hover: "#f3f1e8"
  action-secondary: "#191d1c"
  action-secondary-text: "#d9dbd4"
  action-secondary-hover: "#202524"
typography:
  display:
    fontFamily: '"Inter", "Noto Sans", "Ubuntu", "DejaVu Sans", system-ui, sans-serif'
    fontSize: "clamp(32px, 3.5vw, 49px)"
    fontWeight: 550
    lineHeight: 1.05
    letterSpacing: "-0.035em"
  headline:
    fontFamily: '"Inter", "Noto Sans", "Ubuntu", "DejaVu Sans", system-ui, sans-serif'
    fontSize: "clamp(24px, 2.2vw, 31px)"
    fontWeight: 560
    lineHeight: 1.12
    letterSpacing: "-0.04em"
  body:
    fontFamily: '"Inter", "Noto Sans", "Ubuntu", "DejaVu Sans", system-ui, sans-serif'
    fontSize: "14px"
    lineHeight: 1.45
  label:
    fontSize: "12px"
    fontWeight: 600
  mono:
    fontFamily: '"JetBrains Mono", "DejaVu Sans Mono", ui-monospace, monospace'
rounded:
  window: "9px"
  panel: "7px"
  control: "5px"
  row: "4px"
spacing:
  tight: "8px"
  control-gap: "9px"
  item-gap: "12px"
  compact: "16px"
  section: "20px"
  spacious: "24px"
components:
  button-primary:
    backgroundColor: "{colors.action-light}"
    textColor: "{colors.action-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 14px"
  button-primary-hover:
    backgroundColor: "{colors.action-light-hover}"
  button-secondary:
    backgroundColor: "{colors.action-secondary}"
    textColor: "{colors.action-secondary-text}"
    rounded: "{rounded.control}"
    padding: "0 14px"
  button-secondary-hover:
    backgroundColor: "{colors.action-secondary-hover}"
  button-quiet:
    backgroundColor: "transparent"
    textColor: "#b8bcb5"
    rounded: "{rounded.control}"
    padding: "0 14px"
  button-runtime:
    backgroundColor: "{colors.accent}"
    textColor: "#101614"
    rounded: "{rounded.control}"
    padding: "0 15px"
  input:
    backgroundColor: "{colors.background}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "8px 10px"
  panel:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.panel}"
    padding: "23px 25px 25px"
---

# Design System: LXMI

## Evolución a launcher — 2026-10-15

La interfaz actual autorizada está en `src/launcher.css` de la aplicación operativa. Usa ilustraciones SVG originales a pantalla completa por juego, barra superior translúcida con selector y navegación, título y estado a la izquierda y panel de perfil e inicio a la derecha. La acción principal adopta lima para ZZZ y celeste para WuWa; Mods, Perfiles y Compatibilidad se muestran sobre el fondo del juego difuminado. Los fundamentos y capturas del workspace inicial que siguen registran su diseño de origen. La revisión nativa del launcher está en `SDLC/06_Verificacion/evidencias/interfaz_launcher_nativo_20261015.json`.

## Overview

**Creative North Star: "Workspace LXMI del prototipo autorizado"**

La autoridad es el prototipo existente elegido por el usuario. Este registro describe su traslado a la aplicación operativa: una herramienta oscura, sobria y compacta, con arte geométrico original y una jerarquía clara entre contenido, estado y acción. El nombre de referencia identifica esa continuidad; no introduce una nueva identidad.

Las superficies neutras sostienen controles discretos y acentos salvia. El estado funcional aporta el contenido: instalación detectada, biblioteca, perfil y runtime. La apariencia no debe implicar una carga en juego que todavía no se verificó; véase [PRODUCT.md](PRODUCT.md).

**Key Characteristics:**
- Campo oscuro con capas tonales y bordes finos.
- Sans compacta, titulares moderados y controles de escritorio.
- Acento salvia para selección y estado; acción principal clara.
- Arte SVG original existente como material de la identidad.

## Colors

La paleta combina carbón, texto cálido y salvia apagada; los valores normativos están en el frontmatter y proceden de `src/tokens.css` y de las variantes implementadas.

### Primary
- **Salvia** (`accent`): selección de navegación, foco general y acciones de preparación del runtime.
- **Salvia quieta** (`accent-quiet`): reserva de la paleta heredada; no implica una nueva variante de acción.
- **Marfil de acción** (`action-light`): botón principal del shell, incluido «Jugar sin mods»; usa tinta oscura y su variante hover.

### Neutral
- **Carbón profundo** (`deep`): campo exterior y rail.
- **Carbón de workspace** (`background`): fondo principal y campos.
- **Superficie** (`surface`), **elevada** (`elevated`) y **suave** (`soft-surface`): paneles, controles y respuesta de filas.
- **Texto cálido** (`text`), **secundario** (`secondary`) y **atenuado** (`muted`): niveles de lectura.
- **Borde** (`border`): separación del contenido y las áreas de revisión.
- **Ámbar** (`warning`) y **arcilla** (`danger`): estados de advertencia y error, acompañados por texto.

**The State Rule.** Un color de estado acompaña información verificable; no sustituye su etiqueta ni certifica carga en juego.

## Typography

La familia UI usa la pila sans definida en tokens, con Inter como primera preferencia y alternativas Linux. No se declara descarga ni disponibilidad garantizada de esas fuentes. La pila mono se reserva para información técnica donde la implementación la utiliza.

La jerarquía normativa recoge el título de juego, encabezado de página, cuerpo y etiqueta de acción. En el hero móvil, el título se adapta con `clamp(28px, 6vw, 42px)`. Las notas funcionales usan un interlineado más abierto (1.6); la nota de runtime limita la medida a (38ch). Las rutas largas admiten saltos dentro del contenido.

Los metadatos existentes de (9–11px) se registran como densidad heredada, no como objetivo para nueva información esencial. No elevar los pequeños rótulos en mayúsculas del prototipo a una nueva regla global.

## Layout

El shell combina un rail lateral y un workspace flexible. El rail mide (74px); el encabezado parte de (68px) y el pie de (28px). El contenido usa un máximo de (1440px), mientras gestor y compatibilidad limitan su ancho a (1320px). El padding horizontal de escritorio es `clamp(22px, 3.2vw, 46px)`.

El inicio mantiene el hero del prototipo: arte a la izquierda y estado/acciones a la derecha, en columnas `1.62fr` y un panel de mínimo (335px). A partir de (1150px) tiene altura mínima de (520px). A (1100px) el encabezado permite una segunda fila de navegación y el hero usa proporciones `1.2fr / 1fr`.

A (760px) o menos, el rail baja a (56px), el contenido usa padding (20px 16px), la navegación permite desplazamiento horizontal y el hero se apila. El arte conserva al menos (290px) y el panel de controles (420px). Los metadatos y acciones se envuelven; las rutas no desbordan. La biblioteca tiene además un reflujo propio a (700px). El ancho mínimo implementado del documento es (360px).

## Elevation & Depth

La profundidad procede sobre todo de capas tonales, bordes y del arte SVG sombreado. El hero conserva una sombra ambiental (`0 20px 44px -35px rgba(0, 0, 0, 0.88)`); el titular sobre arte utiliza sombra de texto para legibilidad. Los paneles operativos no añaden esa elevación.

Las transiciones de controles duran (140–160ms), con salida rápida y sin animación continua. El estado activo reduce el botón a (0.97) y el selector de juego a (0.96). La preferencia de movimiento reducido acorta transiciones y animaciones a (0.01ms).

**The Continuity Rule.** Las extensiones conservan la composición y el material del prototipo autorizado; el reflujo adapta su lectura sin inventar otro shell.

## Shapes

Las esquinas son suavemente rectangulares: ventana/hero, panel y control usan las escalas del frontmatter. Los bordes son finos (1px). Los puntos de estado son circulares y el juego activo tiene una pequeña marca lateral; esa geometría se conserva junto con el foco visible.

## Components

### Buttons

El shell usa acciones compactas con icono SVG, espacio interior y una altura mínima de (38px); «Jugar sin mods» crece a (42px). Hay variantes clara, secundaria con borde y quieta. Las acciones del runtime usan el acento salvia y altura mínima de (40px). Son dos familias implementadas: no se deben unificar fingiendo que ya comparten todas sus variantes.

Los botones del shell responden a hover y presión. El foco general tiene contorno de (2px) y separación de (3px). Los controles deshabilitados conservan etiqueta y reducen opacidad. Los snippets del sidecar conservan estados de cada familia.

### Inputs / Fields

Campos y selectores oscuros con borde visible, esquinas de control y altura mínima de (40px). Las etiquetas están asociadas al campo; placeholder no las sustituye. La familia de formularios conserva su foco salvia claro existente (`#9dd7bf`), distinto del foco general. El error se presenta en texto con semántica de alerta.

### Cards / Containers

Los paneles usan superficie neutra y borde, con padding normativo del frontmatter; en móvil se reduce a (20px 16px). Las áreas de revisión se separan con bordes horizontales. Las filas de mods usan una casilla, nombre y volumen, con altura mínima de (64px), radio de fila y respuesta tonal al hover.

### Navigation

El rail selecciona el juego por portada original y marca lateral. La navegación superior agrupa Inicio, Mods, Perfiles y Compatibilidad con subrayado salvia del destino activo. Los botones de icono tienen nombres accesibles. A anchos reducidos la navegación pasa a una fila propia y permite desplazamiento horizontal.

### Hero de juego

El arte existente usa recorte y sombras superpuestas; el título se apoya en la zona inferior izquierda. El panel derecho reúne instalación, perfil, biblioteca, runtime y CTA. Los datos y avisos funcionales sustituyen las muestras del prototipo sin cambiar su jerarquía visual.

## Do's and Don'ts

### Do:
- **Do** conservar los SVG originales y el mundo visual del prototipo autorizado.
- **Do** distinguir selección, preparación privada y carga en juego mediante etiquetas visibles.
- **Do** mantener foco visible, etiquetas accesibles y reducción de movimiento.
- **Do** permitir que rutas, acciones y metadatos se ajusten en anchos estrechos.

### Don't:
- **Don't** sustituir el shell o el arte por un concepto visual nuevo sin un cambio de dirección autorizado.
- **Don't** convertir los rótulos diminutos heredados en el estándar de información esencial.
- **Don't** usar únicamente color para comunicar selección, advertencia o error.
- **Don't** presentar las capturas sintéticas como prueba de ejecución nativa o de mods cargados en juego.
