use ratatui::{
    Frame,
    layout::Alignment,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::{Counts, Mode, Stats};

const LOGO: &str = r"_____  _     _   _ ___________ _____
/  __ \| |   | | | |  ___|  _  \  _  |
| /  \/| |   | | | | |__ | | | | | | |
| |    | |   | | | |  __|| | | | | | |
| \__/\| |___| |_| | |___| |/ /\ \_/ /
 \____/\_____/\___/\____/|___/  \___/ ";

pub fn draw(
    f: &mut Frame,
    ddr_addr: &str,
    obs_addr: &str,
    counts: &Counts,
    stats: &Stats,
    log: &[(String, String)],
    mode: &Mode,
) {
    let area = f.area();

    let outer = Block::default().borders(Borders::ALL);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let lr = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(inner);
    let (left, right) = (lr[0], lr[1]);

    let logo_h = LOGO.lines().count() as u16;
    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_h),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(left);

    f.render_widget(
        Paragraph::new(LOGO).alignment(Alignment::Center),
        left_rows[0],
    );

    f.render_widget(
        Paragraph::new(format!(" {ddr_addr}"))
            .block(Block::default().borders(Borders::ALL).title(" DDR URL ")),
        left_rows[2],
    );
    f.render_widget(
        Paragraph::new(format!(" {obs_addr}"))
            .block(Block::default().borders(Borders::ALL).title(" OBS URL ")),
        left_rows[3],
    );

    let judgment_lines = vec![
        judgment_line("Marvelous", Color::Rgb(238, 232, 170), counts.marvelous),
        judgment_line("Perfect", Color::Yellow, counts.perfect),
        judgment_line("Great", Color::Green, counts.great),
        judgment_line("Good", Color::Rgb(70, 130, 180), counts.good),
        judgment_line("Ok", Color::Rgb(255, 69, 0), counts.ok),
        judgment_line("Miss", Color::Rgb(139, 0, 0), counts.miss),
        judgment_line("Fast", Color::Rgb(65, 105, 225), counts.fast),
        judgment_line("Slow", Color::Rgb(255, 105, 180), counts.slow),
    ];
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(left_rows[4]);

    f.render_widget(
        Paragraph::new(judgment_lines)
            .block(Block::default().borders(Borders::ALL).title(" Judgments ")),
        bottom[0],
    );

    let elapsed = stats.elapsed_secs;
    let (h, m, s) = (elapsed / 3600, elapsed % 3600 / 60, elapsed % 60);
    let stats_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<6}", "Time"),
                Style::default().fg(Color::Rgb(180, 180, 180)),
            ),
            Span::raw(format!("{h:02}:{m:02}:{s:02}")),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!("{:<12}", "Songs"), Style::default().fg(Color::Cyan)),
            Span::raw(stats.songs.to_string()),
        ]),
    ];
    f.render_widget(
        Paragraph::new(stats_lines)
            .block(Block::default().borders(Borders::ALL).title(" Session ")),
        bottom[1],
    );

    f.render_widget(
        Paragraph::new(" [Enter] Reconnect  [S] Settings  [Esc] Quit")
            .style(Style::default().fg(Color::DarkGray)),
        left_rows[6],
    );

    let visible = right.height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = log
        .iter()
        .rev()
        .take(visible)
        .map(|(time, msg)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{time} "), Style::default().fg(Color::DarkGray)),
                Span::raw(msg.clone()),
            ]))
        })
        .collect();
    f.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(" LOGS ")),
        right,
    );

    if let Mode::Settings { ddr, obs, field } = mode {
        let popup = centered_rect(62, 9, area);
        f.render_widget(Clear, popup);

        let outer = Block::default().borders(Borders::ALL).title(" Settings ");
        let inner = outer.inner(popup);
        f.render_widget(outer, popup);

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);

        let active = Style::default().fg(Color::Yellow);
        let inactive = Style::default();

        f.render_widget(
            Paragraph::new(format!(" {ddr}")).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" DDR Address ")
                    .border_style(if *field == 0 { active } else { inactive }),
            ),
            sections[0],
        );
        f.render_widget(
            Paragraph::new(format!(" {obs}")).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" OBS Address ")
                    .border_style(if *field == 1 { active } else { inactive }),
            ),
            sections[1],
        );
        f.render_widget(
            Paragraph::new("  [Tab] Switch field  [Enter] Save  [Esc] Cancel")
                .style(Style::default().fg(Color::DarkGray)),
            sections[2],
        );
    }
}

fn judgment_line(label: &str, color: Color, count: u32) -> Line<'_> {
    Line::from(vec![
        Span::styled(format!(" {label:<12}"), Style::default().fg(color)),
        Span::raw(count.to_string()),
    ])
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
