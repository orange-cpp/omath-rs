//! Small fixed-function DX9 backend. Expands ImGui's indexed triangles to avoid
//! keeping default-pool buffers alive across Reset. Texture IDs are DX9 textures.
use imgui::{DrawCmd, DrawData, internal::RawWrapper};
use windows::{
    Win32::{Foundation::RECT, Graphics::Direct3D9::*},
    core::Interface,
};
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 4],
    color: u32,
    uv: [f32; 2],
}
#[derive(Default)]
pub struct Renderer {
    font: Option<IDirect3DTexture9>,
}
impl Renderer {
    pub fn release(&mut self) {
        self.font = None;
    }
    pub fn prepare(
        &mut self,
        context: &mut imgui::Context,
        device: &IDirect3DDevice9,
    ) -> windows::core::Result<()> {
        if self.font.is_some() {
            return Ok(());
        }
        let atlas = context.fonts().build_rgba32_texture();
        unsafe {
            let mut texture = None;
            device.CreateTexture(
                atlas.width,
                atlas.height,
                1,
                D3DUSAGE_DYNAMIC as u32,
                D3DFMT_A8R8G8B8,
                D3DPOOL_DEFAULT,
                &mut texture,
                std::ptr::null_mut(),
            )?;
            let texture = texture.unwrap();
            let mut lock = D3DLOCKED_RECT::default();
            texture.LockRect(0, &mut lock, std::ptr::null(), D3DLOCK_DISCARD as u32)?;
            for y in 0..atlas.height as usize {
                for x in 0..atlas.width as usize {
                    let at = (y * atlas.width as usize + x) * 4;
                    let rgba = &atlas.data[at..at + 4];
                    let bgra = [rgba[2], rgba[1], rgba[0], rgba[3]];
                    std::ptr::copy_nonoverlapping(
                        bgra.as_ptr(),
                        (lock.pBits as *mut u8).add(y * lock.Pitch as usize + x * 4),
                        4,
                    );
                }
            }
            texture.UnlockRect(0)?;
            context.fonts().tex_id = imgui::TextureId::new(texture.as_raw() as usize);
            self.font = Some(texture);
        }
        Ok(())
    }
    unsafe fn setup(data: &DrawData, device: &IDirect3DDevice9) -> windows::core::Result<()> {
        unsafe {
            device.SetVertexShader(None)?;
            device.SetPixelShader(None)?;
            device.SetFVF(D3DFVF_XYZRHW | D3DFVF_DIFFUSE | D3DFVF_TEX1)?;
            for (key, value) in [
                (D3DRS_ZENABLE, 0),
                (D3DRS_ZWRITEENABLE, 0),
                (D3DRS_CULLMODE, D3DCULL_NONE.0 as u32),
                (D3DRS_LIGHTING, 0),
                (D3DRS_FOGENABLE, 0),
                (D3DRS_ALPHATESTENABLE, 0),
                (D3DRS_STENCILENABLE, 0),
                (D3DRS_SCISSORTESTENABLE, 1),
                (D3DRS_ALPHABLENDENABLE, 1),
                (D3DRS_BLENDOP, D3DBLENDOP_ADD.0 as u32),
                (D3DRS_SRCBLEND, D3DBLEND_SRCALPHA.0 as u32),
                (D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA.0 as u32),
                (D3DRS_SEPARATEALPHABLENDENABLE, 1),
                (D3DRS_SRCBLENDALPHA, D3DBLEND_ONE.0 as u32),
                (D3DRS_DESTBLENDALPHA, D3DBLEND_INVSRCALPHA.0 as u32),
                (D3DRS_COLORWRITEENABLE, 15),
                (D3DRS_SRGBWRITEENABLE, 0),
                (D3DRS_FILLMODE, D3DFILL_SOLID.0 as u32),
                (D3DRS_SHADEMODE, D3DSHADE_GOURAUD.0 as u32),
                (D3DRS_CLIPPING, 1),
                (D3DRS_RANGEFOGENABLE, 0),
                (D3DRS_BLENDOPALPHA, D3DBLENDOP_ADD.0 as u32),
            ] {
                device.SetRenderState(key, value)?;
            }
            for (key, value) in [
                (D3DTSS_COLOROP, D3DTOP_MODULATE.0 as u32),
                (D3DTSS_COLORARG1, D3DTA_TEXTURE),
                (D3DTSS_COLORARG2, D3DTA_DIFFUSE),
                (D3DTSS_ALPHAOP, D3DTOP_MODULATE.0 as u32),
                (D3DTSS_ALPHAARG1, D3DTA_TEXTURE),
                (D3DTSS_ALPHAARG2, D3DTA_DIFFUSE),
                (D3DTSS_TEXCOORDINDEX, 0),
                (D3DTSS_TEXTURETRANSFORMFLAGS, D3DTTFF_DISABLE.0 as u32),
            ] {
                device.SetTextureStageState(0, key, value)?;
            }
            device.SetTextureStageState(1, D3DTSS_COLOROP, D3DTOP_DISABLE.0 as u32)?;
            device.SetTextureStageState(1, D3DTSS_ALPHAOP, D3DTOP_DISABLE.0 as u32)?;
            for key in [D3DSAMP_MINFILTER, D3DSAMP_MAGFILTER] {
                device.SetSamplerState(0, key, D3DTEXF_LINEAR.0 as u32)?;
            }
            device.SetSamplerState(0, D3DSAMP_SRGBTEXTURE, 0)?;
            device.SetSamplerState(0, D3DSAMP_MIPFILTER, D3DTEXF_NONE.0 as u32)?;
            for key in [D3DSAMP_ADDRESSU, D3DSAMP_ADDRESSV] {
                device.SetSamplerState(0, key, D3DTADDRESS_CLAMP.0 as u32)?;
            }
            let scale = data.framebuffer_scale;
            let width = (data.display_size[0] * scale[0]) as u32;
            let height = (data.display_size[1] * scale[1]) as u32;
            device.SetViewport(&D3DVIEWPORT9 {
                X: 0,
                Y: 0,
                Width: width,
                Height: height,
                MinZ: 0.0,
                MaxZ: 1.0,
            })?;
        }
        Ok(())
    }
    pub fn render(&self, data: &DrawData, device: &IDirect3DDevice9) -> windows::core::Result<()> {
        if data.display_size[0] <= 0.0 || data.display_size[1] <= 0.0 {
            return Ok(());
        }
        unsafe {
            let state = device.CreateStateBlock(D3DSBT_ALL)?;
            state.Capture()?;
            // Even an early API error must hand all state back to the host.
            struct Restore(IDirect3DStateBlock9);
            impl Drop for Restore {
                fn drop(&mut self) {
                    let _ = unsafe { self.0.Apply() };
                }
            }
            let _restore = Restore(state);
            Self::setup(data, device)?;
            let scale = data.framebuffer_scale;
            let width = (data.display_size[0] * scale[0]) as u32;
            let height = (data.display_size[1] * scale[1]) as u32;
            let mut vertices = Vec::new();
            for list in data.draw_lists() {
                for command in list.commands() {
                    match command {
                        DrawCmd::Elements {
                            count,
                            cmd_params: p,
                        } => {
                            let clip = p.clip_rect;
                            let x = ((clip[0] - data.display_pos[0]) * scale[0])
                                .clamp(0.0, width as f32)
                                as i32;
                            let y = ((clip[1] - data.display_pos[1]) * scale[1])
                                .clamp(0.0, height as f32)
                                as i32;
                            let right = ((clip[2] - data.display_pos[0]) * scale[0])
                                .clamp(0.0, width as f32)
                                as i32;
                            let bottom = ((clip[3] - data.display_pos[1]) * scale[1])
                                .clamp(0.0, height as f32)
                                as i32;
                            if right <= x || bottom <= y {
                                continue;
                            }
                            device.SetScissorRect(&RECT {
                                left: x,
                                top: y,
                                right,
                                bottom,
                            })?;
                            let raw = p.texture_id.id() as *mut std::ffi::c_void;
                            let texture = IDirect3DBaseTexture9::from_raw_borrowed(&raw);
                            device.SetTexture(0, texture)?;
                            vertices.clear();
                            for &index in &list.idx_buffer()[p.idx_offset..p.idx_offset + count] {
                                let v = list.vtx_buffer()[p.vtx_offset + index as usize];
                                vertices.push(Vertex {
                                    position: [
                                        (v.pos[0] - data.display_pos[0]) * scale[0] - 0.5,
                                        (v.pos[1] - data.display_pos[1]) * scale[1] - 0.5,
                                        0.0,
                                        1.0,
                                    ],
                                    color: u32::from_le_bytes([
                                        v.col[2], v.col[1], v.col[0], v.col[3],
                                    ]),
                                    uv: v.uv,
                                });
                            }
                            device.DrawPrimitiveUP(
                                D3DPT_TRIANGLELIST,
                                (count / 3) as u32,
                                vertices.as_ptr().cast(),
                                std::mem::size_of::<Vertex>() as u32,
                            )?;
                        }
                        DrawCmd::RawCallback { callback, raw_cmd } => callback(list.raw(), raw_cmd),
                        DrawCmd::ResetRenderState => Self::setup(data, device)?,
                    }
                }
            }
        }
        Ok(())
    }
}
