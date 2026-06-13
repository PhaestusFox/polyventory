use bevy::reflect::{Reflect, TypeRegistryArc};

macro_rules! load_test_file {
    ($path: literal) => {
        include_str!($path).replace("\r\n", "\n")
    };
}

#[derive(Reflect)]
struct TestMarker;

#[derive(Reflect)]
struct TestTupleStruct(u32, f32, i32);

impl Default for TestTupleStruct {
    fn default() -> Self {
        TestTupleStruct(2, 4., -8)
    }
}

#[derive(Reflect)]
struct TestStruct {
    int: u32,
    str: String,
    float: f32,
    bool: bool,
    complex: ComplexStruct,
    enum_b: TestEnum,
    marker: TestMarker,
    tuple_struct: TestTupleStruct,
}

impl Default for TestStruct {
    fn default() -> Self {
        TestStruct {
            int: 10,
            str: String::from("Test Struct"),
            float: 420.69,
            bool: true,
            complex: ComplexStruct::default(),
            enum_b: TestEnum::default(),
            marker: TestMarker,
            tuple_struct: TestTupleStruct::default()
        }
    }
}

#[derive(Default, Reflect)]
enum TestEnum {
    A,
    #[default]
    B,
    C
}

#[derive(Default, Reflect)]
struct ComplexStruct {
    int: u32,
    str: String,
    float: f32,
    bool: bool,
}


#[test]
fn serializer_test() {
    let obj = TestStruct::default();
    let mut test_str = String::default();

    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    drop(r);

    let mut ser = super::ComponentSer {
        file: &mut test_str,
        type_repo: repo
    };

    ser.serialize_reflect(obj.as_reflect()).expect("to serialize");

    assert_eq!(test_str, include_str!("struct.comp"))
}

#[test]
fn serializer_marker() {
    let obj = TestMarker;
    let mut test_str = String::default();

    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestMarker>();
    drop(r);

    let mut ser = super::ComponentSer {
        file: &mut test_str,
        type_repo: repo
    };

    ser.serialize_reflect(obj.as_reflect()).expect("to serialize");

    assert_eq!(test_str, include_str!("marker.comp"))
}

#[test]
fn serializer_tuple_struct() {
    let obj = TestTupleStruct::default();
    let mut test_str = String::default();

    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestTupleStruct>();
    drop(r);

    let mut ser = super::ComponentSer {
        file: &mut test_str,
        type_repo: repo
    };

    ser.serialize_reflect(obj.as_reflect()).expect("to serialize");

    assert_eq!(test_str, load_test_file!("tuple_struct.comp"))
}