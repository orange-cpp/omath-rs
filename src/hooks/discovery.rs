use super::*;
use std::ptr::null_mut;
use windows::{
    Win32::{
        Foundation::{HMODULE, HWND},
        Graphics::{
            Direct3D::*,
            Direct3D9::*,
            Direct3D11::*,
            Direct3D12::*,
            Dxgi::{Common::*, *},
        },
        UI::WindowsAndMessaging::*,
    },
    core::{GUID, HRESULT, Interface, s, w},
};
struct Window(HWND);
impl Window {
    fn new() -> io::Result<Self> {
        Ok(Self(
            unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("STATIC"),
                    w!("omath hook discovery"),
                    WS_OVERLAPPEDWINDOW,
                    0,
                    0,
                    100,
                    100,
                    None,
                    None,
                    None,
                    None,
                )
            }
            .map_err(io::Error::other)?,
        ))
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.0) };
    }
}
fn swap_desc(hwnd: HWND, dx12: bool) -> DXGI_SWAP_CHAIN_DESC {
    DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Width: 100,
            Height: 100,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            RefreshRate: DXGI_RATIONAL {
                Numerator: 60,
                Denominator: 1,
            },
            ..Default::default()
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: if dx12 { 2 } else { 1 },
        OutputWindow: hwnd,
        Windowed: true.into(),
        SwapEffect: if dx12 {
            DXGI_SWAP_EFFECT_FLIP_DISCARD
        } else {
            DXGI_SWAP_EFFECT_DISCARD
        },
        ..Default::default()
    }
}
pub(super) unsafe fn dx9(manager: &HooksManager) -> io::Result<()> {
    if manager.is_hooked(HookKind::Dx9) {
        return Ok(());
    }
    type Create = unsafe extern "system" fn(u32) -> *mut c_void;
    let create = module_proc(w!("d3d9.dll"), s!("Direct3DCreate9"))
        .ok_or_else(|| not_loaded("d3d9.dll is not loaded"))?;
    let window = Window::new()?;
    let d3d = unsafe { std::mem::transmute::<usize, Create>(create)(D3D_SDK_VERSION) };
    if d3d.is_null() {
        return Err(io::Error::other("Direct3DCreate9 failed"));
    }
    let d3d = unsafe { IDirect3D9::from_raw(d3d) };
    let mut params = D3DPRESENT_PARAMETERS {
        SwapEffect: D3DSWAPEFFECT_DISCARD,
        hDeviceWindow: window.0,
        Windowed: true.into(),
        ..Default::default()
    };
    let mut device = None;
    if unsafe {
        d3d.CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            window.0,
            D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32,
            &mut params,
            &mut device,
        )
    }
    .is_err()
    {
        unsafe {
            d3d.CreateDevice(
                D3DADAPTER_DEFAULT,
                D3DDEVTYPE_NULLREF,
                window.0,
                D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32,
                &mut params,
                &mut device,
            )
        }
        .map_err(io::Error::other)?;
    }
    let device = device.ok_or_else(|| io::Error::other("DX9 device unavailable"))?;
    unsafe { manager.hook_dx9_device(device.as_raw()) }
}
pub(super) unsafe fn dx11(manager: &HooksManager) -> io::Result<()> {
    if manager.is_hooked(HookKind::Dx11) {
        return Ok(());
    }
    type Create = unsafe extern "system" fn(
        *mut c_void,
        D3D_DRIVER_TYPE,
        HMODULE,
        D3D11_CREATE_DEVICE_FLAG,
        *const D3D_FEATURE_LEVEL,
        u32,
        u32,
        *const DXGI_SWAP_CHAIN_DESC,
        *mut *mut c_void,
        *mut *mut c_void,
        *mut D3D_FEATURE_LEVEL,
        *mut *mut c_void,
    ) -> HRESULT;
    let create = module_proc(w!("d3d11.dll"), s!("D3D11CreateDeviceAndSwapChain"))
        .ok_or_else(|| not_loaded("d3d11.dll is not loaded"))?;
    let window = Window::new()?;
    let desc = swap_desc(window.0, false);
    let levels = [D3D_FEATURE_LEVEL_11_0];
    let (mut chain, mut device, mut context) = (null_mut(), null_mut(), null_mut());
    let result = unsafe {
        std::mem::transmute::<usize, Create>(create)(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG::default(),
            levels.as_ptr(),
            levels.len() as u32,
            D3D11_SDK_VERSION,
            &desc,
            &mut chain,
            &mut device,
            null_mut(),
            &mut context,
        )
    };
    // Owned, so they are released on every path
    let chain = (!chain.is_null()).then(|| unsafe { IDXGISwapChain::from_raw(chain) });
    let _device = (!device.is_null()).then(|| unsafe { ID3D11Device::from_raw(device) });
    let _context = (!context.is_null()).then(|| unsafe { ID3D11DeviceContext::from_raw(context) });
    result.ok().map_err(io::Error::other)?;
    let chain = chain.ok_or_else(|| io::Error::other("DX11 swap chain unavailable"))?;
    unsafe { manager.hook_dx11_swap_chain(chain.as_raw()) }
}
pub(super) unsafe fn dx12(manager: &HooksManager) -> io::Result<()> {
    if manager.is_hooked(HookKind::Dx12) {
        return Ok(());
    }
    type CreateFactory = unsafe extern "system" fn(*const GUID, *mut *mut c_void) -> HRESULT;
    type CreateDevice = unsafe extern "system" fn(
        *mut c_void,
        D3D_FEATURE_LEVEL,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT;
    let create_device = module_proc(w!("d3d12.dll"), s!("D3D12CreateDevice"))
        .ok_or_else(|| not_loaded("d3d12.dll is not loaded"))?;
    let create_factory = module_proc(w!("dxgi.dll"), s!("CreateDXGIFactory"))
        .ok_or_else(|| not_loaded("dxgi.dll is not loaded"))?;
    let window = Window::new()?;
    let mut factory = null_mut();
    unsafe {
        std::mem::transmute::<usize, CreateFactory>(create_factory)(
            &IDXGIFactory::IID,
            &mut factory,
        )
    }
    .ok()
    .map_err(io::Error::other)?;
    let factory = unsafe { IDXGIFactory::from_raw(factory) };
    let adapter = unsafe { factory.EnumAdapters(0) }.map_err(io::Error::other)?;
    let mut device = null_mut();
    unsafe {
        std::mem::transmute::<usize, CreateDevice>(create_device)(
            adapter.as_raw(),
            D3D_FEATURE_LEVEL_11_0,
            &ID3D12Device::IID,
            &mut device,
        )
    }
    .ok()
    .map_err(io::Error::other)?;
    let device = (!device.is_null()).then(|| unsafe { ID3D12Device::from_raw(device) });
    let device = device.ok_or_else(|| io::Error::other("DX12 device unavailable"))?;
    let desc = D3D12_COMMAND_QUEUE_DESC {
        Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
        ..Default::default()
    };
    let queue: ID3D12CommandQueue =
        unsafe { device.CreateCommandQueue(&desc) }.map_err(io::Error::other)?;
    let desc = swap_desc(window.0, true);
    let mut chain = None;
    unsafe { factory.CreateSwapChain(&queue, &desc, &mut chain) }
        .ok()
        .map_err(io::Error::other)?;
    let chain = chain.ok_or_else(|| io::Error::other("DX12 swap chain unavailable"))?;
    unsafe { manager.hook_dx12_objects(chain.as_raw(), queue.as_raw()) }
}
