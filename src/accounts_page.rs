//! "Rclone accounts" page: list, create (Google Drive), reconnect and remove
//! remotes. Every rclone process runs on `AccountRunner`'s thread; this page
//! only polls for results on the GTK main loop.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use drivemirror::{
    account_runner::{AccountResult, AccountRunner},
    accounts::{
        self, ConfigQuestion, ContinueRequest, CreateDriveRequest, Remote, delete_command,
        list_remotes, list_remotes_command, reconnect_command,
    },
    i18n::Message,
};
use gtk::{glib, prelude::*};

use crate::app::{Ui, t, translated_button, translated_label};

const CLIENT_ID_HELP: &str = "https://rclone.org/drive/#making-your-own-client-id";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Job {
    List,
    Configure,
    Delete,
}

struct Page {
    ui: Rc<Ui>,
    window: gtk::Window,
    runner: AccountRunner,
    job: Cell<Job>,
    /// Remote the running configure/delete job acts on.
    target: RefCell<String>,
    /// The creation request, kept until the flow ends: rclone needs it on every continuation.
    creating: RefCell<Option<CreateDriveRequest>>,
    /// A remote whose creation did not finish; offered for removal while it is listed.
    incomplete: RefCell<Option<String>>,
    question: RefCell<Option<ConfigQuestion>>,
    remotes: RefCell<Vec<Remote>>,
    list: gtk::ListBox,
    name: gtk::Entry,
    client_id: gtk::Entry,
    secret: gtk::PasswordEntry,
    question_box: gtk::Box,
    question_help: gtk::Label,
    examples: gtk::DropDown,
    answer: gtk::Entry,
    secret_answer: gtk::PasswordEntry,
    progress: gtk::Box,
    spinner: gtk::Spinner,
    progress_label: gtk::Label,
    auth_link: gtk::LinkButton,
    remove_incomplete: gtk::Button,
    status: gtk::Label,
}

pub(crate) fn build(ui: &Rc<Ui>, window: &gtk::Window) -> gtk::Box {
    let page = Rc::new(Page {
        ui: ui.clone(),
        window: window.clone(),
        runner: AccountRunner::default(),
        job: Cell::new(Job::List),
        target: RefCell::default(),
        creating: RefCell::default(),
        incomplete: RefCell::default(),
        question: RefCell::default(),
        remotes: RefCell::default(),
        list: gtk::ListBox::new(),
        name: gtk::Entry::builder().placeholder_text("gdrive").build(),
        client_id: gtk::Entry::builder().hexpand(true).build(),
        secret: gtk::PasswordEntry::builder().show_peek_icon(true).build(),
        question_box: gtk::Box::new(gtk::Orientation::Vertical, 8),
        question_help: gtk::Label::builder()
            .wrap(true)
            .xalign(0.0)
            .selectable(true)
            .build(),
        examples: gtk::DropDown::from_strings(&[]),
        answer: gtk::Entry::new(),
        secret_answer: gtk::PasswordEntry::builder().show_peek_icon(true).build(),
        progress: gtk::Box::new(gtk::Orientation::Horizontal, 8),
        spinner: gtk::Spinner::new(),
        progress_label: gtk::Label::new(None),
        auth_link: gtk::LinkButton::new(CLIENT_ID_HELP),
        remove_incomplete: gtk::Button::new(),
        status: gtk::Label::builder().wrap(true).xalign(0.0).build(),
    });
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.set_margin_top(12);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let heading = translated_label(ui, Message::ConfiguredRemotes);
    heading.set_hexpand(true);
    heading.set_xalign(0.0);
    heading.add_css_class("heading");
    let refresh = translated_button(ui, Message::Refresh);
    header.append(&heading);
    header.append(&refresh);
    root.append(&header);
    page.list.set_selection_mode(gtk::SelectionMode::None);
    page.list.add_css_class("boxed-list");
    root.append(&page.list);
    page.remove_incomplete.set_visible(false);
    page.remove_incomplete.add_css_class("destructive-action");
    page.remove_incomplete.set_halign(gtk::Align::Start);
    root.append(&page.remove_incomplete);

    let form_title = translated_label(ui, Message::NewDriveRemote);
    form_title.set_xalign(0.0);
    form_title.add_css_class("heading");
    root.append(&form_title);
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(10)
        .build();
    for (row, (message, widget)) in [
        (
            Message::RemoteNameLabel,
            page.name.upcast_ref::<gtk::Widget>(),
        ),
        (Message::ClientIdLabel, page.client_id.upcast_ref()),
        (Message::ClientSecretLabel, page.secret.upcast_ref()),
    ]
    .into_iter()
    .enumerate()
    {
        let label = translated_label(ui, message);
        label.set_xalign(0.0);
        grid.attach(&label, 0, row as i32, 1, 1);
        grid.attach(widget, 1, row as i32, 1, 1);
    }
    root.append(&grid);
    let form_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let create = translated_button(ui, Message::CreateRemote);
    create.add_css_class("suggested-action");
    let help = gtk::LinkButton::new(CLIENT_ID_HELP);
    crate::app::register(ui, &help, Message::ClientIdHelp);
    form_actions.append(&create);
    form_actions.append(&help);
    root.append(&form_actions);

    page.question_box.set_visible(false);
    page.question_box.append(&page.question_help);
    let options_label = translated_label(ui, Message::Options);
    options_label.set_xalign(0.0);
    page.question_box.append(&options_label);
    page.question_box.append(&page.examples);
    page.question_box.append(&page.answer);
    page.question_box.append(&page.secret_answer);
    let question_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let answer = translated_button(ui, Message::ContinueSetup);
    answer.add_css_class("suggested-action");
    let abandon = translated_button(ui, Message::CancelSetup);
    question_actions.append(&answer);
    question_actions.append(&abandon);
    page.question_box.append(&question_actions);
    root.append(&page.question_box);

    page.progress.set_visible(false);
    let cancel = translated_button(ui, Message::Cancel);
    page.progress.append(&page.spinner);
    page.progress.append(&page.progress_label);
    page.auth_link.set_visible(false);
    crate::app::register(ui, &page.auth_link, Message::OpenAuthLink);
    page.progress.append(&page.auth_link);
    page.progress.append(&cancel);
    root.append(&page.progress);
    root.append(&page.status);

    refresh.connect_clicked(glib::clone!(
        #[strong]
        page,
        move |_| refresh_list(&page)
    ));
    create.connect_clicked(glib::clone!(
        #[strong]
        page,
        move |_| start_create(&page)
    ));
    answer.connect_clicked(glib::clone!(
        #[strong]
        page,
        move |_| send_answer(&page)
    ));
    abandon.connect_clicked(glib::clone!(
        #[strong]
        page,
        move |_| {
            page.question.replace(None);
            mark_incomplete(&page);
            show_question(&page);
        }
    ));
    cancel.connect_clicked(glib::clone!(
        #[strong]
        page,
        move |_| page.runner.cancel()
    ));
    page.remove_incomplete.connect_clicked(glib::clone!(
        #[strong]
        page,
        move |_| {
            if let Some(name) = page.incomplete.borrow().clone() {
                confirm_remove(&page, name);
            }
        }
    ));
    page.examples.connect_selected_notify(glib::clone!(
        #[strong]
        page,
        move |examples| {
            // Items are in the same order as the question's examples.
            let index = examples.selected() as usize;
            if let Some(example) = page
                .question
                .borrow()
                .as_ref()
                .and_then(|question| question.option.as_ref())
                .and_then(|option| option.examples.get(index))
            {
                page.answer.set_text(&example.value);
            }
        }
    ));
    glib::timeout_add_local(
        Duration::from_millis(200),
        glib::clone!(
            #[strong]
            page,
            move || {
                // rclone prints the OAuth URL while waiting; offer it in case no browser opened.
                let url = page.runner.auth_url();
                page.auth_link.set_visible(url.is_some());
                if let Some(url) = url
                    && page.auth_link.uri() != url
                {
                    page.auth_link.set_uri(&url);
                    page.auth_link.set_tooltip_text(Some(&url));
                }
                if let Some(result) = page.runner.take_result() {
                    finish(&page, result);
                }
                glib::ControlFlow::Continue
            }
        ),
    );
    refresh_list(&page);
    root
}

fn start(page: &Page, job: Job, command: accounts::AccountCommand, progress: Message) -> bool {
    match page
        .runner
        .start(command, crate::app::selected_language(&page.ui))
    {
        Ok(()) => {
            page.job.set(job);
            page.progress_label.set_text(t(&page.ui, progress));
            page.progress.set_visible(true);
            page.spinner.start();
            page.question_box.set_visible(false);
            true
        }
        Err(error) => {
            page.status.set_text(&error);
            false
        }
    }
}

fn refresh_list(page: &Page) {
    start(
        page,
        Job::List,
        list_remotes_command(),
        Message::RunningRclone,
    );
}

fn start_create(page: &Page) {
    let name = page.name.text().trim().to_string();
    if accounts::validate_remote_name(&name).is_err() {
        page.status
            .set_text(t(&page.ui, Message::InvalidRemoteName));
        return;
    }
    if page
        .remotes
        .borrow()
        .iter()
        .any(|remote| remote.name == name)
    {
        page.status
            .set_text(&t(&page.ui, Message::RemoteExists).replace("{name}", &name));
        return;
    }
    let request = CreateDriveRequest {
        name: name.clone(),
        client_id: page.client_id.text().trim().to_string(),
        client_secret: page.secret.text().to_string(),
    };
    let Ok(command) = request.command() else {
        page.status
            .set_text(t(&page.ui, Message::MissingCredentials));
        return;
    };
    if start(page, Job::Configure, command, Message::RunningRclone) {
        page.secret.set_text("");
        page.question.replace(None);
        page.target.replace(name);
        page.creating.replace(Some(request));
    }
}

fn reconnect(page: &Page, name: String) {
    if let Ok(command) = reconnect_command(&name)
        && start(page, Job::Configure, command, Message::RunningRclone)
    {
        page.question.replace(None);
        page.creating.replace(None);
        page.target.replace(name);
    }
}

fn send_answer(page: &Page) {
    let Some(question) = page.question.borrow().clone() else {
        return;
    };
    let option = question.option.as_ref();
    let is_password = option.is_some_and(|option| option.is_password);
    let value = if is_password {
        page.secret_answer.text().to_string()
    } else {
        page.answer.text().trim().to_string()
    };
    let name = page.target.borrow().clone();
    let command =
        ContinueRequest::answer(&name, page.creating.borrow().as_ref(), &question, &value)
            .and_then(|request| request.command());
    let progress =
        if option.is_some_and(|option| option.name == "config_is_local") && value == "true" {
            Message::WaitingBrowser
        } else {
            Message::RunningRclone
        };
    match command {
        Ok(command) => {
            if start(page, Job::Configure, command, progress) {
                page.secret_answer.set_text("");
            }
        }
        Err(_) => page
            .status
            .set_text(t(&page.ui, Message::MalformedResponse)),
    }
}

fn finish(page: &Rc<Page>, result: AccountResult) {
    page.spinner.stop();
    page.progress.set_visible(false);
    let job = page.job.get();
    let name = page.target.borrow().clone();
    match result {
        AccountResult::Cancelled => {
            page.status
                .set_text(t(&page.ui, Message::OperationCancelled));
            if job == Job::Configure {
                mark_incomplete(page);
            }
        }
        AccountResult::Failed(message) => page.status.set_text(&message),
        AccountResult::Finished(output) if !output.success => {
            let detail = if output.stderr.is_empty() {
                output.stdout
            } else {
                output.stderr
            };
            // The last lines carry rclone's actual error; earlier ones are progress logs.
            let lines: Vec<&str> = detail.lines().collect();
            let detail = lines[lines.len().saturating_sub(3)..].join("\n");
            page.status
                .set_text(&t(&page.ui, Message::RcloneFailed).replace("{error}", &detail));
            if job == Job::Configure {
                mark_incomplete(page);
            }
        }
        AccountResult::Finished(output) => match job {
            Job::List => show_remotes(page, list_remotes(&output.stdout)),
            Job::Delete => {
                page.status
                    .set_text(&t(&page.ui, Message::RemoteRemoved).replace("{name}", &name));
                if page.ui.remote.text() == name {
                    page.ui.remote.set_text("");
                }
                if page.incomplete.borrow().as_deref() == Some(name.as_str()) {
                    page.incomplete.replace(None);
                }
                refresh_list(page);
            }
            Job::Configure => match ConfigQuestion::parse(&output.stdout) {
                Err(_) => {
                    page.status
                        .set_text(t(&page.ui, Message::MalformedResponse));
                    mark_incomplete(page);
                }
                Ok(question) if question.is_done() && question.error.is_empty() => {
                    page.question.replace(None);
                    page.creating.replace(None);
                    page.incomplete.replace(None);
                    page.status
                        .set_text(&t(&page.ui, Message::RemoteReady).replace("{name}", &name));
                    if page.ui.remote.text().trim().is_empty() {
                        page.ui.remote.set_text(&name);
                    }
                    refresh_list(page);
                }
                Ok(question) => {
                    if question.error.is_empty() {
                        page.status.set_text("");
                    } else {
                        page.status.set_text(
                            &t(&page.ui, Message::RcloneFailed).replace("{error}", &question.error),
                        );
                    }
                    if question.is_done() {
                        mark_incomplete(page);
                    } else {
                        page.question.replace(Some(question));
                    }
                }
            },
        },
    }
    show_question(page);
}

/// A creation that stops before `is_done` may leave a remote without a token.
fn mark_incomplete(page: &Page) {
    if let Some(request) = page.creating.take() {
        page.question.replace(None);
        page.incomplete.replace(Some(request.name));
        refresh_list(page);
    }
}

fn show_question(page: &Page) {
    let question = page.question.borrow();
    let Some(option) = question
        .as_ref()
        .and_then(|question| question.option.as_ref())
    else {
        page.question_box
            .set_visible(question.is_some() && !page.runner.is_running());
        page.question_help.set_text("");
        return;
    };
    page.question_help.set_text(if option.help.is_empty() {
        &option.name
    } else {
        &option.help
    });
    let items: Vec<String> = option
        .examples
        .iter()
        .map(|example| {
            if example.help.is_empty() {
                example.value.clone()
            } else {
                format!("{} — {}", example.value, example.help)
            }
        })
        .collect();
    let items: Vec<&str> = items.iter().map(String::as_str).collect();
    page.examples.set_model(Some(&gtk::StringList::new(&items)));
    page.examples.set_visible(!items.is_empty());
    page.answer.set_visible(!option.is_password);
    page.secret_answer.set_visible(option.is_password);
    page.answer.set_text(&option.default);
    if let Some(index) = option
        .examples
        .iter()
        .position(|example| example.value == option.default)
    {
        page.examples.set_selected(index as u32);
    }
    page.question_box.set_visible(!page.runner.is_running());
}

fn show_remotes(page: &Rc<Page>, remotes: Vec<Remote>) {
    while let Some(row) = page.list.first_child() {
        page.list.remove(&row);
    }
    if remotes.is_empty() {
        let empty = translated_label(&page.ui, Message::NoRemotes);
        empty.set_margin_top(8);
        empty.set_margin_bottom(8);
        page.list.append(&empty);
    }
    for remote in &remotes {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_top(6);
        row.set_margin_bottom(6);
        row.set_margin_start(8);
        row.set_margin_end(8);
        let label = gtk::Label::new(Some(&format!("{} ({})", remote.name, remote.kind)));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        row.append(&label);
        // Only Google Drive remotes are managed here; others are listed read-only.
        if remote.kind == "drive" {
            let reconnect_button = translated_button(&page.ui, Message::Reconnect);
            let name = remote.name.clone();
            reconnect_button.connect_clicked(glib::clone!(
                #[strong]
                page,
                move |_| reconnect(&page, name.clone())
            ));
            row.append(&reconnect_button);
        }
        let remove = translated_button(&page.ui, Message::Remove);
        remove.add_css_class("destructive-action");
        let name = remote.name.clone();
        remove.connect_clicked(glib::clone!(
            #[strong]
            page,
            move |_| confirm_remove(&page, name.clone())
        ));
        row.append(&remove);
        page.list.append(&row);
    }
    let incomplete = page.incomplete.borrow().clone();
    let listed = incomplete
        .as_ref()
        .is_some_and(|name| remotes.iter().any(|remote| &remote.name == name));
    if let Some(name) = incomplete.filter(|_| listed) {
        page.remove_incomplete
            .set_label(&t(&page.ui, Message::RemoveIncomplete).replace("{name}", &name));
    } else {
        page.incomplete.replace(None);
    }
    page.remove_incomplete.set_visible(listed);
    page.remotes.replace(remotes);
}

fn confirm_remove(page: &Rc<Page>, name: String) {
    let dialog = gtk::MessageDialog::builder()
        .transient_for(&page.window)
        .modal(true)
        .message_type(gtk::MessageType::Warning)
        .text(t(&page.ui, Message::ConfirmRemoveTitle).replace("{name}", &name))
        .secondary_text(t(&page.ui, Message::ConfirmRemoveBody))
        .build();
    dialog.add_button(t(&page.ui, Message::Cancel), gtk::ResponseType::Cancel);
    let remove = dialog.add_button(t(&page.ui, Message::Remove), gtk::ResponseType::Accept);
    remove.add_css_class("destructive-action");
    dialog.set_default_response(gtk::ResponseType::Cancel);
    dialog.connect_response(glib::clone!(
        #[strong]
        page,
        move |dialog, response| {
            dialog.close();
            if response == gtk::ResponseType::Accept
                && let Ok(command) = delete_command(&name)
                && start(&page, Job::Delete, command, Message::RunningRclone)
            {
                page.target.replace(name.clone());
            }
        }
    ));
    dialog.present();
}
