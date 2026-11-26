import { useEffect, useState } from "react";
import { api } from "../api";
import { dinero, errorTexto, fechaHora } from "../formato";
import type { ItemCarrito, Producto, ResultadoVenta, Venta } from "../types";
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

type Seccion = "Nueva venta" | "Ventas" | "Anular venta" | "Garantías";

function estadoTono(estado: string) {
  if (estado === "CONFIRMADA") return "verde";
  if (estado === "ANULADA") return "rojo";
  return "amarillo";
}

function NuevaVenta({ avisar }: { avisar: Avisar }) {
  const [termino, setTermino] = useState("");
  const [resultados, setResultados] = useState<Producto[]>([]);
  const [carrito, setCarrito] = useState<ItemCarrito[]>([]);
  const [nombre, setNombre] = useState("Consumidor final");
  const [ci, setCi] = useState("00000000");
  const [telefono, setTelefono] = useState("0000000");
  const [factura, setFactura] = useState<ResultadoVenta | null>(null);
  const [proforma, setProforma] = useState<string | null>(null);
  const [proformaTexto, setProformaTexto] = useState("");
  const [cargando, setCargando] = useState(false);

  const buscar = async () => {
    try {
      setResultados(await api.listarProductos(termino));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  const agregar = (producto: Producto) => {
    setCarrito((actual) => {
      const existente = actual.find((item) => item.id_producto === producto.id_producto);
      if (existente) {
        return actual.map((item) =>
          item.id_producto === producto.id_producto
            ? { ...item, cantidad: item.cantidad + 1, subtotal: (item.cantidad + 1) * item.precio_unitario! }
            : item,
        );
      }
      return [
        ...actual,
        {
          id_producto: producto.id_producto,
          nombre_producto: producto.nombre,
          cantidad: 1,
          precio_unitario: producto.precio_venta,
          subtotal: producto.precio_venta,
        },
      ];
    });
  };

  const cambiarCantidad = (id_producto: string, cantidad: number) => {
    setCarrito((actual) =>
      actual
        .map((item) =>
          item.id_producto === id_producto
            ? {
                ...item,
                cantidad,
                subtotal: cantidad * (item.precio_unitario ?? 0),
              }
            : item,
        )
        .filter((item) => item.cantidad > 0),
    );
  };

  const total = carrito.reduce((suma, item) => suma + (item.subtotal ?? 0), 0);

  const prepararOrden = async () => {
    if (carrito.length === 0) {
      avisar("Agrega productos al carrito", "error");
      return null;
    }
    return api.crearOrden({
      nombre_cliente: nombre,
      ci_cliente: ci,
      telefono_cliente: telefono,
      items: carrito,
    });
  };

  const hacerProforma = async () => {
    setCargando(true);
    try {
      const creada = await prepararOrden();
      if (creada) {
        setProforma(creada.id_orden);
        setProformaTexto(await api.facturaOrden(creada.id_orden));
        avisar(`Proforma ${creada.id_orden} creada`, "ok");
      }
    } catch (error) {
      avisar(errorTexto(error), "error");
    } finally {
      setCargando(false);
    }
  };

  const confirmar = async () => {
    setCargando(true);
    try {
      const creada = await prepararOrden();
      if (creada) {
        const resultado = await api.confirmarOrden(creada.id_orden);
        setFactura(resultado);
        setCarrito([]);
        avisar(`Venta ${resultado.id_orden} confirmada`, "ok");
      }
    } catch (error) {
      avisar(errorTexto(error), "error");
    } finally {
      setCargando(false);
    }
  };

  const importarOrden = async () => {
    try {
      const ruta = await api.elegirArchivo();
      if (!ruta) return;
      const cantidad = await api.importarOrdenes(ruta);
      avisar(`${cantidad} orden(es) importadas como proformas`, "ok");
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  const columnasResultados: Columna<Producto>[] = [
    { titulo: "ID", valor: (p) => p.id_producto, ancho: "110px" },
    { titulo: "Producto", valor: (p) => p.nombre },
    { titulo: "Marca", valor: (p) => p.marca },
    {
      titulo: "Precio",
      valor: (p) => <span className="num">{dinero(p.precio_venta)}</span>,
      ancho: "100px",
    },
    { titulo: "Stock", valor: (p) => <span className="num">{p.stock}</span>, ancho: "70px" },
    {
      titulo: "",
      valor: (p) => (
        <button className="mini" onClick={() => agregar(p)}>
          Agregar
        </button>
      ),
      ancho: "80px",
    },
  ];

  const columnasCarrito: Columna<ItemCarrito>[] = [
    { titulo: "Producto", valor: (i) => i.nombre_producto ?? i.id_producto },
    {
      titulo: "Cantidad",
      valor: (i) => (
        <input
          className="mini"
          type="number"
          min={1}
          value={i.cantidad}
          onChange={(e) => cambiarCantidad(i.id_producto, Number(e.target.value))}
          style={{ width: "70px" }}
        />
      ),
      ancho: "110px",
    },
    {
      titulo: "P. unitario",
      valor: (i) => <span className="num">{dinero(i.precio_unitario ?? 0)}</span>,
      ancho: "110px",
    },
    {
      titulo: "Subtotal",
      valor: (i) => <span className="num">{dinero(i.subtotal ?? 0)}</span>,
      ancho: "110px",
    },
    {
      titulo: "",
      valor: (i) => (
        <button className="mini peligro" onClick={() => cambiarCantidad(i.id_producto, 0)}>
          Quitar
        </button>
      ),
      ancho: "80px",
    },
  ];

  return (
    <div className="rejilla dos">
      <Tarjeta titulo="Buscar productos">
        <div className="barra">
          <input
            placeholder="ID, nombre o marca"
            value={termino}
            onChange={(e) => setTermino(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && buscar()}
          />
          <button className="secundario" onClick={buscar}>Buscar</button>
          <button className="neutro" onClick={importarOrden}>Importar orden CSV</button>
        </div>
        <DataTable filas={resultados} columnas={columnasResultados} vacio="Busca un producto para empezar" />
      </Tarjeta>

      <div className="rejilla">
        <Tarjeta titulo="Datos del cliente">
          <div className="formulario">
            <div className="completo">
              <Campo etiqueta="Nombre">
                <input value={nombre} onChange={(e) => setNombre(e.target.value)} />
              </Campo>
            </div>
            <Campo etiqueta="CI">
              <input value={ci} onChange={(e) => setCi(e.target.value)} />
            </Campo>
            <Campo etiqueta="Teléfono">
              <input value={telefono} onChange={(e) => setTelefono(e.target.value)} />
            </Campo>
          </div>
        </Tarjeta>

        <Tarjeta
          titulo="Carrito"
          accion={<span className="total-carrito">{dinero(total)}</span>}
        >
          <DataTable filas={carrito} columnas={columnasCarrito} vacio="El carrito está vacío" />
          <div className="acciones">
            <button className="secundario" onClick={hacerProforma} disabled={cargando}>Hacer proforma</button>
            <button className="primario" onClick={confirmar} disabled={cargando}>Confirmar venta</button>
          </div>
        </Tarjeta>
      </div>

      {factura && (
        <Modal titulo={`Venta ${factura.id_orden}`} onCerrar={() => setFactura(null)}>
          <div className="lista-simple">
            <div>Órdenes de venta: {factura.id_ventas.join(", ")}</div>
            <div>Garantías: {factura.id_garantias.join(", ")}</div>
            <div>Total cobrado: <strong>{dinero(factura.total)}</strong></div>
          </div>
          <div className="separador" />
          <pre className="factura">{factura.factura}</pre>
        </Modal>
      )}
      {proforma && (
        <Modal titulo={`Proforma ${proforma}`} onCerrar={() => setProforma(null)}>
          <pre className="factura">{proformaTexto}</pre>
        </Modal>
      )}
    </div>
  );
}

function ListaVentas({ avisar }: { avisar: Avisar }) {
  const [ventas, setVentas] = useState<Venta[]>([]);
  const [estado, setEstado] = useState("");
  const [fecha, setFecha] = useState("");
  const [factura, setFactura] = useState<string | null>(null);

  const cargar = async () => {
    try {
      setVentas(await api.listarVentas(estado || undefined, fecha || undefined));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };
  useEffect(() => {
    void cargar();
  }, [estado, fecha]);

  const columnas: Columna<Venta>[] = [
    { titulo: "Venta", valor: (v) => v.id_venta, ancho: "130px" },
    { titulo: "Orden", valor: (v) => v.id_orden, ancho: "130px" },
    { titulo: "Cliente", valor: (v) => v.nombre_cliente || "—" },
    {
      titulo: "Estado",
      valor: (v) => <Etiqueta tono={estadoTono(v.estado)}>{v.estado}</Etiqueta>,
      ancho: "120px",
    },
    { titulo: " Ítems", valor: (v) => <span className="num">{v.items.length}</span>, ancho: "70px" },
    {
      titulo: "Total",
      valor: (v) => <span className="num">{dinero(v.total)}</span>,
      ancho: "110px",
    },
    { titulo: "Fecha", valor: (v) => fechaHora(v.fecha_venta), ancho: "150px" },
    {
      titulo: "",
      valor: (v) => (
        <button
          className="mini"
          onClick={async () => {
            try {
              setFactura(await api.facturaOrden(v.id_orden));
            } catch (error) {
              avisar(errorTexto(error), "error");
            }
          }}
        >
          Factura
        </button>
      ),
      ancho: "80px",
    },
  ];

  return (
    <>
      <Tarjeta
        titulo="Historial de ventas"
        accion={
          <div className="barra" style={{ margin: 0 }}>
            <select value={estado} onChange={(e) => setEstado(e.target.value)}>
              <option value="">Todos los estados</option>
              <option value="PROFORMA">Proforma</option>
              <option value="CONFIRMADA">Confirmada</option>
              <option value="ANULADA">Anulada</option>
            </select>
            <input type="date" value={fecha} onChange={(e) => setFecha(e.target.value)} />
          </div>
        }
      >
        <DataTable filas={ventas} columnas={columnas} />
      </Tarjeta>
      {factura && (
        <Modal titulo="Factura" onCerrar={() => setFactura(null)}>
          <pre className="factura">{factura}</pre>
        </Modal>
      )}
    </>
  );
}

function AnularVenta({ avisar }: { avisar: Avisar }) {
  const [referencia, setReferencia] = useState("");
  const [cargando, setCargando] = useState(false);

  const anular = async () => {
    setCargando(true);
    try {
      await api.anularVenta(referencia);
      avisar(`Se anuló ${referencia} y se devolvió el stock`, "ok");
      setReferencia("");
    } catch (error) {
      avisar(errorTexto(error), "error");
    } finally {
      setCargando(false);
    }
  };

  return (
    <Tarjeta titulo="Anular una venta">
      <p className="texto-suave">Puedes usar el ID de la venta o el de la orden. Se devuelve el stock y se anulan sus garantías.</p>
      <div className="barra">
        <input
          placeholder="VEN-XXXXXXXX u ORD-XXXXXXXX"
          value={referencia}
          onChange={(e) => setReferencia(e.target.value)}
        />
        <button className="peligro" onClick={anular} disabled={cargando}>Anular</button>
      </div>
    </Tarjeta>
  );
}

function Garantias({ avisar }: { avisar: Avisar }) {
  const [referencia, setReferencia] = useState("");
  const [lineas, setLineas] = useState<string[]>([]);

  const consultar = async () => {
    try {
      setLineas(await api.consultarGarantia(referencia));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  return (
    <Tarjeta titulo="Consultar garantías">
      <p className="texto-suave">Busca por código de garantía, ID de venta u orden.</p>
      <div className="barra">
        <input
          placeholder="GAR-XXXXXXXX, VEN-… u ORD-…"
          value={referencia}
          onChange={(e) => setReferencia(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && consultar()}
        />
        <button className="secundario" onClick={consultar}>Consultar</button>
      </div>
      <div className="lista-simple">
        {lineas.map((linea, indice) => (
          <div key={indice}>{linea}</div>
        ))}
      </div>
    </Tarjeta>
  );
}

export default function VendedorView({ avisar }: { avisar: Avisar }) {
  const [seccion, setSeccion] = useState<Seccion>("Nueva venta");
  const secciones: Seccion[] = ["Nueva venta", "Ventas", "Anular venta", "Garantías"];
  return (
    <>
      <Pestanas items={secciones} activa={seccion} onCambiar={setSeccion} />
      {seccion === "Nueva venta" && <NuevaVenta avisar={avisar} />}
      {seccion === "Ventas" && <ListaVentas avisar={avisar} />}
      {seccion === "Anular venta" && <AnularVenta avisar={avisar} />}
      {seccion === "Garantías" && <Garantias avisar={avisar} />}
    </>
  );
}
