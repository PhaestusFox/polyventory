use std::time::{Duration, Instant};

use bevy::{
    ecs::system::SystemParam,
    feathers::{
        self,
        theme::{ThemeBackgroundColor, ThemeToken, ThemedText},
    },
    prelude::*,
    ui::FocusPolicy,
    window::PrimaryWindow,
};

use crate::inventory::{Item, ItemDescriptor};

use crate::prelude::*;

pub struct ToolTipPlugin;

impl Plugin for ToolTipPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::feathers::FeathersPlugin>() {
            app.add_plugins(bevy::feathers::FeathersPlugins);
            use feathers::theme::UiTheme;
            app.insert_resource(UiTheme(feathers::dark_theme::create_dark_theme()));
        }

        app.init_resource::<ToolTipSettings>();
        app.add_message::<ToolTipAction>();
        app.add_systems(Startup, spawn_tooltip);
        app.add_systems(PreUpdate, show_tooltip);
        app.add_observer(show_item_tooltip);
        app.add_observer(detect_item_hover);
        app.add_observer(detect_item_hover_exit);
    }
}

#[derive(Component)]
struct ToolTipRoot;

fn spawn_tooltip(mut commands: Commands) {
    commands.spawn((
        Name::new("ToolTipRoot"),
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        FocusPolicy::Pass,
        ToolTipRoot,
        GlobalZIndex(1),
        ThemeBackgroundColor(feathers::tokens::WINDOW_BG),
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
    ));
}

#[derive(Resource)]
pub struct ToolTipSettings {
    pub delay: Duration,
    pub debug_info: bool,
}

impl Default for ToolTipSettings {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(100),
            debug_info: false,
        }
    }
}

#[derive(Resource, Message, Clone)]
pub struct ToolTipAction {
    timestamp: Instant,
    event_type: ToolTipEventType,
}

impl ToolTipAction {
    pub fn close() -> Self {
        Self {
            timestamp: Instant::now(),
            event_type: ToolTipEventType::Close,
        }
    }

    pub fn item_tooltip(item: Entity, location: Option<Vec2>) -> Self {
        Self {
            timestamp: Instant::now(),
            event_type: ToolTipEventType::ItemToolTip(item, location),
        }
    }
}

#[derive(Clone)]
enum ToolTipEventType {
    Close,
    ItemToolTip(Entity, Option<Vec2>),
}
#[derive(SystemParam)]
struct ToolTip<'w, 's> {
    commands: Commands<'w, 's>,
    root: Single<'w, 's, (Entity, &'static mut Node), With<ToolTipRoot>>,
    settings: Res<'w, ToolTipSettings>,
    cursor: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    pointer: Local<'s, Option<Entity>>,
    current: Local<'s, Option<Entity>>,
}

impl ToolTip<'_, '_> {
    fn close(&mut self) {
        self.root.1.display = Display::None;
    }

    fn open(&mut self, location: Option<Vec2>) {
        if let Some(position) = location.or(self.cursor.cursor_position()) {
            self.root.1.left = Val::Px(position.x + 10.);
            self.root.1.top = Val::Px(position.y + 10.);
        }
        self.root.1.display = Display::Flex;
    }

    pub fn settings(&self) -> &ToolTipSettings {
        &self.settings
    }

    fn root(&self) -> Entity {
        self.root.0
    }

    fn clear(&mut self) {
        self.commands.entity(self.root()).despawn_children();
    }

    pub fn add_space(&mut self, size: f32) {
        let p = self.pointer();
        self.commands.spawn((
            Name::new("Space"),
            Node {
                height: Val::Px(size),
                ..Default::default()
            },
            ChildOf(p),
        ));
    }

    fn pointer(&mut self) -> Entity {
        if let Some(entity) = *self.pointer {
            entity
        } else {
            self.root()
        }
    }

    fn spawn(&mut self, components: impl Bundle) -> Entity {
        let entity = self.commands.spawn((components, ChildOf(self.root()))).id();
        entity
    }
}

fn show_tooltip(
    mut commands: Commands,
    mut messages: MessageReader<ToolTipAction>,
    mut last: Local<Option<ToolTipAction>>,
    mut tooltip: ToolTip,
) {
    if let Some(message) = messages.read().last() {
        *last = Some(message.clone());
    }

    let Some(ToolTipAction {
        timestamp,
        event_type,
    }) = last.take()
    else {
        return;
    };
    // if we haven't stop long enough don't show the tooltip yet
    if timestamp.elapsed() < tooltip.settings().delay {
        *last = Some(ToolTipAction {
            timestamp,
            event_type,
        });
        return;
    }

    match event_type {
        ToolTipEventType::Close => {
            tooltip.close();
        }
        ToolTipEventType::ItemToolTip(item, position) => {
            commands.trigger(ItemToolTip { item, position });
        }
    }
}

#[derive(Event)]
struct ItemToolTip {
    item: Entity,
    position: Option<Vec2>,
}

fn show_item_tooltip(
    event: On<ItemToolTip>,
    items: Query<(&Item, &InInventory, &RenderingItem)>,
    descriptors: Res<Assets<ItemDescriptor>>,
    mut tooltip: ToolTip,
    inventorys: Res<Assets<Inventory>>,
) {
    let Ok((item, in_inventory, rendering_item)) = items.get(event.item) else {
        warn!(
            "ItemToolTip event for entity {:?} that does not have an Item component",
            event.item
        );
        return;
    };
    let Some(descriptor) = descriptors.get(item.descriptor.id()) else {
        warn!(
            "ItemToolTip event for entity {:?} with descriptor handle {:?} that does not correspond to an ItemDescriptor asset",
            event.item, item.descriptor
        );
        return;
    };

    if let Some(old) = tooltip.current.replace(event.item)
        && let Ok((.., rendering_item)) = items.get(old)
    {
        for item in rendering_item.iter() {
            tooltip
                .commands
                .entity(item)
                .insert(BackgroundColor(Color::WHITE.with_alpha(0.)));
        }
    }
    for item in rendering_item.iter() {
        tooltip
            .commands
            .entity(item)
            .insert(BackgroundColor(Color::BLACK.lighter(0.5)));
    }

    tooltip.clear();
    tooltip.open(event.position);
    // Spawn the name
    tooltip.commands.spawn((
        Name::new("Item Name"),
        Text::new(descriptor.name()),
        TextLayout {
            justify: Justify::Center,
            ..Default::default()
        },
        TextFont {
            font_size: 20.,
            ..Default::default()
        },
        ChildOf(tooltip.root()),
    ));
    tooltip.add_space(5.);
    if let Some(description) = descriptor.description() {
        tooltip.commands.spawn((
            Name::new("Item Description"),
            Text::new(description),
            TextLayout {
                justify: Justify::Left,
                ..Default::default()
            },
            TextFont {
                font_size: 16.,
                ..Default::default()
            },
            ChildOf(tooltip.root()),
        ));
    }

    tooltip.add_space(20.);
    if tooltip.settings().debug_info {
        tooltip
            .commands
            .spawn((
                Name::new("Debug Info"),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ChildOf(tooltip.root()),
            ))
            .with_children(|dbg| {
                dbg.spawn((
                    Text::new("Debug Info:"),
                    TextLayout {
                        justify: Justify::Left,
                        ..Default::default()
                    },
                    TextFont {
                        font_size: 20.,
                        ..Default::default()
                    },
                ));
                dbg.spawn((
                    Text::new("Sizes:"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                ))
                .with_children(|size| for (slot_type, layout) in descriptor.sizes() {});
                dbg.spawn((
                    Text::new("Images:"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                ))
                .with_children(|size| {
                    for (slot_type, shape) in descriptor.valid_images() {}
                });
                dbg.spawn(Text::new(format!("Slot Type: {:?}", in_inventory.1)));
                let Some(inv_info) = inventorys.get(in_inventory.0) else {
                    return;
                };
                let Some(shape) = inv_info.get_shape(event.item) else {
                    return;
                };
                dbg.spawn(Text::new(format!("{}", shape)));
            });
        if let Some(inv) = descriptor.sub_inventory() {
            let id = match inv.path() {
                Some(path) => {
                    format!("SubInv-Handle: {}", path)
                }
                None => match inv.id() {
                    AssetId::Index { index, .. } => format!("SubInv-Index: {:?}", index),
                    AssetId::Uuid { uuid } => format!("SubInv-Uuid: {}", uuid),
                },
            };
            tooltip
                .commands
                .spawn((Text::new(id), ChildOf(tooltip.root())));
        }
    }
}

fn detect_item_hover(
    mouse: On<Pointer<Over>>,
    mut actions: MessageWriter<ToolTipAction>,
    // dont know what to call the visual representation of an item
    icons: Query<&RenderedItem>,
) {
    // check we just hovered over an item icon
    let Ok(clicked) = icons.get(mouse.entity) else {
        return;
    };
    actions.write(ToolTipAction::item_tooltip(
        clicked.item,
        Some(mouse.pointer_location.position),
    ));
}

fn detect_item_hover_exit(
    mouse: On<Pointer<Out>>,
    mut actions: MessageWriter<ToolTipAction>,
    icons: Query<&RenderedItem>,
) {
    if icons.contains(mouse.entity) {
        actions.write(ToolTipAction::close());
    }
}
