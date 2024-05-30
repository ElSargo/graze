use std::sync::Arc;

use crate::{
    col,
    generational_map::GenerationalMap,
    ingrediant::{Ingrediant, IngrediantKey, IngrediantQuantity},
    meal, row,
    styles::delete_button,
    IngrediantField, Message, State, UNITS,
};

use super::PickerState;

use color_eyre::owo_colors::OwoColorize;
use iced::{
    theme,
    widget::{button, container, pick_list, scrollable, text},
    Command, Element, Length,
};
use itertools::Itertools;
use meal::MealKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealEditorPage {
    pub(crate) meal_id: MealKey,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub ingredaint_picker: Option<PickerState>,
}

impl MealEditorPage {
    pub fn new(id: MealKey) -> Self {
        Self {
            meal_id: id,
            ingredaint_picker: None,
        }
    }

    pub fn close_picker(&mut self) {
        self.ingredaint_picker = None;
    }

    pub fn view<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let Some(meal) = state.meals.get(self.meal_id) else {
            return col![Element::<_>::from(text("Meal not found"))].into();
        };
        let rows = meal
            .ingrediants
            .iter()
            .map(|(ingrediant_id, ingrediant_quantity)| {
                meal_editor_row(state, self.meal_id, *ingrediant_id, ingrediant_quantity)
            });

        let meal_title = text(
            state
                .meals
                .get(self.meal_id)
                .map_or("Unknown Meal", |meal| &meal.name),
        );

        let plus_button = button("Add").on_press(Message::AddMealIngrediant);

        let under_content = col![
            meal_title,
            col(rows).width(Length::Fill).spacing(10),
            plus_button
        ]
        .spacing(10);

        scrollable(
            iced_aw::Modal::new(
                under_content,
                self.ingredaint_picker.as_ref().map(|picker| picker.view()),
            )
            .backdrop(Message::ClosePicker)
            .on_esc(Message::ClosePicker),
        )
        .into()
    }

    pub fn open_ingrediant_picker(
        &mut self,
        ingrediants: &GenerationalMap<Ingrediant>,
    ) -> Command<Message> {
        if !self.ingredaint_picker.is_some() {
            self.ingredaint_picker = Some(PickerState::new(
                ingrediants
                    .iter()
                    .map(|(_key, ing)| ing.name.clone())
                    .collect_vec(),
                Message::IngrediantPickedForMeal,
            ))
        }
        iced::widget::text_input::focus(
            self.ingredaint_picker
                .as_ref()
                .unwrap()
                .input_feild_id
                .clone(),
        )
    }
}

fn meal_editor_row<'a>(
    state: &'a State,
    meal_id: MealKey,
    ingrediant_id: IngrediantKey,
    IngrediantQuantity {
        quantity,
        // ref quantity_input,
        ref unit,
    }: &IngrediantQuantity,
) -> Element<'a, Message> {
    let delete_button = delete_button().on_press(Message::RemoveMealIngrediant {
        meal_id,
        ingrediant_id,
    });

    let ingredaint_name = text(
        state
            .ingrediants
            .get(ingrediant_id)
            .map(|ing| (*ing.name).to_string())
            .unwrap_or("<Select>".to_string()),
    )
    .width(Length::FillPortion(3));

    let quantity_feild = iced_aw::number_input(*quantity, 9999.0, move |input| {
        Message::UpdateMealIngrediant {
            meal_id,
            ingrediant_id,
            field: IngrediantField::Quantity(input),
        }
    })
    .width(Length::FillPortion(2));

    let unit_select = pick_list(UNITS, Some(*unit), move |u| Message::UpdateMealIngrediant {
        meal_id,
        ingrediant_id,
        field: IngrediantField::Unit(u),
    })
    .width(Length::Shrink);
    let inner = row![
        delete_button,
        ingredaint_name,
        // ingredaint_picker_button,
        quantity_feild,
        unit_select
    ]
    .align_items(iced::Alignment::Center)
    .spacing(3)
    .width(Length::Fill)
    .padding(2);
    container(inner)
        .style(theme::Container::Box)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}
