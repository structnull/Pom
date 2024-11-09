use eframe::App;
use egui::{CentralPanel, Color32, Context, FontFamily, FontId, Pos2, Shape, Stroke, Vec2};
use notify_rust::Notification;
use std::time::{Duration, Instant};

const POMODORO_TIME: u64 = 25; // minutes for Pomodoro session
const BREAK_TIME: u64 = 5; // minutes for short break

enum Notify {
    Finished,
    Started,
    Resume,
    Paused,
    BreakStarted,
}

pub struct Pom {
    state: TimerState,
    last_update: Instant,
    remaining_time: Duration,
    total_duration: Duration,
    time_setting: u64,
    sessions_completed: u32,
    break_time_setting: u64,
}

#[derive(PartialEq)]
enum TimerState {
    Ready,
    Running,
    Paused,
    Finished,
    OnBreak,
}

impl Pom {
    pub fn new() -> Self {
        let total_duration = Duration::new(POMODORO_TIME * 60, 0);
        Self {
            state: TimerState::Ready,
            last_update: Instant::now(),
            remaining_time: total_duration,
            total_duration,
            time_setting: POMODORO_TIME,
            sessions_completed: 0,
            break_time_setting: BREAK_TIME, // Default to break time
        }
    }

    // Method to send notifications
    fn send_notification(&self, notify: Notify) {
        match notify {
            Notify::Finished => {
                Notification::new()
                    .summary("Pomodoro Timer")
                    .body("Time's up!.")
                    .show()
                    .unwrap();
            }
            Notify::Started => {
                Notification::new()
                    .summary("Pomodoro Timer")
                    .body("Timer started.")
                    .show()
                    .unwrap();
            }
            Notify::Paused => {
                Notification::new()
                    .summary("Pomodoro Timer")
                    .body("Timer paused.")
                    .show()
                    .unwrap();
            }
            Notify::Resume => {
                Notification::new()
                    .summary("Pomodoro Timer")
                    .body("Timer resumed.")
                    .show()
                    .unwrap();
            }
            Notify::BreakStarted => {
                Notification::new()
                    .summary("Pomodoro Timer")
                    .body("Break started. Relax!")
                    .show()
                    .unwrap();
            }
        }
    }

    // Start the timer for Pomodoro
    fn start_timer(&mut self) {
        let total_duration = Duration::new(self.time_setting * 60, 0);
        self.total_duration = total_duration;
        self.remaining_time = total_duration;
        self.state = TimerState::Running;
        self.last_update = Instant::now();
        self.send_notification(Notify::Started);
    }

    // Start break timer
    fn start_break(&mut self) {
        let break_duration = self.break_time_setting;
        self.total_duration = Duration::new(break_duration * 60, 0);
        self.remaining_time = self.total_duration;
        self.state = TimerState::OnBreak;
        self.last_update = Instant::now();
        self.send_notification(Notify::BreakStarted); // This sends the notification when break starts
    }

    fn pause_timer(&mut self) {
        if let TimerState::Running = self.state {
            self.state = TimerState::Paused;
            self.send_notification(Notify::Paused);
        }
    }

    fn resume_timer(&mut self) {
        if let TimerState::Paused = self.state {
            self.state = TimerState::Running;
            self.last_update = Instant::now();
            self.send_notification(Notify::Resume);
        }
    }

    fn reset_timer(&mut self) {
        self.state = TimerState::Ready;
        self.remaining_time = self.total_duration;
    }

    fn update_timer(&mut self) {
        if let TimerState::Running | TimerState::OnBreak = self.state {
            let now = Instant::now();
            let elapsed = now - self.last_update;
            if self.remaining_time > elapsed {
                self.remaining_time -= elapsed;
            } else {
                self.remaining_time = Duration::new(0, 0);
                if self.state == TimerState::Running {
                    self.state = TimerState::Finished;
                    self.sessions_completed += 1;
                    self.send_notification(Notify::Finished);
                    // Start break after Pomodoro session
                    self.start_break();
                } else if self.state == TimerState::OnBreak {
                    self.state = TimerState::Ready;
                    self.send_notification(Notify::Finished);
                    // Restart Pomodoro after break
                    self.start_timer();
                }
            }
            self.last_update = now;
        }
    }

    // Format the duration into a MM:SS string
    fn format_duration(duration: Duration) -> String {
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn progress(&self) -> f32 {
        (self.total_duration.as_secs_f32() - self.remaining_time.as_secs_f32())
            / self.total_duration.as_secs_f32()
    }

    // Draw an arc representing the progress of the timer
    fn draw_arc(
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        stroke: Stroke,
    ) {
        let segments = 100;
        let angle_step = (end_angle - start_angle) / segments as f32;
        let points: Vec<Pos2> = (0..=segments)
            .map(|i| {
                let angle = start_angle + i as f32 * angle_step;
                Pos2 {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                }
            })
            .collect();
        painter.add(Shape::line(points, stroke));
    }
}

impl App for Pom {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn raw_input_hook(&mut self, _ctx: &Context, _raw_input: &mut egui::RawInput) {}

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_timer();

        // Request a repaint to ensure the UI continuously updates
        ctx.request_repaint();

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pomodoro Timer");

            // Calculate the size and position of the circular timer display
            let available_size = ui.available_size();
            let size = Vec2::new(
                available_size.x.min(available_size.y),
                available_size.x.min(available_size.y),
            );
            let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());

            let painter = ui.painter();
            let center = rect.center();
            let radius = rect.width() / 2.0;
            let nrad = radius - 80.0;

            // Draw the circular progress bar background
            painter.circle_stroke(center, nrad, Stroke::new(10.0, Color32::from_gray(80)));

            match self.state {
                TimerState::Finished => {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "Time's up!",
                        egui::TextStyle::Heading.resolve(ui.style()),
                        Color32::DARK_RED,
                    );
                }
                TimerState::Paused => {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "Paused",
                        egui::TextStyle::Heading.resolve(ui.style()),
                        Color32::YELLOW,
                    );
                }
                TimerState::OnBreak => {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "On Break",
                        egui::TextStyle::Heading.resolve(ui.style()),
                        Color32::LIGHT_BLUE,
                    );
                }
                _ => {
                    let progress_angle = self.progress() * std::f32::consts::TAU;

                    // Draw the progress arc
                    Pom::draw_arc(
                        painter,
                        center,
                        nrad,
                        0.0,
                        progress_angle,
                        Stroke::new(10.0, Color32::from_rgb(100, 200, 100)),
                    );

                    let text = Pom::format_duration(self.remaining_time);
                    let font_id = FontId::new(50.0, FontFamily::Proportional);
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        text,
                        font_id,
                        Color32::WHITE,
                    );
                }
            }

            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.time_setting, 0..=60)
                                .clamp_to_range(true)
                                .text("Pomodoro Timer (min)")
                                .trailing_fill(true)
                                .integer(),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.break_time_setting, 0..=60)
                                .clamp_to_range(true)
                                .text("Break Time (min)")
                                .trailing_fill(true)
                                .integer(),
                        );
                    },
                );
            });

            ui.label(format!("Sessions Completed: {}", self.sessions_completed));

            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized([160.0, 40.0], egui::Button::new("Start"))
                            .clicked()
                            .then(|| self.start_timer());
                        ui.add_sized([160.0, 40.0], egui::Button::new("Pause"))
                            .clicked()
                            .then(|| self.pause_timer());
                        ui.add_sized([160.0, 40.0], egui::Button::new("Resume"))
                            .clicked()
                            .then(|| self.resume_timer());
                        ui.add_sized([160.0, 40.0], egui::Button::new("Reset"))
                            .clicked()
                            .then(|| self.reset_timer());
                    });
                },
            );
        });
    }
}
