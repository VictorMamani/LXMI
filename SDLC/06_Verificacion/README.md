# 06 | Verificación

**Objetivo:** confirmar que el producto hace lo acordado sin exponer datos o romper el flujo principal.

LXMI-0.1/0.2 cuenta con pruebas sintéticas de modelos del core, parser KeyValues/ACF, escaneo de manifests, catálogo de AppIDs y estados compatdata/pfx, además de typecheck, lint, formato y build web. No se ejecutaron la ventana nativa Tauri ni pruebas contra Steam real; los prerrequisitos nativos faltantes se registran en construcción y trabajo pendiente. Los casos futuros de importación y perfiles siguen siendo criterios de diseño.

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

La salida de fase para un piloto requerirá además validar la aplicación nativa en Ubuntu y completar las pruebas de sistema que correspondan; los resultados sintéticos actuales no prueban el descubrimiento en una instalación real de Steam ni compatibilidad o ejecución de juegos con Proton.
