use std::ffi::c_void;
use std::ptr;

use libmpv2_sys as sys;

pub struct RenderContext {
    ctx: *mut sys::mpv_render_context,
}

impl RenderContext {
    pub fn new(mpv: &libmpv2::Mpv) -> libmpv2::Result<Self> {
        let mut params = [
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                data: sys::MPV_RENDER_API_TYPE_SW.as_ptr() as *mut c_void,
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let mut ctx: *mut sys::mpv_render_context = ptr::null_mut();
        let code = unsafe {
            sys::mpv_render_context_create(&mut ctx, mpv.ctx.as_ptr(), params.as_mut_ptr())
        };
        if code < 0 {
            return Err(libmpv2::Error::Raw(code));
        }
        Ok(Self { ctx })
    }
    pub fn update(&self) -> u64 {
        unsafe { sys::mpv_render_context_update(self.ctx) }
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe { sys::mpv_render_context_free(self.ctx) };
    }
}
