use crate::{
    generational_map::GenerationalKey,
    ingrediant::{IngrediantKey, IngrediantQuantity},
    meal_editor::MealEditorPage,
    styles::{delete_button, edit_icon},
    Page,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meal {
    pub name: Arc<str>,
    pub ingrediants: BTreeMap<IngrediantKey, IngrediantQuantity>,
}
pub type MealKey = GenerationalKey<Meal>;

use iced::{
    theme,
    widget::{button, row, text},
    Element, Length,
};

use iced::widget::container;

use iced::theme::Theme;

use super::Message;

use iced;

pub fn meal_row_view(
    meal: &Meal,
    id: MealKey,
    on_delete: Message,
) -> Element<'_, Message, Theme, iced::Renderer> {
    let label = text(&meal.name).size(20).width(Length::Fill);
    let edit_button = button(edit_icon()).on_press(Message::ChangeToPage(Page::MealEditorView(
        MealEditorPage::new(id),
    )));

    let delete_button = delete_button().on_press(on_delete);

    let content = row![label, edit_button, delete_button]
        .spacing(5)
        .align_items(iced::Alignment::Center);

    container(content)
        .style(theme::Container::Box)
        .padding(5)
        .into()
}
