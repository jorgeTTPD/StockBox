pub fn texto_valido(valor: &str) -> bool {
    let texto = valor.trim();
    !texto.is_empty()
        && texto.chars().all(|caracter| {
            caracter.is_alphanumeric() || matches!(caracter, ' ' | '\'' | '-' | '.')
        })
        && texto.chars().next().is_some_and(char::is_alphanumeric)
        && texto.chars().any(char::is_alphabetic)
}

pub fn identificador_valido(valor: &str) -> bool {
    let texto = valor.trim();
    texto.len() >= 2
        && texto
            .chars()
            .all(|caracter| caracter.is_ascii_alphanumeric() || caracter == '-')
}

pub fn telefono_valido(valor: &str) -> bool {
    let digitos: String = valor.chars().filter(char::is_ascii_digit).collect();
    (7..=15).contains(&digitos.len())
}

pub fn contrasena_valida(valor: &str) -> bool {
    valor.len() >= 6
}

pub fn fluido(valor: f64) -> bool {
    valor.is_finite()
}
