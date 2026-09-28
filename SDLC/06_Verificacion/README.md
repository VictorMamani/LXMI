# 06 | Verificación

**Objetivo:** confirmar que el producto hace lo acordado sin exponer datos o romper el flujo principal.

LXMI-0.1/0.2/0.3 cuenta con fixtures Rust de KeyValues/ACF, Steam games/compatdata, Proton/custom tools/Steam Linux Runtime, errores y symlinks; el workspace completo pasó check, Clippy y tests, y el frontend pasó typecheck, lint, formato y build. Los scanners también se ejecutaron contra Steam local en solo lectura y la ventana Tauri se abrió/revisó visualmente con los resultados. No se ejecutaron runtimes/juegos ni se probó compatibilidad XXMI. Importación y perfiles siguen siendo criterios futuros.

## Documentos sugeridos

- `estrategia_de_calidad.md`: revisión manual, pruebas unitarias/integración/e2e según riesgo.
- `casos_del_flujo_principal.md`: pasos, dato de prueba y resultado esperado.
- `accesibilidad_y_dispositivos.md`: teclado, lector/semántica, anchuras móviles y navegadores objetivo.
- `revisiones_de_seguridad.md`: permisos, aislamiento entre clientes, validación y gestión de secretos.
- `defectos_conocidos.md`: problema, impacto, workaround y decisión.

## Documentación de LXMI

- [Estrategia de calidad](estrategia_de_calidad.md)
- [Casos de prueba iniciales](casos_de_prueba_iniciales.md)

## Salida de fase

La salida de fase para un piloto requerirá completar las pruebas de sistema que correspondan. El discovery local de Steam no prueba selección, compatibilidad o ejecución de juegos con Proton.
