use gloo_timers::callback::Interval;
use std::cell::RefCell;
use web_sys::CanvasRenderingContext2d;

use crate::domain::{
    geometry::{
        coordinate::{CartesianP, MatrixP},
        poligon::{
            rect::{get_length, move_by, zoom_in, zoom_out, zoom_to},
            square::Sq,
        },
    },
    preset::{get_preset, get_preset_groups, get_preset_unsafe, Preset},
    render::{get_values_to_render, RenderSettings},
    universe::{get_camera, iterate, toggle_cell, toggle_cell_by_absolute_point, Universe},
};

pub struct PresetOptionItem {
    pub label: String,
    pub value: String,
}

pub struct PresetOptionGroup {
    pub label: String,
    pub value: String,
    pub options: Vec<PresetOptionItem>,
}

pub fn build_presets() -> Vec<Preset> {
    get_preset_groups()
        .iter()
        .flat_map(|group| {
            group
                .sub_groups
                .iter()
                .flat_map(|sub_group| sub_group.items.clone())
                .collect::<Vec<Preset>>()
        })
        .collect()
}

pub fn build_preset_option_groups() -> Vec<PresetOptionGroup> {
    get_preset_groups()
        .iter()
        .map(|group| PresetOptionGroup {
            label: group.info.name.clone(),
            value: group.info.id.clone(),
            options: group
                .sub_groups
                .iter()
                .flat_map(|sub_group| sub_group.items.clone())
                .map(|item| PresetOptionItem { label: item.name, value: item.id })
                .collect(),
        })
        .collect()
}

pub trait DrawContext {
    fn clear(&self, s: Sq);
    fn draw_square(&self, s: Sq, color: String);
}

#[derive(Clone)]
pub struct Holder {
    context: CanvasRenderingContext2d,
}

impl DrawContext for Holder {
    fn clear(&self, s: Sq) {
        self.context.set_fill_style_str("white");
        self.context.fill_rect(s.x as f64, s.y as f64, s.size as f64, s.size as f64);
    }

    fn draw_square(&self, s: Sq, color: String) {
        self.context.set_fill_style(&color.into());
        self.context.fill_rect(s.x as f64, s.y as f64, s.size as f64, s.size as f64);
    }
}

unsafe impl Send for Holder {}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Resumed,
    Paused,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AppSettings {
    pub preset: Option<String>,
    pub fps: u16,
    pub status: Status,
    pub render_settings: RenderSettings,
}

pub struct Model {
    pub universe: Universe,
    pub settings: AppSettings,
    pub holder: Option<Holder>,
}

impl Default for Model {
    fn default() -> Self {
        let universe = get_preset_unsafe("block");
        let cam = get_camera(&universe);
        Model {
            universe,
            settings: AppSettings {
                preset: Some(String::from("block")),
                fps: 4,
                status: Status::Paused,
                render_settings: RenderSettings { cam, dim: 0, gap: 0 },
            },
            holder: None,
        }
    }
}

thread_local! {
    static MODEL: RefCell<Model> = RefCell::new(Model::default());
}

thread_local! {
    static LISTENERS: RefCell<Vec<Box<dyn FnMut(Prop) + 'static>>> = RefCell::new(Vec::new());
}

#[derive(Debug, Clone)]
pub enum Prop {
    Universe,
    Preset,
    Gap,
    FPS,
    Status,
    Dim,
    Cam,
}

pub fn add_on_change_listener<F>(cb: F)
where
    F: FnMut(Prop) + 'static,
{
    LISTENERS.with_borrow_mut(|l| l.push(Box::new(cb)));
}

fn on_change(param: Prop) {
    LISTENERS.with_borrow_mut(|l| {
        for cb in l.iter_mut() {
            cb(param.clone());
        }
    });
}

fn fps_to_mili(fps: u16) -> u16 {
    1000 / fps
}

const DEAD_COLOR: &str = "#dbdbdb";
const ALIVE_COLOR: &str = "#2e2e2e";

fn render() {
    let (universe, settings, holder) = MODEL.with(|i| {
        let m = i.borrow();
        (m.universe.clone(), m.settings.clone(), m.holder.clone())
    });
    if let Some(holder) = holder {
        let bg = Sq { x: 0, y: 0, size: settings.render_settings.dim.into() };
        holder.draw_square(bg, DEAD_COLOR.to_string());
        let values_to_render = get_values_to_render(&universe, &settings.render_settings);
        for sq in values_to_render {
            holder.draw_square(sq, ALIVE_COLOR.to_string());
        }
    }
}

pub enum Command {
    Start,
    Stop,
}

pub fn app_init(context: CanvasRenderingContext2d) {
    MODEL.with(|i| i.borrow_mut().holder = Some(Holder { context }));
    let mut interval: Option<Interval> = None;

    add_on_change_listener({
        move |prop| {
            let status = MODEL.with(|i| i.borrow().settings.status.clone());
            match status {
                Status::Resumed => match prop {
                    Prop::Status | Prop::FPS => {
                        if interval.is_none() {
                            if let Some(i) = interval.take() {
                                i.cancel();
                            }
                        }
                        let fps = MODEL.with(|i| i.borrow().settings.fps);
                        interval = Some(Interval::new(fps_to_mili(fps).into(), || {
                            app_iterate();
                            render();
                        }))
                    }
                    _ => {}
                },
                Status::Paused => match prop {
                    Prop::Gap | Prop::Dim | Prop::Universe | Prop::Cam => {
                        render();
                    }
                    Prop::Status => {
                        if let Some(i) = interval.take() {
                            i.cancel();
                        }
                    }
                    _ => {}
                },
            }
        }
    });
    render();
    //app_pause();
}

pub fn app_pause() {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        model.settings.status = Status::Paused;
    });
    on_change(Prop::Status);
}

pub fn app_resume() {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        model.settings.status = Status::Resumed;
    });
    on_change(Prop::Status);
}

pub fn app_set_dimension(dim: u16) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        model.settings.render_settings.dim = dim;
    });
    on_change(Prop::Dim);
}

pub fn app_set_gap(gap: u8) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        model.settings.render_settings.gap = gap;
    });
    on_change(Prop::Gap);
}

pub fn app_set_fps(fps: u16) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        model.settings.fps = fps;
    });
    on_change(Prop::FPS);
}

pub fn app_set_preset(preset: String) {
    if let Some(selected_preset) = get_preset(&preset) {
        MODEL.with(|i| {
            let mut model = i.borrow_mut();
            model.settings.render_settings.cam = get_camera(&selected_preset);
            model.universe = selected_preset;
            model.settings.preset = Some(preset);
        });
        on_change(Prop::Universe);
        on_change(Prop::Preset);
        on_change(Prop::Cam);
    }
}

pub fn app_single_iteration() {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        model.settings.status = Status::Paused;
        iterate(&mut model.universe);
    });
    on_change(Prop::Status);
    on_change(Prop::Universe);
}

pub fn app_iterate() {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        iterate(&mut model.universe);
    });
    on_change(Prop::Universe);
}

pub fn app_toggle_by_point(p: CartesianP) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        toggle_cell(&mut model.universe, p);
        model.settings.preset = None;
    });
    on_change(Prop::Universe);
    on_change(Prop::Preset);
}

pub fn app_toggle_model_cell_by_absolute_point(p: MatrixP) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        let render_settings = model.settings.render_settings.clone();
        toggle_cell_by_absolute_point(&mut model.universe, &render_settings, p);
        model.settings.preset = None;
    });
    on_change(Prop::Universe);
    on_change(Prop::Preset);
}

pub fn app_zoom_in() {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        zoom_in(&mut model.settings.render_settings.cam);
    });
    on_change(Prop::Cam);
}

pub fn app_zoom_out() {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        zoom_out(&mut model.settings.render_settings.cam);
    });
    on_change(Prop::Cam);
}

pub fn app_zoom_to(new_size: u16) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        zoom_to(&mut model.settings.render_settings.cam, new_size);
    });
    on_change(Prop::Cam);
}

pub fn app_move_model(delta: CartesianP) {
    MODEL.with(|i| {
        let mut model = i.borrow_mut();
        move_by(&mut model.settings.render_settings.cam, delta);
    });
    on_change(Prop::Universe);
}

#[derive(Debug, PartialEq)]
pub struct AppInfo {
    pub preset: Option<String>,
    pub gap: u8,
    pub size: u16,
    pub fps: u16,
    pub status: Status,
    pub age: u64,
}

pub fn app_get_settings() -> AppInfo {
    MODEL.with(|i| {
        let m = i.borrow();
        let s = m.settings.clone();
        let u = m.universe.clone();
        AppInfo {
            preset: s.preset,
            gap: s.render_settings.gap,
            size: get_length(&s.render_settings.cam).try_into().unwrap(),
            fps: s.fps,
            status: s.status,
            age: u.age,
        }
    })
}

#[cfg(test)]
mod test {
    use crate::domain::{cell::State, geometry::poligon::rect::Rect};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_instance() {
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 4,
                status: Status::Paused,
                render_settings: RenderSettings { cam: Rect::from(-5, -4, 4, 5), dim: 0, gap: 0 }
            }
        );
        assert_eq!(MODEL.with(|i| i.borrow().universe.clone()), get_preset_unsafe("block"));
        let settings = app_get_settings();
        assert_eq!(
            AppInfo {
                preset: Some("block".to_string()),
                gap: 0,
                size: 10,
                fps: 4,
                status: Status::Paused,
                age: 0,
            },
            settings
        );

        app_pause();
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 4,
                status: Status::Paused,
                render_settings: RenderSettings { cam: Rect::from(-5, -4, 4, 5), dim: 0, gap: 0 }
            }
        );

        app_resume();
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 4,
                status: Status::Resumed,
                render_settings: RenderSettings { cam: Rect::from(-5, -4, 4, 5), dim: 0, gap: 0 }
            }
        );

        app_set_dimension(1080);
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 4,
                status: Status::Resumed,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -4, 4, 5),
                    dim: 1080,
                    gap: 0
                }
            }
        );

        app_set_gap(2);
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 4,
                status: Status::Resumed,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -4, 4, 5),
                    dim: 1080,
                    gap: 2
                }
            }
        );

        app_set_fps(60);
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Resumed,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -4, 4, 5),
                    dim: 1080,
                    gap: 2,
                }
            }
        );

        app_set_preset("Gaius Julius Caesar".to_string());
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Resumed,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -4, 4, 5),
                    dim: 1080,
                    gap: 2,
                }
            }
        );
        app_set_preset("r_pentomino".to_string());
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("r_pentomino")),
                fps: 60,
                status: Status::Resumed,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -5, 5, 5),
                    dim: 1080,
                    gap: 2,
                }
            }
        );
        app_set_preset("block".to_string());
        app_iterate();
        let block = get_preset_unsafe("block");
        assert_eq!(
            MODEL.with(|i| i.borrow().universe.clone()),
            Universe { age: 1, value: block.value.clone() }
        );
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Resumed,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -4, 4, 5),
                    dim: 1080,
                    gap: 2,
                }
            }
        );

        app_single_iteration();
        assert_eq!(
            MODEL.with(|i| i.borrow().universe.clone()),
            Universe { age: 2, value: block.value.clone() }
        );
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Paused,
                render_settings: RenderSettings {
                    cam: Rect::from(-5, -4, 4, 5),
                    dim: 1080,
                    gap: 2,
                }
            }
        );

        app_zoom_to(40);
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Paused,
                render_settings: RenderSettings {
                    cam: Rect::from(-20, -19, 19, 20),
                    dim: 1080,
                    gap: 2,
                }
            }
        );
        app_zoom_in();
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Paused,
                render_settings: RenderSettings {
                    cam: Rect::from(-19, -18, 18, 19),
                    dim: 1080,
                    gap: 2,
                }
            }
        );
        app_zoom_out();
        assert_eq!(
            MODEL.with(|i| i.borrow().settings.clone()),
            AppSettings {
                preset: Some(String::from("block")),
                fps: 60,
                status: Status::Paused,
                render_settings: RenderSettings {
                    cam: Rect::from(-20, -19, 19, 20),
                    dim: 1080,
                    gap: 2,
                }
            }
        );

        app_move_model(CartesianP::from(20, 20));
        assert_eq!(
            MODEL.with(|i| i.borrow().universe.clone()),
            Universe { age: 2, value: block.value.clone() }
        );

        app_toggle_by_point(CartesianP::from(0, 0));
        assert_eq!(
            MODEL.with(|i| i.borrow().universe.clone()),
            Universe {
                age: 2,
                value: HashMap::from([
                    (CartesianP::from(-1, 1), State::Alive),
                    (CartesianP::from(0, 1), State::Alive),
                    (CartesianP::from(-1, 0), State::Alive),
                ])
            }
        );
    }
}
