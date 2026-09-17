use std::collections::HashMap;

use serde::Deserialize;
use upstream::back::{glsl, hlsl};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
export type ShaderStage = "vertex" | "fragment" | "compute";

export interface GlslParseOptions {
	stage: ShaderStage;
	defines?: Record<string, string>;
}

export interface GlslWriteOptions {
	version: `${number}` | `${number} es`;
	stage: ShaderStage;
	entryPoint: string;
}

export interface HlslWriteOptions {
	shaderModel?: "5_0" | "5_1" | "6_0" | "6_1" | "6_2" | "6_3" | "6_4" | "6_5" | "6_6" | "6_7" | "6_8" | "6_9";
}

export interface MslWriteOptions {
	langVersion?: [major: number, minor: number];
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
    #[serde(default)]
    defines: HashMap<String, String>,
}

impl From<GlslParseOptions> for upstream::front::glsl::Options {
    fn from(options: GlslParseOptions) -> Self {
        Self {
            stage: options.stage.into(),
            defines: options.defines.into_iter().collect(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlslWriteOptions {
    pub version: String,
    pub stage: Stage,
    pub entry_point: String,
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
