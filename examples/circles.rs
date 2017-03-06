extern crate limn;
extern crate glutin;
extern crate graphics;
extern crate petgraph;
extern crate cassowary;

mod util;

use cassowary::strength::*;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::PushButtonBuilder;
use limn::drawable::ellipse::EllipseDrawable;
use limn::widget::property::{Property, PropChange};
use limn::ui::queue::Target;
use limn::ui::LimnSolver;
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
    let (window, mut ui) = util::init_default("Limn circles demo");
    util::load_default_font();

    fn create_undo_redo_buttons(root_widget: &mut WidgetBuilder) -> (WidgetId, WidgetId) {
        let mut button_container = WidgetBuilder::new();
        button_container.layout.center_horizontal(&root_widget);
        button_container.layout.align_bottom(&root_widget, Some(20.0));
        button_container.layout.shrink();

        let undo_widget = PushButtonBuilder::new()
            .set_text("Undo")
            .widget
            .set_inactive()
            .on_click(|_, args| { args.event_queue.push(Target::Ui, CircleEvent::Undo); });
        let mut redo_widget = PushButtonBuilder::new()
            .set_text("Redo")
            .widget
            .set_inactive()
            .on_click(|_, args| { args.event_queue.push(Target::Ui, CircleEvent::Redo); });
        redo_widget.layout.to_right_of(&undo_widget, Some(20.0));

        let (undo_id, redo_id) = (undo_widget.id, redo_widget.id);
        button_container.add_child(undo_widget);
        button_container.add_child(redo_widget);

        root_widget.add_child(button_container);
        (undo_id, redo_id)
    }

    fn create_circle(graph: &mut WidgetGraph, solver: &mut LimnSolver, center: &Point) -> WidgetId {
        let border = graphics::ellipse::Border {
            color: BLACK,
            radius: 2.0,
        };
        let mut widget = WidgetBuilder::new()
            .set_drawable(EllipseDrawable::new(RED, Some(border)));
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
        let root_id = graph.root_id;
        graph.add_widget(widget, Some(root_id), solver);
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
            let graph = &mut args.ui.graph;
            let solver = &mut args.ui.solver;
            match *event {
                CircleEvent::Add(point) => {
                    self.circles.push((point, create_circle(graph, solver, &point)));
                    self.undo.clear();

                    args.event_queue.push(Target::SubTree(self.undo_id), PropChange::Remove(Property::Inactive));
                    args.event_queue.push(Target::SubTree(self.redo_id), PropChange::Add(Property::Inactive));
                }
                CircleEvent::Undo => {
                    if self.circles.len() > 0 {
                        let (point, node_index) = self.circles.pop().unwrap();
                        graph.remove_widget(node_index, solver);
                        self.undo.push(point);
                        args.event_queue.push(Target::SubTree(self.redo_id), PropChange::Remove(Property::Inactive));
                        if self.circles.len() == 0 {
                            args.event_queue.push(Target::SubTree(self.undo_id), PropChange::Add(Property::Inactive));
                        }
                    }
                }
                CircleEvent::Redo => {
                    if self.undo.len() > 0 {
                        let point = self.undo.pop().unwrap();
                        self.circles.push((point, create_circle(graph, solver, &point)));
                        if self.undo.len() == 0 {
                            args.event_queue.push(Target::SubTree(self.redo_id), PropChange::Add(Property::Inactive));
                        }
                    }
                }
            }
        }
    }
    let mut root_widget = WidgetBuilder::new().on_click(|event, args| {
        let event = CircleEvent::Add(event.position);
        args.event_queue.push(Target::Ui, event);
    });
    root_widget.layout.dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });


    let (undo_id, redo_id) = create_undo_redo_buttons(&mut root_widget);

    let circle_handler = ui::HandlerWrapper::new(CircleEventHandler::new(undo_id, redo_id));
    ui.event_handlers.push(circle_handler);

    util::set_root_and_loop(window, ui, root_widget);
}
