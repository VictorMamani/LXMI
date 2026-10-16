# Confirmación de inicio con mods: adaptación de altura

El panel de Inicio usaba una altura fija con recorte. En una ventana de 960 × 600, los botones del aviso quedaban fuera del panel y no podían alcanzarse. Ahora el panel crece con su contenido y el área principal del workspace se desplaza dentro de la ventana. La casilla conserva 18 px de ancho y los botones tienen una altura mínima de 44 px.

Se revisó en navegador con datos sintéticos de Tauri para mostrar el perfil de 12 mods. El fixture rechaza comandos de lanzamiento o escritura; esta revisión no inicia Steam ni el juego y no sustituye una comprobación nativa de WebKit.

| Ventana | Scroll vertical | Sin recorte ni desbordamiento horizontal | Casilla y botones accesibles | Tab llega a Continuar |
|---|---|---|---|---|
| 1280 × 840 | Sí | Sí | Sí | Sí |
| 960 × 600 | Sí | Sí | Sí | Sí |
| 720 × 600 | Sí | Sí | Sí | Sí |
| 390 × 700 | Sí | Sí | Sí | Sí |
| 640 × 420 | Sí | Sí | Sí | Sí |

En todos los tamaños, Continuar permanece deshabilitado hasta marcar la casilla y Volver cierra el aviso. Capturas: `mod-launch-consent-before.png` y `mod-launch-consent-<ancho>x<alto>-after.png`, en este directorio. `npm run typecheck` y Prettier de `src/workspace.css` finalizaron correctamente. El servidor Vite existente sirve el cambio de CSS a la ventana abierta.
