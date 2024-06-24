// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::core::database::{get_all_studysets, upsert_studyset, OboeteDb};
use crate::models::StudySet;
use crate::studysets::StudySets;
use crate::utils::OboeteError;
use crate::{fl, studysets};
use cosmic::app::{message, Core, Message as CosmicMessage};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Command, Element};

const REPOSITORY: &str = "https://github.com/mariinkys/oboete";

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
    /// StudySets Page
    studysets: StudySets,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    DbConnected(OboeteDb),
    StudySets(studysets::Message),
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

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<CosmicMessage<Self::Message>>) {
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
            studysets: StudySets::new(),
        };

        let commands = vec![
            Command::perform(OboeteDb::init(), |database| {
                message::app(Message::DbConnected(database))
            }),
            app.update_titles(),
        ];

        (app, Command::batch(commands))
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

    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::StudySets => self.studysets.view().map(Message::StudySets),
            Page::AllFlashcards => todo!(),
        };

        widget::Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Self::Message>> {
        let mut commands = vec![];

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

                let command = cosmic::app::Command::perform(
                    get_all_studysets(self.db.clone()),
                    |studysets| todo!(),
                );

                // let command = self.update(Message::StudySets(studysets::Message::StudySetsLoaded(
                //     studysets,
                // )));

                commands.push(command);
            }
            Message::StudySets(message) => {
                let studyset_commands = self.studysets.update(message);
                for studyset_command in studyset_commands {
                    match studyset_command {
                        studysets::Command::CreateStudySet(studyset) => {
                            let command = Command::perform(
                                upsert_studyset(self.db.clone(), studyset),
                                |_result| message::none(),
                            );

                            commands.push(command);
                        }
                    }
                }
            }
        }

        Command::batch(commands)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::NewStudySet => todo!(),
            ContextPage::NewFolder => todo!(),
            ContextPage::NewFlashcard => todo!(),
        })
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<CosmicMessage<Self::Message>> {
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
    pub fn update_titles(&mut self) -> Command<CosmicMessage<Message>> {
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
}
