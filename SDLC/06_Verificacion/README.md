# 06 | Verificación

**Objetivo:** confirmar que el producto hace lo acordado sin exponer datos o romper el flujo principal.

LXMI-0.1 a 0.5.1 cuenta con fixtures Rust de KeyValues/ACF, Steam games/compatdata, Proton/custom tools/Steam Linux Runtime, planes, asociaciones WWMI/ZZMI y seguridad de importación. En 0.5.2 se autenticaron/importaron releases oficiales y el dry-run del directorio del juego quedó como comparación histórica. En 0.5.3 se ensambló un runtime privado XDG y se verificó read-only el prefix real. Nada de esto prueba compatibilidad XXMI con Steam/Linux/Proton ni ejecuta el juego, y no hubo apply.

## Documentos sugeridos

- `estrategia_de_calidad.md`: revisión manual, pruebas unitarias/integración/e2e según riesgo.
- `casos_del_flujo_principal.md`: pasos, dato de prueba y resultado esperado.
- `accesibilidad_y_dispositivos.md`: teclado, lector/semántica, anchuras móviles y navegadores objetivo.
- `revisiones_de_seguridad.md`: permisos, aislamiento entre clientes, validación y gestión de secretos.
- `defectos_conocidos.md`: problema, impacto, workaround y decisión.

## Documentación de LXMI

- [Estrategia de calidad](estrategia_de_calidad.md)
- [Casos de prueba iniciales](casos_de_prueba_iniciales.md)
- [Verificación LXMI 0.5.2](verificacion_0_5_2.md)
- [Verificación LXMI 0.5.3](verificacion_0_5_3.md)

## Salida de fase

La salida de fase para un piloto requerirá completar las pruebas de sistema que correspondan. El discovery local de Steam no prueba selección, compatibilidad o ejecución de juegos con Proton.
