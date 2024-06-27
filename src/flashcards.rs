use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length,
    },
    theme, widget, Apply, Element,
};

use crate::{fl, models::Flashcard, utils::select_random_flashcard};

pub struct Flashcards {
    pub current_folder_id: i32,
    pub flashcards: Vec<Flashcard>,
    pub new_edit_flashcard: CreateEditFlashcardState,
    pub currently_studying_flashcard: Flashcard,
    pub currently_studying_flashcard_side: CurrentFlashcardSide,
}

pub struct CreateEditFlashcardState {
    id: Option<i32>,
    front: String,
    back: String,
    status: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Upsert,
    Upserted,
    Load,
    LoadedSingle(Flashcard),
    SetFlashcards(Vec<Flashcard>),
    ToggleCreatePage(Option<Flashcard>),
    StudyFlashcards,
    ContextPageFrontInput(String),
    ContextPageBackInput(String),
    UpdateFlashcardStatus(Flashcard, StudyActions),
    UpdatedStatus(Vec<Flashcard>),
    SwapFlashcardSide,
    Delete(Option<i32>),
}

pub enum Command {
    //The i32 is the Folder Id
    LoadFlashcards(i32),
    ToggleCreateFlashcardPage(Option<Flashcard>),
    UpsertFlashcard(Flashcard),
    OpenStudyFolderFlashcardsPage,
    UpdateFlashcardStatus(Flashcard),
    DeleteFlashcard(Option<i32>),
}

#[derive(Debug, Clone)]
pub enum StudyActions {
    Bad,
    Ok,
    Good,
}

#[derive(Debug, Clone)]
pub enum CurrentFlashcardSide {
    Front,
    Back,
}

impl Flashcards {
    pub fn new() -> Self {
        Self {
            current_folder_id: 0,
            flashcards: Vec::new(),
            currently_studying_flashcard: Flashcard {
                id: None,
                front: String::from("Error"),
                back: String::from("Error"),
                status: 0,
            },
            new_edit_flashcard: CreateEditFlashcardState {
                id: None,
                front: String::new(),
                back: String::new(),
                status: 0,
            },
            currently_studying_flashcard_side: CurrentFlashcardSide::Front,
        }
    }

    pub fn update(&mut self, message: Message) -> Vec<Command> {
        let mut commands = Vec::new();

        match message {
            Message::Upsert => commands.push(Command::UpsertFlashcard(Flashcard {
                id: self.new_edit_flashcard.id,
                front: self.new_edit_flashcard.front.to_string(),
                back: self.new_edit_flashcard.back.to_string(),
                status: self.new_edit_flashcard.status,
            })),
            Message::Upserted => {
                self.new_edit_flashcard = CreateEditFlashcardState {
                    id: None,
                    front: String::new(),
                    back: String::new(),
                    status: 0,
                };
                commands.push(Command::LoadFlashcards(self.current_folder_id))
            }
            Message::LoadedSingle(flashcard) => {
                self.new_edit_flashcard = CreateEditFlashcardState {
                    id: flashcard.id,
                    front: flashcard.front,
                    back: flashcard.back,
                    status: flashcard.status,
                };
            }
            Message::SetFlashcards(flashcards) => self.flashcards = flashcards,
            Message::ToggleCreatePage(flashcard) => {
                if flashcard.is_none() {
                    self.new_edit_flashcard = CreateEditFlashcardState {
                        id: None,
                        front: String::new(),
                        back: String::new(),
                        status: 0,
                    };
                }

                commands.push(Command::ToggleCreateFlashcardPage(flashcard))
            }
            Message::StudyFlashcards => commands.push(Command::OpenStudyFolderFlashcardsPage),
            Message::ContextPageFrontInput(value) => self.new_edit_flashcard.front = value,
            Message::ContextPageBackInput(value) => self.new_edit_flashcard.back = value,
            Message::UpdateFlashcardStatus(mut flashcard, action) => {
                match action {
                    StudyActions::Bad => flashcard.status = 1,
                    StudyActions::Ok => flashcard.status = 2,
                    StudyActions::Good => flashcard.status = 3,
                }

                commands.push(Command::UpdateFlashcardStatus(flashcard))
            }
            Message::UpdatedStatus(flashcards) => {
                self.flashcards = flashcards;
                self.currently_studying_flashcard = select_random_flashcard(&self.flashcards)
                    .unwrap_or(Flashcard {
                        id: None,
                        front: String::from("Error"),
                        back: String::from("Error"),
                        status: 0,
                    });
            }
            Message::SwapFlashcardSide => match self.currently_studying_flashcard_side {
                CurrentFlashcardSide::Front => {
                    self.currently_studying_flashcard_side = CurrentFlashcardSide::Back
                }
                CurrentFlashcardSide::Back => {
                    self.currently_studying_flashcard_side = CurrentFlashcardSide::Front
                }
            },
            Message::Delete(flashcard_id) => commands.push(Command::DeleteFlashcard(flashcard_id)),
            Message::Load => commands.push(Command::LoadFlashcards(self.current_folder_id)),
        }

        commands
    }

    fn flashcard_header_row(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let new_flashcard_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .padding(spacing.space_xxs)
            .on_press(Message::ToggleCreatePage(None));

        let study_button = if self.flashcards.is_empty() == false {
            widget::button(widget::text("Study"))
                .style(theme::Button::Suggested)
                .padding(spacing.space_xxs)
                .on_press(Message::StudyFlashcards)
        } else {
            widget::button(widget::text("Study"))
                .style(theme::Button::Suggested)
                .padding(spacing.space_xxs)
        };

        widget::row::with_capacity(3)
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(widget::text::title3("Flashcards").width(Length::Fill)) //TODO: The Title should be the Folder name
            .push(study_button)
            .push(new_flashcard_button)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        if self.flashcards.is_empty() == false {
            let mut flashcards = widget::list::list_column()
                .style(theme::Container::ContextDrawer)
                .spacing(spacing.space_xxxs)
                .padding([spacing.space_none, spacing.space_xxs]);

            //TODO: Icons & Add Some Kind of Status Badge
            for flashcard in &self.flashcards {
                let edit_button = widget::button(widget::text("Edit"))
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Standard)
                    .on_press(Message::ToggleCreatePage(Some(flashcard.clone())));

                let delete_button = widget::button("Delete")
                    .padding(spacing.space_xxs)
                    .style(theme::Button::Destructive)
                    .on_press(Message::Delete(flashcard.id));

                let flashcard_front = widget::text(flashcard.front.clone())
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Left)
                    .width(Length::Fill);

                let row = widget::row::with_capacity(2)
                    .align_items(Alignment::Center)
                    .spacing(spacing.space_xxs)
                    .padding([spacing.space_xxxs, spacing.space_xxs])
                    .push(flashcard_front)
                    .push(delete_button)
                    .push(edit_button);

                flashcards = flashcards.add(row);
            }

            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.flashcard_header_row())
                .push(flashcards)
                .apply(widget::container)
                .height(Length::Shrink)
                .apply(widget::scrollable)
                .height(Length::Fill)
                .into()
        } else {
            widget::column::with_capacity(2)
                .spacing(spacing.space_xxs)
                .push(self.flashcard_header_row())
                .push(
                    widget::Container::new(widget::Text::new("Empty").size(spacing.space_xl))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(cosmic::iced::alignment::Horizontal::Center)
                        .align_y(cosmic::iced::alignment::Vertical::Center),
                )
                .height(Length::Fill)
                .into()
        }
    }

    /// The create or edit flashcard context page for this app.
    pub fn create_edit_flashcard_contextpage(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::settings::view_column(vec![widget::settings::view_section(fl!(
            "flashcard-details"
        ))
        .add(
            widget::column::with_children(vec![
                widget::text::body(fl!("new-flashcard-front-title")).into(),
                widget::text_input(
                    fl!("new-flashcard-front-inputfield"),
                    &self.new_edit_flashcard.front,
                )
                .on_input(Message::ContextPageFrontInput)
                .into(),
            ])
            .spacing(spacing.space_xxs)
            .padding([0, 15, 0, 15]),
        )
        .add(
            widget::column::with_children(vec![
                widget::text::body(fl!("new-flashcard-back-title")).into(),
                widget::text_input(
                    fl!("new-flashcard-back-inputfield"),
                    &self.new_edit_flashcard.back,
                )
                .on_input(Message::ContextPageBackInput)
                .into(),
            ])
            .spacing(spacing.space_xxs)
            .padding([0, 15, 0, 15]),
        )
        .add(match self.new_edit_flashcard.id {
            Some(_id) => widget::button(
                widget::text(fl!("new-flashcard-edit-button"))
                    .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                    .width(Length::Fill),
            )
            .on_press(Message::Upsert)
            .style(theme::Button::Suggested)
            .padding([10, 0, 10, 0])
            .width(Length::Fill),
            None => widget::button(
                widget::text(fl!("new-flashcard-submit-button"))
                    .horizontal_alignment(cosmic::iced::alignment::Horizontal::Center)
                    .width(Length::Fill),
            )
            .on_press(Message::Upsert)
            .style(theme::Button::Suggested)
            .padding([10, 0, 10, 0])
            .width(Length::Fill),
        })
        .into()])
        .into()
    }

    pub fn view_study_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        //TODO: Remove Button effect on Hover / Beware text size
        let flashcard_container = widget::container(
            widget::button(
                widget::Text::new(match self.currently_studying_flashcard_side {
                    CurrentFlashcardSide::Front => &self.currently_studying_flashcard.front,
                    CurrentFlashcardSide::Back => &self.currently_studying_flashcard.back,
                })
                .size(spacing.space_xxl)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center),
            )
            .on_press(Message::SwapFlashcardSide)
            .style(theme::Button::Text)
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .style(theme::Container::ContextDrawer)
        .width(Length::Fill)
        .height(Length::Fill);

        //TODO: Custom Button Styling
        let options_row = widget::row::with_capacity(3)
            .push(
                widget::button(
                    widget::Text::new("Bad")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Bad,
                ))
                .style(theme::Button::Suggested)
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button(
                    widget::Text::new("Ok")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Ok,
                ))
                .style(theme::Button::Suggested)
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .push(
                widget::button(
                    widget::Text::new("Good")
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .on_press(Message::UpdateFlashcardStatus(
                    self.currently_studying_flashcard.clone(),
                    StudyActions::Good,
                ))
                .style(theme::Button::Suggested)
                .height(Length::Fixed(60.0))
                .width(Length::Fill),
            )
            .align_items(cosmic::iced::Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .width(Length::Fill);

        widget::Column::new()
            .push(flashcard_container)
            .push(options_row)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .into()
    }
}
