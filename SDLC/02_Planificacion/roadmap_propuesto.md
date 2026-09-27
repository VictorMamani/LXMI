# Roadmap técnico propuesto

Las versiones expresan una secuencia de aprendizaje, no compromisos de entrega ni prueba de viabilidad. Cada etapa debe revisarse tras el descubrimiento.

| Hito tentativo | Resultado propuesto | Puerta de decisión |
|---|---|---|
| 0.1 | Steam Discovery: sistema, Steam roots y bibliotecas | Crates probadas con fixtures; falta compilar/abrir Tauri y validar con Steam de desarrollo |
| 0.2 | Game + compatdata Discovery: manifests, Wuthering Waves y candidato `pfx` | Implementado y probado con fixtures; AppID verificado en Steam; sin inferir Proton activo ni compatibilidad |
| 0.3 | Proton Discovery: detectar instalaciones/versiones y asociaciones posibles | No ejecutar ni modificar prefixes; registrar layouts soportados |
| 0.4 | Preparar GameLaunchConfiguration | Solo modelar/mostrar configuración; evaluar permisos y seguridad antes de permitir lanzamiento |
| 0.5 | Integración guiada con XXMI/WWMI Runtime | Revisar upstream, licencia, anti-cheat, compatibilidad y reversión antes de tocar archivos |
| 0.6 | Biblioteca local de mods | Importación segura y preservación de originales |
| 0.7 | Instalar, activar y desactivar mods | Verificar mecanismo reversible con fixture autorizado |
| 0.8 | Perfiles por juego | Comprobar que cambiar perfil no mezcla juegos |
| 0.9 | Parser de INI compatible | Especificación, corpus de fixtures y round-trip preservando comentarios |
| 0.10 | Conflictos potenciales | Validar señal y explicar límites/falsos positivos |
| 0.11 | Generación de configuración gestionada | Originales intactos y regeneración repetible |
| 0.12 | Prueba de controlador live básico | Solo avanzar si existe mecanismo soportado |
| 0.13 | Integración con GameBanana | Revisar API, términos, atribución y seguridad de descargas |
| 0.14–0.16 | Experimento de bridge e IPC local | Revisar upstream/licencia y demostrar necesidad técnica |
| 0.17 | Evaluación de SteamOS | Definir hardware/entorno y repetir matriz de pruebas |
| 1.0 | Versión estable | Matriz de compatibilidad, recuperación y distribución definidas |

Se puede cancelar o reordenar cualquier hito. El runtime live y el fork no son requisitos de la primera versión funcional.
