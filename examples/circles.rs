extern crate limn;
extern crate glutin;
extern crate graphics;
extern crate petgraph;
extern crate cassowary;

mod util;

use cassowary::strength::*;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::PushButtonBuilder;
use limn::widgets::primitives;
use limn::widget::property::Property;
use limn::event::EventAddress;
use limn::ui::{self, WidgetGraph};
use limn::util::{Dimensions, Point};
use limn::resources::WidgetId;
use limn::color::*;

enum CircleEvent {
    Add(Point),
    Undo,
    Redo,
}

fn main() {
    let (window, graph, mut event_queue) = util::init_default("Limn circles demo");
    util::load_default_font();

    fn create_undo_redo_buttons(root_widget: &mut WidgetBuilder) -> (WidgetId, WidgetId) {
        let mut button_container = WidgetBuilder::new();
        button_container.layout.center_horizontal(&root_widget);
        button_container.layout.align_bottom(&root_widget, Some(20.0));
        button_container.layout.shrink();

        let undo_widget = PushButtonBuilder::new()
            .set_text("Undo")
            .widget
            .on_click(|_, args| { args.event_queue.push(EventAddress::Ui, CircleEvent::Undo); });
        let mut redo_widget = PushButtonBuilder::new()
            .set_text("Redo")
            .widget
            .on_click(|_, args| { args.event_queue.push(EventAddress::Ui, CircleEvent::Redo); });
        redo_widget.layout.to_right_of(&undo_widget, Some(20.0));

        let (undo_id, redo_id) = (undo_widget.id, redo_widget.id);
        button_container.add_child(undo_widget);
        button_container.add_child(redo_widget);

        root_widget.add_child(button_container);
        (undo_id, redo_id)
    }

    fn create_circle(graph: &mut WidgetGraph, center: &Point) -> WidgetId {
        let border = graphics::ellipse::Border {
            color: BLACK,
            radius: 2.0,
        };
        let mut widget = WidgetBuilder::new()
            .set_drawable(primitives::ellipse_drawable(RED, Some(border)));
        widget.layout.dimensions(Dimensions {
            width: 30.0,
            height: 30.0,
        });
        let top_left = Point {
            x: center.x - 15.0,
            y: center.y - 15.0,
        };

        widget.layout.top_left(top_left, Some(STRONG));
        let id = widget.id;
        let root_index = graph.root_index.unwrap();
        graph.add_widget(widget, Some(root_index));
        id
    }

    struct CircleEventHandler {
        undo_id: WidgetId,
        redo_id: WidgetId,
        circles: Vec<(Point, WidgetId)>,
        undo: Vec<Point>,
    }
    impl CircleEventHandler {
        fn new(undo_id: WidgetId, redo_id: WidgetId) -> Self {
            CircleEventHandler {
                circles: Vec::new(),
                undo: Vec::new(),
                undo_id: undo_id,
                redo_id: redo_id,
            }
        }
    }
    impl ui::EventHandler<CircleEvent> for CircleEventHandler {
        fn handle(&mut self, event: &CircleEvent, args: ui::EventArgs) {
            match *event {
                CircleEvent::Add(point) => {
                    self.circles.push((point, create_circle(args.graph, &point)));
                    self.undo.clear();

                    args.event_queue.change_prop(self.undo_id, Property::Inactive, false);
                    args.event_queue.change_prop(self.redo_id, Property::Inactive, true);
                }
                CircleEvent::Undo => {
                    if self.circles.len() > 0 {
                        let (point, node_index) = self.circles.pop().unwrap();
                        args.graph.remove_widget(node_index);
                        self.undo.push(point);
                        args.event_queue.change_prop(self.redo_id, Property::Inactive, false);
                        if self.circles.len() == 0 {
                            args.event_queue.change_prop(self.undo_id, Property::Inactive, true);
                        }
                    }
                }
                CircleEvent::Redo => {
                    if self.undo.len() > 0 {
                        let point = self.undo.pop().unwrap();
                        self.circles.push((point, create_circle(args.graph, &point)));
                        if self.undo.len() == 0 {
                            args.event_queue.change_prop(self.redo_id, Property::Inactive, true);
                        }
                    }
                }
            }
        }
    }
    let mut root_widget = WidgetBuilder::new().on_click(|_, args| {
        let event = CircleEvent::Add(args.input_state.mouse);
        args.event_queue.push(EventAddress::Ui, event);
    });
    root_widget.layout.dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });


    let (undo_id, redo_id) = create_undo_redo_buttons(&mut root_widget);
    // todo: better way to set initial props
    event_queue.change_prop(undo_id, Property::Inactive, true);
    event_queue.change_prop(redo_id, Property::Inactive, true);

    let ui_event_handlers: Vec<ui::HandlerWrapper> =
        vec![ui::HandlerWrapper::new(CircleEventHandler::new(undo_id, redo_id))];
    util::set_root_and_loop(window, graph, root_widget, event_queue, ui_event_handlers);
}
