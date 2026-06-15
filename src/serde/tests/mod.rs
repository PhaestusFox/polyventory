use bevy::{ecs::entity::Entity, math::Vec3, platform::collections::{HashMap, HashSet}, reflect::{PartialReflect, Reflect, TypeRegistryArc}, transform::components::Transform};

fn get_repo() -> TypeRegistryArc {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<Entity>();
    r.register::<(u32,)>();
    r.register::<(f32, u32)>();
    r.register::<(u32, f32, String)>();
    drop(r);
    repo
}

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
    set: HashSet<i8>,
    map: HashMap<i8, i8>,
    entity: Entity,
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
            tuple_struct: TestTupleStruct::default(),
            set: HashSet::from([0, 0, 0, 0, 0]),
            map: HashMap::from([(0, 1), (0, 2), (0, 3)]),
            entity: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Default, Reflect, PartialEq, Debug)]
enum TestEnum {
    A,
    #[default]
    B,
    C,
    Struct {
        a: u32,
        b: f32,
    },
    Tuple(u32, f32),
}

#[derive(Default, Reflect)]
struct ComplexStruct {
    int: u32,
    str: String,
    float: f32,
    bool: bool,
}

#[derive(Reflect)]
struct NewTypeStruct(f32);

#[derive(Reflect)]
struct TupleStruct(f32, u32);

macro_rules! serializer_test {
    ($repo: ident, $input: expr, $result: literal) => {
        let mut data = String::default();
        let t = $input;
        let mut ser = super::ComponentSer {
            file: &mut data,
            type_repo: $repo.clone(),
        };
        ser.serialize_reflect(t.as_reflect()).unwrap();
        assert_eq!(data, $result)
    };
    ($repo: ident, $input: expr, $result: pat) => {
        let mut data = String::default();
        let t = $input;
        let mut ser = super::ComponentSer {
            file: &mut data,
            type_repo: $repo.clone(),
        };
        let $result = ser.serialize_reflect(t.as_reflect()) else {
            panic!("Failed to Match pattern");
        };
    };
}

#[test]
fn serializer_all() {
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

    assert_eq!(test_str, load_test_file!("struct.comp"), "{} != {}", test_str, include_str!("struct.comp"))
}

#[test]
fn serializer_opaque() {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<Entity>();
    drop(r);

    serializer_test!(repo, 2i32, "i32: 2");
    serializer_test!(repo, 2u32, "u32: 2");
    serializer_test!(repo, 2f32, "f32: 2.0");
    serializer_test!(repo, String::from("two"), "String: \"two\"");
    serializer_test!(repo, '2', "char: '2'");
    serializer_test!(repo, (), "()()");
    serializer_test!(repo, true, "bool: true");
    serializer_test!(repo, false, "bool: false");
    serializer_test!(repo, Entity::from_bits(3), "Entity: 3");
    serializer_test!(repo, Entity::PLACEHOLDER, "Entity: 1");
    serializer_test!(repo, "two", Err(super::SerdeError::OpaqueNotSerde("&str")));
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

    assert_eq!(test_str, load_test_file!("marker.comp"), "{} != {}", test_str, load_test_file!("marker.comp"))
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

#[test]
fn serializer_enum() {
    let mut test_str = String::default();
    
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<[TestEnum; 3]>();
    r.register::<Vec<TestEnum>>();
    drop(r);
    
    let mut ser = super::ComponentSer {
        file: &mut test_str,
        type_repo: repo.clone(),
    };
    let enums = [
        TestEnum::C,
        TestEnum::Struct { a: 2, b: 10. },
        TestEnum::Tuple(3, 15.),
    ];
    ser.serialize_reflect(enums.as_reflect()).expect("to work");

    assert_eq!(test_str, load_test_file!("enum.comp"));
    
    test_str.clear();
    
    let mut ser = super::ComponentSer {
        file: &mut test_str,
        type_repo: repo.clone()
    };
    let enums = vec![
        TestEnum::C,
        TestEnum::Struct { a: 2, b: 10. },
        TestEnum::Tuple(3, 15.),
        ];
    ser.serialize_reflect(enums.as_reflect()).expect("to work");
    assert!(test_str.starts_with("Vec<TestEnum>["));
    assert_eq!(test_str.replace("Vec<TestEnum>", "[TestEnum; 3]"), load_test_file!("enum.comp"));

    serializer_test!(repo, TestEnum::B, "TestEnum: B");
}
    
macro_rules! serde_test {
    ($repo: ident, $input: expr) => {
        let mut data = String::default();
        let t = $input;
        let mut ser = super::ComponentSer {
            file: &mut data,
            type_repo: $repo.clone(),
        };
        ser.serialize_reflect(t.as_reflect()).unwrap();
        let de = super::ComponentDe {
            type_repo: $repo.clone()
        };
        let v = de.deserialize_str(&data).unwrap();
        assert_eq!(t.reflect_partial_eq(v.as_partial_reflect()), Some(true), "{:?} != {:?}", v, t.as_partial_reflect());
    };
}
    
#[test]
fn serde_enum() {
    let mut test_str = String::default();
    
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<[TestEnum; 3]>();
    r.register::<Vec<TestEnum>>();
    drop(r);
    
    serde_test!(repo, TestEnum::C);
    serde_test!(repo, TestEnum::Struct { a: 2, b: 10. });
    serde_test!(repo, TestEnum::Tuple(3, 15.));
    serde_test!(repo, [
        TestEnum::C,
        TestEnum::Struct { a: 2, b: 10. },
        TestEnum::Tuple(3, 15.),
    ]);
}
    
#[test]
fn serde_opaque() {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<[TestEnum; 3]>();
    r.register::<Vec<TestEnum>>();
    r.register::<Entity>();
    drop(r);

    serde_test!(repo, 2i32);
    serde_test!(repo, 2u32);
    serde_test!(repo, 2f32);
    serde_test!(repo, String::from("two"));
    serde_test!(repo, '2');
    serde_test!(repo, true);
    serde_test!(repo, false);
    serde_test!(repo, Entity::from_bits(3));
}

#[test]
fn serde_all() {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<Entity>();
    drop(r);

    serde_test!(repo, TestStruct::default());
}

#[test]
fn serde_struct() {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<Vec3>();
    r.register::<Transform>();
    r.register::<NewTypeStruct>();
    r.register::<TupleStruct>();
    r.register::<Entity>();
    drop(r);

    serde_test!(repo, TestMarker);
    serde_test!(repo, Vec3::ZERO);
    serde_test!(repo, Transform::IDENTITY);
    serde_test!(repo, ComplexStruct::default());
    serde_test!(repo, NewTypeStruct(10.));
    serde_test!(repo, TupleStruct(5., 1));
}

#[test]
fn serde_tuple() {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<Entity>();
    r.register::<(u32,)>();
    r.register::<(f32, u32)>();
    r.register::<(u32, f32, String)>();
    drop(r);

    serde_test!(repo, ());
    serde_test!(repo, (1u32,));
    serde_test!(repo, (1.0f32,2u32));
    serde_test!(repo, (1u32, 2.0f32, String::from("Test")));
}

#[test]
fn bevy_test() {
    let repo = get_repo();
    let repo = repo.read();
    let test = TestStruct::default();
    let serde = bevy::reflect::serde::ReflectSerializer::new(test.as_partial_reflect(), &repo);
    let r = ron::ser::to_string_pretty(&serde, ron::ser::PrettyConfig::new()).unwrap();
    panic!("{}", r);
}