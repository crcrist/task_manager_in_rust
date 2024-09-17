// builds the GUI
use iced::{ theme::Theme,
    alignment, executor, time, Application, Command, Element, Length, Settings, Subscription,
    widget::{Button, Column, Container, Row, Scrollable, Text},
};

// gathers info about system
use sysinfo::{Pid, System};

// from std library to define time intervals
use std::time::Duration;

struct TaskManager {
    // list of current running procersses using the ProcessInfo struct
    processes: Vec<ProcessInfo>,
    // specifies which column the process list is sorted by 
    sort_column: SortColumn,
    // indicates sorting as ascending or descending
    sort_ascending: bool,
    // instance of sysinfo to gather and refresh system data
    system: System,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    // holds info about each process
    pid: u32,
    name: String,
    memory: u64,
    cpu: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    // defines columns which can be sorted
    Pid,
    Name,
    Memory,
    Cpu,
}

#[derive(Debug, Clone)]

// different types of messages/events the app can handle
enum Message {
    // changes the sorting based on the selected column    
    Sort(SortColumn),

    // kills the process with the given PID
    KillProcess(u32),

    // triggers an update or refresh of the process list every few seconds
    Tick,
}

impl TaskManager {
    // refresh funciton - refreshes the process list by calling self.system.refresh_all(), 
    // and updates the processes vector with the latest system info
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

    // Sorts the process list based on the selected sort column and order (asc/desc)
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

// defining how the GUI behaves
impl Application for TaskManager {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::theme::Theme;

    // new initializes a new TaskManager instance, refreshing the process list immediately
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

    // defines the window title as Task Manager
    fn title(&self) -> String {
        String::from("Task Manager")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    // handles incoming messages (killing or sorting)
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // refreshes the program every 5 seconds
            Message::Tick => {
                self.refresh();
                Command::none()
            }
            // sorts the list by the given column
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
            // attempts to kill the process with the given PID
            Message::KillProcess(pid) => {
                if let Some(process) = self.system.process(Pid::from(pid as usize)) {
                    let _ = process.kill();
                }
                self.refresh();
                Command::none()
            }
        }
    }
    // construct the GUI layout
    fn view(&self) -> Element<Message> {
        // displays buttons for sorting the process list by PID, name, memory, and CPU
        let header = Row::new()
        
            
            // space inbetween header buttons
            .spacing(10)
            .push(Button::new("PID").on_press(Message::Sort(SortColumn::Pid)).width(Length::FillPortion(1)))
            .push(Button::new("Name").on_press(Message::Sort(SortColumn::Name)).width(Length::FillPortion(2)))
            .push(Button::new("Memory (MB)").on_press(Message::Sort(SortColumn::Memory)).width(Length::FillPortion(1)))
            .push(Button::new("CPU (%)").on_press(Message::Sort(SortColumn::Cpu)).width(Length::FillPortion(1)));
        // displays each porcess in a row with it's PID, name, memory, CPU usage, and Kill button
        let processes = self.processes.iter().fold(
            Column::new().spacing(5),
            |column, process| {
                column.push(
                    Row::new()
                    .spacing(10)
                        .push(Text::new(process.pid.to_string()).width(Length::FillPortion(1)))
                        .push(Text::new(&process.name).width(Length::FillPortion(2)))
                        .push(Text::new(process.memory.to_string()).width(Length::FillPortion(1)))
                        .push(Text::new(format!("{:.1}", process.cpu)).width(Length::FillPortion(1)))
                        .push(
                            Button::new("Kill")
                                .on_press(Message::KillProcess(process.pid))
                                .width(Length::Shrink)
                        )
                )
            },
        );
        // process list is scrollable
        let content = Column::new()
            .spacing(10)
            .push(header)
            .push(Scrollable::new(processes));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(30)
            .center_x()
            .center_y()
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }
    // sets up a timer that triggers a tick message every 5 seconds to refresh the process list
    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(5)).map(|_| Message::Tick)
    }
}

// entry point of the application
fn main() -> iced::Result {
    TaskManager::run(Settings::default())
}