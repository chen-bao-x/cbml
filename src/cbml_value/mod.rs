use value::CbmlType;

pub mod value;

pub trait ToCbmlType {
    fn to_cbml_type(&self) -> CbmlType;
}
