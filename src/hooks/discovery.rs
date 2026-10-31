use super::*;
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
    core::{Interface, w},
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
    let window = Window::new()?;
    let d3d = unsafe { Direct3DCreate9(D3D_SDK_VERSION) }
        .ok_or_else(|| io::Error::other("Direct3DCreate9 failed"))?;
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
    let window = Window::new()?;
    let desc = swap_desc(window.0, false);
    let mut chain = None;
    let mut device = None;
    let mut context = None;
    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG::default(),
            None,
            D3D11_SDK_VERSION,
            Some(&desc),
            Some(&mut chain),
            Some(&mut device),
            None,
            Some(&mut context),
        )
    }
    .map_err(io::Error::other)?;
    let chain = chain.ok_or_else(|| io::Error::other("DX11 swap chain unavailable"))?;
    unsafe { manager.hook_dx11_swap_chain(chain.as_raw()) }
}
pub(super) unsafe fn dx12(manager: &HooksManager) -> io::Result<()> {
    if manager.is_hooked(HookKind::Dx12) {
        return Ok(());
    }
    let window = Window::new()?;
    let factory: IDXGIFactory = unsafe { CreateDXGIFactory() }.map_err(io::Error::other)?;
    let adapter = unsafe { factory.EnumAdapters(0) }.map_err(io::Error::other)?;
    let mut device: Option<ID3D12Device> = None;
    unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }
        .map_err(io::Error::other)?;
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
