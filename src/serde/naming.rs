use bevy::reflect::{ArrayInfo, EnumInfo, ListInfo, MapInfo, OpaqueInfo, SetInfo, StructInfo, TupleInfo, TupleStructInfo};

pub trait HasName {
    fn name(&self) -> &str;
}


macro_rules! impl_info_name {
    ($info: ty) => {
        impl HasName for &$info {
            fn name(&self) -> &str {
                self.ty().short_path()
            }
        }
        impl HasName for $info {
            fn name(&self) -> &str {
                self.ty().short_path()
            }
        }
    };
}

impl_info_name!(StructInfo);
impl_info_name!(ArrayInfo);
impl_info_name!(ListInfo);
impl_info_name!(OpaqueInfo);
impl_info_name!(MapInfo);
impl_info_name!(SetInfo);
impl_info_name!(TupleInfo);
impl_info_name!(TupleStructInfo);
impl_info_name!(EnumInfo);