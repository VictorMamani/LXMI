# Product brief — LXMI

## Resumen

Aplicación de escritorio Linux propuesta para detectar instalaciones de juegos y runtimes de modding, organizar mods locales y, si la investigación confirma viabilidad, administrar configuración compatible con Wine/Proton.

## Personas y necesidad hipotéticas

**Persona inicial por validar:** jugador de Linux que instala manualmente runtimes y mods y necesita entender qué se detectó, dónde se instalará y cómo volver atrás.

**Necesidad hipotética:** reducir errores de configuración y dar una biblioteca reversible para mods. No hay entrevistas ni observaciones realizadas; la necesidad no es un hallazgo.

## Propuesta de valor a probar

- Mostrar detecciones y rutas con claridad antes de escribir archivos.
- Mantener los archivos originales de los mods sin cambios.
- Hacer explícitos runtime, prefix y configuración seleccionados.
- Permitir volver al estado previo tras una operación fallida.

## Alcance de plataforma inicial propuesto

Ubuntu como primer entorno de desarrollo. El soporte de juegos y runtimes se decidirá a partir de documentación upstream y pruebas autorizadas. Wuthering Waves + WWMI aparece como candidato inicial en la propuesta del usuario, no como compatibilidad confirmada. Zenless Zone Zero y ZZMI quedan para una fase posterior.

## Incrementos iniciales de implementación

LXMI-0.1 muestra sistema operativo, arquitectura, home y directorios XDG; permite buscar Steam en ubicaciones Linux comunes, leer `steamapps/libraryfolders.vdf` y listar bibliotecas existentes. LXMI-0.2 lee `appmanifest_*.acf`, reconoce Wuthering Waves por el AppID `3513350` verificado en Steam y comprueba si existe `compatdata/<AppID>/pfx`. Ambos flujos son de solo lectura. Detectar `pfx` no demuestra qué runtime se usa, su salud ni compatibilidad del juego.

## Éxito de la exploración

1. Fuentes oficiales, licencias y límites identificados.
2. Prueba técnica aislada de detección, con versiones y rutas documentadas.
3. Formato de mod de prueba autorizado definido.
4. Decisión explícita de continuar, reducir alcance o descartar el componente live.

No se fijan usuarios, ingresos ni metas de ventas en esta etapa.
