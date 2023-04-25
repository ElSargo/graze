use bincode::{deserialize, serialize};
use iced::{
    event, keyboard, subscription,
    theme::{self, Theme},
    widget::{
        self, button, column as col, container, pick_list, row, scrollable, text, text_input,
        Column, Row,
    },
    Application, Color, Command, Element, Event, Length, Renderer, Settings, Subscription,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    mem::replace,
    time::Duration,
};
mod styles;
use styles::*;

fn main() {
    AppState::run(Settings {
        window: iced::window::Settings {
            size: (300, 600),
            ..Default::default()
        },
        ..Default::default()
    })
    .expect("App crashed");
}

enum AppState {
    Loaded(State),
    Loading,
}

type Date = usize;
type MealId = [u8; 32];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    page: Page,
    stack: Vec<Page>,
    days: BTreeMap<Date, Day>,
    meals: BTreeMap<MealId, Meal>,
    meal_creation_input_field: String,
    save: SaveState,
    meal_picker_sate: MealPickerState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MealPickerState {
    input_field: String,
    selected_id: Option<MealId>,
}

#[derive(PartialEq, Copy, PartialOrd, Ord, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub enum Unit {
    #[default]
    Grams,
    KiloGrams,
    MilliLiters,
    Liters,
    TeaSpoon,
    TableSpoon,
    Pinch,
}

static UNITS: &[Unit] = &[
    Unit::Grams,
    Unit::KiloGrams,
    Unit::MilliLiters,
    Unit::Liters,
    Unit::TeaSpoon,
    Unit::TableSpoon,
    Unit::Pinch,
];

impl Unit {
    fn to_grams(&self, quantity: f64) -> f64 {
        quantity
            * match self {
                Unit::Grams => 1.0,
                Unit::KiloGrams => 1000.0,
                Unit::MilliLiters => 1.0,
                Unit::Liters => 1000.0,
                Unit::TeaSpoon => 4.2,
                Unit::TableSpoon => 13.0,
                Unit::Pinch => 0.3,
            }
    }

    fn abreviation(&self) -> &'static str {
        match self {
            Unit::Grams => "g",
            Unit::KiloGrams => "kg",
            Unit::MilliLiters => "ml",
            Unit::Liters => "l",
            Unit::TeaSpoon => "tsp",
            Unit::TableSpoon => "tbsp",
            Unit::Pinch => "pinch",
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abreviation())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SaveState {
    saved: bool,
    saving: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Page {
    DayView(Date),
    MealList,
    MealPicker,
    MealView(MealId),
    ShoppingView {
        from: Date,
        until: Date,
    },
    #[default]
    WeekView,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddDay,
    AddMeal,
    AddMealIngrediant {
        meal_id: MealId,
        new_ingrediant_name: String,
        new_ingrediant_quantity: f64,
        new_unit: Unit,
    },
    AddMealToDay(Date),
    BackPage,
    ChangeToPage(Page),
    Loaded(State),
    MealPickerInput(String),
    MealPickerSubmit,
    None,
    RemoveMeal(MealId),
    RemoveMealIngrediant {
        meal_name_id: MealId,
        ingrediant_idx: usize,
    },
    Saved(bool),
    SetMealCreationInputFeild(String),
    TabPressed {
        shift: bool,
    },
    UpdateMealIngrediant {
        meal_name_id: MealId,
        ingrediant_idx: usize,
        field: IngrediantField,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngrediantField {
    Quantity(Option<f64>, String),
    Name(String),
    Unit(Unit),
}

impl Default for IngrediantField {
    fn default() -> Self {
        IngrediantField::Unit(Unit::Grams)
    }
}

impl State {
    async fn load(path: &str) -> Self {
        match async_std::fs::read(path).await {
            Ok(bytes) => match deserialize(&bytes) {
                Ok(state) => state,
                Err(_) => State::default(),
            },
            Err(_) => State::default(),
        }
    }
    async fn save(self, path: &str) -> bool {
        println!("Saving");
        if let Ok(bytes) = serialize(&self) {
            if let Err(_) = async_std::fs::write(path, &bytes).await {
                return false;
            }
        }
        async_std::task::sleep(Duration::from_secs(1)).await;
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Day {
    date: Date,
    meals: Vec<MealId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Meal {
    name: String,
    ingrediants: Vec<Ingrediant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ingrediant {
    name: String,
    quantity: f64,
    quantity_input: Option<String>,
    unit: Unit,
}

impl Application for AppState {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            AppState::Loading,
            Command::perform(State::load("./data"), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        match self {
            Self::Loaded(state) => {
                let save_indicator = if state.save.saved { "" } else { "*" };
                let mut title = "Graze".to_owned();
                title.push_str(save_indicator);
                title
            }
            Self::Loading => "Loading".to_owned(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Self::Loading => {
                if let Message::Loaded(state) = message {
                    *self = AppState::Loaded(state);
                }
                Command::none()
            }
            Self::Loaded(state) => {
                state.save.saved = false;
                if let Message::Saved(succes) = message {
                    state.save.saved = succes;
                    state.save.saving = false;
                }
                let com = match message {
                    Message::MealPickerInput(input) => {
                        state.meal_picker_sate.input_field = input;
                        Command::none()
                    }
                    Message::AddMeal => {
                        state.meals.insert(
                            hash_str(&state.meal_creation_input_field),
                            Meal {
                                name: state.meal_creation_input_field.to_owned(),
                                ingrediants: Vec::new(),
                            },
                        );
                        state.meal_creation_input_field = String::new();
                        Command::none()
                    }
                    Message::RemoveMealIngrediant {
                        meal_name_id: meal_name_hash,
                        ingrediant_idx,
                    } => {
                        if let Some(meal) = state.meals.get_mut(&meal_name_hash) {
                            meal.ingrediants.remove(ingrediant_idx);
                        }
                        Command::none()
                    }
                    Message::ChangeToPage(page) => {
                        state.stack.push(replace(&mut state.page, page));
                        Command::none()
                    }
                    Message::AddDay => {
                        state.days.insert(
                            state.days.len(),
                            Day {
                                date: state.days.len(),
                                meals: Vec::new(),
                            },
                        );
                        Command::none()
                    }
                    Message::AddMealToDay(date) => {
                        if let Some(id) = state.meal_picker_sate.selected_id {
                            match state.days.get_mut(&date) {
                                Some(day) => day.meals.push(id),
                                None => {
                                    state.days.insert(
                                        date,
                                        Day {
                                            date,
                                            meals: Vec::new(),
                                        },
                                    );
                                }
                            };
                        }
                        Command::none()
                    }
                    Message::BackPage => {
                        if let Some(page) = state.stack.pop() {
                            state.page = page
                        }
                        Command::none()
                    }
                    Message::UpdateMealIngrediant {
                        meal_name_id: meal_name_hash,
                        ingrediant_idx,
                        field,
                    } => {
                        if let Some(meal) = state.meals.get_mut(&meal_name_hash) {
                            if let Some(Ingrediant {
                                name,
                                quantity,
                                quantity_input,
                                unit,
                            }) = meal.ingrediants.get_mut(ingrediant_idx)
                            {
                                match field {
                                    IngrediantField::Quantity(
                                        new_quantity_option,
                                        new_quantity_input,
                                    ) => {
                                        if let Some(new_quantity) = new_quantity_option {
                                            *quantity = new_quantity;
                                        }
                                        *quantity_input = Some(new_quantity_input);
                                    }
                                    IngrediantField::Name(new_name) => {
                                        *name = new_name;
                                    }
                                    IngrediantField::Unit(new_unit) => {
                                        *quantity =
                                            *quantity * unit.to_grams(1.) / new_unit.to_grams(1.);
                                        *unit = new_unit;
                                    }
                                }
                            }
                        }
                        Command::none()
                    }
                    Message::AddMealIngrediant {
                        meal_id: meal_name,
                        new_ingrediant_name,
                        new_ingrediant_quantity,
                        new_unit,
                    } => {
                        if let Some(meal) = state.meals.get_mut(&meal_name) {
                            meal.ingrediants.push(Ingrediant {
                                name: new_ingrediant_name,
                                quantity: new_ingrediant_quantity,
                                quantity_input: None,
                                unit: new_unit,
                            });
                        }
                        Command::none()
                    }
                    Message::None | Message::Saved(_) => Command::none(),
                    Message::SetMealCreationInputFeild(input) => {
                        state.meal_creation_input_field = input;
                        Command::none()
                    }
                    Message::Loaded(_) => unreachable!(),
                    Message::MealPickerSubmit => {
                        let id = hash_str(&state.meal_picker_sate.input_field);
                        if state.meals.contains_key(&id) {
                            state.meal_picker_sate.selected_id = Some(id);
                        };
                        Command::perform(dummy(), |_| Message::BackPage)
                    }
                    Message::RemoveMeal(id) => {
                        state.meals.remove(&id);
                        Command::none()
                    }
                    Message::TabPressed { shift } => {
                        if shift {
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                };
                let save_com = if !state.save.saving && !state.save.saved {
                    let copy = state.to_owned();
                    state.save.saving = true;
                    Command::perform(copy.save("./data"), Message::Saved)
                } else {
                    Command::none()
                };
                Command::batch([com, save_com])
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match self {
            AppState::Loading => text("Loading").into(),
            AppState::Loaded(state) => {
                let page = match &state.page {
                    Page::MealList => meal_list_view(state),
                    Page::DayView(date) => day_view(state, date),
                    Page::MealView(meal_id) => meal_view(state, meal_id),
                    Page::WeekView => week_view(state),
                    Page::ShoppingView { from, until } => shopping_view(state, from, until),
                    Page::MealPicker => meal_picker_view(state),
                };

                col![bar_view(), scrollable(page)]
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .spacing(10)
                    .padding(10)
                    .into()
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        self::Theme::custom(theme::Palette {
            background: Color::from_rgba(0.157, 0.157, 0.157, 1.0),
            text: Color::from_rgba(0.922, 0.859, 0.698, 1.0),
            primary: Color::from_rgba(0.271, 0.522, 0.533, 1.0),
            success: Color::from_rgba(0.722, 0.733, 0.149, 1.0),
            danger: Color::from_rgba(0.984, 0.286, 0.204, 1.0),
        })
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                    ..
                }),
                event::Status::Ignored,
            ) => Some(Message::TabPressed {
                shift: modifiers.shift(),
            }),

            _ => None,
        })
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn shopping_view<'a>(
    state: &State,
    from: &usize,
    until: &usize,
) -> Column<'a, Message, Renderer<Theme>> {
    let mut meals_and_count = HashMap::new();
    for (_, day) in state.days.range(from..until) {
        for meal in day.meals.iter() {
            if let Some(count) = meals_and_count.get_mut(meal) {
                *count += 1.0;
            } else {
                meals_and_count.insert(meal.to_owned(), 1.0);
            }
        }
    }
    let mut list: HashMap<&String, f64> = HashMap::new();
    for (meal_name, count) in meals_and_count {
        if let Some(meal) = state.meals.get(&meal_name) {
            for Ingrediant {
                name,
                quantity,
                quantity_input: _,
                unit,
            } in meal.ingrediants.iter()
            {
                let ammount = unit.to_grams(quantity * count);
                match list.get_mut(&name) {
                    Some(total) => {
                        *total = *total + ammount;
                    }
                    None => {
                        list.insert(&name, ammount);
                    }
                };
            }
        }
    }

    col![
        text("Shopping"),
        col(list
            .iter()
            .map(|(name, ammount)| text(format!("{name}: {ammount}")).into())
            .collect())
    ]
}

fn meal_view<'a>(state: &'a State, meal_id: &'a [u8; 32]) -> Column<'a, Message, Renderer<Theme>> {
    if let Some(meal) = state.meals.get(meal_id) {
        let mut edit_buttons = Vec::with_capacity(meal.ingrediants.len());
        let mut edit_name = Vec::with_capacity(meal.ingrediants.len());
        let mut edit_quantity = Vec::with_capacity(meal.ingrediants.len());
        let mut quantity_was_parsed = Vec::with_capacity(meal.ingrediants.len());
        let mut edit_unit = Vec::with_capacity(meal.ingrediants.len());
        for (
            i,
            Ingrediant {
                name,
                quantity,
                quantity_input,
                unit,
            },
        ) in meal.ingrediants.iter().enumerate()
        {
            let (previous_quantity_was_parsed, quantity_input_text) = match quantity_input {
                Some(raw_input) => {
                    let parsed_quantity: Option<f64> = raw_input.parse().ok();
                    (
                        parsed_quantity.is_some(),
                        match parsed_quantity {
                            Some(q) => format!("{q}"),
                            None => raw_input.to_owned(),
                        },
                    )
                }
                None => (false, format!("{quantity}")),
            };

            edit_buttons.push(
                delete_button()
                    .on_press(Message::RemoveMealIngrediant {
                        meal_name_id: meal_id.to_owned(),
                        ingrediant_idx: i,
                    })
                    .into(),
            );
            edit_name.push(
                text_input(name, name)
                    .on_input(move |s| Message::UpdateMealIngrediant {
                        meal_name_id: meal_id.to_owned(),
                        ingrediant_idx: i,
                        field: IngrediantField::Name(s),
                    })
                    .into(),
            );
            edit_quantity.push(
                text_input("edit quantity", &quantity_input_text)
                    .on_input(move |input| {
                        let new_quantity = input.parse().ok();
                        Message::UpdateMealIngrediant {
                            meal_name_id: meal_id.to_owned(),
                            ingrediant_idx: i,
                            field: IngrediantField::Quantity(new_quantity, input),
                        }
                    })
                    .into(),
            );
            quantity_was_parsed.push(
                if previous_quantity_was_parsed {
                    text("Y")
                } else {
                    text("N")
                }
                .into(),
            );
            edit_unit.push(
                pick_list(UNITS, Some(*unit), move |u| Message::UpdateMealIngrediant {
                    meal_name_id: meal_id.to_owned(),
                    ingrediant_idx: i,
                    field: IngrediantField::Unit(u),
                })
                .into(),
            )
        }
        col![
            text(match state.meals.get(meal_id) {
                Some(meal) => meal.name.as_str(),
                None => "Unknown Meal",
            }),
            row![
                col(edit_buttons),
                col(edit_name).width(Length::FillPortion(2)),
                col(edit_quantity).width(Length::FillPortion(1)),
                col(quantity_was_parsed),
                col(edit_unit).width(Length::FillPortion(1))
            ]
            .width(Length::Fill)
            .spacing(10), // .padding(10),
            button("Add").on_press(Message::AddMealIngrediant {
                meal_id: meal_id.to_owned(),
                new_ingrediant_name: "My ingrediant".to_owned(),
                new_ingrediant_quantity: 0.,
                new_unit: Unit::Grams,
            })
        ]
    } else {
        col![Element::<_>::from(text("Meal not found"))]
    }
}

fn day_view<'a>(state: &State, date: &usize) -> Column<'a, Message, Renderer<Theme>> {
    if let Some(day) = state.days.get(&date) {
        col![
            text(format!("Day {date}")),
            row![
                button(text(format!(
                    "selected: <{}>",
                    &state.meal_picker_sate.input_field
                )))
                .on_press(Message::ChangeToPage(Page::MealPicker)),
                button("Submit").on_press(Message::AddMealToDay(*date))
            ],
            col(day
                .meals
                .iter()
                .map(|id| {
                    button(text(match state.meals.get(id) {
                        Some(meal) => meal.name.as_str(),
                        None => "meal not found",
                    }))
                    .on_press(Message::ChangeToPage(Page::MealView(*id)))
                    .into()
                })
                .collect()),
        ]
    } else {
        col![text("Day is invalid")]
    }
}

fn meal_list_view<'a>(state: &State) -> Column<'a, Message, Renderer<Theme>> {
    let mut list = Vec::with_capacity(state.meals.len());
    for (id, meal) in state.meals.iter() {
        list.push(
            row![
                container(text(meal.name.as_str()).size(20)).width(Length::Fill),
                button(edit_icon()).on_press(Message::ChangeToPage(Page::MealView(*id))),
                delete_button().on_press(Message::RemoveMeal(*id))
            ]
            .spacing(5)
            .into(),
        );
    }
    col![
        col(list).spacing(5),
        row![
            text_input("New meal", &state.meal_creation_input_field)
                .on_input(Message::SetMealCreationInputFeild,),
            button("Add").on_press(Message::AddMeal)
        ]
        .spacing(10),
    ]
}

fn hash_str(s: &str) -> MealId {
    let mut hasher = Sha3_256::new();
    hasher.update(s);
    let data: Vec<_> = hasher.finalize().into_iter().collect();
    let mut slice: [u8; 32] = [0; 32];
    slice.copy_from_slice(&data[0..32]);
    unsafe { std::mem::transmute(slice) }
}
async fn dummy() -> () {}

fn week_view<'a>(state: &State) -> Column<'a, Message, Renderer<Theme>> {
    col![
        text("weeks"),
        button("add day").on_press(Message::AddDay),
        col(state
            .days
            .iter()
            .flat_map(|(_, day)| {
                let mut widgets = Vec::with_capacity(2);
                let content = format!("Day: {}, {} meals added", day.date, day.meals.len());
                if day.date % 7 == 0 {
                    widgets.push(text("Week").into());
                }
                widgets.push(
                    button(text(content))
                        .on_press(Message::ChangeToPage(Page::DayView(day.date)))
                        .into(),
                );
                widgets
            })
            .collect::<Vec<_>>())
    ]
}

fn meal_picker_view<'a>(state: &'a State) -> Column<'a, Message, Renderer<Theme>> {
    col![text_input("Search", &state.meal_picker_sate.input_field)
        .on_input(Message::MealPickerInput)
        .on_submit(Message::MealPickerSubmit),]
}

fn bar_view<'a>() -> Row<'a, Message, Renderer<Theme>> {
    row![
        header_button("Back", BackButtonStyle).on_press(Message::BackPage),
        col![].width(Length::FillPortion(1)),
        header_button("Week", HeaderButtonStyle).on_press(Message::ChangeToPage(Page::WeekView)),
        col![].width(Length::FillPortion(1)),
        header_button("List", HeaderButtonStyle).on_press(Message::ChangeToPage(
            Page::ShoppingView {
                from: 0,
                until: 100
            }
        )),
        col![].width(Length::FillPortion(1)),
        header_button("Meals", HeaderButtonStyle).on_press(Message::ChangeToPage(Page::MealList)),
        col![].width(Length::FillPortion(1)),
        header_button("Calender", HeaderButtonStyle),
    ]
    .width(Length::Fill)
}
