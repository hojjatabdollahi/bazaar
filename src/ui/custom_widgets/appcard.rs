use iced::{
    alignment, event, mouse, overlay, touch, Alignment, Background, Color, Element, Event, Length,
    Padding, Point, Rectangle,
};

use iced_native::{
    layout, renderer,
    widget::{self, Operation, Tree},
    Clipboard, Layout, Pixels, Shell, Widget,
};

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The text [`Color`] of the container.
    pub text_color: Option<Color>,
    /// The [`Background`] of the container.
    pub background: Option<Background>,
    /// The border radius of the container.
    pub border_radius: f32,
    /// The border width of the container.
    pub border_width: f32,
    /// The border [`Color`] of the container.
    pub border_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of a container.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a container.
    fn appearance(&self, style: &Self::Style) -> Appearance;
    fn hovered(&self, sytle: &Self::Style) -> Appearance;
}
pub struct AppCard<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    style: <Renderer::Theme as StyleSheet>::Style,
    content: Element<'a, Message, Renderer>,
    on_click: Message,
}

impl<'a, Message, Renderer> AppCard<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`Container`].
    pub fn new<T, F, V>(content: T, value: V, f: F) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
        V: Eq,
        F: FnOnce(V) -> Message,
    {
        AppCard {
            id: None,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            style: Default::default(),
            content: content.into(),
            on_click: f(value),
        }
    }

    /// Sets the [`Id`] of the [`Container`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height of the [`Container`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = alignment::Horizontal::Center;
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = alignment::Vertical::Center;
        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for AppCard<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |renderer, limits| self.content.as_widget().layout(renderer, limits),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(self.id.as_ref().map(|id| &id.0), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) == event::Status::Ignored
        {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if layout.bounds().contains(cursor_position) {
                        shell.publish(self.on_click.clone());
                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        }
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);
        let style = if is_mouse_over {
            theme.hovered(&self.style)
        } else {
            theme.appearance(&self.style)
        };
        draw_background(renderer, &style, layout.bounds());

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color.unwrap_or(renderer_style.text_color),
            },
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<AppCard<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(column: AppCard<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}

/// Computes the layout of a [`Container`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits
        .loose()
        .max_width(max_width)
        .max_height(max_height)
        .width(width)
        .height(height);

    let mut content = layout_content(renderer, &limits.pad(padding).loose());
    let padding = padding.fit(content.size(), limits.max());
    let size = limits.pad(padding).resolve(content.size());

    content.move_to(Point::new(padding.left, padding.top));
    content.align(
        Alignment::from(horizontal_alignment),
        Alignment::from(vertical_alignment),
        size,
    );

    layout::Node::with_children(size.pad(padding), vec![content])
}

/// Draws the background of a [`Container`] given its [`Appearance`] and its `bounds`.
pub fn draw_background<Renderer>(
    renderer: &mut Renderer,
    appearance: &Appearance,
    bounds: Rectangle,
) where
    Renderer: iced_native::Renderer,
{
    if appearance.background.is_some() || appearance.border_width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: [appearance.border_radius, 0.0, appearance.border_radius, 0.0]
                    .into(),
                border_width: appearance.border_width,
                border_color: appearance.border_color,
            },
            appearance
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}

/// The identifier of a [`Container`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}
