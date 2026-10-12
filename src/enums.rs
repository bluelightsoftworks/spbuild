use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Clone, Default)]
#[repr(u8)]
pub enum CppVersion{
    CPP14 = 14,
    CPP17 = 17,
    #[default]
    CPP20 = 20
}