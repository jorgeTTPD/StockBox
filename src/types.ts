export type Rol = "ADMINISTRADOR" | "CAJERO" | "ALMACENERO";

export interface Sesion {
  ci: string;
  nombre: string;
  rol: Rol;
}

export interface Producto {
  id_producto: string;
  nombre: string;
  marca: string;
  categoria: string;
  descripcion: string;
  precio_entrada: number;
  precio_venta: number;
  stock: number;
}

export interface Proveedor {
  id_proveedor: string;
  nombre: string;
  contacto: string;
}

export interface Cliente {
  id_cliente: string;
  nombre_cliente: string;
  ci: string;
  telefono: string;
}

export interface Empleado {
  ci: string;
  nombre: string;
  rol: Rol;
}

export interface Entrada {
  id_entrada: string;
  id_proveedor: string;
  id_producto: string;
  nombre_producto: string;
  cantidad: number;
  precio_unitario: number;
  fecha_entrada: string;
  ci_empleado: string;
}

export interface ItemVenta {
  id_producto: string;
  nombre_producto: string;
  cantidad: number;
  precio_unitario: number;
  subtotal: number;
}

export interface Venta {
  id_venta: string;
  id_orden: string;
  estado: string;
  nombre_cliente: string;
  telefono_cliente: string;
  id_garantia: string;
  total: number;
  fecha_venta: string;
  ci_empleado: string;
  items: ItemVenta[];
}

export interface Pedido {
  id_pedido: string;
  id_proveedor: string;
  id_producto: string;
  cantidad: number;
  fecha_pedido: string;
  ci_empleado: string;
}

export interface Caja {
  estado: string;
  fecha_apertura: string;
  fecha_cierre: string;
  ci_empleado: string;
}

export interface ItemCarrito {
  id_producto: string;
  nombre_producto?: string;
  cantidad: number;
  precio_unitario?: number;
  subtotal?: number;
}

export interface OrdenCreada {
  id_orden: string;
  total: number;
  lineas: number;
}

export interface ResultadoVenta {
  id_orden: string;
  id_ventas: string[];
  id_garantias: string[];
  total: number;
  factura: string;
}

export interface Estadisticas {
  productos: number;
  stock_critico: number;
  ventas_hoy: number;
  entradas_hoy: number;
  garantias_vigentes: number;
}

export interface Auditoria {
  id: number;
  fecha: string;
  ci_empleado: string;
  accion: string;
  detalle: string;
}

export interface MigrationReport {
  empleados: number;
  proveedores: number;
  clientes: number;
  productos: number;
  entradas: number;
  caja: number;
  ordenes: number;
  ventas: number;
  garantias: number;
  pedidos: number;
  filas_omitidas: number;
}

export interface LineaVenta {
  id_venta: string;
  id_orden: string;
  estado: string;
  id_producto: string;
  nombre_producto: string;
  cantidad: number;
  precio_unitario: number;
  total: number;
}

export interface Devolucion {
  id_devolucion: string;
  id_venta: string;
  id_orden: string;
  id_producto: string;
  nombre_producto: string;
  cantidad: number;
  precio_unitario: number;
  total: number;
  fecha: string;
  ci_empleado: string;
}

export interface PuntoDia { fecha: string; total: number }
export interface Grupo { etiqueta: string; valor: number }

export interface DashboardResumen {
  total: number;
  cantidad_ventas: number;
  unidades: number;
  ticket_promedio: number;
  productos: number;
  stock_critico: number;
  valor_inventario: number;
}

export interface DashboardDatos {
  desde: string;
  hasta: string;
  resumen: DashboardResumen;
  ventas_por_dia: PuntoDia[];
  ventas_por_categoria: Grupo[];
  top_productos: Grupo[];
  stock_por_categoria: Grupo[];
}

export interface ConfigAsistente {
  provider: string;
  modelo: string;
  configurada: boolean;
}

export interface Preferencias {
  tema_oscuro: boolean;
  periodo_dashboard: string;
}

export interface VistaPrevia {
  columnas: string[];
  filas: string[][];
  total: number;
}

export interface ProductoInput {
  id_producto?: string | null;
  nombre: string;
  marca: string;
  categoria: string;
  descripcion: string;
  precio_entrada: number;
  precio_venta: number;
  stock: number;
}

export interface ProveedorInput {
  id_proveedor?: string | null;
  nombre: string;
  contacto: string;
}

export interface EmpleadoInput {
  ci: string;
  nombre: string;
  rol: Rol;
  contrasena: string;
}

export interface ClienteInput {
  nombre_cliente: string;
  ci: string;
  telefono: string;
}

export interface OrdenInput {
  nombre_cliente: string;
  ci_cliente: string;
  telefono_cliente: string;
  items: ItemCarrito[];
}
