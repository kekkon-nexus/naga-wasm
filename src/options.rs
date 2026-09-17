use std::collections::HashMap;

use serde::Deserialize;
use upstream::back::{glsl, hlsl, spv, wgsl};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
export type ShaderStage = "vertex" | "fragment" | "compute";

export interface GlslParseOptions {
	stage: ShaderStage;
	defines?: Record<string, string>;
}

export interface WgslWriteOptions {
	flags?: {
		explicitTypes?: boolean;
	};
}

export interface GlslWriteOptions {
	version: `${number}` | `${number} es`;
	stage: ShaderStage;
	entryPoint: string;
	flags?: {
		/** @default true */
		adjustCoordinateSpace?: boolean;
		forcePointSize?: boolean;
		textureShadowLod?: boolean;
		drawParameters?: boolean;
		includeUnusedItems?: boolean;
	};
}

export interface HlslWriteOptions {
	shaderModel?: "5_0" | "5_1" | "6_0" | "6_1" | "6_2" | "6_3" | "6_4" | "6_5" | "6_6" | "6_7" | "6_8" | "6_9";
}

export interface MslWriteOptions {
	langVersion?: [major: number, minor: number];
}

export interface SpirvWriteOptions {
	flags?: {
		/** @default true */
		adjustCoordinateSpace?: boolean;
		/** @default true */
		labelVaryings?: boolean;
		/** @default true */
		clampFragDepth?: boolean;
		forcePointSize?: boolean;
		debug?: boolean;
	};
}
"#;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Vertex,
    Fragment,
    Compute,
}

impl From<Stage> for upstream::ShaderStage {
    fn from(stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self::Vertex,
            Stage::Fragment => Self::Fragment,
            Stage::Compute => Self::Compute,
        }
    }
}

#[derive(Deserialize)]
pub struct GlslParseOptions {
    stage: Stage,
    defines: Option<HashMap<String, String>>,
}

impl From<GlslParseOptions> for upstream::front::glsl::Options {
    fn from(options: GlslParseOptions) -> Self {
        Self {
            stage: options.stage.into(),
            defines: options.defines.unwrap_or_default().into_iter().collect(),
        }
    }
}

#[derive(Deserialize, Default)]
pub struct WgslWriteOptions {
    pub flags: Option<WgslFlags>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WgslFlags {
    explicit_types: Option<bool>,
}

impl WgslFlags {
    pub fn apply(self, flags: &mut wgsl::WriterFlags) {
        if let Some(value) = self.explicit_types {
            flags.set(wgsl::WriterFlags::EXPLICIT_TYPES, value);
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlslWriteOptions {
    pub version: String,
    pub stage: Stage,
    pub entry_point: String,
    pub flags: Option<GlslFlags>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GlslFlags {
    adjust_coordinate_space: Option<bool>,
    force_point_size: Option<bool>,
    texture_shadow_lod: Option<bool>,
    draw_parameters: Option<bool>,
    include_unused_items: Option<bool>,
}

impl GlslFlags {
    pub fn apply(self, flags: &mut glsl::WriterFlags) {
        use glsl::WriterFlags as F;
        for (flag, value) in [
            (F::ADJUST_COORDINATE_SPACE, self.adjust_coordinate_space),
            (F::FORCE_POINT_SIZE, self.force_point_size),
            (F::TEXTURE_SHADOW_LOD, self.texture_shadow_lod),
            (F::DRAW_PARAMETERS, self.draw_parameters),
            (F::INCLUDE_UNUSED_ITEMS, self.include_unused_items),
        ] {
            if let Some(value) = value {
                flags.set(flag, value);
            }
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HlslWriteOptions {
    pub shader_model: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MslWriteOptions {
    pub lang_version: Option<(u8, u8)>,
}

#[derive(Deserialize, Default)]
pub struct SpirvWriteOptions {
    pub flags: Option<SpirvFlags>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpirvFlags {
    adjust_coordinate_space: Option<bool>,
    label_varyings: Option<bool>,
    clamp_frag_depth: Option<bool>,
    force_point_size: Option<bool>,
    debug: Option<bool>,
}

impl SpirvFlags {
    pub fn apply(self, flags: &mut spv::WriterFlags) {
        use spv::WriterFlags as F;
        for (flag, value) in [
            (F::ADJUST_COORDINATE_SPACE, self.adjust_coordinate_space),
            (F::LABEL_VARYINGS, self.label_varyings),
            (F::CLAMP_FRAG_DEPTH, self.clamp_frag_depth),
            (F::FORCE_POINT_SIZE, self.force_point_size),
            (F::DEBUG, self.debug),
        ] {
            if let Some(value) = value {
                flags.set(flag, value);
            }
        }
    }
}

pub fn glsl_version(version: &str) -> Result<glsl::Version, JsError> {
    let (number, es) = match version.strip_suffix(" es") {
        Some(number) => (number, true),
        None => (version, false),
    };
    let number = number.parse().map_err(|_| {
        JsError::new(&format!("invalid GLSL version: {version}"))
    })?;
    Ok(if es {
        glsl::Version::new_gles(number)
    } else {
        glsl::Version::Desktop(number)
    })
}

pub fn shader_model(model: &str) -> Result<hlsl::ShaderModel, JsError> {
    use hlsl::ShaderModel as M;
    Ok(match model {
        "5_0" => M::V5_0,
        "5_1" => M::V5_1,
        "6_0" => M::V6_0,
        "6_1" => M::V6_1,
        "6_2" => M::V6_2,
        "6_3" => M::V6_3,
        "6_4" => M::V6_4,
        "6_5" => M::V6_5,
        "6_6" => M::V6_6,
        "6_7" => M::V6_7,
        "6_8" => M::V6_8,
        "6_9" => M::V6_9,
        _ => {
            return Err(JsError::new(&format!(
                "invalid HLSL shader model: {model}"
            )));
        }
    })
}
