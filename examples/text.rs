use std::io::Write;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(First, serde_example);
    app.add_systems(Last, exit);
    app.run();
}

fn serde_example(type_repo: Res<AppTypeRegistry>) {
    use serde::Serialize;
    let repo = type_repo.read();

    let t = Text::new("Test");
    let mut out = String::new();

    let s = bevy::reflect::serde::TypedReflectSerializer::new(t.as_partial_reflect(), &repo);
    let mut ser = ron::Serializer::new(&mut out, Some(ron::ser::PrettyConfig::new())).unwrap();
    _ = s.serialize(&mut ser);
    println!("{}", out)
}

fn exit(mut m: MessageWriter<AppExit>) {
    m.write(AppExit::Success);
}