use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_cursor_icon);
    app.add_observer(detect_hover_slot);
}

fn spawn_cursor_icon(mut commands: Commands, cursor: Res<CursorInventory>) {
    commands.spawn((
        CursorSlot,
        Transform::default(),
        Visibility::Visible,
        Name::new("Cursor Slot"),
        RenderedInventory::new(cursor.inventory.clone()),
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
        GlobalZIndex(1),
        BackgroundColor(Color::linear_rgba(0.1, 0.1, 0.8, 0.33)),
    ));
}


pub fn detect_hover_slot(
    event: On<Pointer<Over>>,
    style: InventoryStyler,
    slots: Query<&RenderedSlot>,
    mut cursor: InventoryCursor,
) {
    let Ok(slot) = slots.get(event.entity) else {
        return;
    };
    let new = style.style_handle(slot.inventory);
    let old = style.style_handle(cursor.entity());
    if old == new {
        return;
    }
    cursor.set_style(new);
}