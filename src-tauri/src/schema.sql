CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS empleados (
    ci TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    rol TEXT NOT NULL CHECK (rol IN ('ADMINISTRADOR', 'CAJERO', 'ALMACENERO')),
    contrasena_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS proveedores (
    id_proveedor TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    contacto TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS clientes (
    id_cliente TEXT PRIMARY KEY,
    nombre_cliente TEXT NOT NULL,
    ci TEXT NOT NULL DEFAULT '',
    telefono TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS productos (
    id_producto TEXT PRIMARY KEY COLLATE NOCASE,
    nombre TEXT NOT NULL,
    marca TEXT NOT NULL,
    categoria TEXT NOT NULL,
    descripcion TEXT NOT NULL,
    precio_entrada REAL NOT NULL CHECK (precio_entrada > 0),
    precio_venta REAL NOT NULL CHECK (precio_venta > 0),
    stock INTEGER NOT NULL CHECK (stock >= 0)
);

CREATE INDEX IF NOT EXISTS idx_productos_categoria ON productos(categoria);

CREATE TABLE IF NOT EXISTS entradas (
    id_entrada TEXT PRIMARY KEY,
    id_proveedor TEXT NOT NULL DEFAULT '',
    id_producto TEXT NOT NULL,
    nombre_producto TEXT NOT NULL,
    cantidad INTEGER NOT NULL CHECK (cantidad > 0),
    precio_unitario REAL NOT NULL CHECK (precio_unitario > 0),
    fecha_entrada TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_entradas_fecha ON entradas(fecha_entrada);

CREATE TABLE IF NOT EXISTS caja (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    estado TEXT NOT NULL,
    fecha_apertura TEXT NOT NULL DEFAULT '',
    fecha_cierre TEXT NOT NULL DEFAULT '',
    ci_empleado TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS ordenes (
    id_orden TEXT PRIMARY KEY,
    estado TEXT NOT NULL,
    id_cliente TEXT NOT NULL DEFAULT '',
    nombre_cliente TEXT NOT NULL DEFAULT '',
    ci_cliente TEXT NOT NULL DEFAULT '',
    telefono_cliente TEXT NOT NULL DEFAULT '',
    fecha TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS ventas (
    id_venta TEXT PRIMARY KEY,
    id_orden TEXT NOT NULL,
    estado TEXT NOT NULL,
    id_cliente TEXT NOT NULL DEFAULT '',
    nombre_cliente TEXT NOT NULL DEFAULT '',
    ci_cliente TEXT NOT NULL DEFAULT '',
    telefono_cliente TEXT NOT NULL DEFAULT '',
    id_garantia TEXT NOT NULL DEFAULT '',
    total REAL NOT NULL,
    fecha_venta TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (id_orden) REFERENCES ordenes(id_orden)
);

CREATE INDEX IF NOT EXISTS idx_ventas_estado_fecha ON ventas(estado, fecha_venta);

CREATE TABLE IF NOT EXISTS venta_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    id_venta TEXT NOT NULL,
    id_producto TEXT NOT NULL,
    nombre_producto TEXT NOT NULL,
    cantidad INTEGER NOT NULL,
    precio_unitario REAL NOT NULL,
    subtotal REAL NOT NULL,
    FOREIGN KEY (id_venta) REFERENCES ventas(id_venta) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_venta_items_venta ON venta_items(id_venta);

CREATE TABLE IF NOT EXISTS garantias (
    id_garantia TEXT PRIMARY KEY,
    id_venta TEXT NOT NULL DEFAULT '',
    id_orden TEXT NOT NULL DEFAULT '',
    id_cliente TEXT NOT NULL DEFAULT '',
    ci_cliente TEXT NOT NULL DEFAULT '',
    nombre_cliente TEXT NOT NULL DEFAULT '',
    telefono_cliente TEXT NOT NULL DEFAULT '',
    id_producto TEXT NOT NULL,
    nombre_producto TEXT NOT NULL,
    fecha_inicio TEXT NOT NULL,
    fecha_fin TEXT NOT NULL,
    estado TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS pedidos (
    id_pedido TEXT NOT NULL,
    id_proveedor TEXT NOT NULL,
    id_producto TEXT NOT NULL,
    cantidad INTEGER NOT NULL CHECK (cantidad > 0),
    fecha_pedido TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (id_pedido, id_producto)
);

CREATE TABLE IF NOT EXISTS devoluciones (
    id_devolucion TEXT PRIMARY KEY,
    id_venta TEXT NOT NULL,
    id_orden TEXT NOT NULL DEFAULT '',
    id_producto TEXT NOT NULL,
    nombre_producto TEXT NOT NULL,
    cantidad INTEGER NOT NULL CHECK (cantidad > 0),
    precio_unitario REAL NOT NULL,
    total REAL NOT NULL,
    fecha TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_devoluciones_venta ON devoluciones(id_venta);

CREATE TABLE IF NOT EXISTS preferencias (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO preferencias(key, value) VALUES('tema_oscuro', 'true');
INSERT OR IGNORE INTO preferencias(key, value) VALUES('periodo_dashboard', 'ULTIMOS_7');

CREATE TABLE IF NOT EXISTS auditoria (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha TEXT NOT NULL,
    ci_empleado TEXT NOT NULL DEFAULT '',
    accion TEXT NOT NULL,
    detalle TEXT NOT NULL DEFAULT ''
);
