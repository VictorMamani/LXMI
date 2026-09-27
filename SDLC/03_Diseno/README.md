# 03 | Diseño UX/UI

**Objetivo:** hacer comprensible el flujo principal en móvil y escritorio antes de pulir pantallas secundarias.

Los flujos de importación y perfiles siguen siendo conceptuales; no hay mockup ni prueba con personas usuarias. El flujo técnico de LXMI-0.1/0.2 consulta información del sistema, explora Steam Libraries y manifests, reconoce Wuthering Waves por AppID y observa `compatdata`/`pfx` en solo lectura. La ventana Tauri aún no se ejecutó en este entorno por dependencias nativas ausentes, así que la presentación visual del resultado no está comprobada en la aplicación de escritorio.

## Documentos sugeridos

- `flujos_de_usuario.md`: tarea principal desde inicio hasta resultado.
- `wireframes_y_prototipo.md`: enlace a bocetos, Figma o captura; resolver primero estructura y contenido.
- `contenido_y_estados.md`: textos, vacío, carga, error, éxito y confirmación.
- `criterios_de_accesibilidad.md`: navegación por teclado, etiquetas, contraste y jerarquía.
- `prueba_de_uso.md`: quién probó el prototipo, qué intentó hacer y dónde tuvo dificultad.

## Documentación de LXMI

- [Flujos de inspección, importación y perfiles](flujos_de_usuario.md)

## Salida de fase

Una persona del público elegido puede entender el siguiente paso y completar la tarea clave en un prototipo simple.
