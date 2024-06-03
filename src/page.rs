use std::ops::Range;

use iced::{widget, Command, Element};
use serde::{Deserialize, Serialize};

use crate::{day_page, meal_editor, Date, Message, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Page {
    DayView(day_page::DayPage),
    MealList(meal),
    MealEditorView(meal_editor::MealEditorPage),
    ShoppingView { from: Date, until: Date },
    WeekView(Range<Date>),
}

impl Page {
    fn as_any(&self) -> Box<dyn AnyPage> {
        match self {
            Page::DayView(dp) => dp,
            Page::MealList => self,
            Page::MealEditorView(_) => mev,
            Page::ShoppingView { from, until } => todo!(),
            Page::WeekView(_) => todo!(),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::WeekView(1..100)
    }
}

pub trait AnyPage {
    fn view<'a>(&'a self, state: &'a State) -> Element<'a, Message>;

    fn on_tab(&mut self, shift_down: bool) -> Command<Message> {
        if shift_down {
            widget::focus_previous()
        } else {
            widget::focus_next()
        }
    }

    fn vertical_movement(&mut self, _offset: isize) -> Command<Message> {
        Command::none()
    }

    fn close_popup(&mut self) -> Command<Message> {
        Command::none()
    }

    fn open_picker(&mut self, _data: &State) -> Command<Message> {
        Command::none()
    }

    fn on_focus() -> Command<Message> {
        Command::none()
    }
}
