import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "./api";
import Login from "./components/Login";
import type { Avisar, TipoAviso } from "./components/UI";
import type { Rol, Sesion } from "./types";
import AdminView from "./views/AdminView";
import AlmacenView from "./views/AlmacenView";
import VendedorView from "./views/VendedorView";
import "./styles.css";

const titulos: Record<Rol, { titulo: string; descripcion: string }> = {
  ADMINISTRADOR: {
    titulo: "Panel de administración",
    descripcion: "Caja, inventario, personal, proveedores y reportes",
  },
  CAJERO: {
    titulo: "Punto de venta",
    descripcion: "Carrito, proformas, ventas, devoluciones y garantías",
  },
  ALMACENERO: {
    titulo: "Almacén",
    descripcion: "Inventario, existencias y recepción de mercadería",
  },
};

export default function App() {
  const [sesion, setSesion] = useState<Sesion | null>(null);
  const [listo, setListo] = useState(false);
  const [temaOscuro, setTemaOscuro] = useState(
    () => localStorage.getItem("stockbox-tema") !== "claro",
  );
  const [aviso, setAviso] = useState<{ mensaje: string; tipo: TipoAviso } | null>(null);
  const temporizador = useRef<number | null>(null);

  useEffect(() => {
    document.documentElement.dataset.tema = temaOscuro ? "oscuro" : "claro";
  }, [temaOscuro]);

  useEffect(() => {
    api.sesionActual()
      .then((actual) => {
        setSesion(actual);
        if (actual) {
          api.preferencias()
            .then((preferencias) => setTemaOscuro(preferencias.tema_oscuro))
            .catch(() => {});
        }
      })
      .catch(() => setSesion(null))
      .finally(() => setListo(true));
  }, []);

  const avisar = useCallback<Avisar>((mensaje, tipo = "ok") => {
    setAviso({ mensaje, tipo });
    if (temporizador.current !== null) window.clearTimeout(temporizador.current);
    temporizador.current = window.setTimeout(() => setAviso(null), 4500);
  }, []);

  const cambiarTema = () => {
    const nuevo = !temaOscuro;
    setTemaOscuro(nuevo);
    localStorage.setItem("stockbox-tema", nuevo ? "oscuro" : "claro");
    if (sesion) api.guardarPreferencias(nuevo).catch(() => {});
  };

  const salir = async () => {
    try {
      await api.cerrarSesion();
      setSesion(null);
    } catch (error) {
      avisar(String(error), "error");
    }
  };

  if (!listo) return <div className="vacio">Cargando StockBox…</div>;
  if (!sesion) {
    return <Login onLogin={setSesion} avisar={avisar} temaOscuro={temaOscuro} cambiarTema={cambiarTema} />;
  }

  const info = titulos[sesion.rol];
  return (
    <div className="app">
      <aside className="barra-lateral">
        <div className="marca">▣ <span>StockBox</span></div>
        <div className="usuario">
          <strong>{sesion.nombre}</strong>
          <small>{sesion.ci}</small>
          <br />
          <span className={`etiqueta ${sesion.rol === "ADMINISTRADOR" ? "amarillo" : "azul"}`}>
            {sesion.rol}
          </span>
        </div>
        <nav className="menu">
          <button className="activo">▦ Panel principal</button>
          <button onClick={cambiarTema}>{temaOscuro ? "☀ Tema claro" : "☾ Tema oscuro"}</button>
          <button onClick={() => avisar("StockBox v0.1.0 · Tauri + Rust + SQLite")}>ⓘ Acerca de</button>
        </nav>
        <button className="salir secundario" onClick={salir}>Cerrar sesión</button>
      </aside>
      <main className="principal">
        <header className="cabecera-pagina">
          <div>
            <h1>{info.titulo}</h1>
            <p>{info.descripcion}</p>
          </div>
          <span className="etiqueta verde">SQLite activo</span>
        </header>
        {sesion.rol === "ADMINISTRADOR" && <AdminView avisar={avisar} />}
        {sesion.rol === "CAJERO" && <VendedorView avisar={avisar} />}
        {sesion.rol === "ALMACENERO" && <AlmacenView avisar={avisar} />}
      </main>
      {aviso && <div className={`aviso ${aviso.tipo}`}>{aviso.mensaje}</div>}
    </div>
  );
}
