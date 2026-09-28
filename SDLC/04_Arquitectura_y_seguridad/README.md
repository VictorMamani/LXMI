# 04 | Arquitectura y seguridad

**Objetivo:** decidir la estructura técnica mínima y cuidar los datos desde el comienzo.

La arquitectura mantiene decisiones conceptuales para fases futuras. La rebanada de discovery 0.1–0.3 tiene límites implementados en código: Tauri/React, `lxmi-core`, `lxmi-steam` y `lxmi-proton`, en modo de solo lectura. El análisis de amenazas para importación, instalación y ejecución todavía no está completo.

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
