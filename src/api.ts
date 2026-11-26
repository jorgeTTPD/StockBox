import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  Auditoria,
  Caja,
  Cliente,
  ClienteInput,
  ConfigAsistente,
  DashboardDatos,
  Devolucion,
  Empleado,
  EmpleadoInput,
  Entrada,
  Estadisticas,
  ItemCarrito,
  LineaVenta,
  MigrationReport,
  OrdenCreada,
  OrdenInput,
  Pedido,
  Preferencias,
  Producto,
  ProductoInput,
  Proveedor,
  ProveedorInput,
  ResultadoVenta,
  Rol,
  Sesion,
  VistaPrevia,
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

  buscarVenta: (id_venta: string) => invoke<LineaVenta | null>("buscar_venta", { id_venta }),
  devolverVenta: (id_venta: string, cantidad: number) =>
    invoke<string>("devolver_venta", { id_venta, cantidad }),
  listarDevoluciones: () => invoke<Devolucion[]>("listar_devoluciones", {}),
  dashboard: (periodo: string) => invoke<DashboardDatos>("dashboard", { periodo }),
  vistaPreviaCsv: (ruta: string, tipo: string) =>
    invoke<VistaPrevia>("vista_previa_csv", { ruta, tipo }),

  rutaImagen: (id_producto: string) =>
    invoke<string | null>("ruta_imagen_producto", { id_producto }),
  rutasImagenes: (ids: string[]) => invoke<Record<string, string>>("rutas_imagenes", { ids }),
  guardarImagen: (id_producto: string, ruta: string) =>
    invoke<string>("guardar_imagen_producto", { id_producto, ruta }),
  imagenUrl: (ruta: string) => convertFileSrc(ruta),

  configuracionAsistente: () =>
    invoke<ConfigAsistente>("configuracion_asistente", {}),
  preguntarAsistente: (pregunta: string) =>
    invoke<string>("preguntar_asistente", { pregunta }),
  limpiarAsistente: () => invoke<void>("limpiar_asistente", {}),

  preferencias: () => invoke<Preferencias>("preferencias", {}),
  guardarPreferencias: (tema_oscuro?: boolean, periodo?: string) =>
    invoke<Preferencias>("guardar_preferencias", {
      tema_oscuro: tema_oscuro ?? null,
      periodo: periodo ?? null,
    }),

  elegirImagen: () =>
    open({ multiple: false, filters: [{ name: "Imágenes", extensions: ["png", "jpg", "jpeg"] }] }),
  elegirArchivo: () =>
    open({ multiple: false, filters: [{ name: "CSV o texto", extensions: ["csv", "txt"] }] }),
  elegirDirectorio: () => open({ directory: true, multiple: false }),
  elegirDestino: (nombre: string) =>
    save({
      defaultPath: nombre,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    }),
};
