# Flujos de usuario propuestos

El primer diagrama corresponde al discovery LXMI-0.1/0.2 implementado en código y probado con datos sintéticos; todavía no se comprobó visualmente la ventana nativa ni contra Steam real. Los demás diagramas describen comportamiento futuro, no implementado. Se conserva una etapa de revisión antes de modificar archivos locales.

## Flujo de LXMI-0.1/0.2: Steam, juegos y compatdata

```mermaid
flowchart TD
    A[Abrir LXMI] --> B[Consultar información básica de Linux]
    B --> C[Mostrar sistema, arquitectura, home y directorios XDG]
    C --> D[La persona selecciona Scan Steam]
    D --> E[Buscar raíces conocidas en solo lectura]
    E --> F{¿Se encontró una instalación?}
    F -->|No| G[Mostrar Steam no encontrado]
    F -->|Sí| H[Leer y parsear libraryfolders.vdf]
    H --> I{¿Configuración válida?}
    I -->|No| J[Mostrar causa y diagnóstico]
    I -->|Sí| K[Validar y deduplicar bibliotecas]
    K --> L[Enumerar appmanifest acf en solo lectura]
    L --> M[Validar campos, AppID y directorio]
    M --> N{¿AppID de Wuthering Waves?}
    N -->|No| O[Omitir del catálogo y registrar conteo]
    N -->|Sí| P[Mostrar juego y ruta esperada]
    P --> Q[Inspeccionar compatdata y pfx sin seguir symlinks]
    Q --> R[Mostrar estados observados y límites de inferencia]
    O --> S[Mostrar bibliotecas, resumen y avisos]
    R --> S
```

## Inspeccionar una instalación

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

## Importar un archivo local

```mermaid
flowchart TD
    A[Seleccionar archivo local] --> B[Calcular hash y revisar formato]
    B --> C[Validar cada ruta y entrada]
    C --> D{¿Archivo seguro y compatible?}
    D -->|No| E[Rechazar sin escribir en biblioteca]
    D -->|Sí| F[Extraer a directorio temporal]
    F --> G[Revisar estructura y metadatos]
    G --> H[Mostrar resumen y pedir confirmación]
    H --> I[Guardar copia gestionada en biblioteca]
```

## Activar un perfil

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
