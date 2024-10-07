use bevy::{prelude::*, render::render_resource::TextureFormat};

// --------------------------------------
pub(crate) const COMMON_SHADER: Handle<Shader> = Handle::weak_from_u128(33512314255795372531210625);
pub(crate) const SDF_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
pub(crate) const CASCADE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
