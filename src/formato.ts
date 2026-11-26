export const dinero = (valor: number) =>
  new Intl.NumberFormat("es-BO", { style: "currency", currency: "USD" }).format(valor);

export const fechaCorta = (valor: string) => valor.slice(0, 10) || "—";

export const fechaHora = (valor: string) => valor.replace("T", " ").slice(0, 19) || "—";

export const errorTexto = (error: unknown) =>
  error instanceof Error ? error.message : String(error);
