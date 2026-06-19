use bevy::prelude::*;
use bevy::{
    asset::UntypedAssetId,
    ecs::{
        schedule::{BoxedCondition, SystemCondition},
        system::IntoSystem,
        world::CommandQueue,
    },
};
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin,
    bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext},
    egui::{self, mutex::Mutex},
    reflect_inspector::{Context, InspectorUi},
    restricted_world_view::RestrictedWorldView,
};

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

pub struct InventoryInspectorPlugin {
    condition: Mutex<Option<BoxedCondition>>,
}

impl Default for InventoryInspectorPlugin {
    fn default() -> Self {
        Self {
            condition: Mutex::new(None),
        }
    }
}
impl InventoryInspectorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI of the specified condition is active
    pub fn run_if<M>(mut self, condition: impl SystemCondition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl Plugin for InventoryInspectorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        check_plugins(app, "AssetInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        // let condition = self.condition.lock().unwrap().take();
        let mut system = asset_inspector_ui.into_configs();
        // if let Some(condition) = condition {
        //     system.run_if_dyn(condition);
        // }
        app.add_systems(EguiPrimaryContextPass, system);
    }
}

fn asset_inspector_ui(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    bevy_inspector_egui::egui::Window::new("Inventory Asset Inspector")
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            bevy_inspector_egui::egui::ScrollArea::both().show(ui, |ui| {
                ui_for_assets::<crate::prelude::Inventory>(world, ui);

                ui.allocate_space(ui.available_size());
            });
        });
}

fn check_plugins(app: &App, name: &str) {
    if !app.is_plugin_added::<bevy::app::MainSchedulePlugin>() {
        panic!(
            r#"`{name}` should be added after the default plugins:
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins({name}::default())
            "#,
        );
    }

    if !app.is_plugin_added::<EguiPlugin>() {
        panic!(
            r#"`{name}` needs to be added after `EguiPlugin`:
        .add_plugins(EguiPlugin::default())
        .add_plugins({name}::default())
            "#,
        );
    }
}

/// Display all assets of the specified asset type `A`
pub fn ui_for_assets<A: Asset + Reflect>(
    world: &mut World,
    ui: &mut bevy_inspector_egui::egui::Ui,
) {
    let asset_server = world.get_resource::<AssetServer>().cloned();

    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `R` resource
    let Some((mut assets, world_view)) =
        RestrictedWorldView::new(world).split_off_resource_typed::<Assets<A>>()
    else {
        bevy_inspector_egui::bevy_inspector::errors::nonexistent_resource(ui, "Assets<Inventory>");
        return;
    };

    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(world_view),
        queue: Some(&mut queue),
    };

    let mut handles: Vec<_> = assets.ids().collect();
    handles.sort_by(|a, b| a.cmp(b));
    for handle_id in handles {
        let id = egui::Id::new(handle_id);
        let asset = assets
            .get_mut_untracked(handle_id)
            .expect("This is a list of all current IDs");
        if let Some(changed) =
            egui::CollapsingHeader::new(handle_name(handle_id.untyped(), asset_server.as_ref()))
                .id_salt(id)
                .show(ui, |ui| {
                    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);
                    env.ui_for_reflect_with_options(asset, ui, id, &())
                })
                .body_returned
        {
            if changed {
                assets.get_mut(handle_id); // reborrow the asset to mark it as changed
            }
        }
    }

    queue.apply(world);
}

fn handle_name(handle: UntypedAssetId, asset_server: Option<&AssetServer>) -> String {
    if let Some(path) = asset_server
        .as_ref()
        .and_then(|server| server.get_path(handle))
    {
        return path.to_string();
    }

    match handle {
        UntypedAssetId::Index { index, .. } => {
            format!("{:?}", egui::Id::new(index))
        }
        UntypedAssetId::Uuid { uuid, .. } => {
            format!("{uuid}")
        }
    }
}
