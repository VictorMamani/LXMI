# 05 | Construcción

**Objetivo:** entregar cambios pequeños, entendibles y conectados con criterios acordados.

LXMI-0.1 a 0.7 están implementados en `aplicacion/`. El bridge 0.6 pasó handshake bajo Proton aislado. En 0.7, el export Direct Inject de `3dmloader.dll` cargó la DLL inocua de LXMI dentro del único test target propio; baseline y casos negativos también respondieron como se esperaba. La ventana Tauri recorrió los casos baseline, positivo y target ausente. No se abrió ZZZ ni se usó su compatdata. El prototipo visual continúa aislado en `prototipo_visual/` y no forma parte de la UI funcional.

## Documentos sugeridos

- `guia_de_desarrollo.md`: cómo instalar, ejecutar y configurar en local.
- `estructura_del_codigo.md`: módulos y convenciones que use el proyecto.
- `registro_de_cambios.md`: decisiones y cambios visibles por versión.
- `trabajo_pendiente.md`: tareas pequeñas, responsable y bloqueo.

## Documentación de LXMI

- [Estructura propuesta](estructura_del_codigo.md)
- [Guía de desarrollo inicial](guia_de_desarrollo.md)
- [Registro de cambios](registro_de_cambios.md)
- [Trabajo pendiente](trabajo_pendiente.md)
- [Aplicación](aplicacion/README.md), [prototipo local](prototipo_local/README.md) y [prototipo visual](prototipo_visual/README.md)

## Salida de fase

El flujo prioritario funciona en un entorno de desarrollo, no hay secretos dentro del repositorio y otra persona puede ejecutar el proyecto con las instrucciones disponibles.
