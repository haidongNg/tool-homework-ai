use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, BorderType, Borders, Gauge, Paragraph, Sparkline, Table, Row},
    Frame,
};
use crate::app::App;
use sysinfo::System;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // --- BẢNG MÀU CHUẨN KHOA HỌC & RÕ RÀNG ---
    let matrix_green = Color::LightGreen;
    let dark_green = Color::Green;
    let neon_cyan = Color::Cyan;
    let neon_yellow = Color::Yellow;
    let neon_magenta = Color::Magenta;
    let neon_orange = Color::LightRed;
    let alert_red = Color::Red;

    // --- PHÂN TẦNG KHÔNG GIAN (5 TẦNG LOGIC) ---
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7), // Tầng 1: System Info & Network (Cao 7 dòng)
            Constraint::Length(6), // Tầng 2: RAM & Swap Gauges (Cao 6 dòng)
            Constraint::Min(10),   // Tầng 3: Vùng linh hoạt (CPU Cores / Hoặc Process Table)
            Constraint::Length(5), // Tầng 4: Lịch sử tải tổng quan (Sparkline)
            Constraint::Length(3), // Tầng 5: Thanh điều hướng / Console log ở đáy
        ])
        .split(size);

    // ====================================================================
    // TẦNG 1: THÔNG TIN HỆ THỐNG & MẠNG (Chia 2 cột cân đối)
    // ====================================================================
    let t1_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    // 1. System Identity
    let host = System::host_name().unwrap_or_else(|| "UNKNOWN".to_string());
    let os = System::long_os_version().unwrap_or_else(|| "UNKNOWN".to_string());
    let uptime = System::uptime();
    let uptime_str = format!("{}h {}m", uptime / 3600, (uptime % 3600) / 60);
    let cores = app.sys.cpus().len();

    let sys_info_text = vec![
        Line::from(vec![Span::styled(" OS: ", Style::default().fg(dark_green)), Span::styled(os, Style::default().fg(neon_cyan))]),
        Line::from(vec![Span::styled(" HOST: ", Style::default().fg(dark_green)), Span::styled(host, Style::default().fg(neon_cyan))]),
        Line::from(vec![Span::styled(" UPTIME / CORES: ", Style::default().fg(dark_green)), Span::styled(format!("{} | {} Cores", uptime_str, cores), Style::default().fg(matrix_green))]),
        Line::from(vec![Span::styled(" VIEW MODE: ", Style::default().fg(dark_green)), Span::styled(if app.selected_tab == 0 { "HARDWARE (TAB)" } else { "PROCESSES (TAB)" }, Style::default().fg(neon_magenta).add_modifier(Modifier::BOLD))]),
    ];
    let sys_info_widget = Paragraph::new(sys_info_text)
        .block(Block::default().title(" [ 1. SYSTEM IDENTITY ] ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(neon_cyan)));
    f.render_widget(sys_info_widget, t1_chunks[0]);

    // 2. Network Uplink (Đã lọc card mạng ảo và ưu tiên card chính)
    let mut net_lines = vec![];
    
    // Sắp xếp các interface để card có traffic (hoặc tên chuẩn) lên đầu
    let mut sorted_networks: Vec<_> = app.networks.iter().collect();
    sorted_networks.sort_by(|a, b| b.1.total_received().cmp(&a.1.total_received()));

    for (name, data) in sorted_networks {
        let name_str = name.as_str().to_lowercase();
        
        // Lọc bỏ các card ảo rác thường gặp trên Windows/Linux (như vEthernet, bluetooth, dummy, loopback...)
        let is_virtual = name_str.contains("vethernet") 
            || name_str.contains("bluetooth") 
            || name_str.contains("pseudo") 
            || name_str.contains("loopback")
            || name_str.contains("docker");

        if is_virtual {
            continue; // Bỏ qua không hiển thị
        }

        // Chỉ hiển thị các card có hoạt động hoặc có tên chuẩn (Wi-Fi, Ethernet, en0, wlan0...)
        if data.total_received() > 0 || data.total_transmitted() > 0 || name_str.contains("wi-fi") || name_str.contains("ethernet") || name_str.contains("en0") {
            let rx_kb = data.received() / 1024;
            let tx_kb = data.transmitted() / 1024;
            
            let total_rx_mb = data.total_received() / (1024 * 1024);
            let total_tx_mb = data.total_transmitted() / (1024 * 1024);

            net_lines.push(Line::from(vec![
                Span::styled(format!(" [{}] ", name), Style::default().fg(neon_yellow).add_modifier(Modifier::BOLD)),
            ]));
            net_lines.push(Line::from(vec![
                Span::styled(format!("  ↓ {} KB/s ({} MB)", rx_kb, total_rx_mb), Style::default().fg(neon_cyan)),
            ]));
            net_lines.push(Line::from(vec![
                Span::styled(format!("  ↑ {} KB/s ({} MB)", tx_kb, total_tx_mb), Style::default().fg(neon_orange)),
            ]));
            
            // Chỉ hiển thị tối đa 1-2 card mạng chính đang hoạt động mạnh nhất để tránh tràn khung
            break; 
        }
    }

    if net_lines.is_empty() { 
        net_lines.push(Line::from(Span::styled("  NO ACTIVE NETWORK INTERFACE", Style::default().fg(Color::DarkGray)))); 
    }

    let net_widget = Paragraph::new(net_lines)
        .block(Block::default().title(" [ 2. NETWORK UPLINK ] ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(neon_yellow)));
    f.render_widget(net_widget, t1_chunks[1]);

    // ====================================================================
    // TẦNG 2: BỘ NHỚ VẬT LÝ (RAM & SWAP GAUGES)
    // ====================================================================
    let t2_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    let total_mem = app.sys.total_memory() / 1_048_576; 
    let used_mem = app.sys.used_memory() / 1_048_576;
    let mem_percent = if total_mem > 0 { (used_mem as f64 / total_mem as f64 * 100.0) as u16 } else { 0 };

    let total_swap = app.sys.total_swap() / 1_048_576;
    let used_swap = app.sys.used_swap() / 1_048_576;
    let swap_percent = if total_swap > 0 { (used_swap as f64 / total_swap as f64 * 100.0) as u16 } else { 0 };

    let ram_gauge = Gauge::default()
        .block(Block::default().title(" [ 3. MEMORY ALLOCATION ] ").borders(Borders::ALL).border_style(Style::default().fg(neon_magenta)))
        .gauge_style(Style::default().fg(if mem_percent > 85 { alert_red } else { neon_magenta }))
        .percent(mem_percent)
        .label(format!("RAM: {} / {} MB ({}%)", used_mem, total_mem, mem_percent));
    f.render_widget(ram_gauge, t2_chunks[0]);

    let swap_gauge = Gauge::default()
        .block(Block::default().title(" [ 4. SWAP SPACE ] ").borders(Borders::ALL).border_style(Style::default().fg(neon_magenta)))
        .gauge_style(Style::default().fg(neon_orange))
        .percent(swap_percent)
        .label(format!("SWAP: {} / {} MB ({}%)", used_swap, total_swap, swap_percent));
    f.render_widget(swap_gauge, t2_chunks[1]);

    // ====================================================================
    // TẦNG 3: VÙNG NỘI DUNG LINH HOẠT (Thay đổi theo phím Tab)
    // ====================================================================
    if app.selected_tab == 0 {
        // Tab 0: Phân tích Tải từng Nhân CPU (Trái) kết hợp Dung lượng Ổ đĩa (Phải)
        let t3_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main_chunks[2]);

        // Biểu đồ cột CPU
        let mut cpu_data: Vec<(String, u64)> = Vec::new();
        for (i, cpu) in app.sys.cpus().iter().enumerate() {
            cpu_data.push((format!("C{}", i), cpu.cpu_usage() as u64));
        }
        let barchart_data: Vec<(&str, u64)> = cpu_data.iter().map(|(s, v)| (s.as_str(), *v)).collect();
        let cpu_barchart = BarChart::default()
            .block(Block::default().title(" [ 5. CPU CORES OVERLOAD ANALYSIS ] ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(matrix_green)))
            .data(&barchart_data)
            .bar_width(3).bar_gap(1)
            .bar_style(Style::default().fg(dark_green))
            .value_style(Style::default().fg(Color::Black).bg(matrix_green));
        f.render_widget(cpu_barchart, t3_chunks[0]);

        // Thông tin Storage
        let mut disk_lines = vec![];
        for disk in &app.disks {
            let name = disk.name().to_string_lossy();
            let total = disk.total_space() / 1_073_741_824;
            let available = disk.available_space() / 1_073_741_824;
            let used = total.saturating_sub(available);
            let percent = if total > 0 { (used as f64 / total as f64 * 100.0) as u16 } else { 0 };

            disk_lines.push(Line::from(vec![
                Span::styled(format!(" {}: ", name), Style::default().fg(neon_yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}% used", percent), Style::default().fg(if percent > 85 { alert_red } else { neon_cyan })),
            ]));
            disk_lines.push(Line::from(vec![
                Span::styled(format!("   {} / {} GB", used, total), Style::default().fg(dark_green)),
            ]));
        }
        let disk_widget = Paragraph::new(disk_lines)
            .block(Block::default().title(" [ 6. STORAGE DISKS ] ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(neon_magenta)));
        f.render_widget(disk_widget, t3_chunks[1]);
    } else {
        // Tab 1: Bảng danh sách Top Tiến trình ngốn RAM nhất
        let mut processes: Vec<_> = app.sys.processes().iter().collect();
        processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

        let rows: Vec<Row> = processes.iter().take(10).map(|(pid, proc)| {
            Row::new(vec![
                format!("{}", pid),
                format!("{}", proc.name().to_string_lossy()),
                format!("{} MB", proc.memory() / 1_048_576),
                format!("{:.1}%", proc.cpu_usage()),
            ]).style(Style::default().fg(neon_cyan))
        }).collect();

        let table = Table::new(rows, [Constraint::Length(8), Constraint::Min(25), Constraint::Length(12), Constraint::Length(10)])
            .header(Row::new(vec!["PID", "PROCESS NAME", "RAM", "CPU"]).style(Style::default().fg(neon_orange).add_modifier(Modifier::BOLD)))
            .block(Block::default().title(" [ 5. TOP HEAVY PROCESSES ] ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(neon_orange)));
        f.render_widget(table, main_chunks[2]);
    }

    // ====================================================================
    // TẦNG 4: BIỂU ĐỒ SÓNG LỊCH SỬ TỔNG QUAN
    // ====================================================================
    let sparkline = Sparkline::default()
        .block(Block::default().title(" [ 7. GLOBAL CPU ACTIVITY HISTORY ] ").borders(Borders::ALL).border_style(Style::default().fg(neon_orange)))
        .data(&app.cpu_history)
        .style(Style::default().fg(neon_orange));
    f.render_widget(sparkline, main_chunks[3]);

    // ====================================================================
    // TẦNG 5: THANH TRẠNG THÁI & HƯỚNG DẪN Ở ĐÁY
    // ====================================================================
    let console = Paragraph::new(app.action_message.clone())
        .style(Style::default().fg(neon_yellow))
        .block(Block::default().title(" [ COMMAND BAR // [TAB]: SWITCH VIEW | [P]: FLUSH MEMORY | [ESC]: EXIT ] ").borders(Borders::ALL).border_style(Style::default().fg(neon_yellow)));
    f.render_widget(console, main_chunks[4]);
}