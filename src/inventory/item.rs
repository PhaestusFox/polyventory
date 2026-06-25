use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
};
use indexmap::IndexMap;
use serde::Deserialize;

use crate::inventory::inventory_descriptor::InventoryDescriptorParseError;

use super::*;

#[derive(Component)]
#[require(Name)]
#[component(immutable, on_insert = Self::on_insert)]
pub struct Item {
    pub descriptor: Handle<ItemDescriptor>,
}

impl Item {
    pub fn id(&self) -> AssetId<ItemDescriptor> {
        self.descriptor.id()
    }

    pub fn new(descriptor: Handle<ItemDescriptor>) -> Self {
        Self { descriptor }
    }

    pub fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let item = world
            .get::<Self>(ctx.entity)
            .expect("This is Self::on_insert");
        let Some(descriptor) = world
            .resource::<Assets<ItemDescriptor>>()
            .get(&item.descriptor)
        else {
            warn!(
                "Item entity {:?} has an Item component with a handle that does not correspond to an ItemDescriptor asset",
                ctx.entity
            );
            return;
        };
        let name = Name::new(descriptor.name().to_string());
        *world.get_mut(ctx.entity).expect("Name is required") = name;
    }
}

impl Into<AssetId<ItemDescriptor>> for &Item {
    fn into(self) -> AssetId<ItemDescriptor> {
        self.descriptor.id()
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct ItemDescriptor {
    name: String,
    description: Option<String>,
    size: Vec<(CellType, Layout)>,
    image: IndexMap<CellType, (Handle<Image>, UVec2)>,
    tint: IndexMap<CellType, Color>,
    sub_inventory: Option<Handle<InventoryDescriptor>>,
    icon_components: Vec<Box<dyn Reflect>>,
}

impl ItemDescriptor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self, slot: &CellType) -> Option<Layout> {
        for (slot_type, layout) in &self.size {
            if slot_type == slot {
                return Some(layout.clone());
            }
        }
        None
    }
    pub fn get_image(&self, slot: &CellType) -> Option<(Handle<Image>, UVec2)> {
        self.image
            .get(slot)
            .cloned()
            .or_else(|| self.image.first().map(|(_, i)| i).cloned())
    }

    pub fn image(&self, slot: impl Iterator<Item = CellType>) -> Option<(Handle<Image>, UVec2)> {
        for slot_type in slot {
            for (image_slot, image) in &self.image {
                if image_slot == &slot_type {
                    return Some(image.clone());
                }
            }
        }
        self.image.first().map(|(_, i)| i).cloned()
    }

    pub fn valid_images(&self) -> impl Iterator<Item = (&CellType, &(Handle<Image>, UVec2))> {
        self.image
            .iter()
            .map(|(slot_type, image)| (slot_type, image))
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn spawn(&self) -> impl Bundle {}

    pub fn is_moveable(&self) -> bool {
        true
    }
    pub fn sub_inventory(&self) -> Option<&Handle<InventoryDescriptor>> {
        self.sub_inventory.as_ref()
    }

    pub fn sizes(&self) -> impl Iterator<Item = &(CellType, Layout)> {
        const FALL_BACK: (CellType, Layout) = (
            CellType::Any,
            Layout::Rect {
                size: UVec2::new(1, 1),
            },
        );
        self.size.iter().chain(&[FALL_BACK])
    }

    pub fn tint(&self, cell: &CellType) -> Option<Color> {
        self.tint
            .get(cell)
            .cloned()
            .or_else(|| self.tint.first().map(|(_, s)| *s))
    }
}

#[derive(TypePath, Default)]
pub struct ItemDescriptorLoader;

impl AssetLoader for ItemDescriptorLoader {
    fn extensions(&self) -> &[&str] {
        &["item"]
    }

    type Asset = ItemDescriptor;

    type Settings = ();

    type Error = LoadItemDescriptorError;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<
        Output = std::prelude::v1::Result<Self::Asset, Self::Error>,
    > {
        async move {
            let mut data = String::new();
            reader.read_to_string(&mut data).await?;
            let Some((pre, post)) = data.split_once("[item]") else {
                return load_item_descriptor(&data, load_context);
            };
            let (main, rest) = if pre.trim().is_empty() {
                let mut parts = post.split("[item]");
                let main = parts.next().unwrap_or("");
                (main, parts)
            } else {
                (pre, post.split("[item]"))
            };
            let main: Result<ItemDescriptor, LoadItemDescriptorError> =
                load_item_descriptor(main, load_context);
            for block in rest {
                let mut l = load_context.begin_labeled_asset();
                let new = load_item_descriptor(block, &mut l)?;
                let name = new.name.clone();
                let asset = l.finish(new);
                load_context.add_loaded_labeled_asset(name, asset);
            }
            main
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoadItemDescriptorError {
    #[error("Failed to read item descriptor file")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse item descriptor file")]
    ParseError(String),

    #[error("Missing `:` that splits cell for data")]
    MissingColin,

    #[error("Line missing slot type")]
    MissingSlotType,
    #[error("Missing Size: {0}")]
    MissingSize(&'static str),
    #[error("Invalid Size {0}: {1}")]
    InvalidSize(&'static str, String),
    #[error("Missing Image path")]
    MissingImagePath,
    #[error("Invalid Color")]
    FailedToParseColor,

    #[error("Entity does not have an Item component")]
    NoItemInDescriptor,

    #[error("Failed to parse item inventory descriptor {0}")]
    ParseInventoryDescriptorError(#[from] InventoryDescriptorParseError),
}

fn load_item_descriptor(
    data: &str,
    ctx: &mut bevy::asset::LoadContext,
) -> Result<ItemDescriptor, LoadItemDescriptorError> {
    let mut size = Vec::new();
    let mut image = IndexMap::new();
    let mut name = String::new();
    let mut mode = Mode::Name;
    let mut description: Option<String> = None;
    let mut sub_inventory: Option<Handle<InventoryDescriptor>> = None;
    let mut tints: IndexMap<CellType, Color> = IndexMap::new();
    let mut icon_components = Vec::new();
    let mut used = 0;
    while let Some(((head, body), consumed)) = extract_segment(&data[used..]) {
        used += consumed;
        if head.is_empty() || head.starts_with('[') {
            trace!("Found empty segment: {}-{}", used - consumed, used);
            continue;
        };
        set_mode(head, &mut mode);
        match mode {
            Mode::Description => {
                if let Some(desc) = description.as_mut() {
                    desc.push('\n');
                    desc.push_str(head);
                } else {
                    description = Some(head.to_string());
                }
            }
            Mode::Inventory => {
                if !body.contains('{') {
                    panic!("going to load sub inv: {:?}", body);
                    if sub_inventory.is_some() {
                        warn!(
                            "Multiple inventory blocks found in item descriptor, only the first one will be used"
                        );
                        continue;
                    }
                    sub_inventory = Some(ctx.load(body.to_string()));
                    continue;
                }
                let i =
                    crate::inventory::inventory_descriptor::InventoryDescriptor::from_str(&body)?;
                sub_inventory = Some(ctx.add_labeled_asset(format!("{}.inventory", name), i));
            }
            Mode::Tint => {
                let Some((cell, rest)) = body.split_once(':') else {
                    return Err(LoadItemDescriptorError::MissingColin);
                };
                let cell = CellType::from_str(cell).expect("Infallible");
                let color = parse_color(rest).ok_or(LoadItemDescriptorError::FailedToParseColor)?;
                tints.insert(cell, color);
<<<<<<< Updated upstream
            }
            Mode::Components => {}
=======
            },
            // Mode::Components {
            //     let block = extract_block();
            // },
>>>>>>> Stashed changes
            _ => {
                mode.parse_line(body, &mut name, &mut size, &mut image, ctx)?;
            }
        }
    }

    let item = ItemDescriptor {
        name,
        description,
        size,
        image,
        sub_inventory,
        tint: tints,
        icon_components,
    };

    Ok(item)
}

fn set_mode(line: &str, mode: &mut Mode) -> bool {
    let line = line.to_lowercase();
    if line.starts_with("name") {
        *mode = Mode::Name;
        false
    } else if line.starts_with("size") {
        *mode = Mode::Size;
        true
    } else if line.starts_with("image") {
        *mode = Mode::Image;
        true
    } else if line.starts_with("description") {
        *mode = Mode::Description;
        true
    } else if line.starts_with("inventory") {
        *mode = Mode::Inventory;
        false
    } else if line.starts_with("tint") {
        *mode = Mode::Tint;
        true
    } else if line.starts_with("components") {
        *mode = Mode::Components;
        false
    } else {
        false
    }
}

#[derive(Debug)]
enum Mode {
    Size,
    Image,
    Name,
    Description,
    Inventory,
    Tint,
    Components,
}

impl Mode {
    fn parse_line(
        &self,
        line: &str,
        name: &mut String,
        size: &mut Vec<(CellType, Layout)>,
        image: &mut IndexMap<CellType, (Handle<Image>, UVec2)>,
        ctx: &mut bevy::asset::LoadContext,
    ) -> Result<(), LoadItemDescriptorError> {
        match self {
            Mode::Name => {
                let mut parts = line.split(':');
                parts.next(); // skip "name:"
                if let Some(new_name) = parts.next() {
                    name.push_str(new_name.trim());
                    Ok(())
                } else {
                    Err(LoadItemDescriptorError::ParseError(format!(
                        "Invalid name line: {}",
                        line
                    )))
                }
            }
            Mode::Size => {
                size.push(Self::parse_size(line)?);
                Ok(())
            }
            Mode::Image => {
                let (slot_type, (path, size)) = Self::parse_image(line)?;
                let image_handle: Handle<Image> = ctx.load(path);
                match image.entry(slot_type) {
                    indexmap::map::Entry::Occupied(e) => {
                        warn!(
                            "Multiple images specified for slot type {:?}, only the first one will be used",
                            e.key()
                        );
                    }
                    indexmap::map::Entry::Vacant(entry) => {
                        entry.insert((image_handle, size));
                    }
                }
                Ok(())
            }
            _ => {
                error!("{:?} should never make it here", self);
                Ok(())
            }
        }
    }

    fn parse_size(size_str: &str) -> Result<(CellType, Layout), LoadItemDescriptorError> {
        let mut parts = size_str.split(':');
        let slot_type = parts
            .next()
            .ok_or(LoadItemDescriptorError::MissingSlotType)?;
        let size_str = parts
            .next()
            .ok_or_else(|| LoadItemDescriptorError::MissingSize("x & y"))?;
        let mut size_parts = size_str.split(',');
        let x = size_parts
            .next()
            .ok_or_else(|| LoadItemDescriptorError::MissingSize("x"))?
            .trim();
        let x = x
            .parse::<u32>()
            .map_err(|_| LoadItemDescriptorError::InvalidSize("x", x.to_string()))?;
        let y = size_parts
            .next()
            .ok_or_else(|| LoadItemDescriptorError::MissingSize("y"))?
            .trim();
        let y = y
            .parse::<u32>()
            .map_err(|_| LoadItemDescriptorError::InvalidSize("y", y.to_string()))?;
        Ok((
            slot_type.parse().unwrap(),
            Layout::Rect {
                size: UVec2::new(x, y),
            },
        ))
    }

    fn parse_image(
        image_line: &str,
    ) -> Result<(CellType, (String, UVec2)), LoadItemDescriptorError> {
        let mut parts = image_line.split(':');
        let slot_type = parts
            .next()
            .ok_or(LoadItemDescriptorError::MissingSlotType)?;
        let next = parts
            .next()
            .ok_or(LoadItemDescriptorError::MissingImagePath)?
            .trim();
        let mut parts = next.split(',');
        let path = parts
            .next()
            .ok_or(LoadItemDescriptorError::MissingImagePath)?
            .trim();
        let x = parts
            .next()
            .ok_or(LoadItemDescriptorError::MissingSize("Width"))?
            .trim();
        let x = x
            .parse::<u32>()
            .map_err(|_| LoadItemDescriptorError::InvalidSize("image x", x.to_string()))?;
        let y = parts
            .next()
            .ok_or(LoadItemDescriptorError::MissingSize("Height"))?
            .trim();
        let y = y
            .parse::<u32>()
            .map_err(|_| LoadItemDescriptorError::InvalidSize("image y", y.to_string()))?;
        Ok((
            slot_type.parse().unwrap(),
            (path.to_string(), UVec2::new(x, y)),
        ))
    }
}

fn parse_color(color: &str) -> Option<Color> {
    let color = color.trim();
    if color.starts_with('#') {
        unimplemented!("Hex Color parsing is not done yet");
    }
    if color.starts_with(|c: char| c.is_numeric()) {
        unimplemented!("Raw color paring is not done yet");
    }
    use bevy::color::palettes::basic::*;
    Some(
        match color.to_ascii_uppercase().as_ref() {
            "AQUA" => AQUA,
            "BLACK" => BLACK,
            "BLUE" => BLUE,
            "FUCHSIA" => FUCHSIA,
            "GRAY" => GRAY,
            "GREEN" => GREEN,
            "LIME" => LIME,
            "MAROON" => MAROON,
            "NAVY" => NAVY,
            "OLIVE" => OLIVE,
            "PURPLE" => PURPLE,
            "RED" => RED,
            "SILVER" => SILVER,
            "TEAL" => TEAL,
            "WHITE" => WHITE,
            "YELLOW" => YELLOW,
            other => {
                let mut m = std::hash::DefaultHasher::default();
                other.hash(&mut m);
                let id = m.finish() as i64;
                let a = id & 0xFF - (id >> 8) & 0x1FF;
                let b = (id >> 17) & 0xFF - (id >> 26) & 0x1FF;
                Color::oklab(1., a as f32 / 256., b as f32 / 256.).to_srgba()
            }
        }
        .into(),
    )
}

fn extract_segment(src: &str) -> Option<((&str, &str), usize)> {
    let mut start = usize::MAX;
    let mut split = 0;
    let mut chars = src.char_indices();
    while let Some((i, char)) = chars.next() {
        if char == '\n' && start != usize::MAX {
            return Some(((&src[start..split], &src[split..i]), i));
        }
        if char.is_whitespace() {
            continue;
        }
        if start == usize::MAX {
            start = i;
            split = i;
        }
        if char == ':' && split == 0 {
            split = i + 1;
            if let Some((block, used)) = extract_block(&src[i + 1..]) {
                return Some(((&src[start..split], block), i + used));
            }
        }
    }
    None
}

fn extract_block(src: &str) -> Option<(&str, usize)> {
    let mut stack = Vec::new();
    let mut start = 0;
    for (i, char) in src.char_indices() {
        let Some(block) = BlockType::from_char(char) else {
            continue;
        };
        if BlockType::open(char) {
            if stack.is_empty() {
                start = i;
            }
            stack.push(block);
        } else if let Some(&b) = stack.last()
            && b == block
        {
            stack.pop();
            if stack.is_empty() {
                return Some((&src[start..i], i));
            }
        }
    }
    None
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum BlockType {
    Par,
    Curl,
    Square,
    Angle,
}

impl BlockType {
    fn from_char(c: char) -> Option<BlockType> {
        match c {
            '(' | ')' => Some(BlockType::Par),
            '{' | '}' => Some(BlockType::Curl),
            '[' | ']' => Some(BlockType::Square),
            '<' | '>' => Some(BlockType::Angle),
            _ => None,
        }
    }
    fn open(c: char) -> bool {
        matches!(c, '(' | '{' | '[' | '<')
    }
}

// impl<'de> Deserialize<'de> for ItemDescriptor {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>
//     {
//         deserializer.deserialize_map(ItemVisitor)
//     }
// }

// struct ItemVisitor;

// impl<'de> serde::de::Visitor<'de> for ItemVisitor {
//     type Value = ItemDescriptor;
//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         formatter.write_str("IDK what im doing")
//     }
//     fn visit_map<A>(self, map: A) -> std::prelude::v1::Result<Self::Value, A::Error>
//     where
//         A: serde::de::MapAccess<'de>,
//     {
//         map.next_entry();
//         ron::Deserializer
//         todo!()
//     }
//     fn visit_enum<A>(self, data: A) -> std::prelude::v1::Result<Self::Value, A::Error>
//     where
//         A: serde::de::EnumAccess<'de>,
//     {
//         let v = data.variant::<String>().unwrap();
//     }
// }

// #[non_exhaustive]
// #[repr(u8)]
// #[derive(Debug)]
// enum Test {
//     A,
//     B,
//     C
// }

// #[test]
// fn try_custom_serde() {
//     let item = ron::from_str::<ItemDescriptor>("{
//         name: Athletic
//         size:
//         Untyped: 6, 2
//         image:
//         Untyped: bbg/ui/Skill.png, 6, 2
//         tint:
//         Untyped: blue
//         icon:
//         {
//             components: []+,
//             {child_components: []}*,
//         }
//     }").unwrap();
//     let truth: Vec<Box<dyn Reflect>> = vec![];
//     assert!(item.icon_components.iter().zip(truth.iter()).all(|(a, b)| a.reflect_partial_eq(b.as_partial_reflect()).unwrap_or_default()));
// }
