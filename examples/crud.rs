#[macro_use]
extern crate limn;
extern crate text_layout;
extern crate cassowary;

mod util;

use std::mem;
use std::collections::HashMap;

use limn::event::{Target, UiEventHandler};
use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widget::property::{Property, PropChange};
use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::widgets::edit_text::{EditTextBuilder, TextUpdated};
use limn::widgets::list::{ListBuilder, STYLE_LIST_ITEM};
use limn::drawable::text::{TextDrawable, TextStyleable};
use limn::drawable::rect::RectDrawable;
use limn::resources::WidgetId;
use limn::ui::Ui;
use limn::util::Dimensions;
use limn::color::*;
use limn::layout::constraint::*;

#[derive(Clone, Debug)]
pub struct Person {
    first_name: String,
    last_name: String,
}
impl Person {
    fn new() -> Self {
        Person {
            first_name: String::new(),
            last_name: String::new(),
        }
    }
    fn name(&self) -> String {
        format!("{}, {}", self.last_name, self.first_name)
    }
    fn is_valid(&self) -> bool {
        self.first_name.len() > 0 && self.last_name.len() > 0
    }
}

#[derive(Clone)]
enum PeopleEvent {
    Add,
    Update,
    Delete,
    PersonSelected(Option<WidgetId>),
    ChangeFirstName(String),
    ChangeLastName(String),
}

struct Ids {
    list_widget: WidgetId,
    first_name_box: WidgetId,
    last_name_box: WidgetId,
    create_button: WidgetId,
    update_button: WidgetId,
    delete_button: WidgetId,
}
struct PeopleHandler {
    ids: Ids,
    selected_item: Option<WidgetId>,
    person: Person,
    people: HashMap<WidgetId, Person>,
}
impl PeopleHandler {
    fn new(ids: Ids) -> Self {
        PeopleHandler {
            ids: ids,
            selected_item: None,
            person: Person::new(),
            people: HashMap::new(),
        }
    }
}

impl PeopleHandler {
    fn update_selected(&mut self) {
        let ref ids = self.ids;
        event!(Target::SubTree(ids.first_name_box), TextUpdated(self.person.first_name.clone()));
        event!(Target::SubTree(ids.last_name_box), TextUpdated(self.person.last_name.clone()));
        if self.selected_item.is_some() {
            event!(Target::SubTree(ids.update_button), PropChange::Remove(Property::Inactive));
            event!(Target::SubTree(ids.delete_button), PropChange::Remove(Property::Inactive));
        } else {
            event!(Target::SubTree(ids.update_button), PropChange::Add(Property::Inactive));
            event!(Target::SubTree(ids.delete_button), PropChange::Add(Property::Inactive));
        }
    }
}
impl UiEventHandler<PeopleEvent> for PeopleHandler {
    fn handle(&mut self, event: &PeopleEvent, ui: &mut Ui) {

        let was_valid = self.person.is_valid();
        match event.clone() {
            PeopleEvent::Add => {
                if was_valid {
                    let person = mem::replace(&mut self.person, Person::new());
                    let id = add_person(&person, ui, self.ids.list_widget);
                    self.people.insert(id, person);

                    self.selected_item = None;
                    self.update_selected();
                }
            },
            PeopleEvent::Update => {
                if let Some(selected_widget_id) = self.selected_item {
                    self.people.insert(selected_widget_id, self.person.clone());
                    event!(Target::SubTree(selected_widget_id), TextUpdated(self.person.name()));
                }
            },
            PeopleEvent::Delete => {
                if let Some(selected_widget_id) = self.selected_item {
                    self.people.remove(&selected_widget_id);
                    ui.remove_widget(selected_widget_id);
                }
                self.selected_item = None;
            }
            PeopleEvent::PersonSelected(widget_id) => {
                self.selected_item = widget_id;
                if let Some(widget_id) = widget_id {
                    self.person = self.people[&widget_id].clone();
                } else {
                    self.person = Person::new();
                }
                self.update_selected();
            },
            PeopleEvent::ChangeFirstName(name) => {
                self.person.first_name = name;
            },
            PeopleEvent::ChangeLastName(name) => {
                self.person.last_name = name;
            }
        }
        let is_valid = self.person.is_valid();
        if was_valid != is_valid {
            if is_valid {
                event!(Target::SubTree(self.ids.create_button), PropChange::Remove(Property::Inactive));
            } else {
                event!(Target::SubTree(self.ids.create_button), PropChange::Add(Property::Inactive));
            }
        }
    }
}

use limn::widgets::edit_text;
pub fn add_person(person: &Person, ui: &mut Ui, list_widget_id: WidgetId) -> WidgetId {
    let mut list_item_widget = {
        let text_style = style!(TextStyleable::TextColor: WHITE);
        let text_drawable = TextDrawable::new(&person.name());
        let text_dims = text_drawable.measure();
        let mut list_item_widget = WidgetBuilder::new();
        list_item_widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
            .list_item(list_widget_id)
            .enable_hover();
        layout!(list_item_widget: height(text_dims.height));
        let mut list_text_widget = WidgetBuilder::new();
        list_text_widget
            .set_drawable_with_style(text_drawable, text_style)
            .add_handler_fn(edit_text::text_change_handle);
        layout!(list_text_widget: center(&list_item_widget));
        list_item_widget.add_child(list_text_widget);
        list_item_widget
    };
    let list_item_widget_id = list_item_widget.id();
    ui.add_widget(list_item_widget, list_widget_id);
    list_item_widget_id
}

fn main() {
    let (window, mut app) = util::init_default("Limn edit text demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    layout!(root_widget: min_dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    }));
    let mut container = WidgetBuilder::new();
    layout!(container: bound_by(&root_widget).padding(20.0));

    let create_name_group = |title, container: &mut WidgetBuilder| {
        let mut name_container = WidgetBuilder::new();
        layout!(name_container: match_width(container));

        let mut static_text = WidgetBuilder::new();
        let text = TextDrawable::new(title);
        let text_dims = text.measure();
        static_text.set_drawable(text);
        layout!(static_text:
            center_vertical(&name_container),
            dimensions(text_dims));

        let mut text_box = EditTextBuilder::new();
        layout!(text_box:
            min_height(30.0),
            min_width(200.0),
            align_right(&name_container),
            to_right_of(&static_text).padding(20.0));
        name_container.add_child(static_text);
        (name_container, text_box)
    };

    let (mut first_name_container, mut first_name_box) = create_name_group("First name:", &mut container);
    let (mut last_name_container, mut last_name_box) = create_name_group("Last name:", &mut container);

    layout!(first_name_container: align_top(&container));
    layout!(last_name_container: below(&first_name_container).padding(20.0));
    first_name_box.on_text_changed(|text, _| {
        event!(Target::Ui, PeopleEvent::ChangeFirstName(text.0.clone()));
    });
    last_name_box.on_text_changed(|text, _| {
        event!(Target::Ui, PeopleEvent::ChangeLastName(text.0.clone()));
    });

    let mut button_container = WidgetBuilder::new();
    layout!(button_container: below(&last_name_container).padding(20.0));

    let mut create_button = PushButtonBuilder::new();
    create_button.set_text("Create");
    create_button.set_inactive();
    let mut update_button = PushButtonBuilder::new();
    update_button.set_text("Update");
    update_button.set_inactive();
    update_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Update);
    });
    let mut delete_button = PushButtonBuilder::new();
    delete_button.set_text("Delete");
    delete_button.set_inactive();
    delete_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Delete);
    });
    layout!(update_button: to_right_of(&create_button).padding(20.0));
    layout!(delete_button: to_right_of(&update_button).padding(20.0));

    let mut scroll_container = WidgetBuilder::new();
    scroll_container
        .set_drawable(RectDrawable::new())
        .contents_scroll();
    layout!(scroll_container:
        below(&button_container).padding(20.0),
        min_height(260.0));

    let mut list_widget = ListBuilder::new();
    list_widget.on_item_selected(|selected, _| {
        event!(Target::Ui, PeopleEvent::PersonSelected(selected));
    });
    layout!(list_widget: match_width(&scroll_container));

    create_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Add);
    });
    let ids = Ids {
        list_widget: list_widget.id(),
        first_name_box: first_name_box.id(),
        last_name_box: last_name_box.id(),
        create_button: create_button.id(),
        update_button: update_button.id(),
        delete_button: delete_button.id(),
    };
    first_name_container.add_child(first_name_box);
    last_name_container.add_child(last_name_box);
    scroll_container.add_child(list_widget);
    button_container
        .add_child(create_button)
        .add_child(update_button)
        .add_child(delete_button);

    container
        .add_child(first_name_container)
        .add_child(last_name_container)
        .add_child(button_container)
        .add_child(scroll_container);
    root_widget.add_child(container);

    app.add_handler(PeopleHandler::new(ids));

    util::set_root_and_loop(window, app, root_widget);
}
