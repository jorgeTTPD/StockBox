import { useEffect, useState } from "react";
import { api } from "../api";
import { dinero, errorTexto, fechaCorta } from "../formato";
import type { Entrada, Producto, Proveedor } from "../types";
import {
  Campo,
  DataTable,
  Etiqueta,
  Pestanas,
  Tarjeta,
  type Avisar,
  type Columna,
} from "../components/UI";

type Seccion = "Inventario" | "Entradas" | "Nueva entrada" | "Importar entradas";

const columnasProducto: Columna<Producto>[] = [
  { titulo: "ID", valor: (p) => p.id_producto, ancho: "110px" },
  { titulo: "Producto", valor: (p) => p.nombre },
  { titulo: "Marca", valor: (p) => p.marca },
  { titulo: "Categoría", valor: (p) => p.categoria },
  {
    titulo: "P. entrada",
    valor: (p) => <span className="num">{dinero(p.precio_entrada)}</span>,
    ancho: "110px",
  },
  {
    titulo: "P. venta",
    valor: (p) => <span className="num">{dinero(p.precio_venta)}</span>,
    ancho: "110px",
  },
  {
    titulo: "Stock",
    valor: (p) => (
      <span className="num">
        {p.stock <= 5 ? <Etiqueta tono="rojo">{p.stock}</Etiqueta> : p.stock}
      </span>
    ),
    ancho: "80px",
  },
];

function Inventario({ avisar }: { avisar: Avisar }) {
  const [termino, setTermino] = useState("");
  const [productos, setProductos] = useState<Producto[]>([]);
  const [criticos, setCriticos] = useState<Producto[]>([]);

  const cargar = async () => {
    try {
      setProductos(await api.listarProductos(termino || undefined));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  useEffect(() => {
    void cargar();
    api.productosCriticos().then(setCriticos).catch((error) => avisar(errorTexto(error), "error"));
  }, []);

  return (
    <div className="rejilla">
      <Tarjeta
        titulo="Stock crítico (5 unidades o menos)"
        accion={<Etiqueta tono="rojo">{criticos.length} productos</Etiqueta>}
      >
        <DataTable
          filas={criticos}
          columnas={columnasProducto.slice(0, 2).concat(columnasProducto.slice(6))}
          vacio="No hay productos en nivel crítico"
        />
      </Tarjeta>
      <Tarjeta titulo="Inventario">
        <div className="barra">
          <input
            placeholder="Buscar por ID, nombre, marca o categoría"
            value={termino}
            onChange={(e) => setTermino(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && cargar()}
          />
          <button className="secundario" onClick={cargar}>Buscar</button>
        </div>
        <DataTable filas={productos} columnas={columnasProducto} />
      </Tarjeta>
    </div>
  );
}

function ListaEntradas({ avisar }: { avisar: Avisar }) {
  const [entradas, setEntradas] = useState<Entrada[]>([]);
  const [fecha, setFecha] = useState("");

  useEffect(() => {
    api.listarEntradas(fecha || undefined)
      .then(setEntradas)
      .catch((error) => avisar(errorTexto(error), "error"));
  }, [fecha]);

  const columnas: Columna<Entrada>[] = [
    { titulo: "ID", valor: (e) => e.id_entrada, ancho: "130px" },
    { titulo: "Producto", valor: (e) => e.nombre_producto },
    { titulo: "Cantidad", valor: (e) => <span className="num">{e.cantidad}</span>, ancho: "90px" },
    {
      titulo: "P. unitario",
      valor: (e) => <span className="num">{dinero(e.precio_unitario)}</span>,
      ancho: "110px",
    },
    { titulo: "Proveedor", valor: (e) => e.id_proveedor || "—", ancho: "110px" },
    { titulo: "Fecha", valor: (e) => fechaCorta(e.fecha_entrada), ancho: "110px" },
  ];

  return (
    <Tarjeta titulo="Entradas registradas">
      <div className="barra">
        <input type="date" value={fecha} onChange={(e) => setFecha(e.target.value)} />
        {fecha && <button className="neutro" onClick={() => setFecha("")}>Limpiar</button>}
      </div>
      <DataTable filas={entradas} columnas={columnas} />
    </Tarjeta>
  );
}

function NuevaEntrada({ avisar }: { avisar: Avisar }) {
  const [termino, setTermino] = useState("");
  const [productos, setProductos] = useState<Producto[]>([]);
  const [producto, setProducto] = useState<Producto | null>(null);
  const [cantidad, setCantidad] = useState(1);
  const [precio, setPrecio] = useState(0);
  const [proveedores, setProveedores] = useState<Proveedor[]>([]);
  const [proveedor, setProveedor] = useState("");
  const [cargando, setCargando] = useState(false);

  useEffect(() => {
    api.listarProveedores()
      .then((lista) => {
        setProveedores(lista);
        setProveedor(lista[0]?.id_proveedor ?? "");
      })
      .catch((error) => avisar(errorTexto(error), "error"));
  }, []);

  const buscar = async () => {
    try {
      setProductos(await api.listarProductos(termino || undefined));
    } catch (error) {
      avisar(errorTexto(error), "error");
    }
  };

  const seleccionar = (productoSeleccionado: Producto) => {
    setProducto(productoSeleccionado);
    setPrecio(productoSeleccionado.precio_entrada);
  };

  const registrar = async () => {
    if (!producto) {
      avisar("Selecciona un producto", "error");
      return;
    }
    setCargando(true);
    try {
      const id = await api.registrarEntrada(producto.id_producto, cantidad, precio, proveedor || undefined);
      avisar(`Entrada ${id} registrada`, "ok");
      setProducto(null);
      setCantidad(1);
    } catch (error) {
      avisar(errorTexto(error), "error");
    } finally {
      setCargando(false);
    }
  };

  return (
    <div className="rejilla dos">
      <Tarjeta titulo="Seleccionar producto">
        <div className="barra">
          <input
            placeholder="Buscar producto"
            value={termino}
            onChange={(e) => setTermino(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && buscar()}
          />
          <button className="secundario" onClick={buscar}>Buscar</button>
        </div>
        <DataTable
          filas={productos}
          columnas={[
            { titulo: "ID", valor: (p) => p.id_producto, ancho: "110px" },
            { titulo: "Producto", valor: (p) => p.nombre },
            { titulo: "Stock", valor: (p) => <span className="num">{p.stock}</span>, ancho: "70px" },
            {
              titulo: "",
              valor: (p) => (
                <button className="mini" onClick={() => seleccionar(p)}>Elegir</button>
              ),
              ancho: "70px",
            },
          ]}
          vacio="Busca un producto existente"
        />
      </Tarjeta>
      <Tarjeta titulo="Datos de la entrada">
        {producto ? (
          <div className="lista-simple" style={{ marginBottom: 16 }}>
            <div><strong>{producto.nombre}</strong> ({producto.id_producto})</div>
            <div>Stock actual: {producto.stock}</div>
          </div>
        ) : (
          <p className="texto-suave">Elige un producto de la lista.</p>
        )}
        <div className="formulario">
          <Campo etiqueta="Cantidad">
            <input type="number" min={1} value={cantidad} onChange={(e) => setCantidad(Number(e.target.value))} />
          </Campo>
          <Campo etiqueta="Precio unitario">
            <input type="number" min={0} step="0.01" value={precio} onChange={(e) => setPrecio(Number(e.target.value))} />
          </Campo>
          <div className="completo">
            <Campo etiqueta="Proveedor">
              <select value={proveedor} onChange={(e) => setProveedor(e.target.value)}>
                {proveedores.map((p) => (
                  <option key={p.id_proveedor} value={p.id_proveedor}>
                    {p.nombre}
                  </option>
                ))}
              </select>
            </Campo>
          </div>
        </div>
        <div className="acciones">
          <button className="primario" onClick={registrar} disabled={cargando || !producto}>
            Registrar entrada
          </button>
        </div>
      </Tarjeta>
    </div>
  );
}

function ImportarEntradas({ avisar }: { avisar: Avisar }) {
  const [cargando, setCargando] = useState(false);

  const importar = async () => {
    try {
      const ruta = await api.elegirArchivo();
      if (!ruta) return;
      setCargando(true);
      const cantidad = await api.importarEntradas(ruta);
      avisar(`${cantidad} entrada(s) importadas`, "ok");
    } catch (error) {
      avisar(errorTexto(error), "error");
    } finally {
      setCargando(false);
    }
  };

  return (
    <Tarjeta titulo="Importar entradas desde CSV">
      <p className="texto-suave">
        Columnas requeridas: idProveedor, idProducto, nombreProducto, marca, categoria, precio y cantidad.
        Los productos nuevos se crean con precio de venta = precio × 1.3.
      </p>
      <button className="primario" onClick={importar} disabled={cargando}>
        {cargando ? "Importando…" : "Seleccionar archivo CSV"}
      </button>
    </Tarjeta>
  );
}

export default function AlmacenView({ avisar }: { avisar: Avisar }) {
  const [seccion, setSeccion] = useState<Seccion>("Inventario");
  const secciones: Seccion[] = ["Inventario", "Entradas", "Nueva entrada", "Importar entradas"];
  return (
    <>
      <Pestanas items={secciones} activa={seccion} onCambiar={setSeccion} />
      {seccion === "Inventario" && <Inventario avisar={avisar} />}
      {seccion === "Entradas" && <ListaEntradas avisar={avisar} />}
      {seccion === "Nueva entrada" && <NuevaEntrada avisar={avisar} />}
      {seccion === "Importar entradas" && <ImportarEntradas avisar={avisar} />}
    </>
  );
}
