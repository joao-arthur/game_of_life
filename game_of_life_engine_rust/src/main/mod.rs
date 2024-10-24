use crate::{
    app::{
        add_on_change_listener, app_get_settings, app_init, app_move_model, app_pause, app_resume,
        app_set_dimension, app_set_fps, app_set_gap, app_set_preset, app_single_iteration,
        app_toggle_model_cell, app_toggle_model_cell_by_point, app_zoom, Status,
    },
    domain::{plane::cartesian::CartesianPoint, preset::get_preset_groups},
};
use js_sys::Function;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::CanvasRenderingContext2d;

#[wasm_bindgen]
#[derive(Clone)]
pub struct EngineCartesianPoint {
    pub x: i64,
    pub y: i64,
}

#[wasm_bindgen]
impl EngineCartesianPoint {
    #[wasm_bindgen(constructor)]
    pub fn new(x: i64, y: i64) -> EngineCartesianPoint {
        EngineCartesianPoint { x, y }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum EngineStatus {
    Resumed,
    Paused,
}

#[wasm_bindgen]
pub struct EngineInfo {
    preset: Option<String>,
    pub gap: u16,
    pub size: u16,
    pub fps: u16,
    pub status: EngineStatus,
    pub iter: u64,
}

#[wasm_bindgen]
impl EngineInfo {
    #[wasm_bindgen(getter)]
    pub fn preset(&self) -> Option<String> {
        self.preset.clone()
    }
}

#[derive(Serialize)]
pub struct EnginePresetInfo {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct EnginePresetGroup {
    pub info: EnginePresetInfo,
    pub items: Vec<EnginePresetInfo>,
}

#[wasm_bindgen(js_name = "engineInit")]
pub fn main_init(value: CanvasRenderingContext2d) {
    console::log_1(&"[init]".into());
    console::log_1(&"[init dyn_into!]".into());
    app_init(value);
}

#[wasm_bindgen(js_name = "enginePause")]
pub fn main_pause() {
    console::log_1(&"[pause]".into());
    app_pause();
}

#[wasm_bindgen(js_name = "engineResume")]
pub fn main_resume() {
    console::log_1(&"[resume]".into());
    app_resume();
}

#[wasm_bindgen(js_name = "engineSetDimension")]
pub fn main_set_dimension(dim: u16) {
    console::log_2(&"[set_dimension]".into(), &dim.into());
    app_set_dimension(dim);
}

#[wasm_bindgen(js_name = "engineSetGap")]
pub fn main_set_gap(gap: u16) {
    console::log_2(&"[set_gap]".into(), &gap.into());
    app_set_gap(gap);
}

#[wasm_bindgen(js_name = "engineSetFPS")]
pub fn main_set_fps(fps: u16) {
    console::log_2(&"[set_fps]".into(), &fps.into());
    app_set_fps(fps);
}

#[wasm_bindgen(js_name = "engineSetPreset")]
pub fn main_set_preset(preset: String) {
    console::log_2(&"[set_preset]".into(), &preset.clone().into());
    app_set_preset(preset);
}

#[wasm_bindgen(js_name = "engineSingleIteration")]
pub fn main_single_iteration() {
    console::log_1(&"[iterate]".into());
    app_single_iteration();
}

#[wasm_bindgen(js_name = "engineToggle")]
pub fn main_toggle_model_cell(point: EngineCartesianPoint) {
    console::log_2(&"[toggle]".into(), &point.clone().into());
    let cp = CartesianPoint {
        x: point.x,
        y: point.y,
    };
    app_toggle_model_cell(cp);
}

#[wasm_bindgen(js_name = "engineToggleByPoint")]
pub fn main_toggle_model_cell_point(point: EngineCartesianPoint) {
    console::log_2(&"[toggle by opoint]".into(), &point.clone().into());
    let cp = CartesianPoint {
        x: point.x,
        y: point.y,
    };
    app_toggle_model_cell_by_point(cp);
}

#[wasm_bindgen(js_name = "engineZoom")]
pub fn main_zoom(new_size: u16) {
    console::log_1(&"[zoom]".into());
    app_zoom(new_size);
}

#[wasm_bindgen(js_name = "engineMove")]
pub fn main_move_model(delta: EngineCartesianPoint) {
    console::log_2(&"[move]".into(), &delta.clone().into());
    let cp = CartesianPoint {
        x: delta.x,
        y: delta.y,
    };
    app_move_model(cp);
}

#[wasm_bindgen(js_name = "engineGetSettings")]
pub fn main_get_settings() -> EngineInfo {
    console::log_1(&"[get_settings]".into());
    let settings = app_get_settings();
    EngineInfo {
        preset: settings.preset,
        size: settings.size,
        fps: settings.fps,
        gap: settings.gap,
        status: match settings.status {
            Status::Paused => EngineStatus::Paused,
            Status::Resumed => EngineStatus::Resumed,
        },
        iter: settings.iter,
    }
}

#[wasm_bindgen(js_name = "engineAddOnChangeListener")]
pub fn main_add_on_change_listener(cb: Function) {
    console::log_1(&"[on_change_listener]".into());
    add_on_change_listener(move |_| {
        let this = JsValue::null();
        cb.call0(&this).unwrap();
    });
}

#[wasm_bindgen(js_name = "engineGetPresets")]
pub fn main_get_presets() -> JsValue {
    let groups: Vec<EnginePresetGroup> = get_preset_groups()
        .iter()
        .map(|g| EnginePresetGroup {
            info: EnginePresetInfo {
                id: g.info.id.clone(),
                name: g.info.name.clone(),
            },
            items: g
                .sub_groups
                .iter()
                .flat_map(|sub_group| {
                    sub_group
                        .items
                        .iter()
                        .map(|i| EnginePresetInfo {
                            id: i.id.clone(),
                            name: i.name.clone(),
                        })
                        .collect::<Vec<EnginePresetInfo>>()
                })
                .collect(),
        })
        .collect();
    serde_wasm_bindgen::to_value(&groups).unwrap()
}

