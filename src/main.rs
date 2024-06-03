mod day_page;
mod generational_map;
mod page;
use color_eyre::Result;
use day_page::DayPage;
use page::Page;
mod ingrediant;
mod meal;
mod meal_editor;
mod picker;
mod unit;
use crate::picker::PickerState;
use bincode::{deserialize, serialize};
use generational_map::GenerationalMap;
use iced::{
    event,
    keyboard::{self, key::Named, Key},
    theme::{self, Theme},
    widget::{self, button, column as col, container, row, scrollable, text, text_input, Row},
    Application, Command, Element, Event, Length, Settings, Subscription,
};
use ingrediant::{Ingrediant, IngrediantKey, IngrediantQuantity};
use meal::{Meal, MealKey};
use meal_editor::MealEditorPage;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    mem::{replace, take},
    ops::Range,
    sync::Arc,
    time::Duration,
};
use styles::THEME;
use unit::*;
mod styles;
use styles::{header_button, BackButtonStyle, HeaderButtonStyle};

fn main() -> Result<()> {
    color_eyre::install()?;

    Ok(AppState::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(300.0, 650.0),
            ..Default::default()
        },
        ..Default::default()
    })?)
}

static MEAL_ADDER_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

enum AppState {
    Loaded(Box<State>),
    Loading(LoadingState),
}

struct LoadingState {
    app_state: Option<Box<State>>,
    main_font_loaded: bool,
    icon_font_loaded: bool,
}

type Date = usize;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct State {
    page: Page,
    stack: Vec<Page>,
    days: BTreeMap<Date, Day>,
    meals: GenerationalMap<Meal>,
    ingrediants: GenerationalMap<Ingrediant>,
    meal_creation_input_field: String,
    save: SaveState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SaveState {
    saved: bool,
    saving: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ClosePicker,

    MealAddedToDay(Arc<str>, Date),
    AddDay(Date),
    AddMeal,
    AddMealIngrediant,
    AddMealToDay,
    BackPage,
    ChangeToPage(Page),
    AppStateLoaded(Box<State>),
    MainFontLoaded,
    IconFontLoaded,
    MealPickerInput(String),
    // MealPickerSubmit(Option<MealKey>),
    None,
    RemoveMeal(MealKey),
    RemoveMealFromDay {
        date: Date,
        index: usize,
    },
    RemoveMealIngrediant {
        meal_id: MealKey,
        ingrediant_id: IngrediantKey,
    },
    Saved(bool),
    SetMealCreationInputFeild(String),
    TabPressed {
        shift: bool,
    },
    UpdateMealIngrediant {
        meal_id: MealKey,
        ingrediant_id: IngrediantKey,
        field: IngrediantField,
    },
    VerticalMovement(isize),

    IngrediantPickedForMeal(Arc<str>, MealKey),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngrediantField {
    Quantity(f64),
    Unit(Unit),
}

impl Default for IngrediantField {
    fn default() -> Self {
        Self::Unit(Unit::Solid(SolidUnit::Grams))
    }
}

impl State {
    async fn load(path: &str) -> Box<Self> {
        Box::new(
            async_std::fs::read(path)
                .await
                .ok()
                .and_then(|bytes| deserialize(&bytes).ok())
                .unwrap_or_default(),
        )
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
    meals: Vec<MealKey>,
}

impl Application for AppState {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self::Loading(LoadingState {
                app_state: None,
                main_font_loaded: false,
                icon_font_loaded: false,
            }),
            Command::batch([
                Command::perform(State::load("./data"), |state| {
                    Message::AppStateLoaded(state)
                }),
                iced::font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                    .map(|_| Message::IconFontLoaded),
                iced::font::load(include_bytes!(std::env!("MAIN_FONT_PATH")).as_slice())
                    .map(|_| Message::MainFontLoaded),
            ]),
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
            Self::Loading(_) => "Loading".to_owned(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Self::Loading(load_state) => {
                // if let Message::AppStateLoaded(state) = message {
                //     *self = Self::Loaded(state);
                // }

                match message {
                    Message::MainFontLoaded => {
                        load_state.main_font_loaded = true;
                    }
                    Message::IconFontLoaded => {
                        load_state.icon_font_loaded = true;
                    }
                    Message::AppStateLoaded(state) => {
                        load_state.app_state = Some(state);
                    }
                    _ => unreachable!(),
                }

                if let LoadingState {
                    app_state: Some(_),
                    main_font_loaded: true,
                    icon_font_loaded: true,
                } = load_state
                {
                    *self = Self::Loaded(load_state.app_state.take().unwrap());
                }

                Command::none()
            }
            Self::Loaded(state) => update_ui(state, message),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self {
            Self::Loading(_) => text("Loading").into(),
            Self::Loaded(state) => {
                let page: Element<Message> = match &state.page {
                    Page::MealList => meal_list_view(state).into(),
                    Page::DayView(day_page) => day_page.view(state),
                    Page::MealEditorView(page) => page.view(&state),
                    Page::WeekView(range) => week_view(state, range).into(),
                    Page::ShoppingView { from, until } => {
                        shopping_view(state, *from, *until).into()
                    } // Page::MealPicker => meal_picker_view(state),
                };

                col![bar_view(state), page]
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .spacing(10)
                    .padding(10)
                    .into()
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        // self::Theme::custom(theme::Palette {
        //     background: Color::from_rgba(0.157, 0.157, 0.157, 1.0),
        //     text: Color::from_rgba(0.922, 0.859, 0.698, 1.0),
        //     primary: Color::from_rgba(0.271, 0.522, 0.533, 1.0),
        //     success: Color::from_rgba(0.722, 0.733, 0.149, 1.0),
        //     danger: Color::from_rgba(0.984, 0.286, 0.204, 1.0),
        // })
        // self::Theme::Nord

        // Theme::custom(
        //     "Adw".to_owned(),
        //     Palette {
        //         background: Color::from_rgb(32.0 / 256.0, 32.0 / 256.0, 32.0 / 256.0),
        //         text: Color::from_rgb(242.0 / 256.0, 242.0 / 256.0, 242.0 / 256.0),
        //         primary: Color::from_rgb(67.0 / 256.0, 141.0 / 256.0, 230.0 / 256.0),
        //         success: Color::from_rgb(51.0 / 256.0, 209.0 / 256.0, 122.0 / 256.0),
        //         danger: Color::from_rgb(237.0 / 256.0, 51.0 / 256.0, 59.0 / 256.0),
        //     },
        // )
        Theme::Custom(THEME.clone())
    }

    fn subscription(&self) -> Subscription<Message> {
        // let x: SmolStr = SmolStr::new_inline("J");
        event::listen_with(|event, status| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }),
                event::Status::Ignored,
            ) => {
                let key = key.as_ref();
                match key {
                    // key_code: keyboard::KeyCode::Tab,
                    Key::Named(Named::Tab) => Some(Message::TabPressed {
                        shift: modifiers.shift(),
                    }),
                    Key::Named(Named::ArrowDown) | Key::Character("J") => {
                        Some(Message::VerticalMovement(1))
                    }
                    Key::Named(Named::ArrowUp) | Key::Character("K") => {
                        Some(Message::VerticalMovement(-1))
                    }
                    _ => None,
                }
            }

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
            meal_id,
            ingrediant_id,
        } => on_message_remove_meal_ingrediant(state, meal_id, ingrediant_id),
        Message::ChangeToPage(page) => on_message_change_page(page, state),
        Message::AddDay(date) => on_message_add_day(state, date),
        Message::AddMealToDay => on_message_add_meal_to_day(state),
        Message::BackPage => {
            back_page(state);
            Command::none()
        }
        Message::UpdateMealIngrediant {
            meal_id: meal_name_hash,
            ingrediant_id,
            field,
        } => on_message_update_meal_ingrediant(state, meal_name_hash, ingrediant_id, field),
        Message::AddMealIngrediant => on_message_add_meal_ingrediant(state),
        Message::Saved(succes) => {
            state.save.saved = succes;
            state.save.saving = false;
            Command::none()
        }
        Message::SetMealCreationInputFeild(input) => {
            state.meal_creation_input_field = input;
            Command::none()
        }
        // Message::MealPickerSubmit(meal_id) => {
        //     state.picker_state.selected_id = meal_id;
        //     back_page(state);
        //     Command::none()
        // }
        Message::RemoveMeal(id) => {
            state.meals.remove(id);
            Command::none()
        }
        Message::TabPressed { shift } => {
            state.page.as_any().on_tab(shift)
            // if let Some(picker) = active_picker_mut(state) {
            //     picker.fill_input();
            //     Command::none()
            // } else {
            //     if shift {
            //         widget::focus_previous()
            //     } else {
            //         widget::focus_next()
            //     }
            // }
        }
        Message::None => Command::none(),
        Message::RemoveMealFromDay { date, index } => {
            state.days.get_mut(&date).map(|day| day.meals.remove(index));
            Command::none()
        }
        Message::VerticalMovement(movement) => {
            if let Some(picker) = active_picker_mut(state) {
                picker.vertical_movement(movement);
            }
            Command::none()
        } // _ => Command::none(),
        Message::AppStateLoaded(_) | Message::MainFontLoaded | Message::IconFontLoaded => {
            unreachable!()
        }
        Message::IngrediantPickedForMeal(name, meal_id) => {
            on_ingrediant_picked_for_meal(state, name, meal_id)
        }
        Message::ClosePicker => {
            match state.page {
                Page::MealEditorView(ref mut editor) => editor.close_picker(),
                _ => {}
            };
            Command::none()
        }
        Message::MealAddedToDay(name, date) => on_meal_picked_for_date(state, name, date),
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

fn on_ingrediant_picked_for_meal(
    state: &mut State,
    name: Arc<str>,
    meal_id: MealKey,
) -> Command<Message> {
    let Page::MealEditorView(ref mut editor) = state.page else {
        return Command::none();
    };

    let ingredaint_key = state
        .ingrediants
        .iter()
        .find(|(_key, ing)| *name == *ing.name)
        .map(|(key, _)| key);
    let ingredaint_key =
        ingredaint_key.unwrap_or_else(|| state.ingrediants.push(Ingrediant { name: name.into() }));

    let Some(meal) = state.meals.get_mut(meal_id) else {
        return Command::none();
    };

    meal.ingrediants
        .entry(ingredaint_key)
        .or_insert(IngrediantQuantity {
            quantity: 0.0,
            unit: Unit::default(),
        });

    editor.close_picker();

    // id
    Command::none()
}

fn on_meal_picked_for_date(state: &mut State, name: Arc<str>, date: Date) -> Command<Message> {
    let Page::DayView(ref mut editor) = state.page else {
        return Command::none();
    };

    let meal_key = state
        .meals
        .iter()
        .find(|(_key, ing)| *name == *ing.name)
        .map(|(key, _)| key);
    let meal_key = meal_key.unwrap_or_else(|| {
        state.meals.push(Meal {
            name,
            ingrediants: BTreeMap::new(),
        })
    });

    if let Some(day) = state.days.get_mut(&date) {
        day.meals.push(meal_key);
        editor.close_picker();
    }

    // id
    Command::none()
}

fn on_message_add_meal_ingrediant(state: &mut State) -> Command<Message> {
    // if let (Some(meal), Some(ingrediant)) = (
    //     state.meals.get_mut(meal_id),
    //     state
    //         .ingrediants
    //         .iter()
    //         .find(|(_, ing)| ing.name == ingrediant_name)
    //         .map(|(key, _)| key),
    // ) {
    //     meal.ingrediants.push(IngrediantQuantity {
    //         id: ingrediant,
    //         quantity: 0.0,
    //         unit: Unit::Solid(SolidUnit::Grams),
    //     });
    // }

    match state.page {
        Page::MealEditorView(ref mut meal_editor) => {
            meal_editor.open_ingrediant_picker(&state.ingrediants)
        }

        _ => Command::none(),
    }
}

fn on_message_add_meal_to_day(state: &mut State) -> Command<Message> {
    println!("Open sessemy");
    match state.page {
        Page::DayView(ref mut page) => page.open_meal_picker(&state.meals),
        _ => unreachable!(),
    }
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
        // Page::MealPicker => iced::widget::text_input::focus(MEAL_PICKER_INPUT_ID.clone()),
        Page::MealEditorView(MealEditorPage {
            meal_id: _,
            ingredaint_picker: Some(ref picker),
        }) => iced::widget::text_input::focus(picker.input_feild_id.clone()),
        _ => Command::none(),
    };
    state.stack.push(replace(&mut state.page, page));
    command
}

fn on_message_remove_meal_ingrediant(
    state: &mut State,
    meal_name_hash: MealKey,
    ingrediant_id: IngrediantKey,
) -> Command<Message> {
    if let Some(meal) = state.meals.get_mut(meal_name_hash) {
        meal.ingrediants.remove(&ingrediant_id);
    }
    Command::none()
}

fn on_message_add_meal(state: &mut State) -> Command<Message> {
    state.meals.push(Meal {
        name: state.meal_creation_input_field.clone().into(),
        ingrediants: BTreeMap::new(),
    });
    state.meal_creation_input_field = String::new();
    Command::none()
}

fn on_message_meal_picker_input(state: &mut State, input: String) -> Command<Message> {
    if let Some(picker) = active_picker_mut(state) {
        picker.input(input);
    }

    Command::none()
}

fn on_message_update_meal_ingrediant(
    state: &mut State,
    meal_id: MealKey,
    ingrediant_id: IngrediantKey,
    field: IngrediantField,
) -> Command<Message> {
    if let Some(meal) = state.meals.get_mut(meal_id) {
        if let Some(IngrediantQuantity {
            ref mut quantity,
            // quantity_input,
            ref mut unit,
        }) = meal.ingrediants.get_mut(&ingrediant_id)
        {
            match field {
                IngrediantField::Quantity(new_quantity) => {
                    *quantity = new_quantity;
                }
                IngrediantField::Unit(new_unit) => {
                    // *quantity =
                    *unit = new_unit;
                    // *quantity_input = None;
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

fn shopping_view<'a>(state: &State, from: Date, until: Date) -> Element<'a, Message> {
    let mut meals_and_count = BTreeMap::new();
    for (_, day) in state.days.range(from..until) {
        for meal_id in &day.meals {
            if let Some(count) = meals_and_count.get_mut(meal_id) {
                *count += 1.0;
            } else {
                meals_and_count.insert(*meal_id, 1.0);
            }
        }
    }
    let mut list: BTreeMap<IngrediantKey, (f64, Unit)> = BTreeMap::new();
    for (meal_name, count) in meals_and_count {
        let Some(meal) = state.meals.get(meal_name) else {
            continue;
        };

        for (ingrediant_id, IngrediantQuantity { quantity, unit }) in meal.ingrediants.iter() {
            let to_merge = list.entry(*ingrediant_id).or_insert((0.0, Unit::default()));
            merge_into_unit(to_merge, *quantity, *unit);
        }
    }

    let header = text("Shopping").size(30);

    let format_list_item = |(id, (ammount, unit)): (IngrediantKey, (f64, Unit))| {
        state.ingrediants.get(id).map(|ingrediant| {
            let content = format!(
                "{}: {} {}",
                ingrediant.name,
                (ammount * 10.).round() / 10.,
                unit.abreviation()
            );
            text(content).into()
        })
    };
    let ingrediant_list = col(list.into_iter().flat_map(format_list_item));
    scrollable(col![header, ingrediant_list]).into()
}

fn merge_into_unit(to_merge: &mut (f64, Unit), quantity: f64, unit: Unit) {
    todo!()
}

fn meal_list_view<'a>(state: &'a State) -> Element<'a, Message> {
    let mut list = Vec::with_capacity(state.meals.len());
    for (id, meal) in state.meals.iter() {
        list.push(meal::meal_row_view(meal, id, Message::RemoveMeal(id)));
    }
    scrollable(
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
        .spacing(10),
    )
    .into()
}

fn week_view<'a>(state: &State, range: &Range<Date>) -> Element<'a, Message> {
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
                    col![].width(Length::Fill),
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
                    .on_press(Message::ChangeToPage(Page::DayView(DayPage::new(date)))),
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

    scrollable(col(weeks).spacing(10)).into()
}

fn bar_view<'a>(state: &'a State) -> Row<'a, Message> {
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
