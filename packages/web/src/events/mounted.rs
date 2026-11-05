use aether_html::{
    MountedData,
    geometry::euclid::{Point2D, Size2D},
};
use wasm_bindgen::JsCast;

use super::{Synthetic, WebEventExt};

impl aether_html::RenderedElementBacking for Synthetic<web_sys::Element> {
    fn get_scroll_offset(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = aether_html::MountedResult<aether_html::geometry::PixelsVector2D>,
                >,
        >,
    > {
        let left = self.event.scroll_left();
        let top = self.event.scroll_top();
        let result = Ok(aether_html::geometry::PixelsVector2D::new(
            left as f64,
            top as f64,
        ));
        Box::pin(async { result })
    }

    fn get_scroll_size(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = aether_html::MountedResult<aether_html::geometry::PixelsSize>,
                >,
        >,
    > {
        let width = self.event.scroll_width();
        let height = self.event.scroll_height();
        let result = Ok(aether_html::geometry::PixelsSize::new(
            width as f64,
            height as f64,
        ));
        Box::pin(async { result })
    }

    fn get_client_rect(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = aether_html::MountedResult<aether_html::geometry::PixelsRect>,
                >,
        >,
    > {
        let rect = self.event.get_bounding_client_rect();
        let result = Ok(aether_html::geometry::PixelsRect::new(
            Point2D::new(rect.left(), rect.top()),
            Size2D::new(rect.width(), rect.height()),
        ));
        Box::pin(async { result })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        &self.event
    }

    fn scroll_to(
        &self,
        input_options: aether_html::ScrollToOptions,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = aether_html::MountedResult<()>>>> {
        let options = web_sys::ScrollIntoViewOptions::new();
        options.set_behavior(match input_options.behavior {
            aether_html::ScrollBehavior::Instant => web_sys::ScrollBehavior::Instant,
            aether_html::ScrollBehavior::Smooth => web_sys::ScrollBehavior::Smooth,
        });
        options.set_block(match input_options.vertical {
            aether_html::ScrollLogicalPosition::Start => web_sys::ScrollLogicalPosition::Start,
            aether_html::ScrollLogicalPosition::Center => web_sys::ScrollLogicalPosition::Center,
            aether_html::ScrollLogicalPosition::End => web_sys::ScrollLogicalPosition::End,
            aether_html::ScrollLogicalPosition::Nearest => web_sys::ScrollLogicalPosition::Nearest,
        });
        options.set_inline(match input_options.horizontal {
            aether_html::ScrollLogicalPosition::Start => web_sys::ScrollLogicalPosition::Start,
            aether_html::ScrollLogicalPosition::Center => web_sys::ScrollLogicalPosition::Center,
            aether_html::ScrollLogicalPosition::End => web_sys::ScrollLogicalPosition::End,
            aether_html::ScrollLogicalPosition::Nearest => web_sys::ScrollLogicalPosition::Nearest,
        });
        self.event
            .scroll_into_view_with_scroll_into_view_options(&options);

        Box::pin(async { Ok(()) })
    }

    fn scroll(
        &self,
        coordinates: aether_html::geometry::PixelsVector2D,
        behavior: aether_html::ScrollBehavior,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = aether_html::MountedResult<()>>>> {
        let options = web_sys::ScrollToOptions::new();
        options.set_top(coordinates.y);
        options.set_left(coordinates.x);
        match behavior {
            aether_html::ScrollBehavior::Instant => {
                options.set_behavior(web_sys::ScrollBehavior::Instant);
            }
            aether_html::ScrollBehavior::Smooth => {
                options.set_behavior(web_sys::ScrollBehavior::Smooth);
            }
        }
        self.event.scroll_with_scroll_to_options(&options);

        Box::pin(async { Ok(()) })
    }

    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = aether_html::MountedResult<()>>>> {
        #[derive(Debug)]
        struct FocusError(wasm_bindgen::JsValue);

        impl std::fmt::Display for FocusError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "failed to focus element {:?}", self.0)
            }
        }

        impl std::error::Error for FocusError {}

        let result = self
            .event
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or_else(|| {
                aether_html::MountedError::OperationFailed(Box::new(FocusError(
                    self.event.clone().into(),
                )))
            })
            .and_then(|e| {
                (if focus { e.focus() } else { e.blur() }).map_err(|err| {
                    aether_html::MountedError::OperationFailed(Box::new(FocusError(err)))
                })
            });
        Box::pin(async { result })
    }
}

impl WebEventExt for MountedData {
    type WebEvent = web_sys::Element;

    #[inline(always)]
    fn try_as_web_event(&self) -> Option<web_sys::Element> {
        self.downcast::<web_sys::Element>().cloned()
    }
}
