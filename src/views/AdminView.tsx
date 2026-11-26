import { useEffect, useState, type FormEvent } from "react";
import { api } from "../api";
import { dinero, errorTexto, fechaCorta, fechaHora } from "../formato";
import type {
  Auditoria,
  Caja,
  DashboardDatos,
  Empleado,
  EmpleadoInput,
  Entrada,
  ItemCarrito,
  MigrationReport,
  Pedido,
  Producto,
  ProductoInput,
  Proveedor,
  ProveedorInput,
  Rol,
  VistaPrevia,
  Venta,
} from "../types";
import {
  Campo,
  DataTable,
  Etiqueta,
  Modal,
  Pestanas,
  Tarjeta,
  type Avisar,
  type Columna,
} from "../components/UI";

type Seccion =
  | "Resumen"
  | "Caja"
  | "Inventario"
  | "Ventas"
  | "Entradas"
  | "Productos"
  | "Empleados"
  | "Proveedores"
  | "Pedidos"
  | "Auditoría"
  | "Datos";

const secciones: Seccion[] = [
  "Resumen", "Caja", "Inventario", "Ventas", "Entradas", "Productos",
  "Empleados", "Proveedores", "Pedidos", "Auditoría", "Datos",
];

const columnasProducto: Columna<Producto>[] = [
  { titulo: "ID", valor: (p) => p.id_producto, ancho: "110px" },
  { titulo: "Producto", valor: (p) => p.nombre },
  { titulo: "Marca", valor: (p) => p.marca },
  { titulo: "Categoría", valor: (p) => p.categoria },
  { titulo: "P. entrada", valor: (p) => <span className="num">{dinero(p.precio_entrada)}</span>, ancho: "110px" },
  { titulo: "P. venta", valor: (p) => <span className="num">{dinero(p.precio_venta)}</span>, ancho: "110px" },
  {
    titulo: "Stock",
    valor: (p) => p.stock <= 5 ? <Etiqueta tono="rojo">{p.stock}</Etiqueta> : <span className="num">{p.stock}</span>,
    ancho: "80px",
  },
];

type Periodo = "HOY" | "ULTIMOS_7" | "ULTIMOS_30" | "TODO";
const periodos: { valor: Periodo; texto: string }[] = [
  { valor: "HOY", texto: "Hoy" },
  { valor: "ULTIMOS_7", texto: "Últimos 7 días" },
  { valor: "ULTIMOS_30", texto: "Últimos 30 días" },
  { valor: "TODO", texto: "Todo el historial" },
];

function Barras({
  items,
  secundaria,
  formato,
}: {
  items: { etiqueta: string; valor: number }[];
  secundaria?: boolean;
  formato?: (valor: number) => string;
}) {
  if (items.length === 0) return <div className="vacio">Sin datos en este período</div>;
  const maximo = Math.max(1, ...items.map((item) => item.valor));
  return (
    <div className="grafica">
      {items.map((item, indice) => (
        <div className="barra-fila" key={indice}>
          <span title={item.etiqueta}>
            {item.etiqueta.length > 24 ? `${item.etiqueta.slice(0, 23)}…` : item.etiqueta}
          </span>
          <div className="barra-pista">
            <div
              className={secundaria ? "barra-relleno secundaria" : "barra-relleno"}
              style={{ width: `${Math.max(2, (item.valor / maximo) * 100)}%` }}
            />
          </div>
          <span className="valor-derecha">
            {formato ? formato(item.valor) : Math.round(item.valor)}
          </span>
        </div>
      ))}
    </div>
  );
}

function Dashboard({ avisar }: { avisar: Avisar }) {
  const [periodo, setPeriodo] = useState<Periodo>("ULTIMOS_7");
  const [datos, setDatos] = useState<DashboardDatos | null>(null);
  const [caja, setCaja] = useState<Caja | null>(null);

  const cargar = async () => {
    try {
      setDatos(await api.dashboard(periodo));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  useEffect(() => {
    api.preferencias()
      .then((preferencias) => {
        if (periodos.some((opcion) => opcion.valor === preferencias.periodo_dashboard)) {
          setPeriodo(preferencias.periodo_dashboard as Periodo);
        }
      })
      .catch(() => {});
    api.estadoCaja().then(setCaja).catch((error) => avisar(errorTexto(error), "error"));
  }, []);

  useEffect(() => {
    void cargar();
    api.guardarPreferencias(undefined, periodo).catch(() => {});
  }, [periodo]);

  if (!datos) return <div className="vacio">Cargando dashboard…</div>;
  const resumen = datos.resumen;
  return (
    <div className="rejilla">
      <Tarjeta
        titulo="Resumen general"
        accion={
          <div className="barra" style={{ margin: 0 }}>
            <select value={periodo} onChange={(e) => setPeriodo(e.target.value as Periodo)}>
              {periodos.map((opcion) => <option key={opcion.valor} value={opcion.valor}>{opcion.texto}</option>)}
            </select>
            <button className="neutro" onClick={cargar}>Actualizar</button>
          </div>
        }
      >
        <p className="texto-suave">
          Del {fechaCorta(datos.desde)} al {fechaCorta(datos.hasta)} · ventas confirmadas e inventario actual.
        </p>
        <div className="rejilla cuatro">
          <div className="estadistica"><div className="numero acento">{dinero(resumen.total)}</div><div className="etiqueta-texto">total del período</div></div>
          <div className="estadistica"><div className="numero acento">{dinero(resumen.ticket_promedio)}</div><div className="etiqueta-texto">ticket promedio · {resumen.cantidad_ventas} ventas</div></div>
          <div className="estadistica"><div className="numero acento">{resumen.unidades}</div><div className="etiqueta-texto">unidades vendidas</div></div>
          <div className="estadistica"><div className="numero" style={{ color: "var(--rojo)" }}>{resumen.stock_critico}</div><div className="etiqueta-texto">productos ≤ 5 uds.</div></div>
        </div>
        <div className="separador" />
        <div className="rejilla dos">
          <div className="estadistica"><div className="numero acento">{dinero(resumen.valor_inventario)}</div><div className="etiqueta-texto">valor del inventario · {resumen.productos} productos</div></div>
          <div className="estadistica"><div className="numero">{caja?.estado ?? "—"}</div><div className="etiqueta-texto">estado de caja</div></div>
        </div>
      </Tarjeta>
      <div className="rejilla dos">
        <Tarjeta titulo="Ventas por día">
          <Barras items={datos.ventas_por_dia.map((p) => ({ etiqueta: p.fecha, valor: p.total }))} formato={dinero} />
        </Tarjeta>
        <Tarjeta titulo="Ventas por categoría">
          <Barras items={datos.ventas_por_categoria} formato={dinero} secundaria />
        </Tarjeta>
        <Tarjeta titulo="Top productos por unidades">
          <Barras items={datos.top_productos} />
        </Tarjeta>
        <Tarjeta titulo="Stock por categoría">
          <Barras items={datos.stock_por_categoria} secundaria />
        </Tarjeta>
      </div>
    </div>
  );
}

function Caja({ avisar }: { avisar: Avisar }) {
  const [caja, setCaja] = useState<Caja | null>(null);
  const cargar = () => api.estadoCaja().then(setCaja).catch((e) => avisar(errorTexto(e), "error"));
  useEffect(() => { cargar(); }, []);

  const cambiar = async (abrir: boolean) => {
    try {
      if (abrir) await api.abrirCaja(); else await api.cerrarCaja();
      avisar(abrir ? "Caja abierta" : "Caja cerrada", "ok");
      cargar();
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  return (
    <Tarjeta titulo="Administración de caja" accion={<Etiqueta tono={caja?.estado === "ABIERTA" ? "verde" : "rojo"}>{caja?.estado ?? "—"}</Etiqueta>}>
      <div className="lista-simple">
        <div>Apertura: {caja?.fecha_apertura || "—"}</div>
        <div>Cierre: {caja?.fecha_cierre || "—"}</div>
        <div>Responsable: {caja?.ci_empleado || "—"}</div>
      </div>
      <p className="texto-suave" style={{ marginTop: 16 }}>
        Las ventas solo pueden confirmarse mientras la caja está abierta.
      </p>
      <div className="acciones">
        <button className="secundario" onClick={() => cambiar(true)}>Abrir caja</button>
        <button className="peligro" onClick={() => cambiar(false)}>Cerrar caja</button>
      </div>
    </Tarjeta>
  );
}

function InventarioAdmin({ avisar }: { avisar: Avisar }) {
  const [termino, setTermino] = useState("");
  const [productos, setProductos] = useState<Producto[]>([]);
  const cargar = async () => {
    try { setProductos(await api.listarProductos(termino || undefined)); }
    catch (error) { avisar(errorTexto(error), "error"); }
  };
  useEffect(() => { void cargar(); }, []);
  return (
    <Tarjeta titulo="Inventario">
      <div className="barra">
        <input placeholder="Buscar producto" value={termino} onChange={(e) => setTermino(e.target.value)} onKeyDown={(e) => e.key === "Enter" && cargar()} />
        <button className="secundario" onClick={cargar}>Buscar</button>
      </div>
      <DataTable filas={productos} columnas={columnasProducto} />
    </Tarjeta>
  );
}

async function exportar(avisar: Avisar, tipo: string, nombre: string) {
  try {
    const ruta = await api.elegirDestino(nombre);
    if (!ruta) return;
    const filas = await api.exportarReporte(tipo, ruta);
    avisar(`${filas} filas exportadas a ${ruta}`, "ok");
  } catch (error) {
    avisar(errorTexto(error), "error");
  }
}

function VentasAdmin({ avisar }: { avisar: Avisar }) {
  const [ventas, setVentas] = useState<Venta[]>([]);
  const [estado, setEstado] = useState("");
  const [factura, setFactura] = useState<string | null>(null);
  useEffect(() => {
    api.listarVentas(estado || undefined)
      .then(setVentas)
      .catch((e) => avisar(errorTexto(e), "error"));
  }, [estado]);
  const columnas: Columna<Venta>[] = [
    { titulo: "Venta", valor: (v) => v.id_venta, ancho: "130px" },
    { titulo: "Cliente", valor: (v) => v.nombre_cliente || "—" },
    { titulo: "Estado", valor: (v) => <Etiqueta tono={v.estado === "CONFIRMADA" ? "verde" : v.estado === "ANULADA" ? "rojo" : "amarillo"}>{v.estado}</Etiqueta>, ancho: "120px" },
    { titulo: "Total", valor: (v) => <span className="num">{dinero(v.total)}</span>, ancho: "110px" },
    { titulo: "Fecha", valor: (v) => fechaHora(v.fecha_venta), ancho: "150px" },
    { titulo: "", valor: (v) => <button className="mini" onClick={async () => { try { setFactura(await api.facturaOrden(v.id_orden)); } catch (e) { avisar(errorTexto(e), "error"); } }}>Factura</button>, ancho: "80px" },
  ];
  return (
    <>
      <Tarjeta titulo="Ventas" accion={<button className="secundario" onClick={() => exportar(avisar, "ventas", "ventas.csv")}>Exportar CSV</button>}>
        <div className="barra">
          <select value={estado} onChange={(e) => setEstado(e.target.value)}>
            <option value="">Todos los estados</option>
            <option value="PROFORMA">Proforma</option>
            <option value="CONFIRMADA">Confirmada</option>
            <option value="ANULADA">Anulada</option>
          </select>
        </div>
        <DataTable filas={ventas} columnas={columnas} />
      </Tarjeta>
      {factura && <Modal titulo="Factura" onCerrar={() => setFactura(null)}><pre className="factura">{factura}</pre></Modal>}
    </>
  );
}

function EntradasAdmin({ avisar }: { avisar: Avisar }) {
  const [entradas, setEntradas] = useState<Entrada[]>([]);
  useEffect(() => {
    api.listarEntradas()
      .then(setEntradas)
      .catch((e) => avisar(errorTexto(e), "error"));
  }, []);
  const columnas: Columna<Entrada>[] = [
    { titulo: "ID", valor: (e) => e.id_entrada, ancho: "130px" },
    { titulo: "Producto", valor: (e) => e.nombre_producto },
    { titulo: "Cantidad", valor: (e) => <span className="num">{e.cantidad}</span>, ancho: "90px" },
    { titulo: "Proveedor", valor: (e) => e.id_proveedor || "—", ancho: "110px" },
    { titulo: "Fecha", valor: (e) => fechaCorta(e.fecha_entrada), ancho: "110px" },
  ];
  return (
    <Tarjeta titulo="Entradas" accion={<button className="secundario" onClick={() => exportar(avisar, "entradas", "entradas.csv")}>Exportar CSV</button>}>
      <DataTable filas={entradas} columnas={columnas} />
    </Tarjeta>
  );
}

function ProductosAdmin({ avisar }: { avisar: Avisar }) {
  const [productos, setProductos] = useState<Producto[]>([]);
  const [categorias, setCategorias] = useState<string[]>([]);
  const [imagenes, setImagenes] = useState<Record<string, string>>({});
  const [modal, setModal] = useState(false);
  const [nuevo, setNuevo] = useState<ProductoInput>({
    id_producto: "", nombre: "", marca: "", categoria: "", descripcion: "",
    precio_entrada: 0, precio_venta: 0, stock: 0,
  });

  const cargar = async () => {
    try {
      const lista = await api.listarProductos();
      setProductos(lista);
      setCategorias(await api.listarCategorias());
      api.rutasImagenes(lista.map((producto) => producto.id_producto))
        .then(setImagenes)
        .catch(() => {});
    } catch (error) { avisar(errorTexto(error), "error"); }
  };
  useEffect(() => { void cargar(); }, []);

  const subirImagen = async (id: string) => {
    try {
      const ruta = await api.elegirImagen();
      if (!ruta) return;
      await api.guardarImagen(id, ruta);
      avisar(`Imagen actualizada para ${id}`, "ok");
      cargar();
    } catch (error) { avisar(errorTexto(error), "error"); }
  };

  const guardar = async (evento: FormEvent) => {
    evento.preventDefault();
    try {
      await api.crearProducto(nuevo);
      avisar("Producto creado", "ok");
      setModal(false);
      setNuevo({ id_producto: "", nombre: "", marca: "", categoria: "", descripcion: "", precio_entrada: 0, precio_venta: 0, stock: 0 });
      cargar();
    } catch (error) { avisar(errorTexto(error), "error"); }
  };

  const eliminar = async (id: string) => {
    try { await api.eliminarProducto(id); avisar(`Producto ${id} eliminado`, "ok"); cargar(); }
    catch (error) { avisar(errorTexto(error), "error"); }
  };

  const columnas: Columna<Producto>[] = [
    {
      titulo: "Img",
      valor: (p) => imagenes[p.id_producto]
        ? <img className="miniatura" src={api.imagenUrl(imagenes[p.id_producto])} alt={p.nombre} />
        : <div className="sin-imagen">▣</div>,
      ancho: "60px",
    },
    ...columnasProducto,
    {
      titulo: "",
      valor: (p) => (
        <div className="barra" style={{ margin: 0 }}>
          <button className="mini" onClick={() => subirImagen(p.id_producto)}>Imagen</button>
          <button className="mini peligro" onClick={() => eliminar(p.id_producto)}>Eliminar</button>
        </div>
      ),
      ancho: "150px",
    },
  ];

  return (
    <>
      <Tarjeta titulo="Productos" accion={<button className="primario" onClick={() => setModal(true)}>Nuevo producto</button>}>
        <DataTable filas={productos} columnas={columnas} />
      </Tarjeta>
      {modal && (
        <Modal titulo="Nuevo producto" onCerrar={() => setModal(false)}>
          <form onSubmit={guardar} className="formulario">
            <Campo etiqueta="ID (opcional)"><input value={nuevo.id_producto ?? ""} onChange={(e) => setNuevo({ ...nuevo, id_producto: e.target.value })} /></Campo>
            <Campo etiqueta="Nombre"><input value={nuevo.nombre} onChange={(e) => setNuevo({ ...nuevo, nombre: e.target.value })} /></Campo>
            <Campo etiqueta="Marca"><input value={nuevo.marca} onChange={(e) => setNuevo({ ...nuevo, marca: e.target.value })} /></Campo>
            <Campo etiqueta="Categoría">
              <input list="categorias" value={nuevo.categoria} onChange={(e) => setNuevo({ ...nuevo, categoria: e.target.value })} />
              <datalist id="categorias">{categorias.map((c) => <option key={c} value={c} />)}</datalist>
            </Campo>
            <Campo etiqueta="Precio entrada"><input type="number" step="0.01" min={0} value={nuevo.precio_entrada} onChange={(e) => setNuevo({ ...nuevo, precio_entrada: Number(e.target.value) })} /></Campo>
            <Campo etiqueta="Precio venta"><input type="number" step="0.01" min={0} value={nuevo.precio_venta} onChange={(e) => setNuevo({ ...nuevo, precio_venta: Number(e.target.value) })} /></Campo>
            <Campo etiqueta="Stock inicial"><input type="number" min={0} value={nuevo.stock} onChange={(e) => setNuevo({ ...nuevo, stock: Number(e.target.value) })} /></Campo>
            <Campo etiqueta="Descripción"><input value={nuevo.descripcion} onChange={(e) => setNuevo({ ...nuevo, descripcion: e.target.value })} /></Campo>
            <div className="completo acciones"><button className="secundario" type="button" onClick={() => setModal(false)}>Cancelar</button><button className="primario">Guardar</button></div>
          </form>
        </Modal>
      )}
    </>
  );
}

function EmpleadosAdmin({ avisar }: { avisar: Avisar }) {
  const [empleados, setEmpleados] = useState<Empleado[]>([]);
  const [modal, setModal] = useState(false);
  const [nuevo, setNuevo] = useState<EmpleadoInput>({ ci: "", nombre: "", rol: "CAJERO", contrasena: "" });

  const cargar = async () => {
    try { setEmpleados(await api.listarEmpleados()); }
    catch (error) { avisar(errorTexto(error), "error"); }
  };
  useEffect(() => { void cargar(); }, []);

  const guardar = async (evento: FormEvent) => {
    evento.preventDefault();
    try {
      await api.crearEmpleado(nuevo);
      avisar("Empleado creado", "ok");
      setModal(false);
      setNuevo({ ci: "", nombre: "", rol: "CAJERO", contrasena: "" });
      cargar();
    } catch (error) { avisar(errorTexto(error), "error"); }
  };

  const columnas: Columna<Empleado>[] = [
    { titulo: "CI", valor: (e) => e.ci, ancho: "140px" },
    { titulo: "Nombre", valor: (e) => e.nombre },
    { titulo: "Rol", valor: (e) => <Etiqueta tono="azul">{e.rol}</Etiqueta>, ancho: "150px" },
    { titulo: "", valor: (e) => <button className="mini peligro" onClick={async () => { try { await api.eliminarEmpleado(e.ci); avisar("Empleado eliminado", "ok"); cargar(); } catch (error) { avisar(errorTexto(error), "error"); } }}>Eliminar</button>, ancho: "85px" },
  ];

  return (
    <>
      <Tarjeta titulo="Empleados" accion={<button className="primario" onClick={() => setModal(true)}>Nuevo empleado</button>}>
        <DataTable filas={empleados} columnas={columnas} />
      </Tarjeta>
      {modal && (
        <Modal titulo="Nuevo empleado" onCerrar={() => setModal(false)}>
          <form onSubmit={guardar} className="formulario">
            <Campo etiqueta="CI"><input value={nuevo.ci} onChange={(e) => setNuevo({ ...nuevo, ci: e.target.value })} /></Campo>
            <Campo etiqueta="Nombre"><input value={nuevo.nombre} onChange={(e) => setNuevo({ ...nuevo, nombre: e.target.value })} /></Campo>
            <Campo etiqueta="Rol">
              <select value={nuevo.rol} onChange={(e) => setNuevo({ ...nuevo, rol: e.target.value as Rol })}>
                <option value="ADMINISTRADOR">Administrador</option>
                <option value="CAJERO">Cajero</option>
                <option value="ALMACENERO">Almacenero</option>
              </select>
            </Campo>
            <Campo etiqueta="Contraseña"><input type="password" value={nuevo.contrasena} onChange={(e) => setNuevo({ ...nuevo, contrasena: e.target.value })} /></Campo>
            <div className="completo acciones"><button className="secundario" type="button" onClick={() => setModal(false)}>Cancelar</button><button className="primario">Guardar</button></div>
          </form>
        </Modal>
      )}
    </>
  );
}

function ProveedoresAdmin({ avisar }: { avisar: Avisar }) {
  const [proveedores, setProveedores] = useState<Proveedor[]>([]);
  const [modal, setModal] = useState(false);
  const [nuevo, setNuevo] = useState<ProveedorInput>({ id_proveedor: "", nombre: "", contacto: "" });

  const cargar = async () => {
    try { setProveedores(await api.listarProveedores()); }
    catch (error) { avisar(errorTexto(error), "error"); }
  };
  useEffect(() => { void cargar(); }, []);

  const guardar = async (evento: FormEvent) => {
    evento.preventDefault();
    try {
      await api.crearProveedor(nuevo);
      avisar("Proveedor creado", "ok");
      setModal(false);
      setNuevo({ id_proveedor: "", nombre: "", contacto: "" });
      cargar();
    } catch (error) { avisar(errorTexto(error), "error"); }
  };

  const columnas: Columna<Proveedor>[] = [
    { titulo: "ID", valor: (p) => p.id_proveedor, ancho: "130px" },
    { titulo: "Nombre", valor: (p) => p.nombre },
    { titulo: "Contacto", valor: (p) => p.contacto || "—", ancho: "140px" },
    { titulo: "", valor: (p) => <button className="mini peligro" onClick={async () => { try { await api.eliminarProveedor(p.id_proveedor); avisar("Proveedor eliminado", "ok"); cargar(); } catch (error) { avisar(errorTexto(error), "error"); } }}>Eliminar</button>, ancho: "85px" },
  ];

  return (
    <>
      <Tarjeta titulo="Proveedores" accion={<button className="primario" onClick={() => setModal(true)}>Nuevo proveedor</button>}>
        <DataTable filas={proveedores} columnas={columnas} />
      </Tarjeta>
      {modal && (
        <Modal titulo="Nuevo proveedor" onCerrar={() => setModal(false)}>
          <form onSubmit={guardar} className="formulario">
            <Campo etiqueta="ID (opcional)"><input value={nuevo.id_proveedor ?? ""} onChange={(e) => setNuevo({ ...nuevo, id_proveedor: e.target.value })} /></Campo>
            <Campo etiqueta="Nombre"><input value={nuevo.nombre} onChange={(e) => setNuevo({ ...nuevo, nombre: e.target.value })} /></Campo>
            <div className="completo"><Campo etiqueta="Contacto"><input value={nuevo.contacto} onChange={(e) => setNuevo({ ...nuevo, contacto: e.target.value })} /></Campo></div>
            <div className="completo acciones"><button className="secundario" type="button" onClick={() => setModal(false)}>Cancelar</button><button className="primario">Guardar</button></div>
          </form>
        </Modal>
      )}
    </>
  );
}

function PedidosAdmin({ avisar }: { avisar: Avisar }) {
  const [pedidos, setPedidos] = useState<Pedido[]>([]);
  const [proveedores, setProveedores] = useState<Proveedor[]>([]);
  const [productos, setProductos] = useState<Producto[]>([]);
  const [modal, setModal] = useState(false);
  const [proveedor, setProveedor] = useState("");
  const [items, setItems] = useState<ItemCarrito[]>([]);
  const [producto, setProducto] = useState("");
  const [cantidad, setCantidad] = useState(1);

  const cargar = async () => {
    try {
      setPedidos(await api.listarPedidos());
      setProveedores(await api.listarProveedores());
      setProductos(await api.listarProductos());
    } catch (error) { avisar(errorTexto(error), "error"); }
  };
  useEffect(() => { void cargar(); }, []);

  const agregar = () => {
    if (!producto) return;
    setItems((actual) => [...actual, { id_producto: producto, cantidad }]);
    setProducto("");
    setCantidad(1);
  };

  const guardar = async () => {
    try {
      const id = await api.crearPedido(proveedor, items);
      avisar(`Pedido ${id} creado`, "ok");
      setModal(false);
      setItems([]);
      cargar();
    } catch (error) { avisar(errorTexto(error), "error"); }
  };

  const columnas: Columna<Pedido>[] = [
    { titulo: "Pedido", valor: (p) => p.id_pedido, ancho: "130px" },
    { titulo: "Proveedor", valor: (p) => p.id_proveedor, ancho: "120px" },
    { titulo: "Producto", valor: (p) => p.id_producto },
    { titulo: "Cantidad", valor: (p) => <span className="num">{p.cantidad}</span>, ancho: "90px" },
    { titulo: "Fecha", valor: (p) => fechaCorta(p.fecha_pedido), ancho: "110px" },
  ];

  return (
    <>
      <Tarjeta titulo="Pedidos a proveedores" accion={<button className="primario" onClick={() => setModal(true)}>Nuevo pedido</button>}>
        <DataTable filas={pedidos} columnas={columnas} vacio="No hay pedidos registrados" />
      </Tarjeta>
      {modal && (
        <Modal titulo="Nuevo pedido" onCerrar={() => setModal(false)}>
          <div className="formulario">
            <div className="completo"><Campo etiqueta="Proveedor"><select value={proveedor} onChange={(e) => setProveedor(e.target.value)}>{proveedores.map((p) => <option key={p.id_proveedor} value={p.id_proveedor}>{p.nombre}</option>)}</select></Campo></div>
            <Campo etiqueta="Producto"><select value={producto} onChange={(e) => setProducto(e.target.value)}><option value="">Selecciona…</option>{productos.map((p) => <option key={p.id_producto} value={p.id_producto}>{p.nombre}</option>)}</select></Campo>
            <Campo etiqueta="Cantidad"><input type="number" min={1} value={cantidad} onChange={(e) => setCantidad(Number(e.target.value))} /></Campo>
          </div>
          <div className="acciones"><button className="secundario" onClick={agregar}>Agregar línea</button></div>
          <div className="separador" />
          <DataTable filas={items} columnas={[
            { titulo: "Producto", valor: (i) => i.id_producto },
            { titulo: "Cantidad", valor: (i) => <span className="num">{i.cantidad}</span> },
          ]} vacio="Sin líneas agregadas" />
          <div className="acciones"><button className="secundario" onClick={() => setModal(false)}>Cancelar</button><button className="primario" onClick={guardar}>Guardar pedido</button></div>
        </Modal>
      )}
    </>
  );
}

function AuditoriaAdmin({ avisar }: { avisar: Avisar }) {
  const [registros, setRegistros] = useState<Auditoria[]>([]);
  useEffect(() => {
    api.auditoria()
      .then(setRegistros)
      .catch((e) => avisar(errorTexto(e), "error"));
  }, []);
  const columnas: Columna<Auditoria>[] = [
    { titulo: "Fecha", valor: (a) => fechaHora(a.fecha), ancho: "160px" },
    { titulo: "Empleado", valor: (a) => a.ci_empleado || "—", ancho: "130px" },
    { titulo: "Acción", valor: (a) => <Etiqueta tono="azul">{a.accion}</Etiqueta>, ancho: "170px" },
    { titulo: "Detalle", valor: (a) => a.detalle },
  ];
  return <Tarjeta titulo="Registro de actividad"><DataTable filas={registros} columnas={columnas} /></Tarjeta>;
}

function DatosAdmin({ avisar }: { avisar: Avisar }) {
  const [reporte, setReporte] = useState<MigrationReport | null>(null);
  const [pendiente, setPendiente] = useState<string | null>(null);
  const [vista, setVista] = useState<VistaPrevia | null>(null);

  const elegir = async () => {
    try {
      const ruta = await api.elegirDirectorio();
      if (!ruta) return;
      setPendiente(ruta);
      setVista(await api.vistaPreviaCsv(`${ruta}/inventario.csv`, "importar"));
    } catch (error) { avisar(errorTexto(error), "error"); }
  };

  const importar = async () => {
    if (!pendiente) return;
    try {
      setReporte(await api.importarDatos(pendiente));
      avisar("Importación completada", "ok");
      setVista(null);
      setPendiente(null);
    } catch (error) { avisar(errorTexto(error), "error"); }
  };
  const filas = reporte
    ? Object.entries(reporte).map(([campo, valor]) => ({ campo, valor }))
    : [];

  return (
    <div className="rejilla dos">
      <Tarjeta titulo="Exportar a CSV">
        <p className="texto-suave">SQLite es la fuente de verdad. Puedes generar copias en CSV cuando lo necesites.</p>
        <div className="barra">
          <button className="secundario" onClick={() => exportar(avisar, "inventario", "inventario.csv")}>Inventario</button>
          <button className="secundario" onClick={() => exportar(avisar, "ventas", "ventas.csv")}>Ventas</button>
          <button className="secundario" onClick={() => exportar(avisar, "entradas", "entradas.csv")}>Entradas</button>
        </div>
      </Tarjeta>
      <Tarjeta titulo="Importar CSV originales">
        <p className="texto-suave">Selecciona la carpeta con los nueve CSV. Los registros existentes se conservan; solo se añaden los nuevos.</p>
        <button className="primario" onClick={elegir}>Seleccionar carpeta</button>
        {vista && (
          <Modal titulo="Vista previa de importación" onCerrar={() => setVista(null)}>
            <p className="texto-suave">
              {vista.total} producto(s) en el CSV. Los registros existentes se conservan.
            </p>
            <DataTable
              filas={vista.filas}
              columnas={vista.columnas.slice(0, 4).map((nombre, indice) => ({
                titulo: nombre,
                valor: (fila: string[]) => fila[indice] ?? "",
              }))}
            />
            <div className="acciones">
              <button className="neutro" onClick={() => setVista(null)}>Cancelar</button>
              <button className="primario" onClick={importar}>Confirmar importación</button>
            </div>
          </Modal>
        )}
        {reporte && (
          <div style={{ marginTop: 16 }}>
            <DataTable filas={filas} columnas={[
              { titulo: "Concepto", valor: (f) => f.campo },
              { titulo: "Registros", valor: (f) => <span className="num">{f.valor}</span>, ancho: "100px" },
            ]} />
          </div>
        )}
      </Tarjeta>
    </div>
  );
}

export default function AdminView({ avisar }: { avisar: Avisar }) {
  const [seccion, setSeccion] = useState<Seccion>("Resumen");
  return (
    <>
      <Pestanas items={secciones} activa={seccion} onCambiar={setSeccion} />
      {seccion === "Resumen" && <Dashboard avisar={avisar} />}
      {seccion === "Caja" && <Caja avisar={avisar} />}
      {seccion === "Inventario" && <InventarioAdmin avisar={avisar} />}
      {seccion === "Ventas" && <VentasAdmin avisar={avisar} />}
      {seccion === "Entradas" && <EntradasAdmin avisar={avisar} />}
      {seccion === "Productos" && <ProductosAdmin avisar={avisar} />}
      {seccion === "Empleados" && <EmpleadosAdmin avisar={avisar} />}
      {seccion === "Proveedores" && <ProveedoresAdmin avisar={avisar} />}
      {seccion === "Pedidos" && <PedidosAdmin avisar={avisar} />}
      {seccion === "Auditoría" && <AuditoriaAdmin avisar={avisar} />}
      {seccion === "Datos" && <DatosAdmin avisar={avisar} />}
    </>
  );
}
