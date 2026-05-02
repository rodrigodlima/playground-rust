pub fn dobro(x: i32) -> i32 { x * 2 }

fn main() {

#[cfg(test)]
mod tests {
    use super::*; // importa tudo do módulo pai

    #[test]
    fn dobro_de_dois_e_quatro() {
        assert_eq!(dobro(2), 4);
    }

    #[test]
    fn dobro_de_zero() {
        assert_eq!(dobro(0), 0);
    }
}
}
