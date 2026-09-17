use std::collections::HashMap;

use serde::Serialize;
use upstream::back::glsl;
use upstream::{GlobalVariable, Handle};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
export interface ResourceBinding {
	group: number;
	binding: number;
}

export interface GlslReflection {
	textures: Record<string, { texture?: ResourceBinding; sampler?: ResourceBinding }>;
	uniforms: Record<string, ResourceBinding | undefined>;
	varyings: Record<string, { location: number; index: number }>;
	clipDistanceCount: number;
}

export interface GlslOutput {
	code: string;
	reflection: GlslReflection;
}
"#;

#[derive(Serialize)]
struct Binding {
    group: u32,
    binding: u32,
}

#[derive(Serialize)]
struct Texture {
    texture: Option<Binding>,
    sampler: Option<Binding>,
}

#[derive(Serialize)]
struct Varying {
    location: u32,
    index: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Reflection {
    textures: HashMap<String, Texture>,
    uniforms: HashMap<String, Option<Binding>>,
    varyings: HashMap<String, Varying>,
    clip_distance_count: u32,
}

#[derive(Serialize)]
pub struct GlslOutput {
    code: String,
    reflection: Reflection,
}

impl GlslOutput {
    pub fn new(
        code: String,
        module: &upstream::Module,
        info: glsl::ReflectionInfo,
    ) -> Self {
        let binding = |handle: Handle<GlobalVariable>| {
            module.global_variables[handle]
                .binding
                .as_ref()
                .map(|resource| Binding {
                    group: resource.group,
                    binding: resource.binding,
                })
        };
        Self {
            code,
            reflection: Reflection {
                textures: info
                    .texture_mapping
                    .into_iter()
                    .map(|(name, mapping)| {
                        let texture = Texture {
                            texture: binding(mapping.texture),
                            sampler: mapping.sampler.and_then(binding),
                        };
                        (name, texture)
                    })
                    .collect(),
                uniforms: info
                    .uniforms
                    .into_iter()
                    .map(|(handle, name)| (name, binding(handle)))
                    .collect(),
                varyings: info
                    .varying
                    .into_iter()
                    .map(|(name, varying)| {
                        let varying = Varying {
                            location: varying.location,
                            index: varying.index,
                        };
                        (name, varying)
                    })
                    .collect(),
                clip_distance_count: info.clip_distance_count,
            },
        }
    }

    pub fn into_js(self) -> Result<JsValue, JsError> {
        let serializer = serde_wasm_bindgen::Serializer::new()
            .serialize_maps_as_objects(true);
        Ok(self.serialize(&serializer)?)
    }
}
