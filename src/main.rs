use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use graham::{Action, Point, ScanTrace, Step, graham_scan};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Points};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use serde::Deserialize;

struct Scenario {
    markers: Vec<Point>,
    expected_perimeter: Vec<Point>,
}

#[derive(Deserialize)]
struct ScenarioFile {
    markers: Vec<[i32; 2]>,
    expected_perimeter: Vec<[i32; 2]>,
}

struct App {
    trace: ScanTrace,
    expected_perimeter: Vec<Point>,
    step: usize,
    playing: bool,
    last_advance: Instant,
}

impl App {
    fn new(scenario: Scenario) -> Self {
        Self {
            trace: graham_scan(&scenario.markers),
            expected_perimeter: scenario.expected_perimeter,
            step: 0,
            playing: false,
            last_advance: Instant::now(),
        }
    }

    fn current(&self) -> &Step {
        &self.trace.steps[self.step]
    }

    fn next(&mut self) {
        self.step = (self.step + 1).min(self.trace.steps.len() - 1);
    }

    fn previous(&mut self) {
        self.step = self.step.saturating_sub(1);
    }

    fn tick(&mut self) {
        if self.playing && self.last_advance.elapsed() >= Duration::from_millis(125) {
            if self.step == self.trace.steps.len() - 1 {
                self.playing = false;
            } else {
                self.next();
            }
            self.last_advance = Instant::now();
        }
    }

    fn excluded_marker(&self) -> Option<(usize, Point)> {
        self.expected_perimeter
            .iter()
            .find(|point| !self.trace.hull.contains(point))
            .and_then(|point| {
                self.trace
                    .input
                    .iter()
                    .position(|input| input == point)
                    .map(|index| (index, *point))
            })
    }
}

fn main() -> io::Result<()> {
    let data_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/graham.toml");
    let scenario = load_scenario(&data_path)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, scenario);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    scenario: Scenario,
) -> io::Result<()> {
    let mut app = App::new(scenario);

    loop {
        terminal.draw(|frame| draw(frame, &app))?;
        app.tick();

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Right | KeyCode::Char('l') => {
                app.playing = false;
                app.next();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.playing = false;
                app.previous();
            }
            KeyCode::Char('r') => {
                app.playing = false;
                app.step = 0;
            }
            KeyCode::Enter => {
                app.playing = !app.playing;
                app.last_advance = Instant::now();
            }
            _ => {}
        }
    }
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(16),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [map, _, sidebar] = Layout::horizontal([
        Constraint::Percentage(66),
        Constraint::Length(1),
        Constraint::Min(34),
    ])
    .areas(body);

    draw_header(frame, header, app);
    draw_map(frame, map, app);
    draw_sidebar(frame, sidebar, app);
    draw_footer(frame, footer, app);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let subtitle = if app.step == app.trace.steps.len() - 1 {
        app.excluded_marker().map_or_else(
            || "PASS · every marker is enclosed".to_string(),
            |(index, _)| format!("FAILED · the computed fence does not enclose P{index}"),
        )
    } else {
        "Find the smallest fence enclosing every survey marker".to_string()
    };
    let subtitle = if app.step == app.trace.steps.len() - 1 {
        if app.excluded_marker().is_some() {
            Span::raw(subtitle).red()
        } else {
            Span::raw(subtitle).green()
        }
    } else {
        Span::raw(subtitle).gray()
    };
    let header = Paragraph::new(vec![
        Line::from(Span::raw("Graham Scan TUI").cyan().bold()),
        Line::from(subtitle),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(header, area);
}

fn draw_map(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let current = app.current();
    let final_step = app.step == app.trace.steps.len() - 1;
    let failed = final_step && app.excluded_marker().is_some();
    let hull_color = if failed { Color::Red } else { Color::Cyan };

    let canvas = Canvas::default()
        .block(panel("Survey area"))
        .marker(Marker::Braille)
        .x_bounds([0.5, 9.5])
        .y_bounds([0.5, 9.5])
        .paint(|context| {
            if final_step {
                for index in 0..app.expected_perimeter.len() {
                    let a = app.expected_perimeter[index];
                    let b = app.expected_perimeter[(index + 1) % app.expected_perimeter.len()];
                    for segment in (0..16).step_by(2) {
                        let start = f64::from(segment) / 16.0;
                        let end = f64::from(segment + 1) / 16.0;
                        let x_delta = f64::from(b.x - a.x);
                        let y_delta = f64::from(b.y - a.y);
                        context.draw(&CanvasLine {
                            x1: f64::from(a.x) + x_delta * start,
                            y1: f64::from(a.y) + y_delta * start,
                            x2: f64::from(a.x) + x_delta * end,
                            y2: f64::from(a.y) + y_delta * end,
                            color: Color::Green,
                        });
                    }
                }
            }

            for pair in current.hull.windows(2) {
                context.draw(&CanvasLine {
                    x1: f64::from(pair[0].x),
                    y1: f64::from(pair[0].y),
                    x2: f64::from(pair[1].x),
                    y2: f64::from(pair[1].y),
                    color: hull_color,
                });
            }
            if final_step && current.hull.len() > 2 {
                let first = current.hull[0];
                let last = *current.hull.last().expect("the hull is not empty");
                context.draw(&CanvasLine {
                    x1: f64::from(last.x),
                    y1: f64::from(last.y),
                    x2: f64::from(first.x),
                    y2: f64::from(first.y),
                    color: hull_color,
                });
            }

            if let Some((a, b, candidate)) = current.inspected {
                context.draw(&CanvasLine {
                    x1: f64::from(a.x),
                    y1: f64::from(a.y),
                    x2: f64::from(b.x),
                    y2: f64::from(b.y),
                    color: Color::Cyan,
                });
                let turn_color = if current.turn.is_some_and(|turn| turn > 0) {
                    Color::Green
                } else {
                    Color::Red
                };
                context.draw(&CanvasLine {
                    x1: f64::from(b.x),
                    y1: f64::from(b.y),
                    x2: f64::from(candidate.x),
                    y2: f64::from(candidate.y),
                    color: turn_color,
                });
            }

            for point in &app.trace.input {
                let color = point_color(app, *point);
                let coords = [(f64::from(point.x), f64::from(point.y))];
                context.draw(&Points {
                    coords: &coords,
                    color,
                });
            }

            for (index, point) in app.trace.input.iter().enumerate() {
                let label_x = if point.x >= 8 {
                    f64::from(point.x) - 0.72
                } else {
                    f64::from(point.x) + 0.12
                };
                let label_y = if point.y >= 8 {
                    f64::from(point.y) - 0.32
                } else {
                    f64::from(point.y) + 0.12
                };
                context.print(
                    label_x,
                    label_y,
                    Span::styled(
                        format!("P{index}"),
                        Style::new().fg(point_color(app, *point)),
                    ),
                );
            }
        });

    frame.render_widget(canvas, area);
}

fn draw_sidebar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let [state_area, hull_area, order_area] = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Min(5),
    ])
    .areas(area);
    let current = app.current();

    let turn = current
        .turn
        .map(|value| format!("{value} ({})", if value > 0 { "left" } else { "not left" }))
        .unwrap_or_else(|| "—".to_string());
    let candidate = current
        .candidate
        .map(|point| point.to_string())
        .unwrap_or_else(|| "—".to_string());
    let state = Paragraph::new(vec![
        Line::from(vec![
            state_label("Step"),
            Span::raw(format!("{}/{}", app.step, app.trace.steps.len() - 1)),
        ]),
        Line::from(vec![
            state_label("Action"),
            Span::styled(current.action.to_string(), action_style(current.action)),
        ]),
        Line::from(vec![state_label("Candidate"), Span::raw(candidate)]),
        Line::from(vec![state_label("Turn"), Span::raw(turn)]),
        Line::from(vec![
            state_label("Processed"),
            Span::raw(format!("{}/{}", current.processed, app.trace.ordered.len())),
        ]),
    ])
    .wrap(Wrap { trim: true })
    .block(panel("Current state"));
    frame.render_widget(state, state_area);

    let hull_items = current
        .hull
        .iter()
        .enumerate()
        .map(|(stack_index, point)| {
            let point_index = app
                .trace
                .input
                .iter()
                .position(|input| input == point)
                .expect("hull points come from the input");
            let pointer = if stack_index + 1 == current.hull.len() {
                "→"
            } else {
                " "
            };
            ListItem::new(format!("{pointer} P{point_index:<2} {point}"))
                .style(Style::new().fg(point_color(app, *point)))
        })
        .collect::<Vec<_>>();
    if hull_items.is_empty() {
        frame.render_widget(Paragraph::new("—").block(panel("Hull stack")), hull_area);
    } else {
        frame.render_widget(List::new(hull_items).block(panel("Hull stack")), hull_area);
    }

    let items = app
        .trace
        .ordered
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let style = if Some(*point) == current.candidate {
                Style::new().yellow()
            } else {
                Style::new()
            };
            ListItem::new(format!("{:>2}. {point}", index + 1)).style(style)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(panel("Polar-angle order")),
        order_area,
    );
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let playback = if app.playing {
        "Pause"
    } else if app.step == 0 {
        "Start"
    } else {
        "Resume"
    };
    let mut spans = vec![
        Span::raw("←/→").cyan(),
        Span::raw(" step  "),
        Span::raw("Enter").cyan(),
        Span::raw(format!(" {playback}  ")),
        Span::raw("r").cyan(),
        Span::raw(" reset  "),
        Span::raw("q").cyan(),
        Span::raw(" quit  ·  "),
    ];
    if app.step == app.trace.steps.len() - 1 {
        spans.extend([
            Span::raw("green").green(),
            Span::raw(" expected  "),
            Span::raw("red").red(),
            Span::raw(" computed"),
        ]);
    } else {
        spans.extend([
            Span::raw("magenta").magenta(),
            Span::raw(" pivot  "),
            Span::raw("yellow").yellow(),
            Span::raw(" candidate  "),
            Span::raw("cyan").cyan(),
            Span::raw(" hull"),
        ]);
    }
    let footer = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

fn action_style(action: Action) -> Style {
    let color = match action {
        Action::Remove(_) => Color::Red,
        Action::KeepTurn(_) | Action::Push(_) => Color::Green,
        Action::Consider(_) => Color::White,
        Action::Complete => Color::Magenta,
        _ => Color::White,
    };
    Style::new().fg(color)
}

fn point_color(app: &App, point: Point) -> Color {
    let current = app.current();
    let final_step = app.step == app.trace.steps.len() - 1;

    if final_step
        && app
            .excluded_marker()
            .is_some_and(|(_, excluded)| excluded == point)
    {
        return Color::Red;
    }
    if matches!(current.action, Action::Remove(removed) if removed == point) {
        return Color::Red;
    }
    if current.candidate == Some(point) {
        return Color::Yellow;
    }
    if app.step > 0 && point == app.trace.pivot {
        return Color::Magenta;
    }
    if current.hull.contains(&point) {
        return Color::Cyan;
    }

    let processed = app
        .trace
        .ordered
        .iter()
        .position(|ordered| *ordered == point)
        .is_some_and(|index| index < current.processed);
    if processed {
        Color::DarkGray
    } else {
        Color::White
    }
}

fn state_label(label: &str) -> Span<'static> {
    Span::raw(format!("{label:<9} : ")).white()
}

fn panel(title: &'static str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().dark_gray())
        .title(Line::from(vec![
            Span::raw("┤ ").dark_gray(),
            Span::raw(title).white().bold(),
            Span::raw(" ├").dark_gray(),
        ]))
}

fn load_scenario(path: &Path) -> io::Result<Scenario> {
    let contents = std::fs::read_to_string(path)?;
    let data: ScenarioFile = toml::from_str(&contents)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let markers = data
        .markers
        .into_iter()
        .map(|[x, y]| Point::new(x, y))
        .collect::<Vec<_>>();
    let expected_perimeter = data
        .expected_perimeter
        .into_iter()
        .map(|[x, y]| Point::new(x, y))
        .collect::<Vec<_>>();

    if markers.len() < 3 || expected_perimeter.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "graham data needs at least three markers and three perimeter points",
        ));
    }

    Ok(Scenario {
        markers,
        expected_perimeter,
    })
}
