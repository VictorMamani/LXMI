# Fuentes e hipótesis

## Origen de la idea

La descripción inicial de LXMI y sus componentes fue aportada por el usuario en la conversación. Este documento organiza esas ideas; no equivale a investigación independiente.

## Hechos comprobados en este repositorio

- Se ha creado una carpeta de documentación para el Proyecto 19.
- La implementación de LXMI-0.1/0.2 cubre información del sistema, Steam Libraries, lectura de manifests, registro de Wuthering Waves y observación pasiva de compatdata/pfx. No valida ejecución, compatibilidad de juegos ni runtimes.
- No se recibieron enlaces upstream, código, documentación de compatibilidad ni licencias de XXMI/runtimes. La fuente oficial consultada para el AppID se registra por separado; no valida compatibilidad.

## Fuente verificada para LXMI-0.2

| Fecha | Fuente primaria | Dato comprobado | Límite |
|---|---|---|---|
| 2026-09-27 | [Ficha oficial de Wuthering Waves en Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/) | La URL de la ficha usa AppID `3513350` y el producto se identifica como Wuthering Waves. La ficha también declara uso de Anti-Cheat Expert a nivel de kernel. | Confirma identidad/AppID de Steam, no instalación local, compatibilidad con Linux/Proton, seguridad de mods ni funcionamiento de LXMI. La mención anti-cheat exige revisión antes de cualquier integración de runtime o mods; no se intentará eludir controles. |

## Hipótesis técnicas por validar

| ID | Hipótesis | Evidencia necesaria | Estado |
|---|---|---|---|
| H-01 | Existe un recorrido viable para administrar una versión compatible de XXMI/WWMI mediante Wine o Proton en Ubuntu | Documentación upstream vigente y prueba aislada con versiones registradas | Pendiente |
| H-02 | Se puede detectar de forma segura Steam, una instalación de juego y su prefix | Fuentes oficiales/formato de configuración y prueba con fixture; después, entorno de desarrollo autorizado | Pendiente |
| H-03 | Los mods objetivo pueden instalarse y activarse sin editar sus originales | Estructuras de mods autorizadas y prueba reproducible de instalación reversible | Pendiente |
| H-04 | Un cambio live es posible sin reiniciar el juego en los runtimes objetivo | Documentación o experimento autorizado, con condiciones y límites | Pendiente |
| H-05 | Los usuarios objetivo necesitan una interfaz nativa para estas tareas | Entrevistas o evidencia pública verificable; no se presume demanda comercial | Pendiente |

## Preguntas de investigación

- ¿Qué repositorio y documentación son upstream oficiales de XXMI, WWMI y ZZMI?
- ¿Qué licencias aplican a cada componente y a sus dependencias? ¿Qué obligaciones tendría una distribución o fork?
- ¿Qué versiones, juegos, plataformas y launchers están admitidos actualmente por cada runtime?
- ¿Qué limitaciones de Wine/Proton, Steam, actualizadores del juego o medidas anti-cheat afectan a la integración?
- ¿Qué formato de instalación y metadatos emplean los mods autorizados?
- ¿Hay mecanismos documentados de recarga o variables live? ¿Qué no está soportado?
- ¿Qué distribución Linux y GPU están disponibles para pruebas reproducibles?

## Regla de evidencia

Separar en futuras actualizaciones **fuente primaria**, **observación reproducida**, **inferencia** e **hipótesis**. No afirmar que un juego funciona ni que una función live es posible hasta registrar entorno, versiones, pasos y resultado. No eludir anti-cheat ni restricciones del juego.
