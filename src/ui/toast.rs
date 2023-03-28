use std::fmt;
use std::time::{Duration, Instant};

use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, text};
use iced::{Alignment, Element, Length, Point, Rectangle, Size, Vector};
use iced_native::widget::{tree, Operation, Tree};
use iced_native::{event, layout, mouse, overlay, renderer, window};
use iced_native::{Clipboard, Event, Layout, Shell, Widget};

use crate::ui::appearance::ContainerStyle;

pub const DEFAULT_TIMEOUT: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Primary,
    Secondary,
    Success,
    Danger,
}

impl Status {
    pub const ALL: &[Self] = &[Self::Primary, Self::Secondary, Self::Success, Self::Danger];
}

// impl container::StyleSheet for Status {
//     type Style = Theme;
//
//     fn appearance(&self, theme: &Theme) -> container::Appearance {
//         let palette = theme.extended_palette();
//
//         let pair = match self {
//             Status::Primary => palette.primary.weak,
//             Status::Secondary => palette.secondary.weak,
//             Status::Success => palette.success.weak,
//             Status::Danger => palette.danger.weak,
//         };
//
//         container::Appearance {
//             background: pair.color.into(),
//             text_color: pair.text.into(),
//             ..Default::default()
//         }
//     }
// }

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Primary => "Primary",
            Status::Secondary => "Secondary",
            Status::Success => "Success",
            Status::Danger => "Danger",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Toast {
    pub title: String,
    pub body: String,
    pub status: Status,
}

pub trait StyleSheet {
    type Style;
}

pub struct Manager<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    toasts: Vec<Element<'a, Message, Renderer>>,
    timeout_secs: u64,
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message, Renderer> Manager<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet
        + iced_style::container::StyleSheet
        + iced_style::text::StyleSheet
        + iced_style::button::StyleSheet
        + iced_style::rule::StyleSheet,
    <<Renderer as iced_native::Renderer>::Theme as iced_style::container::StyleSheet>::Style:
        From<ContainerStyle>,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                container(column![
                    container(
                        row![
                            text(toast.title.as_str()),
                            horizontal_space(Length::Fill),
                            button("X").on_press((on_close)(index)).padding(3),
                        ]
                        .align_items(Alignment::Center)
                    )
                    .width(Length::Fill)
                    .padding(5),
                    horizontal_rule(1),
                    container(text(toast.body.as_str()))
                        .width(Length::Fill)
                        .padding(5),
                ])
                .style(ContainerStyle::Toast)
                .padding(5)
                .max_width(200)
                .into()
            })
            .collect();

        Self {
            content: content.into(),
            toasts,
            timeout_secs: DEFAULT_TIMEOUT,
            on_close: Box::new(on_close),
        }
    }

    pub fn timeout(self, seconds: u64) -> Self {
        Self {
            timeout_secs: seconds,
            ..self
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Manager<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(renderer, limits)
    }

    fn tag(&self) -> tree::Tag {
        struct Marker(Vec<Instant>);
        iced_native::widget::tree::Tag::of::<Marker>()
    }

    fn state(&self) -> tree::State {
        iced_native::widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.toasts.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

        // Invalidating removed instants to None allows us to remove
        // them here so that diffing for removed / new toast instants
        // is accurate
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            (old, new) if old > new => {
                instants.truncate(new);
            }
            (old, new) if old < new => {
                instants.extend(std::iter::repeat(Some(Instant::now())).take(new - old));
            }
            _ => {}
        }

        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toasts.iter())
                .collect::<Vec<_>>(),
        );
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.content
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        });
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut state.children[0],
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

        let (content_state, toasts_state) = state.children.split_at_mut(1);

        let content = self
            .content
            .as_widget_mut()
            .overlay(&mut content_state[0], layout, renderer);

        let toasts = (!self.toasts.is_empty()).then(|| {
            overlay::Element::new(
                layout.bounds().position(),
                Box::new(Overlay {
                    toasts: &mut self.toasts,
                    state: toasts_state,
                    instants,
                    on_close: &self.on_close,
                    timeout_secs: self.timeout_secs,
                }),
            )
        });
        let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

struct Overlay<'a, 'b, Message, Renderer> {
    toasts: &'b mut [Element<'a, Message, Renderer>],
    state: &'b mut [Tree],
    instants: &'b mut [Option<Instant>],
    on_close: &'b dyn Fn(usize) -> Message,
    timeout_secs: u64,
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn layout(&self, renderer: &Renderer, bounds: Size, position: Point) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds)
            .width(Length::Fill)
            .height(Length::Fill);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            10.into(),
            10.0,
            Alignment::End,
            self.toasts,
        )
        .translate(Vector::new(position.x, position.y))
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            let mut next_redraw: Option<window::RedrawRequest> = None;

            self.instants
                .iter_mut()
                .enumerate()
                .for_each(|(index, maybe_instant)| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        let remaining = Duration::from_secs(self.timeout_secs)
                            .saturating_sub(instant.elapsed());

                        if remaining == Duration::ZERO {
                            maybe_instant.take();
                            shell.publish((self.on_close)(index));
                            next_redraw = Some(window::RedrawRequest::NextFrame);
                        } else {
                            let redraw_at = window::RedrawRequest::At(*now + remaining);
                            next_redraw = next_redraw
                                .map(|redraw| redraw.min(redraw_at))
                                .or(Some(redraw_at));
                        }
                    }
                });

            if let Some(redraw) = next_redraw {
                shell.request_redraw(redraw);
            }
        }

        self.toasts
            .iter_mut()
            .zip(self.state.iter_mut())
            .zip(layout.children())
            .zip(self.instants.iter_mut())
            .map(|(((child, state), layout), instant)| {
                let mut local_messages = vec![];
                let mut local_shell = Shell::new(&mut local_messages);

                let status = child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    &mut local_shell,
                );

                if !local_shell.is_empty() {
                    instant.take();
                }

                shell.merge(local_shell, std::convert::identity);

                status
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let viewport = layout.bounds();

        for ((child, state), layout) in self
            .toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
        {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                &viewport,
            );
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_native::widget::Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.toasts
                .iter()
                .zip(self.state.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                })
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn is_over(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        layout
            .children()
            .any(|layout| layout.bounds().contains(cursor_position))
    }
}

impl<'a, Message, Renderer> From<Manager<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(manager: Manager<'a, Message, Renderer>) -> Self {
        Element::new(manager)
    }
}
