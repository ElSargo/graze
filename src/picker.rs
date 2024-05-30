use iced::Padding;
use iced_aw::helpers::card;
use crate::col;
use fuse_rust::{Fuse, SearchResult as SR};
use iced::widget::button;
use iced::widget::text_input;
use iced::{
    widget::{row, text, Row},
    Color, Element,
};
use serde::{Deserialize, Serialize};
use std::{ops::Range, sync::Arc};

use crate::Message;

#[derive(Clone)]
pub struct PickerState {
    on_pick: fn(Arc<str>) -> Message,
    input_field: String,
    search_results: Vec<SearchResult>,
    selection_index: usize,
    search_feilds: Vec<Arc<str>>,
    pub input_feild_id: text_input::Id
}

impl PickerState {
    pub fn view(&self) -> Element<'_, Message> {
        let results: Vec<Element<_>> = if self.search_results.is_empty() {
            self.search_feilds
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    if i == self.selection_index {
                        text(format!("> {}", &name))
                    } else {
                        text(&name)
                    }
                    .into()
                })
                .collect()
        } else {
            self.search_results
                .iter()
                .enumerate()
                .filter_map(|(i, result)| {
                    self.higlight_search_result(result, i)
                        .map(|content|  content)
                })
                .map(| row| row.into())
                .collect()
        };

        let result_buttons = col(results.into_iter().zip(self.search_feilds.iter()).map(
            |(ele,name)| button(ele).on_press((self.on_pick)(name.clone())).into()
        ));

        let selected_result = 
        // self
            // .search_feilds
            // .get(self.selection_index)
            // .cloned()
            // .unwrap_or_else(|| 
                self.input_field.clone().into()
                    // .into())
        ;

        let input_feild = 
            text_input("Search", &self.input_field)
                .on_input(Message::MealPickerInput)
                .on_submit((self.on_pick)(selected_result))
                .id( self.input_feild_id.clone() );

        
        let content = col![
            input_feild,
            result_buttons
        ]
        .spacing(30);

        card(text("Picker"),content)
            .padding(Padding::new(20.0))
                    .foot(text::Text::new("Foot"))
                    .style(iced_aw::CardStyles::Primary)
                    // .on_close(Message::CloseCard)
            // .style(theme::Container::Box)
            .into()
    }

    fn higlight_search_result<'a>(
        &self,
        result: &SearchResult,
        i: usize,
    ) -> Option<Row<'a, Message>> {
        let meal_name = self.search_feilds.get(result.index)?;
        let mut text_substrings: Vec<Element<Message>> = Vec::new();
        if i == self.selection_index {
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

    pub fn new(search_feilds: Vec<Arc<str>>, on_pick: fn(Arc<str>) -> Message) -> Self {
        Self {
            on_pick,
            input_field: String::new(),

            search_results: Vec::new(),
            selection_index: 0,
            search_feilds,
            input_feild_id: text_input::Id::unique(),
        }
    }

    pub fn input(&mut self, input: String) {
        self.input_field = input;
        self.selection_index = 0;
        let searcher = Fuse::default();

        self.search_results = if self.input_field.is_empty() {
            Vec::new()
        } else {
            // self.searched_ids = state.meals.values().map(|meal| meal.name).collect();
            searcher
                .search_text_in_iterable(&self.input_field, &self.search_feilds)
                .into_iter()
                .map(std::convert::Into::into)
                .collect()
        };
    }

    pub fn vertical_movement(&mut self, offset: isize) {
        let new_index = self.selection_index.saturating_add_signed(offset).min(
            if self.search_results.is_empty() {
                self.search_feilds.len()
            } else {
                self.search_results.len()
            }
            .saturating_sub(1),
        );

        self.selection_index = new_index;
    }

    pub fn fill_input(&mut self) {
        
       if let Some(text)  =  self.search_results.get(self.selection_index).and_then(|srp|  self.search_feilds.get(srp.index)){
           self.input_field = text.to_string();
       }
    }
}

impl std::fmt::Debug for PickerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "App state")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResult {
    /// corresponding index of the search result in the original list
    pub index: usize,
    /// Search rating of the search result, 0.0 is a perfect match 1.0 is a perfect mismatch
    pub score: f64,
    /// Ranges of matches in the search query, useful if you want to hightlight matches.
    pub ranges: Vec<Range<usize>>,
}

impl From<SR> for SearchResult {
    fn from(value: SR) -> Self {
        Self {
            index: value.index,
            score: value.score,
            ranges: value.ranges,
        }
    }
}
