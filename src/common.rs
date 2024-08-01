use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(Component, Default, ExtractComponent, Clone, Copy)]
pub struct Light2dCameraTag;
