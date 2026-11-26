import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  Auditoria,
  Caja,
  Cliente,
  ClienteInput,
  Empleado,
  EmpleadoInput,
  Entrada,
  Estadisticas,
  ItemCarrito,
  MigrationReport,
  OrdenCreada,
  OrdenInput,
  Pedido,
  Producto,
  ProductoInput,
  Proveedor,
  ProveedorInput,
  ResultadoVenta,
  Rol,
  Sesion,
  Venta,
} from "./types";

export const api = {
  iniciarSesion: (ci: string, contrasena: string, rol: Rol) =>
    invoke<Sesion>("iniciar_sesion", { ci, contrasena, rol }),
  cerrarSesion: () => invoke<void>("cerrar_sesion", {}),
  sesionActual: () => invoke<Sesion | null>("sesion_actual", {}),

  listarProductos: (termino?: string) =>
    invoke<Producto[]>("listar_productos", { termino: termino || null }),
  listarCategorias: () => invoke<string[]>("listar_categorias", {}),
  productosCriticos: () => invoke<Producto[]>("productos_criticos", {}),
  crearProducto: (entrada: ProductoInput) =>
    invoke<string>("crear_producto", { entrada }),
  eliminarProducto: (id: string) => invoke<void>("eliminar_producto", { id }),

  listarProveedores: () => invoke<Proveedor[]>("listar_proveedores", {}),
  crearProveedor: (entrada: ProveedorInput) =>
    invoke<string>("crear_proveedor", { entrada }),
  eliminarProveedor: (id: string) => invoke<void>("eliminar_proveedor", { id }),

  listarClientes: () => invoke<Cliente[]>("listar_clientes", {}),
  crearCliente: (entrada: ClienteInput) =>
    invoke<string>("crear_cliente", { entrada }),

  crearOrden: (entrada: OrdenInput) => invoke<OrdenCreada>("crear_orden", { entrada }),
  confirmarOrden: (id_orden: string) =>
    invoke<ResultadoVenta>("confirmar_orden", { id_orden }),
  anularVenta: (referencia: string) =>
    invoke<void>("anular_venta", { referencia }),
  listarVentas: (estado?: string, fecha?: string) =>
    invoke<Venta[]>("listar_ventas", { estado: estado || null, fecha: fecha || null }),
  consultarGarantia: (referencia: string) =>
    invoke<string[]>("consultar_garantia", { referencia }),
  importarOrdenes: (ruta: string) => invoke<number>("importar_ordenes", { ruta }),
  facturaOrden: (id_orden: string) => invoke<string>("factura_orden", { id_orden }),

  listarEntradas: (fecha?: string) =>
    invoke<Entrada[]>("listar_entradas", { fecha: fecha || null }),
  importarEntradas: (ruta: string) => invoke<number>("importar_entradas", { ruta }),
  registrarEntrada: (
    id_producto: string,
    cantidad: number,
    precio: number,
    proveedor?: string,
  ) =>
    invoke<string>("registrar_entrada", {
      id_producto,
      cantidad,
      precio,
      proveedor: proveedor || null,
    }),

  estadoCaja: () => invoke<Caja>("estado_caja", {}),
  abrirCaja: () => invoke<void>("abrir_caja", {}),
  cerrarCaja: () => invoke<void>("cerrar_caja", {}),
  listarEmpleados: () => invoke<Empleado[]>("listar_empleados", {}),
  crearEmpleado: (entrada: EmpleadoInput) =>
    invoke<void>("crear_empleado", { entrada }),
  eliminarEmpleado: (ci: string) => invoke<void>("eliminar_empleado", { ci }),
  crearPedido: (id_proveedor: string, items: ItemCarrito[]) =>
    invoke<string>("crear_pedido", { id_proveedor, items }),
  listarPedidos: () => invoke<Pedido[]>("listar_pedidos", {}),
  auditoria: (limite?: number) =>
    invoke<Auditoria[]>("auditoria", { limite: limite ?? 100 }),
  estadisticas: () => invoke<Estadisticas>("estadisticas", {}),
  exportarReporte: (tipo: string, ruta: string) =>
    invoke<number>("exportar_reporte", { tipo, ruta }),
  importarDatos: (ruta: string) =>
    invoke<MigrationReport>("importar_datos", { ruta }),

  elegirArchivo: () =>
    open({ multiple: false, filters: [{ name: "CSV o texto", extensions: ["csv", "txt"] }] }),
  elegirDirectorio: () => open({ directory: true, multiple: false }),
  elegirDestino: (nombre: string) =>
    save({
      defaultPath: nombre,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    }),
};
