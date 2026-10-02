pub mod app;
pub mod ui;

use app::{App, AppScreen};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<(), io::Error> {
    // Thiết lập môi trường terminal ban đầu
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Vòng lặp chính của ứng dụng
    loop {
        // Cập nhật dữ liệu hệ thống liên tục nếu đang đứng ở màn hình Dashboard
        if app.screen == AppScreen::Dashboard {
            app.update_dashboard_data();
        }

        // Vẽ giao diện dựa trên trạng thái hiện tại của app
        terminal.draw(|f| ui::render(f, &mut app))?;

        // Lắng nghe sự kiện bàn phím (chờ tối đa 500ms mỗi khung hình)
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    
                    // Phím ESC: Luôn đưa người dùng quay trở lại Menu chính từ bất kỳ đâu
                    if key.code == KeyCode::Esc {
                        app.screen = AppScreen::Menu;
                        continue;
                    }

                    // Phân nhánh xử lý phím tùy thuộc vào màn hình hiện tại
                    match app.screen {
                        AppScreen::Menu => match key.code {
                            // Nhấn 'q' để thoát hẳn chương trình
                            KeyCode::Char('q') => break,
                            
                            // Điều hướng Menu bằng phím mũi tên Lên / Xuống
                            KeyCode::Down => {
                                let i = app.menu_state.selected().unwrap_or(0);
                                app.menu_state.select(Some((i + 1) % app.menu_items.len()));
                            }
                            KeyCode::Up => {
                                let i = app.menu_state.selected().unwrap_or(0);
                                app.menu_state.select(Some(if i == 0 { app.menu_items.len() - 1 } else { i - 1 }));
                            }
                            
                            // Nhấn Enter để chọn mục tương ứng trên Menu
                            KeyCode::Enter => match app.menu_state.selected().unwrap_or(0) {
                                0 => app.screen = AppScreen::Dashboard,
                                1 => app.screen = AppScreen::PomodoroTool,
                                2 => app.screen = AppScreen::NotesTool,
                                3 => break,
                                _ => {}
                            },
                            _ => {}
                        },

                        // --- XỬ LÝ PHÍM TƯƠNG TÁC RIÊNG CHO DASHBOARD ---
                        AppScreen::Dashboard => match key.code {
                            // Bấm phím Tab: Chuyển đổi qua lại giữa giao diện Phần cứng và Danh sách Tiến trình
                            KeyCode::Tab => {
                                app.selected_tab = (app.selected_tab + 1) % 2;
                            }
                            // Bấm phím 'p' hoặc 'P': Kích hoạt lệnh giả lập dọn dẹp bộ nhớ RAM
                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                app.action_message = String::from("⚡ [WARNING] MEMORY FLUSH TRIGGERED. CACHE CLEARED.");
                            }
                            _ => {}
                        },

                        _ => {}
                    }
                }
            }
        }
    }

    // Dọn dẹp trả lại trạng thái ban đầu cho Terminal trước khi tắt ứng dụng
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}