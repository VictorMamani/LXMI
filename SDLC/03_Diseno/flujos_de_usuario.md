# Flujos de usuario propuestos

El primer diagrama corresponde al discovery y runtime planning de LXMI-0.1 a 0.4 implementados; pasó fixtures y se ejecutó contra Steam local en solo lectura. El flujo de importación de carpetas y revisión de plan corresponde al código de LXMI-0.5, aunque la interacción completa por IPC en ventana nativa queda NO COMPROBADA. No hay extracción de archives ni apply al juego. Los flujos de selección manual de instalación y activación de perfiles siguen siendo futuros.

## Flujo de LXMI-0.1 a 0.4: Steam, juegos, compatdata, tools y plan

```mermaid
flowchart TD
    A[Abrir LXMI] --> B[Consultar información básica de Linux]
    B --> C[Mostrar sistema, arquitectura, home y directorios XDG]
    C --> D[La persona selecciona Scan Steam]
    D --> E[Buscar raíces conocidas en solo lectura]
    E --> F{¿Se encontró una instalación?}
    F -->|No| G[Marcar discovery de juego y tools como no disponible]
    F -->|Sí| H[Leer y parsear libraryfolders.vdf]
    H --> I{¿Configuración válida?}
    I -->|No| J[Guardar el estado de discovery y su diagnóstico]
    I -->|Sí| K[Validar y deduplicar bibliotecas]
    K --> L[Enumerar appmanifest acf en solo lectura]
    L --> M[Validar campos, AppID y directorio]
    M --> N{¿AppID de Wuthering Waves?}
    N -->|No| O[Marcar juego como no encontrado si el scan fue completo]
    N -->|Sí| P[Registrar manifest y estado del directorio]
    P --> Q[Inspeccionar compatdata y pfx sin seguir symlinks]
    O --> R[Explorar compatibility tools y metadata en solo lectura]
    Q --> R
    G --> S[Construir plan declarativo para juegos soportados]
    J --> S
    R --> T[Combinar instalación, compatdata y Proton candidates]
    T --> S
    S --> U[Conservar selección como desconocida sin evidencia por juego]
    U --> V[Evaluar readiness y registrar evidencia]
    V --> W[Mostrar resumen, detalles y avisos]
```

## Inspeccionar y guardar una instalación (flujo futuro)

```mermaid
flowchart TD
    A[Abrir LXMI] --> B[Elegir ubicación Steam]
    B --> C[Inspección de solo lectura]
    C --> D{¿Instalación reconocible?}
    D -->|Sí| E[Mostrar juego, rutas y datos detectados]
    D -->|No| F[Explicar qué falta y permitir ruta manual]
    E --> G[Confirmar o corregir datos]
    F --> G
    G --> H[Guardar selección local]
```

## Importar una carpeta runtime local y revisar plan (LXMI-0.5)

```mermaid
flowchart TD
    A[Elegir carpeta local WWMI o XXMI Libraries] --> B[Inspeccionar archivos sin seguir symlinks]
    B --> C[Validar layout soportado y límites]
    C --> D{¿Estructura válida?}
    D -->|No| E[Rechazar sin publicar paquete]
    D -->|Sí| F[Copiar a staging privado de LXMI]
    F --> G[Revalidar inventario y SHA-256]
    G --> H[Promover a packages/xxmi sin overwrite]
    H --> I[Usuario solicita revisar plan]
    I --> J[Generar plan declarativo sin aplicar cambios]
    J --> K[Mostrar destinos administrados y requisitos pendientes]
```

## Activar un perfil (flujo futuro)

```mermaid
flowchart TD
    A[Elegir juego y perfil] --> B[Validar rutas y runtime]
    B --> C{¿Contexto válido?}
    C -->|No| D[No modificar archivos y mostrar diagnóstico]
    C -->|Sí| E[Preparar cambios reversibles]
    E --> F[Aplicar configuración gestionada]
    F --> G{¿Operación completa?}
    G -->|No| H[Restaurar el estado previo]
    G -->|Sí| I[Mostrar mods activos y resultado]
```
