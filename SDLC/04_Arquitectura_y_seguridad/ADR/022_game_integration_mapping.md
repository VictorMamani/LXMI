# ADR-022: Mapping explícito entre juego e integración

**Estado:** Aceptado para LXMI 0.5.1
**Fecha:** 2026-09-28

## Contexto

Una validación estructural de paquete solo tiene sentido en el juego al que apunta. Asociar una carpeta por su nombre o aceptar cualquier `IntegrationKind` permitiría producir planes cruzados, como aplicar conceptualmente ZZMI a Wuthering Waves.

## Decisión

LXMI mantiene el juego detectado y la integración como tipos separados, con relaciones explícitas en el registro:

| Juego | Integración |
|---|---|
| Wuthering Waves | WWMI |
| Zenless Zone Zero | ZZMI |

La validación y planificación rechazan asociaciones distintas. La distribución observada, compatibilidad Steam y compatibilidad Linux/Proton son propiedades independientes y no se deducen de esta relación.

## Consecuencias

- Se conserva la extensibilidad para otras integraciones sin acoplar el scanner Steam al paquete.
- El soporte nominal del juego no significa que el paquete sea auténtico ni ejecutable.
- Steam y Linux/Proton se mantienen `Unverified` hasta contar con evidencia específica.
- No se modifica el juego ni se ejecuta el paquete en 0.5.1.

## Evidencia

El mapping ZZZ/ZZMI y los nombres de ejecutable se contrastaron con [la configuración ZZMI del XXMI Launcher](https://github.com/SpectrumQT/XXMI-Launcher/blob/main/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py). El contrato se revisó en la rama `main` el 2026-09-28 y no está fijado a un commit.
