// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::core::database::{get_all_studysets, upsert_studyset, OboeteDb};
use crate::fl;
use crate::models::StudySet;
use crate::utils::OboeteError;
use cosmic::app::{Command, Core};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Element};

const REPOSITORY: &str = "https://github.com/mariinkys/oboete";
const STUDYSETS_PER_ROW: usize = 5;

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct Oboete {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// A model that contains all of the pages assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Currently selected Page
    current_page: Page,
    /// Database of the application
    db: Option<OboeteDb>,
    /// Application State Holder
    state: OboeteState,
}

pub struct OboeteState {
    studysets: Vec<StudySet>,
    new_studyset: NewStudySetState,
}

impl Default for OboeteState {
    fn default() -> Self {
        Self {
            studysets: Default::default(),
            new_studyset: NewStudySetState {
                name: String::from(""),
            },
        }
    }
}

pub struct NewStudySetState {
    name: String,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    DbConnected(OboeteDb),
    LoadedStudySets(Result<Vec<StudySet>, OboeteError>),
    NewStudySetNameInput(String),
    CreateStudySet,
    StudySetCreated,
}

/// Identifies a page in the application.
pub enum Page {
    StudySets,
    AllFlashcards,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    NewStudySet,
    NewFolder,
    NewFlashcard,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::NewStudySet => fl!("new-studyset"),
            Self::NewFolder => fl!("new-folder"),
            Self::NewFlashcard => fl!("new-flashcard"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for Oboete {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.mariinkys.Oboete";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Instructs the cosmic runtime to use this model as the nav bar model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("Study Sets")
            .data::<Page>(Page::StudySets)
            .icon(icon::from_name("applications-science-symbolic"))
            .activate();

        nav.insert()
            .text("All Flashcards")
            .data::<Page>(Page::AllFlashcards)
            .icon(icon::from_name("applications-system-symbolic"));

        let mut app = Oboete {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            nav,
            current_page: Page::StudySets,
            db: None,
            state: OboeteState::default(),
        };

        let cmd = cosmic::app::Command::perform(OboeteDb::init(), |database| {
            cosmic::app::message::app(Message::DbConnected(database))
        });
        let commands = Command::batch(vec![app.update_titles(), cmd]);

        (app, commands)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::StudySets => self.studysets(),
            Page::AllFlashcards => self.all_flashcards(),
        };

        widget::Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                // Set the title of the context drawer.
                self.set_context_title(context_page.title());
            }
            Message::DbConnected(db) => {
                self.db = Some(db);
                //TODO: How to not clone the DB for every operation
                // return cosmic::app::Command::perform(
                //     get_all_studysets(&self.db),
                //     |studysets| cosmic::app::message::app(Message::LoadedStudySets(studysets)),
                // );
                // borrowed data escapes outside of method argument requires that `'1` must outlive `'static`
                // app.rs(181, 15): `self` is a reference that is only valid in the method body
                // app.rs(181, 15): let's call the lifetime of this reference `'1`

                return cosmic::app::Command::perform(
                    get_all_studysets(self.db.clone()),
                    |studysets| cosmic::app::message::app(Message::LoadedStudySets(studysets)),
                );
            }
            Message::LoadedStudySets(studysets) => match studysets {
                Ok(studysets) => self.state.studysets = studysets,
                Err(_) => self.state.studysets = Vec::new(),
            },
            Message::NewStudySetNameInput(value) => self.state.new_studyset.name = value,
            Message::CreateStudySet => {
                return cosmic::app::Command::perform(
                    upsert_studyset(
                        self.db.clone(),
                        StudySet {
                            id: None,
                            name: self.state.new_studyset.name.to_string(),
                            folders: Vec::new(),
                        },
                    ),
                    |_result| cosmic::app::message::app(Message::StudySetCreated),
                );
            }
            Message::StudySetCreated => {
                self.core.window.show_context = false;
                self.state.new_studyset.name = String::new();
                return cosmic::app::Command::perform(
                    get_all_studysets(self.db.clone()),
                    |studysets| cosmic::app::message::app(Message::LoadedStudySets(studysets)),
                );
            }
        }
        Command::none()
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::NewStudySet => self.new_studyset(),
            ContextPage::NewFolder => todo!(),
            ContextPage::NewFlashcard => todo!(),
        })
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        // Activate the page in the model.
        self.nav.activate(id);

        //Update the current page
        let current_page: Option<&Page> = self.nav.active_data();
        match current_page {
            Some(page) => match page {
                Page::StudySets => self.current_page = Page::StudySets,
                Page::AllFlashcards => self.current_page = Page::AllFlashcards,
            },
            None => self.current_page = Page::StudySets,
        }

        self.update_titles()
    }
}

impl Oboete {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/128x128/apps/dev.mariinkys.Oboete.svg")[..],
        ));

        let title = widget::text::title3(fl!("app-title"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_titles(&mut self) -> Command<Message> {
        let mut window_title = fl!("app-title");
        let mut header_title = String::new();

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
            header_title.push_str(page);
        }

        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }

    /// The studysets page for this app.
    pub fn studysets(&self) -> Element<Message> {
        let mut studysets_grid = widget::Grid::new().width(Length::Fill);

        for (index, studyset) in self.state.studysets.iter().enumerate() {
            let studyset_button =
                widget::button(widget::text(studyset.name.as_str())).style(theme::Button::Text);

            if index % STUDYSETS_PER_ROW == 0 {
                studysets_grid = studysets_grid.insert_row();
            }

            studysets_grid = studysets_grid.push(studyset_button);
        }

        let new_studyset_button = widget::button(widget::text("New"))
            .style(theme::Button::Suggested)
            .on_press(Message::ToggleContextPage(ContextPage::NewStudySet));
        let header_row = widget::Row::new()
            .push(new_studyset_button)
            .width(Length::Fill);

        widget::Column::new()
            .push(header_row)
            .push(studysets_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The new studyset context page for this app.
    pub fn new_studyset(&self) -> Element<Message> {
        let new_studyset_name_inputfield = widget::TextInput::new(
            fl!("new-studyset-name-inputfield"),
            &self.state.new_studyset.name,
        )
        .on_input(Message::NewStudySetNameInput);

        let submit_button = widget::button(widget::text(fl!("new-studyset-submit-button")))
            .on_press(Message::CreateStudySet)
            .style(theme::Button::Suggested);

        widget::Column::new()
            .push(new_studyset_name_inputfield)
            .push(submit_button)
            .width(Length::Fill)
            .into()
    }

    /// The flashcards page for this app.
    pub fn all_flashcards(&self) -> Element<Message> {
        // let mut flashcards_grid = widget::Grid::new().width(Length::Fill);

        // for (index, flashcard) in self.flashcards.iter().enumerate() {
        //     let flashcard_button =
        //         widget::button(widget::text(flashcard.front.as_str())).style(theme::Button::Text);

        //     if index % FLASHCARDS_PER_ROW == 0 {
        //         flashcards_grid = flashcards_grid.insert_row();
        //     }

        //     flashcards_grid = flashcards_grid.push(flashcard_button);
        // }

        let study_button = widget::button(widget::text("Study")).style(theme::Button::Suggested);
        let header_row = widget::Row::new().push(study_button).width(Length::Fill);

        widget::Column::new()
            .push(header_row)
            //.push(studysets_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
