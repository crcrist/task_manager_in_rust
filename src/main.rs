use iced::{
    alignment, executor, time, Application, Command, Element, Length, Settings, Subscription,
    widget::{Button, Column, Container, Row, Scrollable, Text},
};
use sysinfo::{Pid, System};
use std::time::Duration;

struct TaskManager {
    processes: Vec<ProcessInfo>,
    sort_column: SortColumn,
    sort_ascending: bool,
    system: System,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    memory: u64,
    cpu: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Pid,
    Name,
    Memory,
    Cpu,
}

#[derive(Debug, Clone)]
enum Message {
    Sort(SortColumn),
    KillProcess(u32),
    Tick,
}

impl TaskManager {
    fn refresh(&mut self) {
        self.system.refresh_all();
        self.processes = self.system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                memory: process.memory() / 1024 / 1024,
                cpu: process.cpu_usage(),
            })
            .collect();
        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Memory => a.memory.cmp(&b.memory),
                SortColumn::Cpu => a.cpu.partial_cmp(&b.cpu).unwrap_or(std::cmp::Ordering::Equal),
            };
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }
}

impl Application for TaskManager {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::theme::Theme;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut task_manager = TaskManager {
            processes: Vec::new(),
            sort_column: SortColumn::Pid,
            sort_ascending: true,
            system: System::new_all(),
        };
        task_manager.refresh();
        (task_manager, Command::none())
    }

    fn title(&self) -> String {
        String::from("Task Manager")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                self.refresh();
                Command::none()
            }
            Message::Sort(column) => {
                if self.sort_column == column {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = column;
                    self.sort_ascending = true;
                }
                self.sort_processes();
                Command::none()
            }
            Message::KillProcess(pid) => {
                if let Some(process) = self.system.process(Pid::from(pid as usize)) {
                    let _ = process.kill();
                }
                self.refresh();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = Row::new()
            .push(Button::new("PID").on_press(Message::Sort(SortColumn::Pid)))
            .push(Button::new("Name").on_press(Message::Sort(SortColumn::Name)))
            .push(Button::new("Memory (MB)").on_press(Message::Sort(SortColumn::Memory)))
            .push(Button::new("CPU (%)").on_press(Message::Sort(SortColumn::Cpu)));

        let processes = self.processes.iter().fold(
            Column::new().spacing(5),
            |column, process| {
                column.push(
                    Row::new()
                        .push(Text::new(process.pid.to_string()).width(Length::Fixed(100.0)))
                        .push(Text::new(&process.name).width(Length::Fixed(200.0)))
                        .push(Text::new(process.memory.to_string()).width(Length::Fixed(100.0)))
                        .push(Text::new(format!("{:.1}", process.cpu)).width(Length::Fixed(100.0)))
                        .push(
                            Button::new("Kill")
                                .on_press(Message::KillProcess(process.pid))
                        )
                )
            },
        );

        let content = Column::new()
            .push(header)
            .push(Scrollable::new(processes));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(5)).map(|_| Message::Tick)
    }
}

fn main() -> iced::Result {
    TaskManager::run(Settings::default())
}