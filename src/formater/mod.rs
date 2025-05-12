pub trait ToCbmlCode {
    fn to_cbml_code(&self, deepth: usize) -> String;
}

