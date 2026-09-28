# 06 | Verificación

**Objetivo:** confirmar que el producto hace lo acordado sin exponer datos o romper el flujo principal.

LXMI-0.1 a 0.5.1 cuenta con fixtures Rust de KeyValues/ACF, Steam games/compatdata, Proton/custom tools/Steam Linux Runtime, planes, asociaciones WWMI/ZZMI y seguridad de importación de paquetes. La verificación 0.5.1 está detallada en [verificacion_0_5_1.md](verificacion_0_5_1.md): el host Steam encontró ZZZ y el scan fue de solo lectura; los paquetes ZZMI de la suite son fixtures. No se ejecutaron juegos ni se comprobó compatibilidad XXMI con Steam/Linux/Proton. No se importó una release ZZMI real ni se hizo apply.

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
