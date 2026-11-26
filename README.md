# StockBox

Reescritura de aplicación de escritorio con **Tauri 2 + Rust + React/TypeScript + SQLite**.

Gestión de inventario, entradas de almacén, ventas, proformas, anulaciones, garantías y reportes para una tienda. SQLite es la única fuente de verdad; los CSV se usan solo para migración, importación y exportación.

## Cambios respecto al original

- Java 17 + JavaFX/FXML → Tauri 2 (Rust) + React/TypeScript.
- 9 archivos CSV → base de datos SQLite con esquema normalizado, llaves foráneas, índices y transacciones.
- Contraseñas en texto plano → hash Argon2id.
- Registro de actividad (auditoría) de las operaciones principales.
- Importar/exportar CSV desde la interfaz; la siembra inicial usa los CSV incluidos en `data/`.

Mejoras incorporadas desde la versión Java mejorada (`exp`):

- **Dashboard analítico** con períodos (hoy, 7 días, 30 días, todo), KPIs, ticket promedio, valor del inventario y gráficas de ventas por día/categoría, top productos y stock por categoría. El período elegido se guarda como preferencia.
- **Devoluciones parciales**: se devuelven unidades al stock, se ajusta la línea y el total de la venta; si no queda saldo, la venta y su garantía pasan a `ANULADA`. Queda registro en la tabla `devoluciones`.
- **Asistente IA** para el cajero: recomienda y compara productos usando el inventario real como contexto, sin revelar precios de costo. Soporta OpenRouter y NVIDIA Build.
- **Imágenes de producto** en `app_data_dir/imagenes/{idProducto}.png|jpg|jpeg`, con miniaturas y carga desde la interfaz. Se copian las imágenes de ejemplo incluidas en `data/imagenes`.
- **Tema claro/oscuro** persistente y **vista previa de CSV** antes de confirmar cualquier importación.

### Configuración del asistente IA

Copia `data/ia.properties.example` a `ia.properties` en el directorio de datos de la aplicación y coloca tu key, o exporta la variable de entorno:

```bash
export OPENROUTER_API_KEY=sk-or-...   # proveedor por defecto
export NVIDIA_API_KEY=nvapi-...       # con provider=nvidia
```

Si no hay key configurada, el chat muestra las instrucciones. Nunca se guarda ni muestra la key en la interfaz.

## Roles y credenciales de prueba

Los datos del proyecto original se conservan como datos de muestra. **No los reutilices en producción.**

| Rol | CI | Contraseña |
| --- | --- | --- |
| Administrador | `ADM-001` | `user` |
| Cajero | `DEMO-VEN-001` | `ventas123` |
| Almacenero | `DEMO-ALM-001` | `almacen123` |

En la pantalla de acceso puedes elegir el rol y rellenar automáticamente sus credenciales de prueba.

## Desarrollo

Requisitos: Rust 1.90+, Node 20+, GTK3 y WebKit2GTK (Linux).

```bash
npm install
npm run tauri dev
```

Comandos útiles:

```bash
cargo test -p stockbox-core -p stockbox            # pruebas de Rust del núcleo y Tauri
npm run build                                      # typecheck + build del frontend
npm run tauri build                                # instalador Tauri (.deb/.rpm/.AppImage)
cargo run -p stockbox-gpui                          # versión GPUI (sin WebView)
```

La base de datos se crea automáticamente en el directorio de datos de la aplicación. La versión Tauri usa `app_data_dir/stockbox.sqlite` y la GPUI `~/.local/share/stockbox-gpui/stockbox.sqlite`; ambas se siembran con los CSV de `data/`.

## Dos interfaces, un mismo backend

El proyecto es un workspace de Cargo. Todo el backend vive en `stockbox-core` y es reutilizado por las dos aplicaciones:

| App | Estado | Comando | Descripción |
| --- | --- | --- | --- |
| Tauri + React | Estable / recomendada | `npm run tauri dev` | Interfaz completa con todos los roles y funciones |
| GPUI (Rust puro) | Completa / sin WebView | `cargo run -p stockbox-gpui` | Misma lógica, UI acelerada por GPU con `gpui-component` |

Nota: la versión GPUI cubre login, dashboard, inventario, ventas, entradas, productos, empleados, proveedores, devoluciones, garantías y asistente IA. La selección de archivos CSV y de imágenes solo está implementada en la versión Tauri.

Los artefactos se publican en GitHub Releases con tags separados: `stockbox-tauri-vX` y `stockbox-gpui-vX`.

## Estructura

```
crates/
  stockbox-core/        backend compartido (SQLite, dominio, CSV, IA, imágenes)
    src/db/             conexión, esquema y migrador
    src/servicios/      catalogo, ventas, almacen, admin, dashboard, asistente, imagenes
    src/models.rs       tipos serializables
    src/security.rs     Argon2, IDs y fechas
    src/validacion.rs   reglas de validación
    src/csv_io.rs       lectura/escritura CSV
  stockbox-gpui/        interfaz GPUI (usa stockbox-core)
src-tauri/
  src/commands.rs       comandos Tauri (delgados, delegan en el core)
  tests/                pruebas de integración
src/                    interfaz React del frontend Tauri
data/                   CSV originales, imágenes y ejemplo de config de IA
```
