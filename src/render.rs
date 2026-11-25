//! Windows.UI.Composition visual tree for the island. Retained-mode: no render loop.
use windows_numerics::Vector2;
use windows::{
    System::DispatcherQueueController,
    UI::Color,
    UI::Composition::{CompositionRoundedRectangleGeometry, CompositionSpriteShape, Compositor, Desktop::DesktopWindowTarget, ShapeVisual},
    Win32::{
        Foundation::HWND,
        System::WinRT::{Composition::ICompositorDesktopInterop, CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_STA, DQTYPE_THREAD_CURRENT},
    },
    core::{Interface, Result},
};

pub struct Renderer {
    _queue: DispatcherQueueController,
    _target: DesktopWindowTarget,
    visual: ShapeVisual,
    shape: CompositionSpriteShape,
    geometry: CompositionRoundedRectangleGeometry,
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
        // ponytail: literal black until the design-system tokens land (Phase 2).
        shape.SetFillBrush(&compositor.CreateColorBrushWithColor(Color { A: 255, R: 0, G: 0, B: 0 })?)?;
        let visual = compositor.CreateShapeVisual()?;
        visual.Shapes()?.Append(&shape)?;
        target.SetRoot(&visual)?;
        Ok(Self { _queue: queue, _target: target, visual, shape, geometry })
    }

    /// All values in physical pixels (DesktopWindowTarget units = px for a DPI-aware process).
    pub fn set_capsule(&self, window_w: f32, window_h: f32, offset: f32, w: f32, h: f32) -> Result<()> {
        self.visual.SetSize(Vector2 { X: window_w, Y: window_h })?;
        self.shape.SetOffset(Vector2 { X: offset, Y: offset })?;
        self.geometry.SetSize(Vector2 { X: w, Y: h })?;
        self.geometry.SetCornerRadius(Vector2 { X: h / 2.0, Y: h / 2.0 })
    }
}
