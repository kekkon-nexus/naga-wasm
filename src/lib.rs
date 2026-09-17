mod options;

use serde::de::DeserializeOwned;
use upstream::back::{glsl, hlsl, msl, spv, wgsl};
use upstream::valid::{Capabilities, ValidationFlags, Validator};
use wasm_bindgen::prelude::*;

use crate::options::{
    GlslParseOptions, GlslWriteOptions, HlslWriteOptions, MslWriteOptions,
    SpirvWriteOptions, WgslWriteOptions,
};

#[wasm_bindgen]
pub struct Module {
    module: upstream::Module,
    source: String,
    path: &'static str,
}

#[wasm_bindgen]
pub struct ModuleInfo(upstream::valid::ModuleInfo);

fn options<T: DeserializeOwned + Default>(
    options: JsValue,
) -> Result<T, JsError> {
    let options: Option<T> = serde_wasm_bindgen::from_value(options)?;
    Ok(options.unwrap_or_default())
}

#[wasm_bindgen(js_name = parseWgsl)]
pub fn parse_wgsl(source: String) -> Result<Module, JsError> {
    match upstream::front::wgsl::parse_str(&source) {
        Ok(module) => Ok(Module {
            module,
            source,
            path: "wgsl",
        }),
        Err(error) => Err(JsError::new(&error.emit_to_string(&source))),
    }
}

#[wasm_bindgen(js_name = parseGlsl)]
pub fn parse_glsl(
    source: String,
    #[wasm_bindgen(unchecked_param_type = "GlslParseOptions")] options: JsValue,
) -> Result<Module, JsError> {
    let options: GlslParseOptions = serde_wasm_bindgen::from_value(options)?;
    match upstream::front::glsl::Frontend::default()
        .parse(&options.into(), &source)
    {
        Ok(module) => Ok(Module {
            module,
            source,
            path: "glsl",
        }),
        Err(error) => Err(JsError::new(&error.emit_to_string(&source))),
    }
}

#[wasm_bindgen(js_name = parseSpirv)]
pub fn parse_spirv(bytes: &[u8]) -> Result<Module, JsError> {
    match upstream::front::spv::parse_u8_slice(bytes, &Default::default()) {
        Ok(module) => Ok(Module {
            module,
            source: String::new(),
            path: "spv",
        }),
        Err(error) => Err(JsError::new(&error.emit_to_string(""))),
    }
}

#[wasm_bindgen]
pub fn validate(module: &Module) -> Result<ModuleInfo, JsError> {
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module.module)
        .map(ModuleInfo)
        .map_err(|error| {
            JsError::new(
                &error.emit_to_string_with_path(&module.source, module.path),
            )
        })
}

#[wasm_bindgen(js_name = writeWgsl)]
pub fn write_wgsl(
    module: &Module,
    info: &ModuleInfo,
    #[wasm_bindgen(unchecked_optional_param_type = "WgslWriteOptions")]
    options: JsValue,
) -> Result<String, JsError> {
    let options: WgslWriteOptions = self::options(options)?;
    let mut flags = wgsl::WriterFlags::empty();
    options.flags.unwrap_or_default().apply(&mut flags);
    Ok(wgsl::write_string(&module.module, &info.0, flags)?)
}

#[wasm_bindgen(js_name = writeGlsl)]
pub fn write_glsl(
    module: &Module,
    info: &ModuleInfo,
    #[wasm_bindgen(unchecked_param_type = "GlslWriteOptions")] options: JsValue,
) -> Result<String, JsError> {
    let options: GlslWriteOptions = serde_wasm_bindgen::from_value(options)?;
    let mut glsl_options = glsl::Options {
        version: options::glsl_version(&options.version)?,
        ..Default::default()
    };
    options
        .flags
        .unwrap_or_default()
        .apply(&mut glsl_options.writer_flags);
    let mut out = String::new();
    glsl::Writer::new(
        &mut out,
        &module.module,
        &info.0,
        &glsl_options,
        &glsl::PipelineOptions {
            shader_stage: options.stage.into(),
            entry_point: options.entry_point,
            multiview: None,
        },
        upstream::proc::BoundsCheckPolicies::default(),
    )?
    .write()?;
    Ok(out)
}

#[wasm_bindgen(js_name = writeHlsl)]
pub fn write_hlsl(
    module: &Module,
    info: &ModuleInfo,
    #[wasm_bindgen(unchecked_optional_param_type = "HlslWriteOptions")]
    options: JsValue,
) -> Result<String, JsError> {
    let options: HlslWriteOptions = self::options(options)?;
    let mut hlsl_options = hlsl::Options::default();
    if let Some(model) = options.shader_model {
        hlsl_options.shader_model = options::shader_model(&model)?;
    }
    let mut out = String::new();
    hlsl::Writer::new(&mut out, &hlsl_options, &Default::default()).write(
        &module.module,
        &info.0,
        None,
    )?;
    Ok(out)
}

#[wasm_bindgen(js_name = writeMsl)]
pub fn write_msl(
    module: &Module,
    info: &ModuleInfo,
    #[wasm_bindgen(unchecked_optional_param_type = "MslWriteOptions")]
    options: JsValue,
) -> Result<String, JsError> {
    let options: MslWriteOptions = self::options(options)?;
    let mut msl_options = msl::Options::default();
    if let Some(version) = options.lang_version {
        msl_options.lang_version = version;
    }
    let (out, _) = msl::write_string(
        &module.module,
        &info.0,
        &msl_options,
        &Default::default(),
    )?;
    Ok(out)
}

#[wasm_bindgen(js_name = writeSpirv)]
pub fn write_spirv(
    module: &Module,
    info: &ModuleInfo,
    #[wasm_bindgen(unchecked_optional_param_type = "SpirvWriteOptions")]
    options: JsValue,
) -> Result<Vec<u32>, JsError> {
    let options: SpirvWriteOptions = self::options(options)?;
    let mut spv_options = spv::Options::default();
    options
        .flags
        .unwrap_or_default()
        .apply(&mut spv_options.flags);
    Ok(spv::write_vec(&module.module, &info.0, &spv_options, None)?)
}
