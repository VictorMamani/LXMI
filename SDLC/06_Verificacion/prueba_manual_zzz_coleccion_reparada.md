# Prueba manual de ZZZ con la colección reparada

Estado: carga y funcionamiento confirmados por el usuario el 2026-10-15 (hora local). [Registro del arranque correcto](evidencias/zzz_coleccion_reparada_en_juego_20261015.json).

## Resultado confirmado

Tras liberar espacio en el disco principal, el usuario volvió a iniciar ZZZ con **ZZZ — 12 mods (4 reparados)** y confirmó: «ok ya carga» y «ya funca todo». El último arranque registró `HookLibrary` 0, ZZMI presente en el juego, `UnhookLibrary` 0 y proceso vivo durante la observación de 15 segundos. LXMI terminó el flujo en `Running` con `mods_loaded = true`.

Esta confirmación cubre el arranque y el funcionamiento reportado en este equipo. No se registró una duración medida de la partida ni un resultado individual de cada personaje o menú.

## Selección preparada

- Perfil activo: **ZZZ — 12 mods (4 reparados)**, 12 paquetes.
- Sustituciones: Remielle, Lucia Nurse, Sunna y Alice desde `~/LXMI-mods-reparados/`.
- Los otros ocho paquetes conservan sus identificadores de contenido.
- Perfil anterior conservado: **ZZZ — Todos los mods**.
- Runtime privado: 550 archivos, 1 571 355 247 bytes (aproximadamente 1,46 GiB).
- Las cuatro copias se importaron mediante la API de dominio del gestor; la selección se aplicó y resolvió con verificación de inventario.

## Comprobaciones previas

Pasaron `cargo fmt --all -- --check`, Clippy del workspace con todos los targets/features y `-D warnings`, y la compilación de la aplicación. `cargo test --offline --workspace`: 413 pruebas aprobadas, ninguna fallida y 13 ignoradas. Las pruebas ignoradas corresponden a los casos optativos del workspace; no se ejecutaron como parte de esta revisión.

El frontend pasó typecheck, lint, formato y build. Una inspección de los INI activos de las cuatro carpetas reparadas, con las utilidades del reparador, no encontró referencias `filename` ausentes ni secciones con nombre propio duplicadas. Esto es una comprobación de archivos, no una validación visual o de estabilidad del juego.

## Pasos en el juego

1. Con ZZZ cerrado, abrir LXMI y seleccionar Zenless Zone Zero. Comprobar **ZZZ — 12 mods (4 reparados)** y **12 mods**.
2. Pulsar **Jugar con mods** y aceptar el aviso si aparece.
3. Esperar el mensaje **Listo: pulsa Jugar en HoYoPlay**. En ese momento pulsar **Jugar** en HoYoPlay.
4. Comprobar **Mods cargados** y **En juego** en LXMI. Registrar cualquier error exacto si el inicio falla.
5. Comprobar Calidad de personajes **Alta** y Animación de personajes de alta precisión **Desactivada**.
6. Inspeccionar Remielle, Lucia, Sunna y Alice donde estén disponibles; comprobar modelos, ojos, prendas y geometría. Probar `/` en los menús de Remielle y Lucia. Los sustitutos de Remielle no recuperan los adornos originales del autor.
7. Jugar al menos diez minutos y cambiar entre personajes o zonas. Registrar avisos rojos, recursos invisibles, errores de menú o cierres. Una prueba breve satisfactoria no garantiza estabilidad en todas las situaciones.

## Criterio para el commit solicitado

El usuario confirmó el resultado correcto y solicitó explícitamente hacer el commit y subirlo al repositorio. Las comprobaciones de código y la prueba de carga ya están registradas. Los mods locales y sus texturas permanecen fuera del repositorio.
