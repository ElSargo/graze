use crate::{
    generational_map::GenerationalMap, meal::Meal, picker::PickerState, Date, Message, State,
};
use iced::{
    widget::{button, column as col, scrollable, text},
    Command, Element, Length,
};
use iced_aw::{floating_element, modal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct DayPage {
    pub date: Date,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    meal_picker: Option<PickerState>,
}

impl DayPage {
    pub fn new(date: Date) -> Self {
        Self {
            date,
            meal_picker: None,
        }
    }

    pub fn close_picker(&mut self) {
        self.meal_picker = None;
    }

    pub fn open_meal_picker(&mut self, meals: &GenerationalMap<Meal>) -> Command<Message> {
        if !self.meal_picker.is_some() {
            self.meal_picker = Some(PickerState::new(
                meals.iter().map(|(_key, ing)| ing.name.clone()).collect(),
                Message::MealAddedToDay,
            ))
        }
        // iced::widget::text_input::focus(self.meal_picker.as_ref().unwrap().input_feild_id.clone())
        Command::none()
    }

    pub fn view<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let Some(day) = state.days.get(&self.date) else {
            return col!["Day not found"].into();
        };

        let days = day.meals.iter().enumerate().flat_map(|(i, id)| {
            state.meals.get(*id).map(|meal| {
                crate::meal::meal_row_view(
                    meal,
                    *id,
                    Message::RemoveMealFromDay {
                        date: self.date,
                        index: i,
                    },
                )
            })
        });

        let main_content = col![
            text(format!("Day {}", self.date)).size(30),
            // adder_widget,
            col(days).spacing(10),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10);

        let under_picker = floating_element(
            scrollable(main_content),
            button("+").on_press(Message::AddMealToDay),
        )
        .anchor(floating_element::Anchor::SouthEast);

        modal(
            under_picker,
            self.meal_picker.as_ref().map(|picker| picker.view()),
        )
        .on_esc(Message::ClosePicker)
        .backdrop(Message::ClosePicker)
        .into()
    }
}
