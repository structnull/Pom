use eframe::{egui, App};
use egui::{Align2, CentralPanel, Color32, Context, FontFamily, FontId, Pos2, Stroke, Vec2};
use notify_rust::Notification;
use std::f32::consts::TAU;
use std::time::{Duration, Instant};

const DEFAULT_POM_MIN: u64 = 25;
const DEFAULT_BREAK_MIN: u64 = 5;
const ARC_SEGMENTS: usize = 96; 
const FRAME_MS: u64 = 16; 
const TIMER_TICK_MS: u64 = 200; 
const VISUAL_SMOOTHING: f32 = 0.12; 

#[derive(Debug, PartialEq, Clone, Copy)]
enum TimerState {
    Ready,
    Running,
    Paused,
    OnBreak,
}

#[derive(Clone, Copy)]
enum NotifyKind {
    Started,
    Paused,
    Resumed,
    Finished,
    BreakStarted,
}

pub struct Pom {
    state: TimerState,
    pom_minutes: u64,
    break_minutes: u64,
    sessions_completed: u32,

    duration: Duration,
    remaining: Duration,
    last_tick: Instant,
    last_coarse_tick: Instant, 

    visual_progress: f32,
}

impl Pom {
    pub fn new() -> Self {
        let duration = Duration::from_secs(DEFAULT_POM_MIN * 60);
        let now = Instant::now();
        Self {
            state: TimerState::Ready,
            pom_minutes: DEFAULT_POM_MIN,
            break_minutes: DEFAULT_BREAK_MIN,
            sessions_completed: 0,
            duration,
            remaining: duration,
            last_tick: now,
            last_coarse_tick: now,
            visual_progress: 0.0,
        }
    }

    fn notify(&self, kind: NotifyKind) {
        let (summary, body) = match kind {
            NotifyKind::Started => ("Pomodoro", "Timer started."),
            NotifyKind::Paused => ("Pomodoro", "Timer paused."),
            NotifyKind::Resumed => ("Pomodoro", "Timer resumed."),
            NotifyKind::Finished => ("Pomodoro", "Session finished."),
            NotifyKind::BreakStarted => ("Pomodoro", "Break started. Relax!"),
        };

        let _ = Notification::new().summary(summary).body(body).show();
    }

    fn start_session(&mut self) {
        self.duration = Duration::from_secs(self.pom_minutes * 60);
        self.remaining = self.duration;
        self.state = TimerState::Running;
        self.last_tick = Instant::now();
        self.last_coarse_tick = Instant::now();
        self.notify(NotifyKind::Started);
    }

    fn start_break(&mut self) {
        self.duration = Duration::from_secs(self.break_minutes * 60);
        self.remaining = self.duration;
        self.state = TimerState::OnBreak;
        self.last_tick = Instant::now();
        self.last_coarse_tick = Instant::now();
        self.notify(NotifyKind::BreakStarted);
    }

    fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
            self.notify(NotifyKind::Paused);
        }
    }

    fn resume(&mut self) {
        if self.state == TimerState::Paused {
            self.state = TimerState::Running;
            self.last_tick = Instant::now();
            self.last_coarse_tick = Instant::now();
            self.notify(NotifyKind::Resumed);
        }
    }

    fn reset(&mut self) {
        self.state = TimerState::Ready;
        self.duration = Duration::from_secs(self.pom_minutes * 60);
        self.remaining = self.duration;
        self.visual_progress = 0.0;
    }

    /* ---------- Formatting + progress ---------- */
    fn format_duration(d: Duration) -> String {
        let total = d.as_secs();
        let m = total / 60;
        let s = total % 60;
        format!("{:02}:{:02}", m, s)
    }

    fn true_progress(&self) -> f32 {
        let total = self.duration.as_secs_f32();
        if total <= 0.0 {
            return 0.0;
        }
        let rem = self.remaining.as_secs_f32().clamp(0.0, total);
        ((total - rem) / total).clamp(0.0, 1.0)
    }

    fn tick_timer(&mut self) -> bool {
        let now = Instant::now();
        let since_coarse = now.duration_since(self.last_coarse_tick);
        if since_coarse < Duration::from_millis(TIMER_TICK_MS) {
            return false;
        }
        self.last_coarse_tick = now;

        if !matches!(self.state, TimerState::Running | TimerState::OnBreak) {
            self.last_tick = now;
            return false;
        }

        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;

        if elapsed < self.remaining {
            self.remaining -= elapsed;
            return true;
        }

        self.remaining = Duration::ZERO;

        match self.state {
            TimerState::Running => {
                self.sessions_completed = self.sessions_completed.saturating_add(1);
                self.notify(NotifyKind::Finished);

                self.start_break();
            }
            TimerState::OnBreak => {
                self.notify(NotifyKind::Finished);

                self.state = TimerState::Ready;
                self.duration = Duration::from_secs(self.pom_minutes * 60);
                self.remaining = self.duration;
            }
            _ => {}
        }

        true
    }

    fn draw_arc(
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        progress: f32,
        stroke: Stroke,
    ) {
        if progress <= 0.0001 {
            return;
        }
        let end_angle = progress.clamp(0.0, 1.0) * TAU;
        let segments = ARC_SEGMENTS.max(4) as usize;
        let step = end_angle / segments as f32;

        let mut prev_angle = 0.0_f32;
        let mut prev_pos = Pos2 {
            x: center.x + radius * prev_angle.cos(),
            y: center.y + radius * prev_angle.sin(),
        };

        for i in 1..=segments {
            let angle = i as f32 * step;
            let next_pos = Pos2 {
                x: center.x + radius * angle.cos(),
                y: center.y + radius * angle.sin(),
            };
            painter.line_segment([prev_pos, next_pos], stroke);
            prev_pos = next_pos;
            prev_angle = angle;
        }
    }
}

impl App for Pom {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let timer_changed = self.tick_timer();

        let target = self.true_progress();
        self.visual_progress += (target - self.visual_progress) * VISUAL_SMOOTHING;

        ctx.request_repaint_after(Duration::from_millis(FRAME_MS));

        if timer_changed {
            ctx.request_repaint();
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pomodoro Timer");

            let avail = ui.available_size();
            let size = avail.x.min(avail.y).max(200.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(size, size), egui::Sense::hover());
            let painter = ui.painter();
            let center = rect.center();
            let radius = size * 0.36;

            painter.circle_stroke(center, radius, Stroke::new(8.0, Color32::from_gray(40)));

            Pom::draw_arc(
                painter,
                center,
                radius,
                self.visual_progress,
                Stroke::new(10.0, Color32::from_rgb(110, 200, 110)),
            );

            let middle_text = match self.state {
                TimerState::Paused => "Paused".to_owned(),
                TimerState::OnBreak => format!("Break\n{}", Pom::format_duration(self.remaining)),
                _ => Pom::format_duration(self.remaining),
            };

            painter.text(
                center,
                Align2::CENTER_CENTER,
                middle_text,
                FontId::new(36.0, FontFamily::Proportional),
                Color32::WHITE,
            );

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let pom_before = self.pom_minutes;
                let break_before = self.break_minutes;

                ui.add(
                    egui::Slider::new(&mut self.pom_minutes, 1..=90)
                        .clamp_to_range(true)
                        .text("Pomodoro (min)")
                        .integer(),
                );
                ui.add(
                    egui::Slider::new(&mut self.break_minutes, 1..=30)
                        .clamp_to_range(true)
                        .text("Break (min)")
                        .integer(),
                );

                if self.state == TimerState::Ready
                    && (pom_before != self.pom_minutes || break_before != self.break_minutes)
                {
                    self.duration = Duration::from_secs(self.pom_minutes * 60);
                    self.remaining = self.duration;
                    self.visual_progress = 0.0;
                }
            });

            ui.label(format!("Sessions completed: {}", self.sessions_completed));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let start_enabled = self.state == TimerState::Ready;
                let pause_enabled = self.state == TimerState::Running;
                let resume_enabled = self.state == TimerState::Paused;

                if ui
                    .add_enabled(start_enabled, egui::Button::new("Start"))
                    .clicked()
                {
                    self.start_session();
                    ctx.request_repaint();
                }

                if ui
                    .add_enabled(pause_enabled, egui::Button::new("Pause"))
                    .clicked()
                {
                    self.pause();
                    ctx.request_repaint();
                }

                if ui
                    .add_enabled(resume_enabled, egui::Button::new("Resume"))
                    .clicked()
                {
                    self.resume();
                    ctx.request_repaint();
                }

                if ui.button("Reset").clicked() {
                    self.reset();
                    ctx.request_repaint();
                }
            });
        });
    }
}

