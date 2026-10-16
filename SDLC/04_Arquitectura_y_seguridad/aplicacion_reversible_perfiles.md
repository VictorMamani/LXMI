# Aplicación reversible de perfiles al runtime privado

Estado: **implementado y verificado** el 2026-10-14. La aplicación se refiere a archivos privados y selección del resolver de LXMI. La carga dentro del juego y el cambio sin recarga siguen pendientes; el experimento de prefix 0.8.8 conserva su estado y consentimiento independiente.

## Ensamblado y resolución

`assembly.rs` comparte `plan_importer_assembly`, `assemble_importer_runtime` y `existing_importer_runtime` entre WWMI y ZZMI. Los wrappers anteriores ZZMI se conservan. Ambos ensambladores verifican paquetes e inventarios; el importer y las bibliotecas oficiales se autentican antes de ofrecer adquisición. WWMI usa el ejecutable real `Client/Binaries/Win64/Client-Win64-Shipping.exe`, requiere `-dx11` y conserva la identidad upstream del loader. El bootstrap `Wuthering Waves.exe` no sustituye al payload.

`activation.rs` implementa:

| Operación | Resultado |
|---|---|
| Revisar | Revalida base, perfil guardado e inventarios; vincula plan a revisión de perfiles y revisión de selección. Devuelve ruta, tamaño y número de archivos |
| Aplicar | Crea runtime derivado completo en staging privado; comprueba inventario y promueve sin reemplazar copias previas |
| Seleccionar | Sustituye atómicamente un JSON por juego bajo bloqueo y comparación de revisión; retiene la selección anterior |
| Repetir | Una selección idéntica conserva revisión y destino de undo; una copia existente se revalida |
| Revertir/rehacer | Intercambia referencias después de comprobar revisión e integridad del destino; no elimina payloads |
| Resolver | Verifica integridad y vinculación a la misma base; usa la selección en los puntos existentes de bridge/topología/readiness |

El ID deriva de la base y perfil/inventario. Los manifiestos anteriores ZZMI se leen mediante aliases de serde, conservando los IDs y textos históricos necesarios para su comparación exacta. No se reescribe el runtime base.

## Persistencia

| Ruta relativa a LXMI | Contenido |
|---|---|
| `profile-runtimes/<game>/profile-<hash>/<WWMI o ZZMI>/` | Copia completa de runtime con Mods seleccionados |
| `profile-runtimes/<game>/profile-<hash>/runtime-manifest.json` | Manifiesto compatible con el resolver existente |
| `profile-runtimes/<game>/profile-<hash>/lxmi-profile-runtime.json` | Base, perfil e inventario completo del derivado |
| `mods/<game>/active-runtime.json` | Selección actual/anterior y revisión |

Los adaptadores `game_runtime_commands.rs` reciben juego, IDs, perfil, revisiones y plan. Derivan rutas en Rust mediante el store protegido existente; el frontend no elige DLL, PID, ejecutable ni destino de escritura.

## Interfaz y límites

**Compatibilidad** permite elegir paquetes, preparar runtime, revisar y aplicar el perfil. La UI distingue perfil guardado, copia preparada, runtime seleccionado y carga dentro del juego pendiente. «Jugar sin mods» solicita `steam -applaunch` con AppID del catálogo descubierto y `-dx11` para WuWa; no usa shell ni invoca el loader, y no se ha probado su efecto con un juego real en este incremento.

No se modifican árboles en uso: cada aplicación crea o reutiliza una copia inmutable. No hay protocolo live ni pruebas de recuperación frente a pérdida de energía. Los tests cubren planes/revisiones atrasados, binding incorrecto, integridad, aplicación idempotente y undo/redo; los límites de la biblioteca y los guards de rutas siguen vigentes. La selección no reabre el gate del loader ni consume autorizaciones del prefix.

[Verificación y evidencia](../06_Verificacion/verificacion_activacion_runtime.md).
