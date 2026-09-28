# SDLC | Proyecto 19 — LXMI

Estructura de ciclo de vida conservada desde la plantilla común. LXMI-0.1 a 0.5 están implementados en `05_Construccion/aplicacion/`. El workspace, incluido Tauri y `lxmi-xxmi`, pasó check, Clippy y 114 tests; el frontend pasó typecheck, lint, formato y build. El discovery se ejecutó contra Steam local en modo de solo lectura; la importación de paquetes se probó en fixtures y corpus upstream dentro de almacenamiento temporal, nunca en el juego. La ventana Tauri inició; el flujo IPC de import/review no se recorrió clic a clic. No se probó lanzamiento ni compatibilidad Proton/XXMI.

## Fases

1. [Descubrimiento](01_Descubrimiento/README.md)
2. [Planificación](02_Planificacion/README.md)
3. [Diseño](03_Diseno/README.md)
4. [Arquitectura y seguridad](04_Arquitectura_y_seguridad/README.md)
5. [Construcción](05_Construccion/README.md)
6. [Verificación](06_Verificacion/README.md)
7. [Lanzamiento](07_Lanzamiento/README.md)
8. [Operación](08_Operacion/README.md)
9. [Mejora y cierre](09_Mejora_y_cierre/README.md)

Se conservaron las nueve etapas de la plantilla. El estado de cada fase debe distinguir decisiones documentadas, implementación y validaciones realmente ejecutadas; la documentación inicial no constituye evidencia de compatibilidad con juegos o runtimes.
