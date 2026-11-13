use std::ffi::c_void;
use std::ptr;

use libmpv2_sys as sys;

pub struct RenderContext {
    ctx: *mut sys::mpv_render_context,
}

impl RenderContext {
    pub fn new(ctx: *mut sys::mpv_render_context) -> Self {
        Self { ctx }
    }
}
