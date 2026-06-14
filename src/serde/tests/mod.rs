use bevy::{ecs::entity::Entity, platform::collections::{HashMap, HashSet}, reflect::{PartialReflect, Reflect, TypeRegistryArc}};

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
            set: HashSet::from([0, 0, -1, 1, 10, 15, -2]),
            map: HashMap::from([(0, 1), (-1, 2), (1, 3), (10, 4), (15, 5), (-2, 6)]),
            entity: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Default, Reflect, PartialEq)]
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

    assert_eq!(test_str, load_test_file!("struct.comp"), "{} != {}", test_str, include_str!("struct.comp"))
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
        type_repo: repo
    };
    let enums = vec![
        TestEnum::C,
        TestEnum::Struct { a: 2, b: 10. },
        TestEnum::Tuple(3, 15.),
        ];
        ser.serialize_reflect(enums.as_reflect()).expect("to work");
        assert_eq!(test_str, load_test_file!("enum.comp"));
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
    
    let mut de = super::ComponentDe {
        type_repo: repo
    };

    let v = de.deserialize_str(&test_str);
    assert_eq!(enums.reflect_partial_eq(v.as_partial_reflect()), Some(true));
}

macro_rules! serde_test {
    ($input: expr, $repo: ident) => {
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
        let v = de.deserialize_str(&data);
        assert_eq!(t.reflect_partial_eq(v.as_partial_reflect()), Some(true));
    };
}

#[test]
fn serde_opaque() {
    let repo = TypeRegistryArc::default();
    let mut r = repo.write();
    r.register::<TestStruct>();
    r.register::<[TestEnum; 3]>();
    r.register::<Vec<TestEnum>>();
    drop(r);
    
    serde_test!(2i32, repo);
    serde_test!(2u32, repo);
    serde_test!(2f32, repo);
    serde_test!("two", repo);
    serde_test!(String::from("two"), repo);
    serde_test!('2', repo);
    serde_test!((), repo);
    serde_test!(true, repo);
    serde_test!(false, repo);
}