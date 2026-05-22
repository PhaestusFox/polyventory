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
        ThemeBackgroundColor(feathers::tokens::WINDOW_BG),
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
}

impl ToolTip<'_, '_> {
    fn close(&mut self) {
        self.root.1.display = Display::None;
    }

    fn open(&mut self, location: Option<Vec2>) {
        if let Some(position) = location.or(self.cursor.cursor_position()) {
            self.root.1.left = Val::Px(position.x);
            self.root.1.top = Val::Px(position.y);
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
    items: Query<&Item>,
    descriptors: Res<Assets<ItemDescriptor>>,
    mut tooltip: ToolTip,
) {
    let Ok(item) = items.get(event.item) else {
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
                .with_children(|size| for (slot_type, shape) in descriptor.valid_sizes() {});
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
            });
    }
}

fn detect_item_hover(
    mouse: On<Pointer<Over>>,
    mut actions: MessageWriter<ToolTipAction>,
    // dont know what to call the visual representation of an item
    icons: Query<&DisplayedItem>,
) {
    // check we just hovered over an item icon
    let Ok(clicked) = icons.get(mouse.entity) else {
        return;
    };
    actions.write(ToolTipAction::item_tooltip(
        clicked.entity,
        Some(mouse.pointer_location.position),
    ));
}

fn detect_item_hover_exit(
    mouse: On<Pointer<Out>>,
    mut actions: MessageWriter<ToolTipAction>,
    icons: Query<&DisplayedItem>,
) {
    if icons.contains(mouse.entity) {
        actions.write(ToolTipAction::close());
    }
}
