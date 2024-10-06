use crate::{config::GiConfig, constant, prelude::ComputedSize};
use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::{GpuImage, ImageSampler},
    },
};

#[derive(Resource, ExtractResource, Clone)]
pub struct RenderTargets {
    pub sdf_target: Handle<Image>,
    pub merge_targets: Vec<MergeTarget>,
    pub light_mipmap_target: Handle<Image>,
}

impl RenderTargets {
    #[rustfmt::skip]
    pub fn from_size(size: &ComputedSize, cfg: &GiConfig, images: &mut Assets<Image>) -> Self {
        let sdf_target          = Handle::weak_from_u128(905214787963254423236589025);
        let light_mipmap_target = Handle::weak_from_u128(139987876583680013788430531);

        images.insert(
            &sdf_target,
            create_image(
                size.scaled.as_vec2(),
                constant::SDF_FORMAT,
                ImageSampler::nearest(),
            ),
        );

        images.insert(
            &light_mipmap_target,
            create_image(
                size.scaled.as_vec2()/( cfg.probe_stride as f32 * 4. ),
                constant::MERGE_FORMAT,
                ImageSampler::linear(),
            ),
        );

        let mut merge_targets : Vec<MergeTarget> = Vec::new();
        // ping pong later
        for i in 0 .. 2_u32 {
            let handle = Handle::weak_from_u128(2708123423123005630984328769 + u128::from(i));
            images.insert(
                &handle,
                create_image(
                    size.scaled.as_vec2(),
                    constant::MERGE_FORMAT,
                    ImageSampler::nearest(),
                ),
            );
            merge_targets.push(MergeTarget {
                size: size.scaled.as_vec2(),
                img: handle
            })
        }

        Self {
            sdf_target,
            merge_targets,
            light_mipmap_target,
        }
    }

    pub fn sorted_merge_targets<'a>(
        &'a self,
        images: &'a RenderAssets<GpuImage>,
    ) -> impl Iterator<Item = &'a GpuImage> {
        // joke, they are already sorted
        self.merge_targets
            .iter()
            .map(|target| images.get(&target.img).unwrap())
    }
}
#[derive(Clone)]
pub struct MergeTarget {
    pub img: Handle<Image>,
    pub size: Vec2,
}

fn create_image(size: Vec2, format: TextureFormat, sampler: ImageSampler) -> Image {
    let size = Extent3d {
        width: size.x as u32,
        height: size.y as u32,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler,
        ..default()
    };
    image.resize(size);
    image
}
