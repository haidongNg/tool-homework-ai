use ratatui::widgets::ListState;
use sysinfo::{System, Networks, Disks};

#[derive(PartialEq)]
pub enum AppScreen {
    Menu,
    Dashboard,
    PomodoroTool,
    NotesTool,
}

pub struct App {
    pub screen: AppScreen,
    pub menu_state: ListState,
    pub menu_items: Vec<&'static str>,
    pub sys: System,
    pub networks: Networks,
    pub disks: Disks,
    pub cpu_history: Vec<u64>,
    
    // --- TÍNH NĂNG MỚI CHO DASHBOARD ---
    pub selected_tab: usize,            // Tab đang chọn trên Dashboard (0: Tổng quan, 1: Tiến trình)
    pub action_message: String,         // Dòng thông báo trạng thái khi bấm phím
}

impl App {
    pub fn new() -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));

        let mut sys = System::new_all();
        sys.refresh_all();
        
        App {
            screen: AppScreen::Menu,
            menu_state,
            menu_items: vec![
                "1. 💻 System Dashboard (Giám sát hệ thống)",
                "2. ⏱️  Đồng hồ Pomodoro",
                "3. 📓 Ghi chú nhanh (Quick Note)",
                "4. ❌ Thoát",
            ],
            sys,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            cpu_history: vec![0; 150],
            selected_tab: 0,
            action_message: String::from("SYSTEM READY. PRESS [Tab] TO SWITCH VIEWS, [P] TO FLUSH MEMORY."),
        }
    }

    pub fn update_dashboard_data(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true); // Quét danh sách tiến trình chi tiết
        
        self.networks.refresh(true);
        self.disks.refresh(true);

        let mut total_cpu = 0.0;
        for cpu in self.sys.cpus() {
            total_cpu += cpu.cpu_usage();
        }
        let cpu_usage = total_cpu / self.sys.cpus().len() as f32;
        
        self.cpu_history.push(cpu_usage as u64);
        if self.cpu_history.len() > 150 {
            self.cpu_history.remove(0);
        }
    }
}