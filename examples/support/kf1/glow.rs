//! DX9 silhouette, separable blur and premultiplied composite passes from the KF1 example.
use std::{io, mem::size_of};
use windows::{
    Win32::Graphics::{
        Direct3D::{Fxc::D3DCompile, ID3DBlob},
        Direct3D9::*,
    },
    core::{PCSTR, s},
};
pub const VERTEX_SHADER: &str = include_str!("shaders/vertex.hlsl");
pub const MASK_SHADER: &str = include_str!("shaders/mask.hlsl");
pub const BLUR_SHADER: &str = include_str!("shaders/blur.hlsl");
pub const CLASSIC_SHADER: &str = include_str!("shaders/classic.hlsl");
pub fn compile(source: &str, profile: PCSTR) -> io::Result<ID3DBlob> {
    let mut code = None;
    let mut messages = None;
    let result = unsafe {
        D3DCompile(
            source.as_ptr().cast(),
            source.len(),
            PCSTR::null(),
            None,
            None,
            s!("main"),
            profile,
            0,
            0,
            &mut code,
            Some(&mut messages),
        )
    };
    if let Err(error) = result {
        let text = messages
            .map(|blob| unsafe {
                String::from_utf8_lossy(std::slice::from_raw_parts(
                    blob.GetBufferPointer().cast::<u8>(),
                    blob.GetBufferSize(),
                ))
                .trim_end_matches('\0')
                .to_owned()
            })
            .unwrap_or_else(|| error.to_string());
        return Err(io::Error::other(text));
    }
    code.ok_or_else(|| io::Error::other("shader compiler returned no code"))
}
#[derive(Default)]
struct Target {
    texture: Option<IDirect3DTexture9>,
    surface: Option<IDirect3DSurface9>,
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ClipVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}
const CLIP_FVF: u32 = D3DFVF_XYZ | D3DFVF_TEX1;
const SILHOUETTE_STATES: [(D3DRENDERSTATETYPE, u32); 7] = [
    (D3DRS_ZENABLE, 0),
    (D3DRS_STENCILENABLE, 0),
    (D3DRS_ALPHABLENDENABLE, 0),
    (D3DRS_ALPHATESTENABLE, 0),
    (D3DRS_FOGENABLE, 0),
    (D3DRS_SRGBWRITEENABLE, 0),
    (D3DRS_COLORWRITEENABLE, 15),
];
#[derive(Default)]
pub struct ActorGlow {
    pub error: String,
    width: u32,
    height: u32,
    vertex: Option<IDirect3DVertexShader9>,
    mask: Option<IDirect3DPixelShader9>,
    blur: Option<IDirect3DPixelShader9>,
    classic: Option<IDirect3DPixelShader9>,
    state: Option<IDirect3DStateBlock9>,
    targets: [Target; 3],
    mask_drawn: bool,
}
struct Restore {
    device: IDirect3DDevice9,
    target: IDirect3DSurface9,
    depth: Option<IDirect3DSurface9>,
    state: IDirect3DStateBlock9,
}
impl Drop for Restore {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.SetTexture(0, None);
            let _ = self.device.SetTexture(1, None);
            let _ = self.device.SetRenderTarget(0, &self.target);
            let _ = self.device.SetDepthStencilSurface(self.depth.as_ref());
            let _ = self.state.Apply();
        }
    }
}
impl ActorGlow {
    pub fn release(&mut self) {
        self.targets = Default::default();
        self.state = None;
        self.classic = None;
        self.blur = None;
        self.mask = None;
        self.vertex = None;
        self.mask_drawn = false;
    }
    pub fn prepare(&mut self, device: &IDirect3DDevice9, width: u32, height: u32) -> bool {
        if !self.error.is_empty() {
            return false;
        }
        if self.state.is_some() {
            return true;
        }
        self.width = width;
        self.height = height;
        let result = (|| -> io::Result<()> {
            unsafe {
                let code = compile(VERTEX_SHADER, s!("vs_3_0"))?;
                self.vertex = Some(
                    device
                        .CreateVertexShader(code.GetBufferPointer().cast())
                        .map_err(io::Error::other)?,
                );
                for (slot, source, profile) in [
                    (&mut self.mask, MASK_SHADER, s!("ps_2_0")),
                    (&mut self.blur, BLUR_SHADER, s!("ps_3_0")),
                    (&mut self.classic, CLASSIC_SHADER, s!("ps_3_0")),
                ] {
                    let code = compile(source, profile)?;
                    *slot = Some(
                        device
                            .CreatePixelShader(code.GetBufferPointer().cast())
                            .map_err(io::Error::other)?,
                    );
                }
                for target in &mut self.targets {
                    for format in [D3DFMT_X8R8G8B8, D3DFMT_A8R8G8B8] {
                        let mut texture = None;
                        if device
                            .CreateTexture(
                                width,
                                height,
                                1,
                                D3DUSAGE_RENDERTARGET as u32,
                                format,
                                D3DPOOL_DEFAULT,
                                &mut texture,
                                std::ptr::null_mut(),
                            )
                            .is_ok()
                        {
                            let texture = texture.unwrap();
                            if let Ok(surface) = texture.GetSurfaceLevel(0) {
                                target.texture = Some(texture);
                                target.surface = Some(surface);
                                break;
                            }
                        }
                    }
                    if target.surface.is_none() {
                        return Err(io::Error::other("render targets refused"));
                    }
                }
                self.state = Some(
                    device
                        .CreateStateBlock(D3DSBT_ALL)
                        .map_err(io::Error::other)?,
                );
                Ok(())
            }
        })();
        if let Err(error) = result {
            self.error = error.to_string();
            self.release();
            return false;
        }
        true
    }
    fn capture(&self, device: &IDirect3DDevice9) -> windows::core::Result<Restore> {
        unsafe {
            let state = self.state.as_ref().unwrap().clone();
            state.Capture()?;
            Ok(Restore {
                device: device.clone(),
                target: device.GetRenderTarget(0)?,
                depth: device.GetDepthStencilSurface().ok(),
                state,
            })
        }
    }
    pub fn draw_silhouette(
        &mut self,
        device: &IDirect3DDevice9,
        draw_again: impl FnOnce() -> windows::core::Result<()>,
    ) -> windows::core::Result<bool> {
        if self.state.is_none() {
            return Ok(false);
        }
        unsafe {
            let restore = self.capture(device)?;
            let mut desc = D3DSURFACE_DESC::default();
            restore.target.GetDesc(&mut desc)?;
            if desc.Width != self.width || desc.Height != self.height {
                return Ok(false);
            }
            let mut viewport = D3DVIEWPORT9::default();
            device.GetViewport(&mut viewport)?;
            if !self.mask_drawn {
                device.ColorFill(
                    self.targets[1].surface.as_ref(),
                    std::ptr::null(),
                    0xff000000,
                )?;
            }
            self.mask_drawn = true;
            for (state, value) in SILHOUETTE_STATES {
                device.SetRenderState(state, value)?;
            }
            device.SetRenderTarget(0, self.targets[1].surface.as_ref())?;
            device.SetDepthStencilSurface(None)?;
            device.SetViewport(&viewport)?;
            device.SetPixelShader(self.mask.as_ref())?;
            draw_again()?;
            drop(restore);
            Ok(true)
        }
    }
    pub fn clip_quad(&self) -> [ClipVertex; 4] {
        let x = 1.0 / self.width as f32;
        let y = 1.0 / self.height as f32;
        [
            ClipVertex {
                x: -1.0 - x,
                y: 1.0 + y,
                z: 0.0,
                u: 0.0,
                v: 0.0,
            },
            ClipVertex {
                x: 1.0 - x,
                y: 1.0 + y,
                z: 0.0,
                u: 1.0,
                v: 0.0,
            },
            ClipVertex {
                x: -1.0 - x,
                y: -1.0 + y,
                z: 0.0,
                u: 0.0,
                v: 1.0,
            },
            ClipVertex {
                x: 1.0 - x,
                y: -1.0 + y,
                z: 0.0,
                u: 1.0,
                v: 1.0,
            },
        ]
    }
    pub fn composite(
        &mut self,
        device: &IDirect3DDevice9,
        color: [f32; 3],
        thickness: f32,
    ) -> windows::core::Result<bool> {
        if !self.mask_drawn {
            return Ok(false);
        }
        self.mask_drawn = false;
        let restore = self.capture(device)?;
        let quad = self.clip_quad();
        unsafe {
            self.set_composite_states(device)?;
            device.SetPixelShader(self.blur.as_ref())?;
            for (from, to, axis, scale) in [
                (1, 2, [1.0 / self.width as f32, 0.0], 1.0),
                (2, 0, [0.0, 1.0 / self.width as f32], thickness),
            ] {
                device.SetTexture(0, None)?;
                device.SetRenderTarget(0, self.targets[to].surface.as_ref())?;
                device.SetTexture(0, self.targets[from].texture.as_ref().unwrap())?;
                let constants = [axis[0], axis[1], 0.0, 0.0, scale, 0.0, 0.0, 0.0];
                device.SetPixelShaderConstantF(0, constants.as_ptr(), 2)?;
                device.DrawPrimitiveUP(
                    D3DPT_TRIANGLESTRIP,
                    2,
                    quad.as_ptr().cast(),
                    size_of::<ClipVertex>() as u32,
                )?;
            }
            device.SetTexture(0, None)?;
            device.SetRenderTarget(0, &restore.target)?;
            for (state, value) in [
                (D3DRS_ALPHABLENDENABLE, 1),
                (D3DRS_BLENDOP, D3DBLENDOP_ADD.0 as u32),
                (D3DRS_SRCBLEND, D3DBLEND_ONE.0 as u32),
                (D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA.0 as u32),
                (D3DRS_SEPARATEALPHABLENDENABLE, 0),
            ] {
                device.SetRenderState(state, value)?;
            }
            device.SetTexture(0, self.targets[0].texture.as_ref().unwrap())?;
            device.SetTexture(1, self.targets[1].texture.as_ref().unwrap())?;
            device.SetPixelShader(self.classic.as_ref())?;
            let constant = [color[0] * 0.5, color[1] * 0.5, color[2] * 0.5, 1.0];
            device.SetPixelShaderConstantF(0, constant.as_ptr(), 1)?;
            device.DrawPrimitiveUP(
                D3DPT_TRIANGLESTRIP,
                2,
                quad.as_ptr().cast(),
                size_of::<ClipVertex>() as u32,
            )?;
        }
        drop(restore);
        Ok(true)
    }
    unsafe fn set_composite_states(&self, device: &IDirect3DDevice9) -> windows::core::Result<()> {
        unsafe {
            for (state, value) in [
                (D3DRS_ZENABLE, 0),
                (D3DRS_ZWRITEENABLE, 0),
                (D3DRS_ALPHATESTENABLE, 0),
                (D3DRS_SCISSORTESTENABLE, 0),
                (D3DRS_CULLMODE, D3DCULL_NONE.0 as u32),
                (D3DRS_FILLMODE, D3DFILL_SOLID.0 as u32),
                (D3DRS_COLORWRITEENABLE, 15),
                (D3DRS_SRGBWRITEENABLE, 0),
                (D3DRS_FOGENABLE, 0),
                (D3DRS_CLIPPING, 1),
                (D3DRS_ALPHABLENDENABLE, 0),
                (D3DRS_STENCILENABLE, 0),
                (D3DRS_TWOSIDEDSTENCILMODE, 0),
            ] {
                device.SetRenderState(state, value)?;
            }
            for stage in 0..2 {
                for (state, value) in [
                    (D3DSAMP_ADDRESSU, D3DTADDRESS_CLAMP.0 as u32),
                    (D3DSAMP_ADDRESSV, D3DTADDRESS_CLAMP.0 as u32),
                    (D3DSAMP_MINFILTER, D3DTEXF_LINEAR.0 as u32),
                    (D3DSAMP_MAGFILTER, D3DTEXF_LINEAR.0 as u32),
                    (D3DSAMP_MIPFILTER, D3DTEXF_NONE.0 as u32),
                    (D3DSAMP_SRGBTEXTURE, 0),
                ] {
                    device.SetSamplerState(stage, state, value)?;
                }
                device.SetTexture(stage, None)?;
            }
            device.SetVertexShader(self.vertex.as_ref())?;
            device.SetFVF(CLIP_FVF)?;
            device.SetDepthStencilSurface(None)?;
            Ok(())
        }
    }
}
