# 05 | Construcción

**Objetivo:** entregar cambios pequeños, entendibles y conectados con criterios acordados.

LXMI-0.1 a 0.6 están implementados en `aplicacion/`. El helper Windows se compiló con MinGW, fue staged y se ejecutó mediante Proton Experimental en el prefix aislado de LXMI; handshake, mapping de path y lectura/hash del marker pasaron, junto con una respuesta negativa estructurada. No se abrió ZZZ ni se usó su compatdata. Proton sí inicializó el prefix de prueba y actualizó su propio `dist.lock`. El prototipo visual continúa aislado en `prototipo_visual/` y no forma parte de la UI funcional.

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
