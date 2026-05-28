use crate::theme::{THEME, Theme};
use ratatui::prelude::{Style as PStyle, Stylize as PStylize};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{
        Color, Style, Stylize,
        palette::{
            material::{BLUE_GRAY, PINK},
            tailwind::{FUCHSIA, PURPLE, ROSE, SLATE, STONE},
        },
    },
    symbols,
    symbols::border::*,
    text::Span,
    text::Text,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};
use std::sync::Arc;

const BACKGROUND: Color = STONE.c400;
const TEXT_COLOR: Color = ROSE.c800;
use crate::app::App;

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
    let top_border_set = symbols::border::Set {
        horizontal_top: symbols::line::NORMAL.horizontal,
        ..symbols::border::ROUNDED
    };
    let menu_block = Block::default()
        .title_top("MENU")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::ALL);

    let menu_list_items = vec![
        ListItem::new(Text::styled("Open Playlist", TEXT_COLOR)),
        ListItem::new(Text::styled("Change Theme", TEXT_COLOR)),
    ];

    // let menu = Paragraph::new(Text::styled("", TEXT_COLOR)).block(menu_block);
    let menu = List::new(menu_list_items)
        .block(menu_block)
        .highlight_symbol(">>")
        .highlight_style(THEME.highlight)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let about_mfp_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .title_top("About")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let about_mfp = Paragraph::new(Text::raw(
        "Through years of trial and error — skipping around radio streams, playing entire collections on shuffle, or repeating certain tracks over and over, we have found that the most compelling music for sustained concentration, tends to contain a mixture of the following:
        Noise, Drones, Arpeggios, Atmospheres, Field Recordings, Arrhythmic Textures, Vagueness (Hypnagogia), Microtones / Dissonance, Detail / Finery / Patterns, Awesome / Daunting / Foreboding, Vast / Transcendental / Meditative, etc.",
    ))
    .wrap(Wrap { trim: false })
    .scroll((0, 0))
    .block(about_mfp_block);

    let mfp_credits_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .title_top("Credits")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let mfp_credits = Paragraph::new(Text::raw(
        "Music For Programming is maintained by Datassette, the first episode was released in 2009.
        This incarnation of the site was built with Svelte, and the typeface is IBM Plex Mono.",
    ))
    .wrap(Wrap { trim: false })
    .block(mfp_credits_block);

    let middle_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(50),
            Constraint::Percentage(40),
        ])
        .split(inner_layout[1]);

    // let inner_middle_layout = Layout:
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
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));
    let ep_title = Paragraph::new(Text::styled(
        episode_title,
        Style::default().fg(Color::Rgb(175, 196, 219)),
    ))
    .block(ep_title_block);

    let ep_info_block = Block::default()
        .borders(Borders::ALL)
        .border_set(ROUNDED)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let episode_information = format!(
        "Duration: {}\nRelease Date: {}",
        episodes_clone.read().unwrap()[app.selected_episode]
            .duration
            .clone(),
        episodes_clone.read().unwrap()[app.selected_episode]
            .pub_date
            .clone(),
    );
    let ep_info = Paragraph::new(Text::styled(episode_information, Color::Rgb(175, 196, 219)))
        .block(ep_info_block);

    let play_status_bar_block = Block::default()
        .title_top("Status Bar")
        .title_alignment(Alignment::Center)
        .border_set(ROUNDED)
        .borders(Borders::TOP)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let play_status_bar =
        Paragraph::new(Text::styled("Play Status Bar", Color::Rgb(175, 196, 219)))
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
        .borders(Borders::ALL);

    let search_bar =
        Paragraph::new(Text::styled("", Style::default().fg(TEXT_COLOR))).block(search_bar_block);

    // TODO: Add SaveToPlaylist & RemoveFromPlaylist Button/Icon on list items.
    let ep_list_block = Block::bordered()
        .title_top("Episode List")
        .title_alignment(Alignment::Center)
        .title_style(THEME.title)
        .border_set(ROUNDED)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let mut episode_list_items: Vec<_> = Vec::new();

    for ep in episodes_clone.read().unwrap().iter() {
        let ep_list_item = ListItem::new(Text::from(ep.title.clone()));
        episode_list_items.push(ep_list_item);
    }

    let episode_list = List::new(episode_list_items)
        .block(ep_list_block)
        .highlight_symbol(">>")
        .highlight_style(THEME.highlight)
        .style(Style::default().fg(Color::Rgb(175, 196, 219)));

    let title = Span::styled("Dataglass", THEME.app_title);

    // frame.render_widget(title, outer_layout[0]);
    frame.render_stateful_widget(menu, left_layout[0], &mut app.menu_list_state);
    frame.render_widget(about_mfp, left_layout[1]);
    frame.render_widget(mfp_credits, left_layout[2]);
    frame.render_widget(ep_title, middle_layout[0]);
    frame.render_widget(ep_info, middle_layout[1]);
    frame.render_widget(play_status_bar, middle_layout[2]);
    frame.render_widget(search_bar, right_layout[0]);
    frame.render_stateful_widget(episode_list, right_layout[1], &mut app.episode_list_state);

    // frame.render_widget(composed_commit, inner_right_layout[0]);
    // frame.render_widget(interaction_panel, inner_right_layout[1]);
}
