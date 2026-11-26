import type { ReactNode } from "react";

export type TipoAviso = "ok" | "error";
export type Avisar = (mensaje: string, tipo?: TipoAviso) => void;

export interface Columna<T> {
  titulo: string;
  valor: (fila: T) => ReactNode;
  ancho?: string;
}

export function DataTable<T>({
  filas,
  columnas,
  vacio = "Sin registros",
}: {
  filas: T[];
  columnas: Columna<T>[];
  vacio?: string;
}) {
  if (filas.length === 0) {
    return <div className="vacio">{vacio}</div>;
  }
  return (
    <div className="tabla-contenedor">
      <table className="tabla">
        <thead>
          <tr>
            {columnas.map((columna, indice) => (
              <th key={indice} style={columna.ancho ? { width: columna.ancho } : undefined}>
                {columna.titulo}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {filas.map((fila, indice) => (
            <tr key={indice}>
              {columnas.map((columna, columnaIndice) => (
                <td key={columnaIndice}>{columna.valor(fila)}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function Modal({
  titulo,
  onCerrar,
  children,
}: {
  titulo: string;
  onCerrar: () => void;
  children: ReactNode;
}) {
  return (
    <div className="telon" onClick={onCerrar}>
      <div className="modal" onClick={(evento) => evento.stopPropagation()}>
        <div className="modal-cabecera">
          <h2>{titulo}</h2>
          <button className="icono" onClick={onCerrar} aria-label="Cerrar">
            ×
          </button>
        </div>
        <div className="modal-cuerpo">{children}</div>
      </div>
    </div>
  );
}

export function Campo({
  etiqueta,
  children,
}: {
  etiqueta: string;
  children: ReactNode;
}) {
  return (
    <label className="campo">
      <span>{etiqueta}</span>
      {children}
    </label>
  );
}

export function Tarjeta({
  titulo,
  accion,
  children,
}: {
  titulo: string;
  accion?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="tarjeta">
      <div className="tarjeta-cabecera">
        <h2>{titulo}</h2>
        {accion}
      </div>
      <div className="tarjeta-cuerpo">{children}</div>
    </section>
  );
}

export function Pestanas<T extends string>({
  items,
  activa,
  onCambiar,
}: {
  items: T[];
  activa: T;
  onCambiar: (valor: T) => void;
}) {
  return (
    <div className="pestanas">
      {items.map((item) => (
        <button
          key={item}
          className={item === activa ? "pestana activa" : "pestana"}
          onClick={() => onCambiar(item)}
        >
          {item}
        </button>
      ))}
    </div>
  );
}

export function Etiqueta({ children, tono = "neutro" }: { children: ReactNode; tono?: string }) {
  return <span className={`etiqueta ${tono}`}>{children}</span>;
}
