//! Windows.UI.Composition visual tree for the island (design.md D1, D4, D5).
//! Retained-mode: springs run on the compositor; the app has no frame loop.
use crate::{island::Visual, log, theme};
use windows::{
    Foundation::{IReference, PropertyValue, TimeSpan},
    Graphics::{
        DirectX::{DirectXAlphaMode, DirectXPixelFormat},
        SizeInt32,
    },
    System::DispatcherQueueController,
    UI::Color,
    UI::Composition::{
        CompositionDrawingSurface, CompositionGraphicsDevice, CompositionVisualSurface, ContainerVisual, DropShadow, ShapeVisual, CompositionRoundedRectangleGeometry, CompositionSpriteShape, Compositor,
        Desktop::DesktopWindowTarget, SpriteVisual,
    },
    Win32::{
        Foundation::{HANDLE, HMODULE, HWND, POINT},
        Globalization::{GetDateFormatEx, GetTimeFormatEx, DATE_LONGDATE, TIME_NOSECONDS},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        Graphics::{
            Direct2D::{
                Common::{D2D1_COLOR_F, D2D_RECT_F},
                D2D1CreateDevice, ID2D1Device, ID2D1DeviceContext, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_ROUNDED_RECT,
            },
            Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL},
            Direct3D11::{ID3D11Device, ID3D11Device4, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION},
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_WEIGHT_SEMI_BOLD,
                DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_TRAILING,
                DWRITE_WORD_WRAPPING_NO_WRAP,
            },
            Dxgi::IDXGIDevice,
        },
        System::WinRT::{
            Composition::{ICompositionDrawingSurfaceInterop, ICompositorDesktopInterop, ICompositorInterop},
            CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_STA, DQTYPE_THREAD_CURRENT,
        },
    },
    core::{h, s, w, Interface, Result, HRESULT, HSTRING, PCWSTR},
};
use windows_numerics::{Vector2, Vector3};

/// Target frame of the shape, in visual-window pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
}

/// One content layer; two of them swap on every content change so the old one can fade out
/// while the new one (already drawn) fades in.
struct Content {
    visual: SpriteVisual,
    surface: CompositionDrawingSurface,
}

/// Lazily created text stack (design.md D5): Idle-only use never loads D3D/D2D.
struct Gpu {
    _d3d: ID3D11Device,
    graphics: CompositionGraphicsDevice,
    dwrite: IDWriteFactory,
    layers: [Content; 2],
    front: usize,
}

pub struct Renderer {
    _queue: DispatcherQueueController,
    _target: DesktopWindowTarget,
    compositor: Compositor,
    root: ContainerVisual,
    shadow_visual: SpriteVisual,
    shadow_source: CompositionVisualSurface,
    shape_visual: ShapeVisual,
    shape: CompositionSpriteShape,
    geometry: CompositionRoundedRectangleGeometry,
    gpu: Option<Gpu>,
    frame: Option<Frame>,
    content: Option<(Visual, bool, u32)>,
    /// Signalled by D3D when the device is removed; window waits on it (no polling).
    pub device_removed: Option<HANDLE>,
}

fn color(rgb: u32) -> Color {
    Color { A: 255, R: (rgb >> 16) as u8, G: (rgb >> 8) as u8, B: rgb as u8 }
}

fn color_f(rgb: u32) -> D2D1_COLOR_F {
    let c = color(rgb);
    D2D1_COLOR_F { r: c.R as f32 / 255.0, g: c.G as f32 / 255.0, b: c.B as f32 / 255.0, a: 1.0 }
}

fn ms(v: i64) -> TimeSpan {
    TimeSpan { Duration: v * 10_000 }
}

fn v2(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let options = DispatcherQueueOptions {
            dwSize: size_of::<DispatcherQueueOptions>() as u32,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_STA,
        };
        let queue = unsafe { CreateDispatcherQueueController(options)? };
        let compositor = Compositor::new()?;
        let target = unsafe { compositor.cast::<ICompositorDesktopInterop>()?.CreateDesktopWindowTarget(hwnd, true)? };

        let geometry = compositor.CreateRoundedRectangleGeometry()?;
        let shape = compositor.CreateSpriteShapeWithGeometry(&geometry)?;
        shape.SetFillBrush(&compositor.CreateColorBrushWithColor(color(theme::FILL))?)?;
        shape.SetStrokeBrush(&compositor.CreateColorBrushWithColor(color(theme::STROKE))?)?;
        let shape_visual = compositor.CreateShapeVisual()?;
        shape_visual.Shapes()?.Append(&shape)?;

        // Shadow masked by a live VisualSurface of the shape, so it follows the springs with no redraw.
        // (LayerVisual.Shadow paints the whole DesktopWindowTarget black — tried and discarded.)
        let root = compositor.CreateContainerVisual()?;
        let shadow_visual = compositor.CreateSpriteVisual()?;
        let shadow_source = compositor.CreateVisualSurface()?;
        let shadowed = (|| -> Result<()> {
            shadow_source.SetSourceVisual(&shape_visual)?;
            let shadow = compositor.CreateDropShadow()?;
            shadow.SetColor(color(0))?;
            shadow.SetOpacity(theme::SHADOW_OPACITY)?;
            shadow.SetMask(&compositor.CreateSurfaceBrushWithSurface(&shadow_source)?)?;
            shadow_visual.SetShadow(&shadow)
        })();
        if let Err(e) = shadowed {
            log::warn(&format!("shadow unavailable: {e}"));
        }
        root.Children()?.InsertAtTop(&shadow_visual)?;
        root.Children()?.InsertAtTop(&shape_visual)?;
        target.SetRoot(&root)?;
        Ok(Self {
            _queue: queue,
            _target: target,
            compositor,
            root,
            shadow_visual,
            shadow_source,
            shape_visual,
            shape,
            geometry,
            gpu: None,
            frame: None,
            content: None,
            device_removed: None,
        })
    }

    /// Visual-window size and DPI scale; resets the shadow/stroke metrics that depend on scale.
    pub fn set_window(&mut self, w: f32, h: f32, scale: f32) -> Result<()> {
        // Every child is sized from the root so they cover the whole visual window.
        let size = v2(w, h);
        self.root.SetSize(size)?;
        self.shadow_visual.SetSize(size)?;
        self.shadow_source.SetSourceSize(size)?;
        self.shape_visual.SetSize(size)?;
        if let Ok(shadow) = self.shadow_visual.Shadow().and_then(|s| s.cast::<DropShadow>()) {
            shadow.SetBlurRadius(theme::SHADOW_BLUR_DIP * scale)?;
            shadow.SetOffset(Vector3 { X: 0.0, Y: theme::SHADOW_OFFSET_Y_DIP * scale, Z: 0.0 })?;
        }
        self.shape.SetStrokeThickness((theme::STROKE_DIP * scale).round().max(1.0))?;
        Ok(())
    }

    /// Move the shape to `frame` (spring when `animate`), and swap content if `visual`/`activity`/`dpi` changed.
    pub fn apply(&mut self, frame: Frame, visual: Visual, activity: bool, dpi: u32, animate: bool) -> Result<()> {
        if self.frame != Some(frame) {
            let animate = animate && self.frame.is_some();
            self.frame = Some(frame);
            self.move_shape(frame, animate)?;
        }
        if self.content != Some((visual, activity, dpi)) {
            self.content = Some((visual, activity, dpi));
            self.swap_content(frame, visual, activity, dpi as f32 / 96.0, animate)?;
        }
        Ok(())
    }

    fn move_shape(&self, f: Frame, animate: bool) -> Result<()> {
        let g = &self.geometry;
        if !animate {
            for name in ["Offset", "Size", "CornerRadius"] {
                g.StopAnimation(&HSTRING::from(name))?;
            }
            g.SetOffset(v2(f.x, f.y))?;
            g.SetSize(v2(f.w, f.h))?;
            return g.SetCornerRadius(v2(f.radius, f.radius));
        }
        // One scalar spring per component: windows-rs can box f32 (not Vector2) into the IReference FinalValue needs.
        // Same spring for all, so size, radius and position arrive together.
        let channels = [("Offset.X", f.x), ("Offset.Y", f.y), ("Size.X", f.w), ("Size.Y", f.h), ("CornerRadius.X", f.radius), ("CornerRadius.Y", f.radius)];
        for (name, value) in channels {
            let spring = self.compositor.CreateSpringScalarAnimation()?;
            spring.SetDampingRatio(theme::SPRING_DAMPING)?;
            spring.SetPeriod(ms(theme::SPRING_PERIOD_MS))?;
            spring.SetFinalValue(&PropertyValue::CreateSingle(value)?.cast::<IReference<f32>>()?)?;
            // No InitialValue: NaturalMotion starts from the current animated value, so interruptions don't jump.
            g.StartAnimation(&HSTRING::from(name), &spring)?;
        }
        Ok(())
    }

    fn fade(&self, visual: &SpriteVisual, to: f32, delay_ms: i64, dur_ms: i64, animate: bool) -> Result<()> {
        if !animate {
            visual.StopAnimation(h!("Opacity"))?;
            return visual.SetOpacity(to);
        }
        let a = self.compositor.CreateScalarKeyFrameAnimation()?;
        a.InsertKeyFrame(1.0, to)?;
        a.SetDuration(ms(dur_ms))?;
        a.SetDelayTime(ms(delay_ms))?;
        visual.StartAnimation(h!("Opacity"), &a)
    }

    fn swap_content(&mut self, frame: Frame, visual: Visual, activity: bool, scale: f32, animate: bool) -> Result<()> {
        let has_content = matches!(visual, Visual::Compact | Visual::Expanded);
        if self.gpu.is_none() {
            if !has_content {
                return Ok(()); // Hidden/Idle have no content: keep D3D unloaded.
            }
            self.gpu = Some(self.create_gpu()?);
        }
        let Some(gpu) = self.gpu.as_ref() else { return Ok(()) };
        let old = &gpu.layers[gpu.front];
        let new = &gpu.layers[1 - gpu.front];
        self.fade(&old.visual, 0.0, 0, theme::CONTENT_FADE_OUT_MS, animate)?;
        new.visual.SetOffset(Vector3 { X: frame.x, Y: frame.y, Z: 0.0 })?;
        new.visual.SetSize(v2(frame.w, frame.h))?;
        new.visual.SetOpacity(0.0)?;
        if has_content {
            let size = SizeInt32 { Width: frame.w.ceil() as i32, Height: frame.h.ceil() as i32 };
            draw(gpu, &new.surface, size, visual, activity, scale)?;
            self.fade(&new.visual, 1.0, theme::CONTENT_FADE_IN_DELAY_MS, theme::CONTENT_FADE_IN_MS, animate)?;
        }
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.front = 1 - gpu.front;
        }
        Ok(())
    }

    fn create_gpu(&mut self) -> Result<Gpu> {
        let (d3d, d2d) = create_device()?;
        let graphics = unsafe { self.compositor.cast::<ICompositorInterop>()?.CreateGraphicsDevice(&d2d)? };
        let dwrite: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let layer = |_| -> Result<Content> {
            let surface = graphics.CreateDrawingSurface2(SizeInt32 { Width: 1, Height: 1 }, DirectXPixelFormat::B8G8R8A8UIntNormalized, DirectXAlphaMode::Premultiplied)?;
            let brush = self.compositor.CreateSurfaceBrushWithSurface(&surface)?;
            let visual = self.compositor.CreateSpriteVisual()?;
            visual.SetBrush(&brush)?;
            visual.SetOpacity(0.0)?;
            self.root.Children()?.InsertAtTop(&visual)?;
            Ok(Content { visual, surface })
        };
        let layers = [layer(0)?, layer(1)?];
        self.device_removed = watch_device(&d3d);
        log::info("gpu text device created");
        Ok(Gpu { _d3d: d3d, graphics, dwrite, layers, front: 0 })
    }

    /// Device removed (driver reset, GPU switch): new D3D/D2D on the same CompositionGraphicsDevice, then redraw.
    pub fn recover_device(&mut self) -> Result<()> {
        let Some(gpu) = self.gpu.as_mut() else { return Ok(()) };
        let (d3d, d2d) = create_device()?;
        unsafe { gpu.graphics.cast::<windows::Win32::System::WinRT::Composition::ICompositionGraphicsDeviceInterop>()?.SetRenderingDevice(&d2d)? };
        self.device_removed = watch_device(&d3d);
        gpu._d3d = d3d;
        self.content = None; // force redraw on next apply
        log::warn("gpu device recovered");
        Ok(())
    }
}

/// `D3D11CreateDevice` resolved at runtime: a static import loads dxgi.dll at startup, which reserves
/// ~54 MB of private commit as soon as the thread creates a window — even if no device is ever made.
/// ponytail: once a device exists the reservation stays for the process lifetime (unloading doesn't return it);
/// it is untouched commit, not RAM — the private working set stays small.
type D3D11CreateDeviceFn = unsafe extern "system" fn(
    adapter: *mut core::ffi::c_void,
    driver: D3D_DRIVER_TYPE,
    software: HMODULE,
    flags: D3D11_CREATE_DEVICE_FLAG,
    levels: *const D3D_FEATURE_LEVEL,
    level_count: u32,
    sdk: u32,
    device: *mut Option<ID3D11Device>,
    level: *mut D3D_FEATURE_LEVEL,
    context: *mut *mut core::ffi::c_void,
) -> HRESULT;

fn create_device() -> Result<(ID3D11Device, ID2D1Device)> {
    let create: D3D11CreateDeviceFn = unsafe {
        let module = LoadLibraryW(w!("d3d11.dll"))?;
        let proc = GetProcAddress(module, s!("D3D11CreateDevice")).ok_or_else(windows::core::Error::from_thread)?;
        std::mem::transmute::<unsafe extern "system" fn() -> isize, D3D11CreateDeviceFn>(proc)
    };
    let make = |driver: D3D_DRIVER_TYPE| -> Result<ID3D11Device> {
        let mut device = None;
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
        let null = std::ptr::null_mut();
        unsafe { create(null, driver, HMODULE::default(), flags, std::ptr::null(), 0, D3D11_SDK_VERSION, &mut device, null.cast(), null.cast()).ok()? };
        device.ok_or_else(|| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))
    };
    let d3d = make(D3D_DRIVER_TYPE_HARDWARE).or_else(|_| make(D3D_DRIVER_TYPE_WARP))?;
    let d2d = unsafe { D2D1CreateDevice(&d3d.cast::<IDXGIDevice>()?, None)? };
    Ok((d3d, d2d))
}

fn watch_device(d3d: &ID3D11Device) -> Option<HANDLE> {
    let device4 = d3d.cast::<ID3D11Device4>().ok()?;
    let event = unsafe { windows::Win32::System::Threading::CreateEventW(None, false, false, None) }.ok()?;
    unsafe { device4.RegisterDeviceRemovedEvent(event) }.ok()?;
    Some(event)
}

/// Current time (no seconds) and long date in the user's locale, e.g. "20:45" / "quinta-feira, 26 de novembro de 2026".
fn now_strings() -> (String, String) {
    let fmt = |f: &dyn Fn(&mut [u16]) -> i32| {
        let mut buf = [0u16; 128];
        let n = f(&mut buf).max(1) as usize;
        String::from_utf16_lossy(&buf[..n - 1])
    };
    let time = fmt(&|b| unsafe { GetTimeFormatEx(PCWSTR::null(), TIME_NOSECONDS, None, PCWSTR::null(), Some(b)) });
    let date = fmt(&|b| unsafe { GetDateFormatEx(PCWSTR::null(), DATE_LONGDATE, None, PCWSTR::null(), Some(b), PCWSTR::null()) });
    (time, date)
}

fn draw(gpu: &Gpu, surface: &CompositionDrawingSurface, size: SizeInt32, visual: Visual, activity: bool, scale: f32) -> Result<()> {
    surface.Resize(size)?;
    let interop = surface.cast::<ICompositionDrawingSurfaceInterop>()?;
    let mut offset = POINT::default();
    let dc: ID2D1DeviceContext = unsafe { interop.BeginDraw(None, &mut offset)? };
    let result = (|| -> Result<()> {
        let (ox, oy) = (offset.x as f32, offset.y as f32);
        let (w, h) = (size.Width as f32, size.Height as f32);
        let px = |dip: f32| dip * scale;
        let format = |size_dip: f32, weight: DWRITE_FONT_WEIGHT| -> Result<IDWriteTextFormat> {
            let family: Vec<u16> = theme::FONT_FAMILY.encode_utf16().chain([0]).collect();
            let f = unsafe {
                gpu.dwrite.CreateTextFormat(PCWSTR(family.as_ptr()), None, weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, px(size_dip), windows::core::w!("pt-BR"))?
            };
            unsafe {
                f.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
                f.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
            }
            Ok(f)
        };
        let text = |s: &str, f: &IDWriteTextFormat, rect: D2D_RECT_F, rgb: u32| -> Result<()> {
            let brush = unsafe { dc.CreateSolidColorBrush(&color_f(rgb), None)? };
            let s: Vec<u16> = s.encode_utf16().collect();
            unsafe { dc.DrawText(&s, f, &rect, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL) };
            Ok(())
        };
        unsafe {
            dc.Clear(Some(&D2D1_COLOR_F::default()));
        }
        // ponytail: fixed placeholder content (debug Demo only); real activities bring their own payloads.
        match visual {
            Visual::Hidden | Visual::Idle => {}
            Visual::Expanded if !activity => {
                // ponytail: drawn once when expanding, not a live clock (content phase brings real widgets).
                let inset = px(theme::EXPANDED_INSET_DIP);
                let line = |top: f32, height: f32| D2D_RECT_F { left: ox + inset, top: oy + px(top), right: ox + w - inset, bottom: oy + px(top + height) };
                let clock = format(theme::FONT_CLOCK_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                let date = format(theme::FONT_SUBTITLE_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                let (time_s, date_s) = now_strings();
                text(&time_s, &clock, line(30.0, 56.0), theme::TEXT_PRIMARY)?;
                text(&date_s, &date, line(90.0, 20.0), theme::TEXT_SECONDARY)?;
            }
            Visual::Compact => {
                let inset = px(theme::CONTENT_INSET_DIP);
                let r = px(4.0);
                let accent = unsafe { dc.CreateSolidColorBrush(&color_f(theme::ACCENT), None)? };
                let dot = D2D1_ELLIPSE { point: v2(ox + inset + r, oy + h / 2.0), radiusX: r, radiusY: r };
                unsafe { dc.FillEllipse(&dot, &accent) };
                let f = format(theme::FONT_COMPACT_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                unsafe { f.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING)? };
                text("0:08", &f, D2D_RECT_F { left: ox + inset, top: oy, right: ox + w - inset, bottom: oy + h }, theme::ACCENT)?;
            }
            Visual::Expanded => {
                let inset = px(theme::EXPANDED_INSET_DIP);
                let title = format(theme::FONT_TITLE_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                let sub = format(theme::FONT_SUBTITLE_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                let line = |top: f32, height: f32| D2D_RECT_F { left: ox + inset, top: oy + px(top), right: ox + w - inset, bottom: oy + px(top + height) };
                text("Atividade de exemplo", &title, line(26.0, 22.0), theme::TEXT_PRIMARY)?;
                text("Demonstração da Fase 2", &sub, line(48.0, 18.0), theme::TEXT_SECONDARY)?;
                // Placeholder "controls" row: a progress track.
                let track = unsafe { dc.CreateSolidColorBrush(&color_f(0x3A3A3C), None)? };
                let fill = unsafe { dc.CreateSolidColorBrush(&color_f(theme::TEXT_PRIMARY), None)? };
                let bar = |right: f32| D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F { left: ox + inset, top: oy + px(110.0), right, bottom: oy + px(116.0) },
                    radiusX: px(3.0),
                    radiusY: px(3.0),
                };
                unsafe {
                    dc.FillRoundedRectangle(&bar(ox + w - inset), &track);
                    dc.FillRoundedRectangle(&bar(ox + inset + (w - 2.0 * inset) * 0.6), &fill);
                }
            }
        }
        Ok(())
    })();
    unsafe { interop.EndDraw()? };
    result
}
