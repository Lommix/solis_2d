use bevy::{prelude::*, render::render_resource::TextureFormat};

// --------------------------------------
pub(crate) const SDF_SHADER: Handle<Shader> = Handle::weak_from_u128(53086988700321314503262798);
pub(crate) const LIGHT_SHADER: Handle<Shader> = Handle::weak_from_u128(58961304766503833016970315);
pub(crate) const BOUNCE_SHADER: Handle<Shader> = Handle::weak_from_u128(58961304766503833016970315);
pub(crate) const PROBE_SHADER: Handle<Shader> = Handle::weak_from_u128(97061304766503863016972215);
pub(crate) const COMPOSITE_SHADER: Handle<Shader> =
    Handle::weak_from_u128(43212314255795302531210625);
pub(crate) const COMMON_SHADER: Handle<Shader> = Handle::weak_from_u128(33512314255795372531210625);
pub(crate) const SHAPES_SHADER: Handle<Shader> = Handle::weak_from_u128(63212314255795362531210427);
pub(crate) const RAYMARCH_SHADER: Handle<Shader> =
    Handle::weak_from_u128(652123142557973608531210428);
// --------------------------------------
pub(crate) const SDF_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
pub(crate) const LIGHT_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
pub(crate) const BOUNCE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
pub(crate) const PROBE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
pub(crate) const MERGE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
