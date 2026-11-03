//! Real DX9 shader/target/state-restoration smoke test, without attaching to a game.
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
#[allow(dead_code)]
#[path = "support/desktop.rs"]
mod desktop;
#[path = "support/kf1/mod.rs"]
mod kf1;
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    unsafe {
        use windows::Win32::Graphics::Direct3D9::*;
        let window = desktop::Window::new(false)?;
        let d3d = Direct3DCreate9(D3D_SDK_VERSION).ok_or("Direct3DCreate9 failed")?;
        let mut pp = D3DPRESENT_PARAMETERS {
            BackBufferWidth: 256,
            BackBufferHeight: 256,
            BackBufferFormat: D3DFMT_A8R8G8B8,
            SwapEffect: D3DSWAPEFFECT_DISCARD,
            hDeviceWindow: window.hwnd,
            Windowed: true.into(),
            ..Default::default()
        };
        let mut device = None;
        d3d.CreateDevice(
            0,
            D3DDEVTYPE_HAL,
            window.hwnd,
            D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32,
            &mut pp,
            &mut device,
        )?;
        let device = device.unwrap();
        let mut glow = kf1::glow::ActorGlow::default();
        if !glow.prepare(&device, 256, 256) {
            return Err(glow.error.into());
        }
        device.Clear(
            0,
            std::ptr::null(),
            D3DCLEAR_TARGET as u32,
            0xff000000,
            1.0,
            0,
        )?;
        device.BeginScene()?;
        #[repr(C)]
        struct Vertex {
            x: f32,
            y: f32,
            z: f32,
            rhw: f32,
            color: u32,
        }
        let tri = [
            Vertex {
                x: 96.0,
                y: 96.0,
                z: 0.0,
                rhw: 1.0,
                color: 0xffffffff,
            },
            Vertex {
                x: 160.0,
                y: 96.0,
                z: 0.0,
                rhw: 1.0,
                color: 0xffffffff,
            },
            Vertex {
                x: 128.0,
                y: 160.0,
                z: 0.0,
                rhw: 1.0,
                color: 0xffffffff,
            },
        ];
        device.SetFVF(D3DFVF_XYZRHW | D3DFVF_DIFFUSE)?;
        let target = device.GetRenderTarget(0)?;
        let mut before = 0;
        device.GetRenderState(D3DRS_CULLMODE, &mut before)?;
        assert!(glow.draw_silhouette(&device, || device.DrawPrimitiveUP(
            D3DPT_TRIANGLELIST,
            1,
            tri.as_ptr().cast(),
            std::mem::size_of::<Vertex>() as u32
        ))?);
        assert!(glow.blur(&device, 5.0)?);
        assert!(!glow.blur(&device, 1.0)?);
        assert!(glow.draw_cutout(&device, || device.DrawPrimitiveUP(
            D3DPT_TRIANGLELIST,
            1,
            tri.as_ptr().cast(),
            std::mem::size_of::<Vertex>() as u32
        ))?);
        assert!(glow.lay(&device, [0.31, 0.86, 0.47])?);
        assert!(!glow.lay(&device, [1.0; 3])?);
        let mut after = 0;
        device.GetRenderState(D3DRS_CULLMODE, &mut after)?;
        assert_eq!(before, after);
        assert_eq!(device.GetRenderTarget(0)?, target);
        device.EndScene()?;
        let mut description = D3DSURFACE_DESC::default();
        target.GetDesc(&mut description)?;
        let mut readback = None;
        device.CreateOffscreenPlainSurface(
            256,
            256,
            description.Format,
            D3DPOOL_SYSTEMMEM,
            &mut readback,
            std::ptr::null_mut(),
        )?;
        let readback = readback.unwrap();
        device.GetRenderTargetData(&target, &readback)?;
        let mut locked = D3DLOCKED_RECT::default();
        readback.LockRect(&mut locked, std::ptr::null(), D3DLOCK_READONLY as u32)?;
        let mut pixels = 0;
        for y in 0..256 {
            for x in 0..256 {
                let value = std::ptr::read_unaligned(
                    (locked.pBits as *const u8)
                        .offset(y * locked.Pitch as isize + x * 4)
                        .cast::<u32>(),
                );
                if value & 0xffffff != 0 {
                    pixels += 1;
                }
            }
        }
        readback.UnlockRect()?;
        assert!(pixels > 0, "blur/composite produced no pixels");
        glow.release();
        drop(readback);
        drop(target);
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        context.io_mut().display_size = [256.0, 256.0];
        context.io_mut().delta_time = 1.0 / 60.0;
        let mut renderer = kf1::dx9_imgui::Renderer::default();
        for pass in 0..2 {
            renderer.prepare(&mut context, &device)?;
            let ui = context.frame();
            ui.get_background_draw_list()
                .add_rect([20.0, 20.0], [80.0, 80.0], [1.0, 0.0, 0.0, 1.0])
                .filled(true)
                .build();
            ui.get_background_draw_list()
                .add_text([30.0, 90.0], [1.0; 4], "DX9 font");
            device.Clear(
                0,
                std::ptr::null(),
                D3DCLEAR_TARGET as u32,
                0xff000000,
                1.0,
                0,
            )?;
            device.BeginScene()?;
            device.SetRenderState(D3DRS_CULLMODE, D3DCULL_CW.0 as u32)?;
            renderer.render(context.render(), &device)?;
            let mut cull = 0;
            device.GetRenderState(D3DRS_CULLMODE, &mut cull)?;
            assert_eq!(cull, D3DCULL_CW.0 as u32);
            device.EndScene()?;
            let surface = device.GetRenderTarget(0)?;
            let mut readback = None;
            device.CreateOffscreenPlainSurface(
                256,
                256,
                D3DFMT_A8R8G8B8,
                D3DPOOL_SYSTEMMEM,
                &mut readback,
                std::ptr::null_mut(),
            )?;
            let readback = readback.unwrap();
            device.GetRenderTargetData(&surface, &readback)?;
            let mut lock = D3DLOCKED_RECT::default();
            readback.LockRect(&mut lock, std::ptr::null(), D3DLOCK_READONLY as u32)?;
            let pixel = std::ptr::read_unaligned(
                (lock.pBits as *const u8)
                    .add(40 * lock.Pitch as usize + 40 * 4)
                    .cast::<u32>(),
            );
            readback.UnlockRect()?;
            assert_eq!(
                pixel & 0xffffff,
                0xff0000,
                "ImGui rectangle did not render in pass {pass}"
            );
            drop(readback);
            drop(surface);
            renderer.release();
            if pass == 0 {
                device.Reset(&mut pp)?;
            }
        }
        println!(
            "DX9 mask, blur, composite and state restoration passed ({pixels} non-black pixels)"
        );
        println!(
            "DX9 ImGui texture, indexed geometry, state restoration and Reset recreation passed"
        );
        Ok(())
    }
}
#[cfg(not(all(windows, feature = "hooks", feature = "imgui")))]
fn main() {
    println!("Enable hooks,imgui on Windows to exercise the DX9 glow renderer.");
}
