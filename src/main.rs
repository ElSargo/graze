#![forbid(clippy::all, clippy::pedantic, clippy::nursery)]
mod generational_map;
use bincode::{deserialize, serialize};
use fuse_rust::{Fuse, SearchResult};
use generational_map::{GenerationalKey, GenerationalMap};
use iced::{
    event, keyboard, subscription,
    theme::{self, Theme},
    widget::{
        self, button, column as col, container, pick_list, row, scrollable, text, text_input,
        Column, Row,
    },
    Application, Color, Command, Element, Event, Length, Renderer, Settings, Subscription,
};
use itertools::Itertools;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::Display,
    mem::{replace, take},
    ops::Range,
    time::Duration,
};
mod styles;
use styles::{delete_button, edit_icon, header_button, BackButtonStyle, HeaderButtonStyle};

fn main() {
    AppState::run(Settings {
        window: iced::window::Settings {
            size: (300, 650),
            ..Default::default()
        },
        ..Default::default()
    })
    .expect("App crashed");
}

static MEAL_ADDER_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static MEAL_PICKER_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static MEAL_INGREDIANT_ADDER_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

enum AppState {
    Loaded(State),
    Loading,
}

type Date = usize;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    page: Page,
    stack: Vec<Page>,
    days: BTreeMap<Date, Day>,
    meals: GenerationalMap<Meal>,
    meal_creation_input_field: String,
    save: SaveState,
    meal_picker_sate: MealPickerState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MealPickerState {
    input_field: String,
    selected_id: Option<GenerationalKey>,
    search_results: Vec<SearchResultPlus>,
    searched_meal_ids: Vec<GenerationalKey>,
    selection_index: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResultPlus {
    /// corresponding index of the search result in the original list
    pub index: usize,
    /// Search rating of the search result, 0.0 is a perfect match 1.0 is a perfect mismatch
    pub score: f64,
    /// Ranges of matches in the search query, useful if you want to hightlight matches.
    pub ranges: Vec<Range<usize>>,
}

impl From<SearchResult> for SearchResultPlus {
    fn from(value: SearchResult) -> Self {
        Self {
            index: value.index,
            score: value.score,
            ranges: value.ranges,
        }
    }
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
    fn to_grams(self, quantity: f64) -> f64 {
        quantity * self.in_grams()
    }

    const fn is_liquid(self) -> bool {
        match self {
            Self::Liters | Self::MilliLiters => true,
            _ => false,
        }
    }

    const fn in_grams(self) -> f64 {
        match self {
            Self::Grams | Self::MilliLiters => 1.0,
            Self::KiloGrams | Self::Liters => 1000.0,
            Self::TeaSpoon => 4.2,
            Self::TableSpoon => 13.0,
            Self::Pinch => 0.3,
        }
    }

    fn from_grams(self, quantity: f64) -> f64 {
        quantity / self.in_grams()
    }

    const fn abreviation(self) -> &'static str {
        match self {
            Self::Grams => "g",
            Self::KiloGrams => "kg",
            Self::MilliLiters => "ml",
            Self::Liters => "L",
            Self::TeaSpoon => "tsp",
            Self::TableSpoon => "tbsp",
            Self::Pinch => "pinch",
        }
    }
}

fn apropriate_unit(grams: f64, is_liquid: bool) -> (f64, Unit) {
    if is_liquid {
        if grams < Unit::Liters.in_grams() {
            (grams, Unit::MilliLiters)
        } else {
            (Unit::Liters.from_grams(grams), Unit::Liters)
        }
    } else {
        if grams < Unit::KiloGrams.in_grams() {
            (grams, Unit::Grams)
        } else {
            (Unit::KiloGrams.from_grams(grams), Unit::KiloGrams)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Page {
    DayView(Date),
    MealList,
    MealPicker,
    MealEditorView(GenerationalKey),
    ShoppingView { from: Date, until: Date },
    WeekView(Range<Date>),
}

impl Default for Page {
    fn default() -> Self {
        Self::WeekView(1..100)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddDay(Date),
    AddMeal,
    AddMealIngrediant {
        meal_id: GenerationalKey,
        new_ingrediant_name: String,
        new_ingrediant_quantity: f64,
        new_unit: Unit,
    },
    AddMealToDay(Date, GenerationalKey),
    BackPage,
    ChangeToPage(Page),
    Loaded(State),
    MealPickerInput(String),
    MealPickerSubmit(Option<GenerationalKey>),
    None,
    RemoveMeal(GenerationalKey),
    RemoveMealIngrediant {
        meal_name_id: GenerationalKey,
        ingrediant_idx: usize,
    },
    Saved(bool),
    SetMealCreationInputFeild(String),
    TabPressed {
        shift: bool,
    },
    UpdateMealIngrediant {
        meal_name_id: GenerationalKey,
        ingrediant_idx: usize,
        field: IngrediantField,
    },
    VerticalMovement(isize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngrediantField {
    Quantity(Option<f64>, String),
    Name(String),
    Unit(Unit),
}

impl Default for IngrediantField {
    fn default() -> Self {
        Self::Unit(Unit::Grams)
    }
}

impl State {
    async fn load(path: &str) -> Self {
        async_std::fs::read(path)
            .await
            .ok()
            .and_then(|bytes| deserialize(&bytes).ok())
            .unwrap_or_default()
    }
    async fn save(self, path: &str) -> bool {
        println!("Saving");
        if let Ok(bytes) = serialize(&self) {
            if async_std::fs::write(path, &bytes).await.is_err() {
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
    meals: Vec<GenerationalKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            Self::Loading,
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
                    *self = Self::Loaded(state);
                }
                Command::none()
            }
            Self::Loaded(state) => update_ui(state, message),
        }
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match self {
            Self::Loading => text("Loading").into(),
            Self::Loaded(state) => {
                let page = match &state.page {
                    Page::MealList => meal_list_view(state),
                    Page::DayView(date) => day_view(state, *date),
                    Page::MealEditorView(meal_id) => meal_editor_view(state, meal_id),
                    Page::WeekView(range) => week_view(state, range),
                    Page::ShoppingView { from, until } => shopping_view(state, *from, *until),
                    Page::MealPicker => meal_picker_view(state),
                };

                col![bar_view(state), scrollable(page)]
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
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Down | keyboard::KeyCode::J,
                    ..
                }),
                event::Status::Ignored,
            ) => Some(Message::VerticalMovement(1)),
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Up | keyboard::KeyCode::K,
                    ..
                }),
                event::Status::Ignored,
            ) => Some(Message::VerticalMovement(-1)),

            _ => None,
        })
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn update_ui(state: &mut State, message: Message) -> Command<Message> {
    state.save.saved = false;
    let com = match message {
        Message::MealPickerInput(input) => on_message_meal_picker_input(state, input),
        Message::AddMeal => on_message_add_meal(state),
        Message::RemoveMealIngrediant {
            meal_name_id: meal_name_hash,
            ingrediant_idx,
        } => on_message_remove_meal_ingrediant(state, meal_name_hash, ingrediant_idx),
        Message::ChangeToPage(page) => on_message_change_page(page, state),
        Message::AddDay(date) => on_message_add_day(state, date),
        Message::AddMealToDay(date, id) => on_message_add_meal_to_day(state, date, id),
        Message::BackPage => {
            back_page(state);
            Command::none()
        }
        Message::UpdateMealIngrediant {
            meal_name_id: meal_name_hash,
            ingrediant_idx,
            field,
        } => on_message_update_meal_ingrediant(state, meal_name_hash, ingrediant_idx, field),
        Message::AddMealIngrediant {
            meal_id: meal_name,
            new_ingrediant_name,
            new_ingrediant_quantity,
            new_unit,
        } => on_message_add_meal_ingrediant(
            state,
            meal_name,
            new_ingrediant_name,
            new_ingrediant_quantity,
            new_unit,
        ),
        Message::Saved(succes) => {
            state.save.saved = succes;
            state.save.saving = false;
            Command::none()
        }
        Message::SetMealCreationInputFeild(input) => {
            state.meal_creation_input_field = input;
            Command::none()
        }
        Message::Loaded(_) => unreachable!(),
        Message::MealPickerSubmit(meal_id) => {
            state.meal_picker_sate.selected_id = meal_id;
            back_page(state);
            Command::none()
        }
        Message::RemoveMeal(id) => {
            state.meals.remove(id);
            Command::none()
        }
        Message::TabPressed { shift } => {
            if shift {
                widget::focus_previous()
            } else {
                widget::focus_next()
            }
        }
        Message::VerticalMovement(movment) if state.page == Page::MealPicker => {
            on_message_vertical_movement(movment, state)
        }
        _ => Command::none(),
    };
    let save_com = if state.save.saving || state.save.saved {
        Command::none()
    } else {
        let copy = state.clone();
        state.save.saving = true;
        Command::perform(copy.save("./data"), Message::Saved)
    };
    Command::batch([com, save_com])
}

fn on_message_vertical_movement(offset: isize, state: &mut State) -> Command<Message> {
    let new_index = state
        .meal_picker_sate
        .selection_index
        .saturating_add_signed(offset)
        .min(
            if state.meal_picker_sate.search_results.is_empty() {
                state.meals.len()
            } else {
                state.meal_picker_sate.search_results.len()
            }
            .saturating_sub(1),
        );

    state.meal_picker_sate.selection_index = new_index;

    Command::none()
}

fn on_message_add_meal_ingrediant(
    state: &mut State,
    meal_name: GenerationalKey,
    new_ingrediant_name: String,
    new_ingrediant_quantity: f64,
    new_unit: Unit,
) -> Command<Message> {
    if let Some(meal) = state.meals.get_mut(meal_name) {
        meal.ingrediants.push(Ingrediant {
            name: new_ingrediant_name,
            quantity: new_ingrediant_quantity,
            quantity_input: None,
            unit: new_unit,
        });
    }
    Command::none()
}

fn on_message_add_meal_to_day(
    state: &mut State,
    date: usize,
    id: GenerationalKey,
) -> Command<Message> {
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
    }
    Command::none()
}

fn on_message_add_day(state: &mut State, date: Date) -> Command<Message> {
    state.days.insert(
        date,
        Day {
            date,
            meals: Vec::new(),
        },
    );
    Command::none()
}

fn on_message_change_page(page: Page, state: &mut State) -> Command<Message> {
    let command = match page {
        Page::MealList => iced::widget::text_input::focus(MEAL_ADDER_INPUT_ID.clone()),
        Page::MealPicker => iced::widget::text_input::focus(MEAL_PICKER_INPUT_ID.clone()),
        Page::MealEditorView(_) => {
            iced::widget::text_input::focus(MEAL_INGREDIANT_ADDER_INPUT_ID.clone())
        }
        _ => Command::none(),
    };
    state.stack.push(replace(&mut state.page, page));
    command
}

fn on_message_remove_meal_ingrediant(
    state: &mut State,
    meal_name_hash: GenerationalKey,
    ingrediant_idx: usize,
) -> Command<Message> {
    if let Some(meal) = state.meals.get_mut(meal_name_hash) {
        meal.ingrediants.remove(ingrediant_idx);
    }
    Command::none()
}

fn on_message_add_meal(state: &mut State) -> Command<Message> {
    state.meals.push(Meal {
        name: state.meal_creation_input_field.clone(),
        ingrediants: Vec::new(),
    });
    state.meal_creation_input_field = String::new();
    Command::none()
}

fn on_message_meal_picker_input(state: &mut State, input: String) -> Command<Message> {
    state.meal_picker_sate.input_field = input;
    state.meal_picker_sate.selection_index = 0;
    let searcher = Fuse::default();

    state.meal_picker_sate.search_results = if state.meal_picker_sate.input_field.is_empty() {
        Vec::new()
    } else {
        state.meal_picker_sate.searched_meal_ids = state.meals.keys().collect();
        searcher
            .search_text_in_iterable(
                &state.meal_picker_sate.input_field,
                state.meals.values().map(|meal| &meal.name),
            )
            .into_iter()
            .map(std::convert::Into::into)
            .collect()
    };

    Command::none()
}

fn on_message_update_meal_ingrediant(
    state: &mut State,
    meal_id: GenerationalKey,
    ingrediant_idx: usize,
    field: IngrediantField,
) -> Command<Message> {
    if let Some(meal) = state.meals.get_mut(meal_id) {
        if let Some(Ingrediant {
            name,
            quantity,
            quantity_input,
            unit,
        }) = meal.ingrediants.get_mut(ingrediant_idx)
        {
            match field {
                IngrediantField::Quantity(new_quantity_option, new_quantity_input) => {
                    if let Some(new_quantity) = new_quantity_option {
                        *quantity = new_quantity;
                    }
                    *quantity_input = Some(new_quantity_input);
                }
                IngrediantField::Name(new_name) => {
                    *name = new_name;
                }
                IngrediantField::Unit(new_unit) => {
                    *quantity = unit.to_grams(*quantity) / new_unit.to_grams(1.);
                    *unit = new_unit;
                    *quantity_input = None;
                }
            }
        }
    }
    Command::none()
}

fn back_page(state: &mut State) {
    if let Some(page) = state.stack.pop() {
        state.page = page;
    }
}

fn shopping_view<'a>(
    state: &State,
    from: Date,
    until: Date,
) -> Column<'a, Message, Renderer<Theme>> {
    let mut meals_and_count = BTreeMap::new();
    for (_, day) in state.days.range(from..until) {
        for meal_id in &day.meals {
            #[allow(clippy::option_if_let_else)]
            if let Some(count) = meals_and_count.get_mut(meal_id) {
                *count += 1.0;
            } else {
                meals_and_count.insert(*meal_id, 1.0);
            }
        }
    }
    let mut list: BTreeMap<&String, (f64, bool)> = BTreeMap::new();
    for (meal_name, count) in meals_and_count {
        if let Some(meal) = state.meals.get(meal_name) {
            for Ingrediant {
                name,
                quantity,
                quantity_input: _,
                unit,
            } in &meal.ingrediants
            {
                let ammount = unit.to_grams(quantity * count);
                #[allow(clippy::option_if_let_else)]
                match list.get_mut(&name) {
                    Some((total, is_liquid)) => {
                        *total += ammount;
                        *is_liquid = *is_liquid && unit.is_liquid()
                    }
                    None => {
                        list.insert(name, (ammount, unit.is_liquid()));
                    }
                };
            }
        }
    }

    col![
        text("Shopping"),
        col(list
            .iter()
            .map(|(name, (ammount, is_liquid))| {
                let (new_ammount, unit) = apropriate_unit(*ammount, *is_liquid);
                text(format!(
                    "{name}: {} {unit}",
                    (new_ammount * 10.).round() / 10.
                ))
                .into()
            })
            .collect())
    ]
}

fn meal_editor_view<'a>(
    state: &'a State,
    meal_id: &'a GenerationalKey,
) -> Column<'a, Message, Renderer<Theme>> {
    state.meals.get(*meal_id).map_or_else(
        || col![Element::<_>::from(text("Meal not found"))],
        |meal| {
            let (delete_buttons, edit_name, edit_quantity, quantity_was_parsed, edit_unit) = meal
                .ingrediants
                .iter()
                .enumerate()
                .map(
                    |(
                        i,
                        Ingrediant {
                            name,
                            quantity,
                            quantity_input,
                            unit,
                        },
                    )| {
                        let (previous_quantity_was_parsed, quantity_input_text) =
                            quantity_input.as_ref().map_or_else(
                                || (false, format!("{quantity}")),
                                |raw_input| {
                                    let parsed_quantity: Option<f64> = raw_input.parse().ok();
                                    (
                                        parsed_quantity.is_some(),
                                        parsed_quantity
                                            .map_or_else(|| raw_input.clone(), |q| format!("{q}")),
                                    )
                                },
                            );

                        (
                            delete_button()
                                .on_press(Message::RemoveMealIngrediant {
                                    meal_name_id: *meal_id,
                                    ingrediant_idx: i,
                                })
                                .into(),
                            text_input(name, name)
                                .on_input(move |s| Message::UpdateMealIngrediant {
                                    meal_name_id: *meal_id,
                                    ingrediant_idx: i,
                                    field: IngrediantField::Name(s),
                                })
                                .into(),
                            text_input("edit quantity", &quantity_input_text)
                                .on_input(move |input| {
                                    let new_quantity = input.parse().ok();
                                    Message::UpdateMealIngrediant {
                                        meal_name_id: *meal_id,
                                        ingrediant_idx: i,
                                        field: IngrediantField::Quantity(new_quantity, input),
                                    }
                                })
                                .into(),
                            if previous_quantity_was_parsed || quantity_input.is_none() {
                                text("Y")
                            } else {
                                text("N")
                            }
                            .into(),
                            pick_list(UNITS, Some(*unit), move |u| Message::UpdateMealIngrediant {
                                meal_name_id: *meal_id,
                                ingrediant_idx: i,
                                field: IngrediantField::Unit(u),
                            })
                            .into(),
                        )
                    },
                )
                .multiunzip();

            col![
                text(
                    state
                        .meals
                        .get(*meal_id)
                        .map_or("Unknown Meal", |meal| &meal.name)
                ),
                row![
                    col(delete_buttons).spacing(5),
                    col(edit_name).spacing(5).width(Length::FillPortion(2)),
                    col(edit_quantity).spacing(5).width(Length::FillPortion(1)),
                    col(quantity_was_parsed).spacing(5),
                    col(edit_unit).spacing(5).width(Length::FillPortion(1))
                ]
                .width(Length::Fill)
                .spacing(10), // .padding(10),
                button("Add").on_press(Message::AddMealIngrediant {
                    meal_id: *meal_id,
                    new_ingrediant_name: "My ingrediant".to_owned(),
                    new_ingrediant_quantity: 0.,
                    new_unit: Unit::Grams,
                })
            ]
            .spacing(10)
        },
    )
}

fn day_view<'a>(state: &State, date: Date) -> Column<'a, Message, Renderer<Theme>> {
    let Some(day )= state.days.get(&date) else {
         return col!["Day not found"]
    };
    let days = day
        .meals
        .iter()
        .map(|id| {
            button(text(
                state
                    .meals
                    .get(*id)
                    .map_or("meal not found", |meal| meal.name.as_str()),
            ))
            .on_press(Message::ChangeToPage(Page::MealEditorView(*id)))
            .into()
        })
        .collect();
    let adder_widget = state.meal_picker_sate.selected_id.map_or_else(
        || {
            row![
                button(text("select: ",)).on_press(Message::ChangeToPage(Page::MealPicker)),
                button("Submit")
            ]
        },
        |id| {
            let meal_name = state
                .meals
                .get(id)
                .map_or("Meal not found", |meal| &meal.name);
            row![
                button(text(format!("selected: {meal_name}")))
                    .on_press(Message::ChangeToPage(Page::MealPicker)),
                button("Submit").on_press(Message::AddMealToDay(date, id))
            ]
        },
    );

    col![
        text(format!("Day {date}")).size(30),
        adder_widget,
        col(days),
    ]
}

fn meal_list_view<'a>(state: &State) -> Column<'a, Message, Renderer<Theme>> {
    let mut list = Vec::with_capacity(state.meals.len());
    for (id, meal) in state.meals.iter() {
        list.push(
            container(
                row![
                    button(text(meal.name.as_str()).size(20)).width(Length::Fill),
                    button(edit_icon()).on_press(Message::ChangeToPage(Page::MealEditorView(id))),
                    delete_button().on_press(Message::RemoveMeal(id))
                ]
                .spacing(5),
            )
            .style(theme::Container::Box)
            .padding(5)
            .into(),
        );
    }
    col![
        col(list).spacing(10),
        row![
            text_input("New meal", &state.meal_creation_input_field)
                .on_input(Message::SetMealCreationInputFeild,)
                .on_submit(Message::AddMeal)
                .id(MEAL_ADDER_INPUT_ID.clone()),
            button("Add").on_press(Message::AddMeal)
        ]
        .spacing(10),
    ]
    .spacing(10)
}

fn week_view<'a>(state: &State, range: &Range<Date>) -> Column<'a, Message, Renderer<Theme>> {
    let mut week_start = range.start;
    let mut weeks = Vec::new();
    let mut week = Vec::new();
    let mut push_week = |week, week_start| {
        weeks.push(
            col![
                text(format!("Week {}", week_start / 7)).size(30),
                container(
                    container(col(week).spacing(5))
                        .style(theme::Container::Box)
                        .padding(5)
                        .width(Length::Fill),
                )
                .padding(20)
                .width(Length::Fill)
            ]
            .into(),
        );
    };
    for date in range.clone() {
        week.push(state.days.get(&date).map_or_else(
            || {
                container(row![
                    text(format!("Day {date}")),
                    button("+").on_press(Message::AddDay(date))
                ])
                .style(theme::Container::Box)
                .into()
            },
            |day| {
                container(
                    button(text(format!(
                        "Day {date}: {} meals addded",
                        day.meals.len()
                    )))
                    .on_press(Message::ChangeToPage(Page::DayView(date))),
                )
                .style(theme::Container::Box)
                .into()
            },
        ));
        if date % 7 == 0 && !week.is_empty() {
            push_week(take(&mut week), week_start);
            week_start = date;
            continue;
        }
    }
    // if !week.is_empty() {
    //     push_week(week);
    // }

    col(weeks).spacing(10)
}

fn meal_picker_view(state: &State) -> Column<'_, Message, Renderer<Theme>> {
    let results: Vec<(_, Element<_>)> = if state.meal_picker_sate.search_results.is_empty() {
        state
            .meals
            .iter()
            .enumerate()
            .map(|(i, (id, meal))| {
                (
                    Some(id),
                    if i == state.meal_picker_sate.selection_index {
                        text(format!("> {}", &meal.name))
                    } else {
                        text(&meal.name)
                    }
                    .into(),
                )
            })
            .collect()
    } else {
        state
            .meal_picker_sate
            .search_results
            .iter()
            .enumerate()
            .filter_map(|(i, result)| {
                higlight_search_result(state, result, i).map(|content| (result, content))
            })
            .map(|(result, row)| {
                (
                    state
                        .meal_picker_sate
                        .searched_meal_ids
                        .get(result.index)
                        .copied(),
                    row.into(),
                )
            })
            .collect()
    };

    let result_buttons = col(results
        .into_iter()
        .map(|(id, ele)| button(ele).on_press(Message::MealPickerSubmit(id)).into())
        .collect());
    //     col::<Message, Renderer<Theme>>(
    //     .collect(),
    // );

    let selected_id = if state.meal_picker_sate.search_results.is_empty() {
        state
            .meals
            .keys()
            .nth(state.meal_picker_sate.selection_index)
    } else {
        state
            .meal_picker_sate
            .search_results
            .get(state.meal_picker_sate.selection_index)
            .and_then(|result| state.meal_picker_sate.searched_meal_ids.get(result.index))
            .and_then(|id| {
                if state.meals.contains_key(*id) {
                    Some(*id)
                } else {
                    None
                }
            })
    };

    col![
        text_input("Search", &state.meal_picker_sate.input_field)
            .on_input(Message::MealPickerInput)
            .on_submit(Message::MealPickerSubmit(selected_id))
            .id(MEAL_PICKER_INPUT_ID.clone()),
        result_buttons
    ]
    .spacing(30)
}

fn higlight_search_result<'a>(
    state: &'a State,
    result: &SearchResultPlus,
    i: usize,
) -> Option<Row<'a, Message, Renderer<Theme>>> {
    let meal_id = state.meal_picker_sate.searched_meal_ids.get(result.index)?;
    let meal_name = &state.meals.get(*meal_id)?.name;
    let mut text_substrings: Vec<Element<Message>> = Vec::new();
    if i == state.meal_picker_sate.selection_index {
        text_substrings.push(text("> ").size(20).into());
    }

    let mut i = 0usize;
    let mut segments: Vec<(Range<usize>, bool)> = vec![];

    result.ranges.iter().for_each(|range| {
        if i < range.start {
            segments.push((i..range.start, false));
        }
        segments.push((range.clone(), true));
        i = range.end;
    });
    if i < meal_name.len() {
        segments.push((i..meal_name.len(), false));
    }

    text_substrings.extend(segments.iter().map(|(range, is_match)| {
        let text_label = text(String::from(&meal_name[range.clone()])).size(20);
        if *is_match {
            text_label.style(Color::from([1.0, 0.2, 0.2])).into()
        } else {
            text_label.into()
        }
    }));

    Some(row(text_substrings))
}

fn bar_view<'a>(state: &'a State) -> Row<'a, Message, Renderer<Theme>> {
    let on_press_and = |button: widget::Button<'a, Message, _>, message, predicate| {
        if predicate {
            button.on_press(message)
        } else {
            button
        }
    };
    row![
        on_press_and(
            header_button("Back", BackButtonStyle),
            Message::BackPage,
            !state.stack.is_empty()
        ),
        col![].width(Length::FillPortion(1)),
        on_press_and(
            header_button("Week", HeaderButtonStyle),
            Message::ChangeToPage(Page::WeekView(1..100)),
            !matches!(state.page, Page::WeekView(_))
        ),
        col![].width(Length::FillPortion(1)),
        on_press_and(
            header_button("List", HeaderButtonStyle),
            Message::ChangeToPage(Page::ShoppingView {
                from: 0,
                until: 100
            }),
            !matches!(state.page, Page::ShoppingView { .. })
        ),
        col![].width(Length::FillPortion(1)),
        on_press_and(
            header_button("Meals", HeaderButtonStyle),
            Message::ChangeToPage(Page::MealList),
            !matches!(state.page, Page::MealList)
        ),
        col![].width(Length::FillPortion(1)),
        header_button("Calender", HeaderButtonStyle),
    ]
    .width(Length::Fill)
}
