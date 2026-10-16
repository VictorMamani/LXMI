# ADR-031 — Biblioteca local y perfiles independientes del loader

Estado: implementado, 2026-10-14.

## Contexto

El usuario priorizó el gestor de mods para WuWa y ZZZ. La preparación de paquetes y los experimentos de contexto Proton ya existen; no hay evidencia suficiente para ofrecer activación en el juego o cambios live. La biblioteca y sus selecciones pueden verificarse sin ejecutar ese contexto.

## Decisión

Extender `lxmi-xxmi` con `ModLibrary`, reutilizando `Directory`, inventario, fingerprint y extracción ZIP existentes. Los comandos Tauri derivan el store mediante `xxmi_service::store_for`, incluidas sus comprobaciones de solapamiento con Steam/juegos/prefixes. No se aceptan roots de escritura desde IPC.

Los mods se asignan explícitamente a `wuthering-waves` o `zenless-zone-zero`; sus paquetes y perfiles viven en árboles distintos. Las copias importadas se promueven sin reemplazo y se verifican antes de listar, guardar o preparar. Un duplicado conserva su identidad y nombre inicial. Los perfiles guardan nombre e IDs del mismo árbol de juego.

Las mutaciones usan `flock` no bloqueante sobre un archivo privado. Las ediciones incluyen la revisión observada; un escritor atrasado recibe `Busy`. El estado se escribe en un archivo nuevo y se sustituye por rename, sin ediciones parciales visibles.

Preparar produce un árbol `Mods` completo por contenido y selección, sin reemplazar el runtime existente ni tocar Steam, juego o prefix. Cada mod ocupa una carpeta por hash, evitando colisiones de nombres de carpeta; esto no resuelve conflictos entre namespaces u overrides INI. Preparaciones alteradas bloquean su reutilización y se conservan para revisión.

## Consecuencias

No se añade una base de datos, dependencia o formato INI propio. Las selecciones se recuperan y cambian sin modificar payloads. Los snapshots consumen espacio y no se eliminan automáticamente. La copia preparada no se anuncia como perfil activo en el juego. La aplicación/reversión en el runtime y el mecanismo live requieren decisiones y verificación posteriores.

El historial y los consentimientos de 0.8.8 permanecen independientes; no se cambia la versión ni se ejecutan helpers por esta función.
