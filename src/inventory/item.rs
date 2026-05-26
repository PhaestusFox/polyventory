use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
};

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
    size: Vec<(SlotType, Shape)>,
    image: Vec<(SlotType, (Handle<Image>, UVec2))>,
}

impl ItemDescriptor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self, slot: &SlotType) -> Option<Shape> {
        for (slot_type, shape) in &self.size {
            if slot_type == slot {
                return Some(shape.clone());
            }
        }
        None
    }
    pub fn get_image(&self, slot: &SlotType) -> Option<(Handle<Image>, UVec2)> {
        for (slot_type, image) in &self.image {
            if slot_type == slot {
                return Some(image.clone());
            }
        }
        None
    }

    pub fn image(&self, slot: impl Iterator<Item = SlotType>) -> Option<(Handle<Image>, UVec2)> {
        for slot_type in slot {
            for (image_slot, image) in &self.image {
                if image_slot == &slot_type {
                    return Some(image.clone());
                }
            }
        }
        None
    }

    pub fn valid_sizes(&self) -> impl Iterator<Item = (&SlotType, &Shape)> {
        self.size
            .iter()
            .map(|(slot_type, shape)| (slot_type, shape))
    }

    pub fn valid_images(&self) -> impl Iterator<Item = (&SlotType, &(Handle<Image>, UVec2))> {
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
            println!("Loading item descriptor from");
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
            let main: Result<ItemDescriptor, LoadItemDescriptorError> = load_item_descriptor(main, load_context);
            for block in rest {
                let mut l = load_context.begin_labeled_asset();
                let new = load_item_descriptor(block, &mut l)?;
                println!("Loaded item descriptor: {:?}", new);
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

    #[error("Line missing slot type")]
    MissingSlotType,
    #[error("Missing Size: {0}")]
    MissingSize(&'static str),
    #[error("Invalid Size {0}: {1}")]
    InvalidSize(&'static str, String),
    #[error("Missing Image path")]
    MissingImagePath,

    #[error("Entity does not have an Item component")]
    NoItemInDescriptor,
}

fn load_item_descriptor(
    data: &str,
    ctx: &mut bevy::asset::LoadContext,
) -> Result<ItemDescriptor, LoadItemDescriptorError> {
    let mut size = Vec::new();
    let mut image = Vec::new();
    let mut name = String::new();
    let mut mode = Mode::Name;
    let mut description: Option<String> = None;
    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        if set_mode(line, &mut mode) {
            continue;
        }
        match mode {
            Mode::Description => {
                if let Some(desc) = description.as_mut() {
                    desc.push('\n');
                    desc.push_str(line);
                } else {
                    description = Some(line.to_string());
                }
            }
            _ => {
                mode.parse_line(line, &mut name, &mut size, &mut image, ctx)?;
            }
        }
    }

    let item = ItemDescriptor {
        name,
        description,
        size,
        image,
    };

    Ok(item)
}

fn set_mode(line: &str, mode: &mut Mode) -> bool {
    let line = line.to_lowercase();
    if line.starts_with("name") {
        *mode = Mode::Name;
        return false;
    } else if line.starts_with("size") {
        *mode = Mode::Size;
        return true;
    } else if line.starts_with("image") {
        *mode = Mode::Image;
        return true;
    } else if line.starts_with("description") {
        *mode = Mode::Description;
        return true;
    } else {
        return false;
    }
}

enum Mode {
    Size,
    Image,
    Name,
    Description,
}

impl Mode {
    fn parse_line(
        &self,
        line: &str,
        name: &mut String,
        size: &mut Vec<(SlotType, Shape)>,
        image: &mut Vec<(SlotType, (Handle<Image>, UVec2))>,
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
                println!("Loading image for slot type {:?} from path {}", slot_type, path);
                let image_handle: Handle<Image> = ctx.load(path);
                image.push((slot_type, (image_handle, size)));
                Ok(())
            }
            Mode::Description => {
                error!("Description should never make it here");
                Ok(())
            }
        }
    }

    fn parse_size(size_str: &str) -> Result<(SlotType, Shape), LoadItemDescriptorError> {
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
            Shape {
                cells: Cells::Rect {
                    size: UVec2::new(x, y),
                },
                position: IVec2::ZERO,
                orientation: Orientation::Identity,
            },
        ))
    }

    fn parse_image(
        image_line: &str,
    ) -> Result<(SlotType, (String, UVec2)), LoadItemDescriptorError> {
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
