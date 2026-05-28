use std::path::Path;

use bevy::{asset::AssetPath, input::common_conditions::input_just_pressed, log::LogPlugin, prelude::*};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use polyventory::prelude::*;
use rand::{RngExt, seq::IndexedRandom};

fn main() {
    let mut app = App::new();
    let mut filter = bevy::log::DEFAULT_FILTER.to_string();
    filter.push_str("polyventory=trace,");
    app.add_plugins(DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(LogPlugin {
        filter: filter,
        ..Default::default()
    }));
    app.add_plugins(EguiPlugin::default())
    .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    app.add_plugins(polyventory::PolyventoryPlugin)
    .add_plugins(polyventory::InventoryRenderPlugin {
        default_inventory_style: Some(polyventory::prelude::InventoryStyleAsset {
            cell_size: Vec2::new(10.0, 10.0),
            cell_icon: Some("bbg/ui/GUICell.png".to_string()),
            background: None,
        }),
        pipeline: InventoryRenderPipeline::Node,
    });
    app.insert_resource(polyventory::prelude::ToolTipSettings {
        debug_info: true,
        ..Default::default()
    });
    app.add_plugins((
        // polyventory::prelude::MouseInventoryPlugin,
        polyventory::prelude::ToolTipPlugin,
    ));
    app.init_resource::<LootTable>();
    app.add_systems(Startup, spawn_camera);
    app.add_systems(OnExit(Loaded::False), spawn_inventory);
    app.init_state::<Loaded>();
    app.add_systems(Update, check_loaded.run_if(in_state(Loaded::False)));
    app.add_plugins(bevy_inspector_egui::quick::AssetInspectorPlugin::<polyventory::prelude::Inventory>::default());
    

    app.add_systems(Update, kill);
    app.add_systems(Update, detect_drop);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn check_loaded(
    mut state: ResMut<NextState<Loaded>>,
    loot: Res<LootTable>,
    asset_server: Res<AssetServer>,
) {
    info!("Checking if loot items are loaded...");
    for item in &loot.items {
        match asset_server.get_load_state(item.id()) {
            Some(bevy::asset::LoadState::Loading) => {
                info!("Item {:?} is still loading", item);
                return;
            }
            _ => {}
        }
    }
    info!("All Items loaded");
    state.set(Loaded::True);
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum Loaded {
    #[default]
    False,
    True,
}

#[derive(Resource)]
pub struct LootTable {
    fixed: Vec<Handle<ItemDescriptor>>,
    items: Vec<Handle<ItemDescriptor>>,
}

impl FromWorld for LootTable {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();
        let items = vec![
            asset_server.load("items/bottle.item"),
            asset_server.load("items/bottle_water.item"),
            asset_server.load("items/lighter.item"),
            asset_server.load("items/battery_phone.item"),
            asset_server.load("items/phone_on.item"),
            asset_server.load("items/phone_off.item"),
            asset_server.load("items/can.item"),
            asset_server.load("items/ketchup_packet.item"),
            asset_server.load("items/torch.item"),
            asset_server.load("items/backpack.item"),
            asset_server.load("items/backpack.item#SmallPocket"),
            asset_server.load("items/backpack.item#BigPocket"),
        ];
            let known = vec![
                asset_server.load("items/battery_phone.item"),
                asset_server.load("items/lighter.item"),
            ];
        Self { items, fixed: known }
    }
}

fn spawn_inventory(
    mut commands: Commands,
    mut inventory_manager: InventoryManager,
    loot: Res<LootTable>,
    mut styles: ResMut<Assets<InventoryStyle>>,
    asset_server: Res<AssetServer>,
) {
    let mut test_inventory = Inventory::new("Ui Inventory");
    test_inventory.add_slot(CellType::Untyped, Shape {
        offset: IVec2::ZERO,
        orientation: Orientation::Deg0,
        layout: Layout::Rect { size: UVec2::new(30, 50) },
    });
    let s = &mut inventory_manager;
    let test_inventory_handle = s.create_inventory(test_inventory);
    let mut test_inventory = s
        .open_inventory(&test_inventory_handle)
        .expect("Just created Inventory");
    let empty_bottle = loot.items[0].clone();
    let bottle = match test_inventory.spawn_item(empty_bottle.clone()) {
        Ok(item) => item,
        Err(f) => {
            panic!("Failed to spawn empty bottle: {:?}", f);
        }
    };
    match test_inventory.spawn_item_at(empty_bottle, IVec2::new(0, 0), Orientation::Deg180) {
        Ok(_) => {},
        Err(f) => {
            error!("Failed to spawn empty bottle: {:?}", f);
        }
    };
    _ = dbg!(test_inventory.remove_item(bottle));
    test_inventory.commands.entity(bottle).insert(Kill);
    let water_bottle = loot.items[1].clone();
    let r = test_inventory.spawn_item_at(
        water_bottle.clone(),
        IVec2::new(1, 8),
        Orientation::Deg270,
    );
    info!(
        "Spawning water bottle at 1,8 with identity orientation: {:?}",
        r
    );
    let r = test_inventory.spawn_item_at(water_bottle.clone(), IVec2::new(1, 8), Orientation::Deg90);
    info!("Spawning water bottle at 1,8 with 90 rotation: {:?}", r);
    let r = test_inventory.spawn_item_at(water_bottle.clone(), IVec2::new(1, 7), Orientation::Deg180);
    info!("Spawning water bottle at 1,7 with 180 rotation: {:?}", r);
    let r = test_inventory.spawn_item_at(water_bottle.clone(), IVec2::new(3, 9), Orientation::Deg270);
    info!("Spawning water bottle at 3,9 with 270 rotation: {:?}", r);

    let mut rng = rand::rng();
    let rotations = [Orientation::Deg90];
    let mut spawned = 0;
    while spawned < 15 {
        let item = loot.items.choose(&mut rng).expect("At least one item").clone();
        let orientation = *rotations.choose(&mut rng).expect("At least one orientation");
        match test_inventory.spawn_item_at(item, IVec2::new(rng.random_range(0..30), rng.random_range(0..50)), orientation) {
            Ok(_) => {
                spawned += 1;
            },
            Err(f) => error!("Failed to spawn random item: {:?}", f),
        }
    }
    let item = loot.fixed[0].clone();
    let container = test_inventory.commands.spawn(Name::new("fill")).id();
    while let Ok(e) = test_inventory.spawn_item(item.clone()) {
        test_inventory.commands.entity(e).insert(ChildOf(container));
        spawned += 1;
        if spawned > 30 * 50 {
            panic!("Too many items spawned, something is wrong");
        }
    }
    let item = loot.fixed[1].clone();
    while let Ok(e) = test_inventory.spawn_item(item.clone()) {
        test_inventory.commands.entity(e).insert(ChildOf(container));
        spawned += 1;
        if spawned > 30 * 50 {
            panic!("Too many items spawned, something is wrong");
        }
    }

    commands.spawn((
        Name::new("Player"),
        polyventory::prelude::ItemInventory(test_inventory_handle.clone()),
    ));

    let style = InventoryStyle {
        cell_size: Vec2::new(30.0, 30.0),
        cell_icon: asset_server.load("bbg/ui/GUICell.png"),
        background: None,
    };
    let style = styles.add(style);
    commands.spawn((
        RenderedInventory::new(test_inventory_handle.clone()),
        Name::new("Test Inventory Node"),
        Node {
            left: Val::Px(50.0),
            top: Val::Px(50.0),
            ..Default::default()
        },
    ));
    commands.spawn((
    RenderedInventory::new(test_inventory_handle.clone()),
    InventoryNode,
    Name::new("Test Inventory Node"),
    InventoryStyleHandle(style)));
}

#[derive(Component)]
pub struct Kill;

fn kill(mut commands: Commands, items: Populated<Entity, With<Kill>>) {
    for item in &items {
        commands.entity(item).despawn();
    }
}

fn detect_drop(
    mut messages: MessageReader<AssetEvent<Inventory>>,
) {
    for message in messages.read() {
        match message {
            AssetEvent::Removed { id } => info!("Received inventory asset event: {:?}", id),
            _ => {}
        }
    }
}