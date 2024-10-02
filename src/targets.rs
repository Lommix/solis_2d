use crate::{config::GiConfig, constant, merge::MergeUniform, prelude::ComputedSize};
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
    pub probe_target: Handle<Image>,
    pub merge_targets: Vec<MergeTarget>,
    pub light_target: Handle<Image>,
    pub bounce_target: Handle<Image>,
    pub light_mipmap_target: Handle<Image>,
}

impl RenderTargets {
    #[rustfmt::skip]
    pub fn from_size(size: &ComputedSize, cfg: &GiConfig, images: &mut Assets<Image>) -> Self {
        let sdf_target          = Handle::weak_from_u128(905214787963254423236589025);
        let probe_target        = Handle::weak_from_u128(531037848998654123701989143);
        let light_target        = Handle::weak_from_u128(432123084179531435312554421);
        let bounce_target       = Handle::weak_from_u128(139987876583680013788430019);
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
                size.scaled.as_vec2(),
                constant::PROBE_FORMAT,
                ImageSampler::nearest(),
            ),
        );

        images.insert(
            &probe_target,
            create_image(
                size.cascade_size.as_vec2(),
                constant::PROBE_FORMAT,
                ImageSampler::nearest(),
            ),
        );

        images.insert(
            &light_target,
            create_image(
                size.scaled.as_vec2()/2.,
                constant::LIGHT_FORMAT,
                ImageSampler::nearest(),
            ),
        );

        images.insert(
            &bounce_target,
            create_image(
                size.scaled.as_vec2(),
                constant::BOUNCE_FORMAT,
                ImageSampler::nearest(),
            ),
        );


        let mut merge_targets : Vec<MergeTarget> = Vec::new();

        info!("--------------------------");


        for i in 0 .. (cfg.cascade_count) {
            // in reverse order small to large
            // following the cascade order
            // skip last one
            let index = cfg.cascade_count - i - 1;
            let handle = Handle::weak_from_u128(2708123423123005630984328769 + u128::from(i));
            let mut probe_stride = (cfg.probe_stride as i32) * (2_i32).pow(index);

            // if index == 0 {
            //     let mut probe_stride = cfg.probe_stride as i32;
            // }

            let mut merge_size = IVec2::new(
                size.scaled.x/probe_stride,
                size.scaled.y/probe_stride,
            );

            // if i >  0 {
            //     merge_size = merge_targets.last().unwrap().size.as_ivec2() * 2;
            // }

            // if size.scaled.x % probe_stride > 0 {
            //     merge_size.x += probe_stride - size.scaled.x%probe_stride;
            // }
            //
            // if size.scaled.y % probe_stride > 0 {
            //     merge_size.y += probe_stride - size.scaled.y%probe_stride;
            // }

            info!("[{i}] -- size {merge_size} -- stride {probe_stride} -- original {}", size.scaled);

            images.insert(
                &handle,
                create_image(
                    merge_size.as_vec2(),
                    constant::MERGE_FORMAT,
                    ImageSampler::nearest(),
                ),
            );
            merge_targets.push(MergeTarget {
                size: merge_size.as_vec2(),
                img: handle
            })
        }

        Self {
            sdf_target,
            probe_target,
            light_target,
            merge_targets,
            bounce_target,
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
