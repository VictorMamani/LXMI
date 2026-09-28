# ADR-026 — LXMI Windows Bridge Protocol

- Estado: aceptada para prototipo de conectividad
- Fecha: 2026-09-28
- Alcance: LXMI 0.6

## Contexto

LXMI es un control plane Linux nativo, mientras que el runtime de XXMI que se estudia es Windows. Antes de investigar un loader, necesitamos comprobar de forma acotada que un proceso Windows inocuo puede iniciarse mediante Proton, recibir el entorno esperado y leer un marker bajo el almacenamiento administrado de LXMI.

Ejecutar Proton puede inicializar/actualizar CompatData y mantener la distribución Proton. LXMI no debe usar el prefix de ZZZ ni arrancar un test sin que una persona acepte esos efectos.

## Decisión

1. Añadir una crate lxmi-bridge y una crate compartida lxmi-bridge-protocol.
2. Construir un helper Windows propiedad de LXMI, sin código de injector ni dependencias de proceso/inyección.
3. Usar stdin/stdout con JSON y protocolo versión 1 para la operación única InspectManagedRuntime.
4. Correlacionar cada respuesta con un nonce aleatorio de 128 bits. El nonce no se considera autenticación.
5. Invocar el script Proton seleccionado explícitamente como argv: `proton runinprefix helper.exe`. En el Proton local, `run` inicia `steam.exe`; `runinprefix` ejecuta directamente Wine con el helper. No usar `sh -c` ni aceptar una cadena de comando desde la UI.
6. Revalidar en Rust la herramienta Proton seleccionada y el runtime administrado. El frontend solo envía IDs de paquetes y path de un candidato; el backend los vuelve a resolver.
7. Restringir helper y runtime bajo el root administrado de LXMI. El helper lee solo el runtime-manifest.json; no recibe rutas arbitrarias.
8. Usar un test-prefix creado bajo storage LXMI y rechazar el modo de ejecución con compatdata de juego.
9. Limitar ejecución a 20 segundos y stdout/stderr a 1 MiB cada uno; terminar el grupo de procesos propio si excede el timeout.
10. Limpiar el entorno heredado y pasar solo un conjunto controlado de variables. No registrar volcados completos del entorno.

## Diagrama

~~~mermaid
sequenceDiagram
    participant UI as Tauri Advanced UI
    participant App as Rust bridge service
    participant Proton as Explicit Proton tool
    participant Helper as LXMI Windows helper
    participant Data as LXMI managed runtime

    UI->>App: package IDs + explicit Proton candidate
    App->>App: re-scan tools, verify packages/runtime/helper
    App->>Proton: argv proton runinprefix helper.exe + protocol JSON
    Proton->>Helper: execute in LXMI isolated CompatData
    Helper->>Data: read runtime-manifest.json only
    Data-->>Helper: marker bytes
    Helper-->>Proton: JSON response + nonce + hashes
    Proton-->>App: bounded stdout/stderr + exit code
    App->>App: validate version, nonce, hashes and status
    App-->>UI: structured bridge result
~~~

## Límites de confianza

- La UI no decide la ruta del helper, root Steam, CompatData ni runtime root.
- El backend revalida el Proton candidato contra el discovery actual y asocia su Steam root.
- El helper Windows tiene capacidad funcional para leer únicamente el marker relativo a su almacenamiento; no enumera procesos ni invoca APIs de inyección.
- El nonce evita atribuir una respuesta vieja a la petición actual, pero no protege frente a un Proton/helper comprometido.
- SHA-256 comprueba identidad local del helper y coincidencia del manifest; no acredita firma upstream ni protege si helper y sidecar se reemplazan juntos.
- El test puede escribir únicamente bajo el contexto aislado LXMI. El script Proton puede también realizar mantenimiento en su propia instalación; la UI requiere confirmación explícita.

## Alternativas

| Alternativa | Decisión |
|---|---|
| Archivo temporal | Rechazada para el prototipo: deja estado temporal y complica ownership/limpieza. |
| Socket localhost | Rechazada: no se necesita servicio persistente; agrega puertos, firewall y lifecycle. |
| Stdin/stdout JSON | Elegida para una llamada finita; límites de entrada/salida y timeout son verificables. |
| Ejecutar XXMI Launcher portable | No evaluado en ejecución. Tiene UI/operación de juego y no es necesario para este handshake. |
| Reutilizar 3dmloader o escribir injector | Fuera de alcance y prohibido en 0.6. |

## Consecuencias

- No existe daemon ni IPC persistente; una futura comunicación live requerirá otro diseño.
- El helper se construye para `x86_64-pc-windows-gnu` y se stagea en storage LXMI antes de habilitar el host test.
- El test ejecutable permanece opt-in y usa un Proton distinto de la selección real/unknown de ZZZ.
- El host handshake se comprobó con Proton Experimental y prefix aislado; esta evidencia valida conectividad del helper/marker, no compatibilidad ZZMI ni ejecución del juego.

## Evidencia upstream

La invocación toma como referencia el código estático de Valve Proton tag proton-11.0-2, commit corto db9e6ff; README y script se registran en SDLC/01_Descubrimiento/bridge_linux_windows.md. El código del script también muestra operaciones que pueden borrar dist legacy, aplicar fixups y crear/actualizar pfx, razón por la que el test-prefix de LXMI es obligatorio.

## Verificación

- **FIXTURE:** serialización, protocolo incompatible, nonce, hash, confinamiento de path, contexto de prefix, argv directo, timeout y output cap.
- **COMPROBADO / HOST TEST:** helper PE Windows construido y staged; handshake por Proton v1, ruta Windows legible, hash del marker coincidente y error negativo estructurado.
- **NO COMPROBADO:** cargar XXMI/DLL, acceder o lanzar ZZZ, o compatibilidad Steam/Linux/Proton.
