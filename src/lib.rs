use naga::back::glsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Module {
    module: naga::Module,
    source: String,
}

#[wasm_bindgen]
pub struct ModuleInfo(naga::valid::ModuleInfo);

#[wasm_bindgen(js_name = parseWgsl)]
pub fn parse_wgsl(source: String) -> Result<Module, JsError> {
    match naga::front::wgsl::parse_str(&source) {
        Ok(module) => Ok(Module { module, source }),
        Err(error) => Err(JsError::new(&error.emit_to_string(&source))),
    }
}

#[wasm_bindgen]
pub fn validate(module: &Module) -> Result<ModuleInfo, JsError> {
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module.module)
        .map(ModuleInfo)
        .map_err(|error| JsError::new(&error.emit_to_string(&module.source)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlslOptions {
    version: String,
    stage: Stage,
    entry_point: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Stage {
    Vertex,
    Fragment,
    Compute,
}

#[wasm_bindgen(js_name = writeGlsl)]
pub fn write_glsl(
    module: &Module,
    info: &ModuleInfo,
    options: JsValue,
) -> Result<String, JsError> {
    let options: GlslOptions = serde_wasm_bindgen::from_value(options)?;
    let shader_stage = match options.stage {
        Stage::Vertex => naga::ShaderStage::Vertex,
        Stage::Fragment => naga::ShaderStage::Fragment,
        Stage::Compute => naga::ShaderStage::Compute,
    };

    let mut out = String::new();
    glsl::Writer::new(
        &mut out,
        &module.module,
        &info.0,
        &glsl::Options {
            version: glsl_version(&options.version)?,
            ..Default::default()
        },
        &glsl::PipelineOptions {
            shader_stage,
            entry_point: options.entry_point,
            multiview: None,
        },
        naga::proc::BoundsCheckPolicies::default(),
    )?
    .write()?;
    Ok(out)
}

fn glsl_version(version: &str) -> Result<glsl::Version, JsError> {
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
