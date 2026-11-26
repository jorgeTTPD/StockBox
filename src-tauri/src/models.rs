use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Producto {
    pub id_producto: String,
    pub nombre: String,
    pub marca: String,
    pub categoria: String,
    pub descripcion: String,
    pub precio_entrada: f64,
    pub precio_venta: f64,
    pub stock: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Empleado {
    pub ci: String,
    pub nombre: String,
    pub rol: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proveedor {
    pub id_proveedor: String,
    pub nombre: String,
    pub contacto: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cliente {
    pub id_cliente: String,
    pub nombre_cliente: String,
    pub ci: String,
    pub telefono: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrada {
    pub id_entrada: String,
    pub id_proveedor: String,
    pub id_producto: String,
    pub nombre_producto: String,
    pub cantidad: i64,
    pub precio_unitario: f64,
    pub fecha_entrada: String,
    pub ci_empleado: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemVenta {
    pub id_producto: String,
    pub nombre_producto: String,
    pub cantidad: i64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Venta {
    pub id_venta: String,
    pub id_orden: String,
    pub estado: String,
    pub nombre_cliente: String,
    pub telefono_cliente: String,
    pub id_garantia: String,
    pub total: f64,
    pub fecha_venta: String,
    pub ci_empleado: String,
    pub items: Vec<ItemVenta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Garantia {
    pub id_garantia: String,
    pub id_venta: String,
    pub id_orden: String,
    pub nombre_cliente: String,
    pub telefono_cliente: String,
    pub id_producto: String,
    pub nombre_producto: String,
    pub fecha_inicio: String,
    pub fecha_fin: String,
    pub estado: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pedido {
    pub id_pedido: String,
    pub id_proveedor: String,
    pub id_producto: String,
    pub cantidad: i64,
    pub fecha_pedido: String,
    pub ci_empleado: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caja {
    pub estado: String,
    pub fecha_apertura: String,
    pub fecha_cierre: String,
    pub ci_empleado: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sesion {
    pub ci: String,
    pub nombre: String,
    pub rol: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ItemCarrito {
    pub id_producto: String,
    #[serde(default)]
    pub nombre_producto: String,
    pub cantidad: i64,
    #[serde(default)]
    pub precio_unitario: f64,
    #[serde(default)]
    pub subtotal: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProductoInput {
    pub id_producto: Option<String>,
    pub nombre: String,
    pub marca: String,
    pub categoria: String,
    pub descripcion: String,
    pub precio_entrada: f64,
    pub precio_venta: f64,
    pub stock: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProveedorInput {
    pub id_proveedor: Option<String>,
    pub nombre: String,
    pub contacto: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClienteInput {
    pub nombre_cliente: String,
    pub ci: String,
    pub telefono: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmpleadoInput {
    pub ci: String,
    pub nombre: String,
    pub rol: String,
    pub contrasena: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrdenInput {
    pub nombre_cliente: String,
    pub ci_cliente: String,
    pub telefono_cliente: String,
    pub items: Vec<ItemCarrito>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrdenCreada {
    pub id_orden: String,
    pub total: f64,
    pub lineas: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResultadoVenta {
    pub id_orden: String,
    pub id_ventas: Vec<String>,
    pub id_garantias: Vec<String>,
    pub total: f64,
    pub factura: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Devolucion {
    pub id_devolucion: String,
    pub id_venta: String,
    pub id_orden: String,
    pub id_producto: String,
    pub nombre_producto: String,
    pub cantidad: i64,
    pub precio_unitario: f64,
    pub total: f64,
    pub fecha: String,
    pub ci_empleado: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LineaVenta {
    pub id_venta: String,
    pub id_orden: String,
    pub estado: String,
    pub id_producto: String,
    pub nombre_producto: String,
    pub cantidad: i64,
    pub precio_unitario: f64,
    pub total: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DashboardResumen {
    pub total: f64,
    pub cantidad_ventas: i64,
    pub unidades: i64,
    pub ticket_promedio: f64,
    pub productos: i64,
    pub stock_critico: i64,
    pub valor_inventario: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PuntoDia {
    pub fecha: String,
    pub total: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Grupo {
    pub etiqueta: String,
    pub valor: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DashboardDatos {
    pub desde: String,
    pub hasta: String,
    pub resumen: DashboardResumen,
    pub ventas_por_dia: Vec<PuntoDia>,
    pub ventas_por_categoria: Vec<Grupo>,
    pub top_productos: Vec<Grupo>,
    pub stock_por_categoria: Vec<Grupo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConfigAsistente {
    pub provider: String,
    pub modelo: String,
    pub configurada: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preferencias {
    pub tema_oscuro: bool,
    pub periodo_dashboard: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VistaPrevia {
    pub columnas: Vec<String>,
    pub filas: Vec<Vec<String>>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Estadisticas {
    pub productos: i64,
    pub stock_critico: i64,
    pub ventas_hoy: f64,
    pub entradas_hoy: i64,
    pub garantias_vigentes: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auditoria {
    pub id: i64,
    pub fecha: String,
    pub ci_empleado: String,
    pub accion: String,
    pub detalle: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResumenDia {
    pub fecha: String,
    pub total_ventas: f64,
    pub cantidad_ventas: i64,
    pub unidades_entradas: i64,
    pub registros_entradas: i64,
    pub stock_critico: i64,
}
