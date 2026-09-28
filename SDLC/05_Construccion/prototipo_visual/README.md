# Prototipo visual de LXMI

## Objetivo

Prototipo navegable de alta fidelidad para explorar la interfaz futura de **LXMI — Linux Model Importer**. Está separado de la aplicación Tauri funcional y utiliza datos locales de muestra.

## Ejecutar

Requisitos: Node.js 20.19 o superior y npm.

```bash
cd SDLC/05_Construccion/prototipo_visual/lxmi-desktop
npm install
npm run dev
```

Vite muestra la dirección local al iniciar. Para generar y revisar la compilación:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
npm run preview
```

## Dirección visual

LXMI se presenta primero como un launcher de escritorio para jugadores Linux: selector de juegos compacto, navegación contextual y un espacio principal que pone el artwork, el profile activo y la acción de jugar por delante de los detalles del sistema.

La interfaz usa una base oscura y cálida, superficies sobrias y una paleta emocional contenida dentro del artwork de cada juego. La información de Steam, Proton, Wine prefix y registros vive en Compatibilidad y Diagnósticos, no en la pantalla de inicio.

## Paleta y tipografía

Los tokens están centralizados en `lxmi-desktop/src/tokens.css`.

| Token | Color | Uso |
| --- | --- | --- |
| Deep Background | `#090B0C` | rail y marco |
| Background | `#0E1011` | espacio de trabajo |
| Surface | `#151819` | paneles principales |
| Elevated | `#1B1F20` | controles seleccionados |
| Soft Surface | `#202526` | detalles y superficies elevadas |
| Primary Text | `#F0EEE9` | títulos y contenido principal |
| Secondary Text | `#AAA9A4` | explicación secundaria |
| Muted Text | `#858C86` | metadatos discretos con contraste legible |
| Border | `#292D2E` | separadores y contornos puntuales |
| LXMI Accent | `#86B8A9` | foco, selección y estado positivo |

La interfaz usa una sans-serif de sistema con preferencias por Inter, Noto Sans y Ubuntu; en Linux se conserva una alternativa legible disponible localmente. Monospace se limita a rutas, IDs, versiones y valores de diagnóstico.

## Componentes y pantallas

- **Inicio:** artwork abstracto original, juego, profile, número de mods activos, runtime y botón Jugar.
- **Mods:** biblioteca en lista visual, búsqueda, filtros, orden, conflicto potencial, detalle y estado activado/desactivado.
- **Profiles:** selección y activación de profiles, resumen de contenido, edición local de mods, duplicar y crear.
- **Compatibilidad:** runtimes de muestra y explicación clara cuando el prefix aún no está inicializado.
- **Configuración:** General, Juego, Launcher, Compatibilidad, XXMI/WWMI, Mods y Avanzado.
- **Diagnósticos:** vistas de manifiesto, Wine prefix, runtime, entorno y registros.

Las acciones principales tienen estados de foco, hover, pressed, desactivado, aviso, carga y vacío donde corresponde. El movimiento es breve y se reduce con `prefers-reduced-motion`.

## Datos de muestra

Los perfiles, mods, runtimes, paths, autores, fechas, conflictos y mensajes de actividad son ficticios y viven en `lxmi-desktop/src/data.ts`. Las acciones solo cambian estado React durante la sesión del navegador. Los botones de lanzar, copiar, abrir ubicaciones e importar no ejecutan operaciones del sistema.

Los assets de juego son SVG abstractos creados para este prototipo; no se copiaron artworks ni capturas protegidas. Wuthering Waves usa el AppID de referencia `3513350` proporcionado para el proyecto; el resto de los valores se presenta como ilustrativo.

## Límites

- Este prototipo no es la UI de producción y no forma parte de la aplicación Tauri.
- No conecta con Steam, Proton, Wine, XXMI, WWMI, ZZMI ni GameBanana.
- No escanea, descarga, extrae, configura ni modifica archivos.
- Los estados de runtime, prefix, WWMI, mods y profiles no representan una instalación real.
- No constituye evidencia de compatibilidad o de que el producto final ya implemente estas funciones.

## Ubicación

El código de esta experiencia vive en `lxmi-desktop/`. La aplicación funcional se conserva por separado en `../aplicacion/` y no fue modificada como parte de este prototipo.
