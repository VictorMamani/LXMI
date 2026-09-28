# 04 | Arquitectura y seguridad

**Objetivo:** decidir la estructura técnica mínima y cuidar los datos desde el comienzo.

La arquitectura mantiene decisiones conceptuales para fases futuras. LXMI-0.1–0.4 implementa discovery y planificación efímera con Tauri/React, `lxmi-core`, `lxmi-steam`, `lxmi-proton` y `lxmi-runtime`, en modo de solo lectura. LXMI 0.5 agrega `lxmi-xxmi`: importa explícitamente paquetes validados a storage XDG administrado; la inspección externa sigue siendo read-only y el plan todavía no se aplica. El análisis de amenazas y límites para importación está documentado; la seguridad de instalación y ejecución sigue pendiente.

## Documentos sugeridos

- `arquitectura.md`: diagrama/contexto, componentes, servicios externos y hosting.
- `modelo_de_datos.md`: entidades, relaciones, dueño de cada dato y retención.
- `api_e_integraciones.md`: endpoints/eventos, permisos, costos y dependencias de terceros.
- `decisiones_tecnicas.md`: decisiones (ADR), opciones consideradas, motivo y fecha para revisar.
- `seguridad_y_privacidad.md`: amenazas básicas, autenticación, permisos, secretos, backups y manejo/borrado de datos.

## Documentación de LXMI

- [Arquitectura conceptual](arquitectura.md)
- [Modelo de datos preliminar](modelo_de_datos.md)
- [Decisiones técnicas](decisiones_tecnicas.md)
- [Seguridad de archivos e integraciones](seguridad_archivos.md)

## Salida de fase

La solución propuesta es posible de operar con el equipo disponible; se sabe dónde viven los datos, quién puede verlos y cómo se restaura o elimina información importante.
