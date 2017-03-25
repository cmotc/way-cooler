use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use rustwlc::{Geometry, Size, WlcOutput};
use rustwlc::render::{calculate_stride};
use cairo::{ImageSurface, Format};

use uuid::Uuid;
use ::registry;
use ::render::{Color, Renderable, drop_data};

/// A notification to the user that an event has happened.
pub struct Notification {
    /// The surface that contains the bytes we give wlc to draw.
    surface: ImageSurface,
    /// The output that the notification is displayed on
    output: WlcOutput,
    /// The string that's displayed as the "title"
    /// or "header" of the notificaiton.
    title: String,
    /// The string that's displayed as the body text of the notification.
    text: String,
    /// The geometry of where the buffer is drawn.
    ///
    /// The width and height should be the same size as the surface's
    /// width * height * 4. If it's not, then the buffer will fail to draw.
    geometry: Geometry,
    /// The background color that the text is overlayed on to.
    ///
    /// If unspecified, the default from the registry is used.
    background_color: Option<Color>,
    /// The color that the header/title text is displayed in.
    ///
    /// If unspecified, the default from the registry is used.
    title_color: Option<Color>,
    /// The color that the body text is displayed in.
    ///
    /// If unspecified, the default from the registry is used.
    text_color: Option<Color>
}

/// Builder for a Notification. This ensures that when it's created
/// it can construct an appropriately sized buffer for the text.
///
/// Also allows you to pre-set the colors.
pub struct NotificationBuilder {
    /// The string that's displayed as the "title"
    /// or "header" of the notificaiton.
    title: String,
    /// The string that's displayed as the body text of the notification.
    text: String,
    /// The background color that the text is overlayed on to.
    ///
    /// If unspecified, the default from the registry is used.
    background_color: Option<Color>,
    /// The color that the header/title text is displayed in.
    ///
    /// If unspecified, the default from the registry is used.
    title_color: Option<Color>,
    /// The color that the body text is displayed in.
    ///
    /// If unspecified, the default from the registry is used.
    text_color: Option<Color>
}

/// Calculates how big the notification should be, based on the given title
/// and text strings.
///
/// Notifications are limited to 1/4 the width and 1/4 the height of the screen.
///
/// If the title text is too long, it is truncated and has ellipses appended.
///
/// If the body text is too long, then it is split into multiple lines,
/// with the final line being truncated with ellipses if it would push the
/// height to be bigger than 1/4 the height of the screen.
///
/// The passed in strings are modified to have these changes reflected.
///
/// Each element in the text `Vec` corresponds to a line of input,
/// if the passed in `Vec` has a length > 1, then it collapses them down
/// into one string and then processes it, as if each line had been part
/// of the first line. This is in order to simplify the algorithm.
fn calc_notification_geo(title: &mut String, text: &mut Vec<String>) -> Geometry {
    panic!()
}

impl NotificationBuilder {
    pub fn new() -> Self {
        NotificationBuilder {
            title: "".into(),
            text: "".into(),
            background_color: None,
            title_color: None,
            text_color: None
        }
    }

    /// Construct the notification for the given output.
    pub fn build(mut self, output: WlcOutput) -> Option<Notification> {
        let mut text = vec![self.text];
        let geo = calc_notification_geo(&mut self.title, &mut text);
        Notification::new(geo, output)
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub fn background_color(mut self, color: Option<Color>) -> Self {
        self.background_color = color;
        self
    }

    pub fn title_color(mut self, color: Option<Color>) -> Self {
        self.title_color = color;
        self
    }

    pub fn text_color(mut self, color: Option<Color>) -> Self {
        self.text_color = color;
        self
    }
}


impl Renderable for Notification {
    fn new(mut geometry: Geometry, output: WlcOutput) -> Option<Self> {
        Notification::allocate_buffer(geometry, drop_data)
            .and_then(|surface| {
                Some(Notification {
                    surface: surface,
                    output: output,
                    title: "".into(),
                    text: "".into(),
                    geometry: geometry,
                    background_color: None,
                    title_color: None,
                    text_color: None
                })})
    }

    fn allocate_buffer<F>(geometry: Geometry, drop_f: F) -> Option<ImageSurface>
        where F: FnOnce(Box<[u8]>) + 'static {
        let Size { w, h } = geometry.size;
        if w == 0 && h == 0 {
            return None
        }
        let stride = calculate_stride(w);
        let size = (stride * w * h) as usize;
        let data: Vec<u8> = iter::repeat(0).take(h as usize * stride as usize)
            .collect();
        let buffer = data.into_boxed_slice();
        Some(ImageSurface::create_for_data(buffer,
                                           drop_f,
                                           Format::ARgb32,
                                           w as i32,
                                           h as i32,
                                           stride as i32))
    }

    fn set_surface(&mut self, surface: ImageSurface) {
        self.surface = surface;
    }

    fn get_surface(&mut self) -> &mut ImageSurface {
        &mut self.surface
    }

    fn get_geometry(&self) -> Geometry {
        self.geometry
    }

    fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    fn get_output(&self) -> WlcOutput {
        self.output
    }
}

impl Notification {
    /// Fetches the default background color for a notification from the
    /// registry.
    ///
    /// If the vaule is unset, an all black color is returned.
    pub fn default_background_color() -> Color {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("notifications".into()).ok()
            .and_then(|notifications| notifications.get("default"))
            .and_then(|default| default.as_object()
                      .and_then(|default| default.get("background_color"))
                      .and_then(|color| color.as_f64()))
            .map(|num| num as u32)
            .unwrap_or(0u32).into()
    }

    /// Fetches the default title color for a notification from the registry.
    ///
    /// If the vaule is unset, an all white color is returned.
    pub fn default_title_color() -> Color {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("notifications".into()).ok()
            .and_then(|notifications| notifications.get("default"))
            .and_then(|default| default.as_object()
                      .and_then(|default| default.get("title_color"))
                      .and_then(|color| color.as_f64()))
            .map(|num| num as u32)
            .unwrap_or(0xffffff).into()
    }

    /// Fetches the default body text color for a notification from the
    /// registry.
    ///
    /// If the vaule is unset, an all white color is returned.
    pub fn default_text_color() -> Color {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("notifications".into()).ok()
            .and_then(|notifications| notifications.get("default"))
            .and_then(|default| default.as_object()
                      .and_then(|default| default.get("text_color"))
                      .and_then(|color| color.as_f64()))
            .map(|num| num as u32)
            .unwrap_or(0xffffff).into()
    }

    pub fn background_color(&self) -> Color {
        self.background_color
            .unwrap_or_else(Notification::default_background_color)
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color
    }

    pub fn title_color(&self) -> Color {
        self.title_color
            .unwrap_or_else(Notification::default_title_color)
    }

    pub fn set_title_color(&mut self, color: Option<Color>) {
        self.title_color = color
    }

    pub fn text_color(&self) -> Color {
        self.text_color
            .unwrap_or_else(Notification::default_text_color)
    }

    pub fn set_text_color(&mut self, color: Option<Color>) {
        self.text_color = color
    }
}

impl Debug for Notification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Notification")
            .field("geometry", &self.geometry as &Debug)
            .finish()
    }
}

impl PartialEq for Notification {
    fn eq(&self, other: &Notification) -> bool {
        self.geometry == other.geometry
    }
}

impl Eq for Notification {}

unsafe impl Send for Notification {}
unsafe impl Sync for Notification {}