use bevy::{input::common_conditions::input_just_pressed, log::LogPlugin, prelude::*};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use polyventory::prelude::*;
use rand::seq::IndexedRandom;

fn main() {
    let mut app = App::new();
    let mut filter = bevy::log::DEFAULT_FILTER.to_string();
    filter.push_str("polyventory=trace,");
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: filter,
        ..Default::default()
    }));
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    app.add_plugins(polyventory::PolyventoryPlugin);
    app.add_plugins(polyventory::InventoryRenderPlugin {
        default_inventory_style: Some(polyventory::prelude::InventoryStyleAsset {
            cell_size: Vec2::new(10.0, 10.0),
            cell_icon: Some("bbg/ui/GUICell.png".to_string()),
            background: None,
        }),
        pipeline: InventoryRenderPipeline::Custom,
    });
    app.insert_resource(polyventory::prelude::ToolTipSettings {
        debug_info: true,
        ..Default::default()
    });
    app.add_plugins((polyventory::prelude::ToolTipPlugin,));
    app.init_resource::<LootTable>();
    app.add_systems(Startup, spawn_camera);
    app.add_systems(OnExit(Loaded::False), spawn_inventory);
    app.init_state::<Loaded>();
    app.add_systems(Update, check_loaded.run_if(in_state(Loaded::False)));
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
    items: Vec<Handle<ItemDescriptor>>,
}

impl FromWorld for LootTable {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();
        let items = vec![
            asset_server.load("items/bottle.item"),
            asset_server.load("items/bottle_water.item"),
            asset_server.load("items/battery_phone.item"),
            asset_server.load("items/phone_on.item"),
            asset_server.load("items/phone_off.item"),
        ];
        Self { items }
    }
}

fn spawn_inventory(
    mut commands: Commands,
    mut inventory_manager: InventoryManager,
    loot: Res<LootTable>,
) {
    let (test_inventory_handle, mut test_inventory) =
        inventory_manager.create_inventory("Test Inventory");
    test_inventory.add_slot(
        CellType::Untyped,
        Shape {
            offset: IVec2::ZERO,
            orientation: Orientation::DEG0,
            layout: Layout::Rect {
                size: UVec2::new(5, 7),
            },
        },
    );
    let empty_bottle = loot.items[0].clone();
    let r = test_inventory.spawn_item(empty_bottle);
    info!("Spawning empty bottle: {:?}", r);
    let water_bottle = loot.items[1].clone();
    let r = test_inventory.spawn_item_at(water_bottle.clone(), IVec2::new(1, 8), Orientation::DEG0);
    info!(
        "Spawning water bottle at 1,8 with identity orientation: {:?}",
        r
    );
    let r = test_inventory.spawn_item_at(water_bottle, IVec2::new(1, 8), Orientation::DEG270);
    info!("Spawning water bottle at 1,8 with 270 rotation: {:?}", r);

    let mut rng = rand::rng();
    for _ in 0..15 {
        let item = loot
            .items
            .choose(&mut rng)
            .expect("At least one item")
            .clone();
        match test_inventory.spawn_item(item) {
            Ok(item) => info!("Spawned random item: {:?}", item),
            Err(f) => error!("Failed to spawn random item: {:?}", f),
        }
    }

    // let inventory = inventorys.add(test_inventory);
    commands.spawn((
        RenderedInventory::new(test_inventory_handle.clone()),
        Node {
            margin: UiRect::all(Val::Auto),
            left: Val::Px(100.0),
            ..Default::default()
        },
        InventoryNode,
    ));

    commands.spawn((
        RenderedInventory::new(test_inventory_handle.clone()),
        InventorySprite,
    ));
}
