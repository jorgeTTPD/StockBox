# StockBox

Reescritura de aplicación de escritorio con **Tauri 2 + Rust + React/TypeScript + SQLite**.

Gestión de inventario, entradas de almacén, ventas, proformas, anulaciones, garantías y reportes para una tienda. SQLite es la única fuente de verdad; los CSV se usan solo para migración, importación y exportación.

## Cambios respecto al original

- Java 17 + JavaFX/FXML → Tauri 2 (Rust) + React/TypeScript.
- 9 archivos CSV → base de datos SQLite con esquema normalizado, llaves foráneas, índices y transacciones.
- Contraseñas en texto plano → hash Argon2id.
- Registro de actividad (auditoría) de las operaciones principales.
- Importar/exportar CSV desde la interfaz; la siembra inicial usa los CSV incluidos en `data/`.

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
cargo test --manifest-path src-tauri/Cargo.toml   # pruebas de Rust
npm run build                                     # typecheck + build del frontend
npm run tauri build                               # instalador/binario de producción
```

La base de datos se crea automáticamente en el directorio de datos de la aplicación (`app_data_dir/stockbox.sqlite`) y se siembra con los CSV de `data/` la primera vez.

## Estructura

```
src/                    interfaz React (login, vistas por rol, componentes)
src-tauri/src/
  commands.rs           comandos Tauri y control de sesión/roles
  db/                   conexión, esquema y migrador de CSV
  servicios/            catalogo, ventas, almacen, admin
  models.rs             tipos serializables compartidos
  security.rs           Argon2, IDs y fechas
  validacion.rs         reglas de validación del dominio
  csv_io.rs             lectura/escritura CSV
data/                   CSV originales (siembra y ejemplos)
src-tauri/tests/        pruebas de integración de Rust
```

## Licencia

Proyecto derivado de Tech-Flow. Sin licencia declarada en el repositorio original.
