use crate::theme::THEME;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color},
    symbols::border::*,
    text::Span,
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::sync::Arc;

use crate::app::{App, SelectedList};

pub fn render(app: &mut App, frame: &mut Frame) {
    let outer_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Percentage(95)])
        .split(frame.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(outer_layout[1]);

    let left_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(inner_layout[0]);
    
    let menu_block = Block::default()
        .title_top("MENU")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::ALL)
        .border_style(if app.selected_list == SelectedList::Menu { THEME.active_borders } else { THEME.borders });

    let menu_list_items = vec![
        ListItem::new(Text::styled("Open Playlist", THEME.text)),
        ListItem::new(Text::styled("Change Theme", THEME.text)),
    ];

    let menu = List::new(menu_list_items)
        .block(menu_block)
        .highlight_symbol(">>")
        .highlight_style(THEME.highlight)
        .style(THEME.text);

    let about_mfp_block = Block::default()
        .borders(Borders::ALL)
        .title_top("About")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .border_style(if app.selected_list == SelectedList::About { THEME.active_borders } else { THEME.borders })
        .style(THEME.text);

    let about_mfp = Paragraph::new(Text::raw(
        "Through years of trial and error — skipping around radio streams, playing entire collections on shuffle, or repeating certain tracks over and over, we have found that the most compelling music for sustained concentration, tends to contain a mixture of the following:
        Noise, Drones, Arpeggios, Atmospheres, Field Recordings, Arrhythmic Textures, Vagueness (Hypnagogia), Microtones / Dissonance, Detail / Finery / Patterns, Awesome / Daunting / Foreboding, Vast / Transcendental / Meditative, etc.",
    ))
    .wrap(Wrap { trim: false })
    .scroll((app.about_scroll, 0))
    .block(about_mfp_block);

    let mfp_credits_block = Block::default()
        .borders(Borders::ALL)
        .title_top("Credits")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .border_style(if app.selected_list == SelectedList::Credits { THEME.active_borders } else { THEME.borders })
        .style(THEME.text);

    let mfp_credits = Paragraph::new(Text::raw(
        "Music For Programming is maintained by Datassette, the first episode was released in 2009.
        This incarnation of the site was built with Svelte, and the typeface is IBM Plex Mono.",
    ))
    .wrap(Wrap { trim: false })
    .scroll((app.credits_scroll, 0))
    .block(mfp_credits_block);

    let middle_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(50),
            Constraint::Percentage(40),
        ])
        .split(inner_layout[1]);

    let episodes_clone = Arc::clone(&app.episodes);
    let full_episode_title = episodes_clone.read().unwrap()[app.selected_episode]
        .title
        .clone();
    let mut split_title = full_episode_title.splitn(2, ":");
    let episode_number = split_title.next().unwrap();
    let episode_title = split_title.next().unwrap();

    let ep_title_block = Block::default()
        .title_top(episode_number)
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::ALL)
        .border_style(THEME.borders)
        .style(THEME.text);

    let ep_title = Paragraph::new(Text::styled(
        episode_title,
        THEME.text,
    ))
    .block(ep_title_block);

    let ep_info_block = Block::default()
        .borders(Borders::ALL)
        .border_set(ROUNDED)
        .border_style(THEME.borders)
        .style(THEME.text);

    let episode_information = format!(
        "Duration: {}\nRelease Date: {}",
        episodes_clone.read().unwrap()[app.selected_episode]
            .duration
            .clone(),
        episodes_clone.read().unwrap()[app.selected_episode]
            .pub_date
            .clone(),
    );
    let ep_info = Paragraph::new(Text::styled(episode_information, THEME.text))
        .block(ep_info_block);

    let play_status_bar_block = Block::default()
        .title_top("Status Bar")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::TOP)
        .border_style(THEME.borders)
        .style(THEME.text);

    let playback_info = match &app.current_track {
        Some(track) => format!(
            "{} | {:?} | Vol: {:.0}%",
            track.title,
            app.playback_state,
            app.volume * 100.0
        ),
        None => "No track playing".to_string(),
    };

    let play_status_bar = Paragraph::new(Text::styled(
        playback_info,
        THEME.text,
    ))
    .block(play_status_bar_block);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(inner_layout[2]);

    let search_bar_block = Block::default()
        .title_top("Search Bar")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::ALL)
        .border_style(THEME.borders);

    let search_bar =
        Paragraph::new(Text::styled("", THEME.text)).block(search_bar_block);

    let ep_list_block = Block::bordered()
        .title_top("Episode List")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::ALL)
        .border_style(if app.selected_list == SelectedList::Episodes { THEME.active_borders } else { THEME.borders })
        .style(THEME.text);

    let mut episode_list_items: Vec<_> = Vec::new();

    for ep in episodes_clone.read().unwrap().iter() {
        let ep_list_item = ListItem::new(Text::from(ep.title.clone()));
        episode_list_items.push(ep_list_item);
    }

    let episode_list = List::new(episode_list_items)
        .block(ep_list_block)
        .highlight_symbol(">>")
        .highlight_style(THEME.highlight)
        .style(THEME.text);

    let title = Span::styled("Petalblade", THEME.app_title);

    frame.render_widget(title, outer_layout[0]);
    frame.render_stateful_widget(menu, left_layout[0], &mut app.menu_list_state);
    frame.render_widget(about_mfp, left_layout[1]);
    frame.render_widget(mfp_credits, left_layout[2]);
    frame.render_widget(ep_title, middle_layout[0]);
    frame.render_widget(ep_info, middle_layout[1]);
    frame.render_widget(play_status_bar, middle_layout[2]);
    frame.render_widget(search_bar, right_layout[0]);
    frame.render_stateful_widget(episode_list, right_layout[1], &mut app.episode_list_state);
}
